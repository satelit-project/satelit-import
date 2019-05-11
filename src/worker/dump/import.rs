use futures::future::Either;
use futures::prelude::*;
use futures::try_ready;
use log::{trace, warn};
use tokio::codec::{FramedRead, FramedWrite, LinesCodec};
use tokio::fs;
use tokio_threadpool::{blocking, BlockingError};

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use crate::anidb::{AniDb, Anime, XmlError};
use crate::db::{entity::NewSchedule, schedules, ConnectionPool, QueryError, Table};

/// Creates AniDB dump importer configured with global app settings
///
/// Returned future will block your current task until it's ready
pub fn importer<P, C>(
    old_dump_path: P,
    dump_path: P,
    connection_pool: C,
) -> impl Future<Item = (), Error = ImportError>
where
    P: AsRef<Path> + Clone + Send + 'static,
    C: ConnectionPool + Send,
{
    let provider = AniDbAnimeProvider::new(old_dump_path, dump_path);
    let schedules = schedules::Schedules::new(connection_pool);
    let scheduler = AniDbImportScheduler::new(schedules);

    DumpImporter::new(provider, scheduler, None).and_then(|_| Ok(()))
}

/// Creates AniDB dump importer configured with global app settings and ability to keep track of
/// failed-to-import anime entities. Entity id's are stored at `reimport_path`.
///
/// This future does it's best to keep track of failed-to-imported entities and will try to import them
/// again at next invocation, but due to possible I/O errors some entities may be lost.
/// Moreover, it doesn't keep track of entities that should be removed but failed to do so. That
/// means that DB may have "dead" entities (i.e. not in AniDB db anymore). It's
/// advised to do full DB update from times to times to import lost and remove "dead"
/// entities
pub fn tracking_importer<P, C>(
    old_dump_path: P,
    dump_path: P,
    reimport_path: P,
    connection_pool: C,
) -> impl Future<Item = (), Error = ImportError>
where
    P: AsRef<Path> + Clone + Send + 'static,
    C: ConnectionPool + Send,
{
    let provider = AniDbAnimeProvider::new(old_dump_path, dump_path);
    let schedules = schedules::Schedules::new(connection_pool);
    let scheduler = AniDbImportScheduler::new(schedules);

    track_importer(reimport_path, move |reimport| {
        DumpImporter::new(provider, scheduler, reimport)
    })
}

// TODO: omg, please refactor
fn track_importer<P, B, F>(
    reimport_path: P,
    builder: B,
) -> impl Future<Item = (), Error = ImportError>
where
    P: AsRef<Path> + Clone + Send + 'static,
    B: FnOnce(Option<HashSet<i32>>) -> F,
    F: Future<Item = Option<HashSet<i32>>, Error = ImportError>,
{
    // open file with failed-to-import ids
    fs::File::open(reimport_path.clone())
        .and_then(|f| {
            // read them line-by-line and put to HashSet
            let reader = FramedRead::new(f, LinesCodec::new());
            reader.fold(HashSet::new(), |mut set, line| {
                if let Ok(id) = line.parse() {
                    set.insert(id);
                }

                Result::<_, std::io::Error>::Ok(set)
            })
        })
        .then(move |reimport| {
            // if we got an I/O error, transform result to None and log error
            let reimport = reimport
                .map_err(|e| warn!("Failed to load id's to reimport: {}", e))
                .ok();

            builder(reimport)
        })
        .and_then(|result| {
            match result {
                // if we successfully read the reimport file, then now we have an updated HashSet of
                // failed-to-import ids, so we can rewrite the file with new content
                Some(failed) => {
                    // the last `map_err` will never be executed but it needed to pass type checking
                    let fut = fs::File::create(reimport_path)
                        .and_then(|f| {
                            let writer = FramedWrite::new(f, LinesCodec::new());
                            writer.send_all(futures::stream::iter_ok::<_, std::io::Error>(
                                failed.into_iter().map(|id| format!("{}", id)),
                            ))
                        })
                        .map_err(|e| warn!("Failed to write reimport data: {}", e))
                        .then(|_| Result::<_, std::io::Error>::Ok(()))
                        .map_err(|_| ImportError::InternalError("".to_owned()));

                    Either::A(fut)
                }
                // if not then don't do anything
                None => Either::B(futures::finished(())),
            }
        })
}

/// Performs anime import from AniDB dump asynchronously
///
/// ## Note
/// This future will block your current task until it's ready due to moving into blocking
/// section of tokio thread pool. If this is not desired behavior, spawn a separate task and use
/// this future there. For more info see docs for `tokio_threadpool::blocking`
pub struct DumpImporter<I, P, S>(AnimeImporter<P, S>)
where
    I: Iterator<Item = Anime>,
    P: AnimeProvider<Iterator = I>,
    S: ImportScheduler;

impl<I, P, S> DumpImporter<I, P, S>
where
    I: Iterator<Item = Anime>,
    P: AnimeProvider<Iterator = I>,
    S: ImportScheduler,
{
    pub fn new(provider: P, scheduler: S, reimport: Option<HashSet<i32>>) -> Self {
        let importer = AnimeImporter::new(provider, scheduler, reimport);
        DumpImporter(importer)
    }
}

impl<I, P, S> Future for DumpImporter<I, P, S>
where
    I: Iterator<Item = Anime>,
    P: AnimeProvider<Iterator = I>,
    S: ImportScheduler,
{
    type Item = Option<HashSet<i32>>;
    type Error = ImportError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let importer = self.0.clone();

        let inner = blocking(move || {
            let mut importer = importer;
            importer.begin()
        });

        match try_ready!(inner) {
            Ok(v) => Ok(Async::Ready(v)),
            Err(e) => Err(e),
        }
    }
}

/// Performs anime import with titles from `provider` and schedules changes in `scheduler`.
#[derive(Clone)]
pub struct AnimeImporter<P, S>
where
    P: AnimeProvider,
    S: ImportScheduler,
{
    /// Data source for anime titles
    provider: P,

    /// Scheduler for importing changes to db
    scheduler: S,

    /// Anime id's that should be re-imported
    reimport: Option<HashSet<i32>>,
}

impl<P, S> AnimeImporter<P, S>
where
    P: AnimeProvider,
    S: ImportScheduler,
{
    /// Creates new instance with provided parameters
    pub fn new(provider: P, scheduler: S, reimport: Option<HashSet<i32>>) -> Self {
        AnimeImporter {
            provider,
            scheduler,
            reimport,
        }
    }

    /// Starts importing anime titles by using id's to determine diff that should be processes.
    /// This method assumes that id's sorted in ascending order and no duplicates exists
    ///
    /// ## Returns
    /// ID's of anime entries that should be imported but has been skipped because of an scheduler
    /// error (like failed to write to db)
    ///
    /// ## Note
    /// This method will block current thread until import is done
    pub fn begin(&mut self) -> Result<Option<HashSet<i32>>, ImportError> {
        let mut iter_old = match self.provider.old_anime_titles() {
            Ok(iter) => iter,
            Err(e) => return Err(e.into()),
        };

        let mut iter_new = match self.provider.new_anime_titles() {
            Ok(iter) => iter,
            Err(e) => return Err(e.into()),
        };

        let mut old = iter_old.next();
        let mut new = iter_new.next();

        while old.is_some() || new.is_some() {
            if old.is_none() & &new.is_some() {
                self.add_title(new.as_ref().unwrap());
                new = iter_new.next();
            } else if old.is_some() & &new.is_none() {
                self.remove_title(old.as_ref().unwrap());
                old = iter_old.next();
            } else {
                let o = old.as_ref().unwrap();
                let n = new.as_ref().unwrap();

                match o.id.cmp(&n.id) {
                    Ordering::Less => {
                        self.remove_title(o);
                        old = iter_old.next();
                    }
                    Ordering::Greater => {
                        self.add_title(n);
                        new = iter_new.next();
                    }
                    Ordering::Equal => {
                        if self
                            .reimport
                            .as_ref()
                            .map(|v| v.contains(&n.id))
                            .unwrap_or(false)
                        {
                            self.add_title(n)
                        }

                        old = iter_old.next();
                        new = iter_new.next();
                    }
                }
            }
        }

        Ok(self.reimport.clone())
    }

    fn add_title(&mut self, anime: &Anime) {
        match self.scheduler.add_title(anime) {
            Err(e) => {
                warn!("adding schedule failed for id:{}: {}", anime.id, e);
                self.reimport.as_mut().map(|v| v.insert(anime.id));
            }
            Ok(()) => {
                trace!("added new schedule for id:{}", anime.id);
                self.reimport.as_mut().map(|v| v.remove(&anime.id));
            }
        }
    }

    fn remove_title(&mut self, anime: &Anime) {
        match self.scheduler.remove_title(anime) {
            Err(e) => {
                warn!("removing schedule failed for id:{}: {}", anime.id, e);
            }
            Ok(()) => {
                trace!("removed old schedule for id:{}", anime.id);
            }
        }
    }
}

/// Represents an error that may occur during anime import
#[derive(Debug)]
pub enum ImportError {
    /// Failed to read data from data source
    ///
    /// For example, situation where `AnimeProvider` will not be able to provide anime titles
    /// will cause that error.
    DataSourceFailed(String),

    /// Something bad happened and import can't be finished
    ///
    /// This is an error that may be caused due to errors in tokio runtime.
    InternalError(String),
}

impl From<XmlError> for ImportError {
    fn from(e: XmlError) -> Self {
        ImportError::DataSourceFailed(format!("data source error: {}", e))
    }
}

impl From<BlockingError> for ImportError {
    fn from(e: BlockingError) -> Self {
        ImportError::InternalError(format!("failed to enter blocking section: {}", e))
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ImportError::*;

        match self {
            DataSourceFailed(e) => e.fmt(f),
            InternalError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ImportError {}

/// Data source for anime records that should be imported
pub trait AnimeProvider: Clone + Send {
    /// Iterator for anime entities that should be processes. Entities should be sorted by id
    /// and returned in ascended order
    type Iterator: Iterator<Item = Anime>;

    /// If provider can't return an iterator this error type will be used to determine a cause of
    /// the error
    type Error: Into<ImportError>;

    /// Returns iterator for previously imported anime titles
    ///
    /// It used to build a diff of changed anime entities and process them only. The iterator may
    /// return `None` at any time. In that case all titles returned from `new_anime_titles`
    /// iterator would be imported as new titles
    fn old_anime_titles(&self) -> Result<Self::Iterator, Self::Error>;

    /// Returns iterator for anime titles that should be imported
    ///
    /// If non-empty iterator is returned from `old_anime_titles` then only diff will be processes
    fn new_anime_titles(&self) -> Result<Self::Iterator, Self::Error>;
}

/// Data source for anime entities from AniDB dumps
#[derive(Clone)]
pub struct AniDbAnimeProvider<P: AsRef<Path> + Clone + Send> {
    old_dump_path: P,
    new_dump_path: P,
}

impl<P: AsRef<Path> + Clone + Send> AniDbAnimeProvider<P> {
    /// Creates instance with AniDB anime dumps
    ///
    /// * `old_dump_path` – path to previously imported dump
    /// * `new_dump_path` - path to dump that should be imported
    pub fn new(old_dump_path: P, new_dump_path: P) -> Self {
        AniDbAnimeProvider {
            old_dump_path,
            new_dump_path,
        }
    }
}

impl<P: AsRef<Path> + Clone + Send> AnimeProvider for AniDbAnimeProvider<P> {
    type Iterator = AniDb;
    type Error = XmlError;

    fn old_anime_titles(&self) -> Result<Self::Iterator, Self::Error> {
        AniDb::new(self.old_dump_path.as_ref())
    }

    fn new_anime_titles(&self) -> Result<Self::Iterator, Self::Error> {
        AniDb::new(self.new_dump_path.as_ref())
    }
}

/// Processes changes to anime entities storage
pub trait ImportScheduler: Clone + Send {
    type Error: std::error::Error;

    /// Adds new anime title to anime storage
    fn add_title(&mut self, anime: &Anime) -> Result<(), Self::Error>;

    /// Removes anime title from anime storage
    fn remove_title(&mut self, anime: &Anime) -> Result<(), Self::Error>;
}

/// Schedules for anime titles from AniDB dump
#[derive(Clone)]
pub struct AniDbImportScheduler<P: ConnectionPool + Send> {
    /// Db table for scheduled imports
    schedules: schedules::Schedules<P>,
}

impl<P: ConnectionPool + Send> AniDbImportScheduler<P> {
    pub fn new(schedules: schedules::Schedules<P>) -> Self {
        AniDbImportScheduler { schedules }
    }
}

impl<P: ConnectionPool + Send> ImportScheduler for AniDbImportScheduler<P> {
    type Error = QueryError;

    fn add_title(&mut self, anime: &Anime) -> Result<(), Self::Error> {
        use crate::schedules_insert;

        let schedule = NewSchedule::new(anime.id);
        schedules_insert!(self.schedules, &schedule)
    }

    fn remove_title(&mut self, anime: &Anime) -> Result<(), Self::Error> {
        use crate::schedules_delete;

        schedules_delete!(self.schedules, anidb_id(anime.id))
    }
}

#[cfg(test)]
mod tests_notrack {
    use super::super::test_utils::import::*;
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_import_no_diff() {
        let provider = FakeProvider {
            old: vec![],
            new: gen_anime([1, 2, 3, 4, 5]),
        };

        let scheduler = FakeScheduler {
            added: Arc::new(Mutex::new(vec![])),
            removed: Arc::new(Mutex::new(vec![])),
            skip_add: Arc::new(None),
        };

        let importer = DumpImporter::new(provider.clone(), scheduler.clone(), Some(HashSet::new()));
        tokio::run(
            importer
                .map_err(|e| panic!("unexpected err: {}", e))
                .and_then(|_| Ok(())),
        );

        assert!(scheduler.removed.lock().unwrap().is_empty());
        assert_eq!(*scheduler.added.lock().unwrap(), provider.new);
    }

    #[test]
    fn test_import_diff_add() {
        let provider = FakeProvider {
            old: gen_anime([1, 3, 5]),
            new: gen_anime([1, 2, 3, 4, 5]),
        };

        let scheduler = FakeScheduler {
            added: Arc::new(Mutex::new(vec![])),
            removed: Arc::new(Mutex::new(vec![])),
            skip_add: Arc::new(None),
        };

        let importer = DumpImporter::new(provider.clone(), scheduler.clone(), Some(HashSet::new()));
        tokio::run(
            importer
                .map_err(|e| panic!("unexpected err: {}", e))
                .and_then(|_| Ok(())),
        );

        assert!(scheduler.removed.lock().unwrap().is_empty());
        assert_eq!(*scheduler.added.lock().unwrap(), gen_anime([2, 4]));
    }

    #[test]
    fn test_import_diff_remove() {
        let provider = FakeProvider {
            old: gen_anime([1, 2, 3, 4, 5]),
            new: gen_anime([1, 3, 5]),
        };

        let scheduler = FakeScheduler {
            added: Arc::new(Mutex::new(vec![])),
            removed: Arc::new(Mutex::new(vec![])),
            skip_add: Arc::new(None),
        };

        let importer = DumpImporter::new(provider.clone(), scheduler.clone(), Some(HashSet::new()));
        tokio::run(
            importer
                .map_err(|e| panic!("unexpected err: {}", e))
                .and_then(|_| Ok(())),
        );

        assert!(scheduler.added.lock().unwrap().is_empty());
        assert_eq!(*scheduler.removed.lock().unwrap(), gen_anime([2, 4]));
    }

    #[test]
    fn test_import_diff_add_remove() {
        let provider = FakeProvider {
            old: gen_anime([1, 3, 5]),
            new: gen_anime([2, 4, 5, 7]),
        };

        let scheduler = FakeScheduler {
            added: Arc::new(Mutex::new(vec![])),
            removed: Arc::new(Mutex::new(vec![])),
            skip_add: Arc::new(None),
        };

        let importer = DumpImporter::new(provider.clone(), scheduler.clone(), Some(HashSet::new()));
        tokio::run(
            importer
                .map_err(|e| panic!("unexpected err: {}", e))
                .and_then(|_| Ok(())),
        );

        assert_eq!(*scheduler.removed.lock().unwrap(), gen_anime([1, 3]));
        assert_eq!(*scheduler.added.lock().unwrap(), gen_anime([2, 4, 7]));
    }
}

#[cfg(test)]
mod tests_track {
    use super::super::test_utils::import::*;
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_no_reimport() -> Result<(), std::io::Error> {
        let track_file = tempfile::Builder::new().tempfile()?;

        Ok(())
    }
}

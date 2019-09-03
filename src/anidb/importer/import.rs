use futures::prelude::*;
use log::{debug, warn};

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

use crate::anidb::parser::{Anidb, Anime, XmlError};
use crate::db::entity::{ExternalSource, QueuedTask};
use crate::db::{schedules, ConnectionPool, QueryError};
use crate::block::{blocking, BlockingError};

/// Creates AniDB dump importer configured with global app settings
///
/// Returned future will block your current task until it's ready
pub fn importer<P>(
    old_dump_path: P,
    new_dump_path: P,
    reimport_ids: HashSet<i32>,
    connection_pool: ConnectionPool,
) -> impl Future<Item = HashSet<i32>, Error = ImportError> + Send
where
    P: AsRef<Path> + Clone + Send + 'static,
{
    blocking(move || {
        let provider = AniDbAnimeProvider::new(old_dump_path, new_dump_path, reimport_ids);
        let schedules = schedules::Schedules::new(connection_pool);
        let scheduler = AniDbImportScheduler::new(schedules);
        let mut importer = AnimeImporter::new(provider, scheduler);

        importer.begin()
    }).from_err()
}

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

    /// Returns `true` if anime title with provided `id` should be imported again
    fn should_reimport(&self, id: i32) -> bool;
}

/// Processes changes to anime entities storage
pub trait ImportScheduler: Clone + Send {
    type Error: std::error::Error;

    /// Adds new anime title to anime storage
    fn add_title(&mut self, anime: &Anime) -> Result<(), Self::Error>;

    /// Removes anime title from anime storage
    fn remove_title(&mut self, anime: &Anime) -> Result<(), Self::Error>;
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

    /// Anime IDs that has not been imported
    skipped_ids: HashSet<i32>,
}

/// Data source for anime entities from AniDB dumps
#[derive(Clone)]
pub struct AniDbAnimeProvider<P: AsRef<Path> + Clone + Send> {
    old_dump_path: P,
    new_dump_path: P,
    reimport_ids: HashSet<i32>,
}

/// Schedules for anime titles from AniDB dump
#[derive(Clone)]
pub struct AniDbImportScheduler {
    /// Db table for scheduled imports
    schedules: schedules::Schedules,
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
    /// This is an error that may be caused due to errors with tokio or rayon.
    InternalError(String),
}

// MARK: impl AnimeImporter

impl<P, S> AnimeImporter<P, S>
where
    P: AnimeProvider,
    S: ImportScheduler,
{
    /// Creates new instance with provided parameters
    pub fn new(provider: P, scheduler: S) -> Self {
        AnimeImporter {
            provider,
            scheduler,
            skipped_ids: HashSet::new(),
        }
    }

    /// Starts importing anime titles by using id's to determine diff that should be processes.
    /// This method assumes that id's sorted in ascending order and no duplicates exists
    ///
    /// ## Returns
    /// ID's of anime entries that should be imported but has been skipped because of an scheduler
    /// error (like failed to write to db) or error in case if import failed to start
    ///
    /// ## Note
    /// This method will block current thread until import is done
    pub fn begin(&mut self) -> Result<HashSet<i32>, ImportError> {
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
            if old.is_none() && new.is_some() {
                self.add_title(new.as_ref().unwrap());
                new = iter_new.next();
            } else if old.is_some() && new.is_none() {
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
                        if self.provider.should_reimport(n.id) {
                            self.add_title(n)
                        }

                        old = iter_old.next();
                        new = iter_new.next();
                    }
                }
            }
        }

        Ok(self.skipped_ids.clone())
    }

    fn add_title(&mut self, anime: &Anime) {
        match self.scheduler.add_title(anime) {
            Err(e) => {
                warn!("adding schedule failed for id:{}: {}", anime.id, e);
                self.skipped_ids.insert(anime.id);
            }
            Ok(()) => {
                debug!("added new schedule for id:{}", anime.id);
                self.skipped_ids.remove(&anime.id);
            }
        }
    }

    fn remove_title(&mut self, anime: &Anime) {
        match self.scheduler.remove_title(anime) {
            Err(e) => {
                warn!("removing schedule failed for id:{}: {}", anime.id, e);
            }
            Ok(()) => {
                debug!("removed old schedule for id:{}", anime.id);
            }
        }
    }
}

// MARK: impl AnidbAnimeProvider

impl<P: AsRef<Path> + Clone + Send> AniDbAnimeProvider<P> {
    /// Creates instance with AniDB anime dumps
    ///
    /// * `old_dump_path` – path to previously imported dump
    /// * `new_dump_path` - path to dump that should be imported
    /// * `reimport_ids` – IDs of anime titles that should be imported again
    pub fn new(old_dump_path: P, new_dump_path: P, reimport_ids: HashSet<i32>) -> Self {
        AniDbAnimeProvider {
            old_dump_path,
            new_dump_path,
            reimport_ids,
        }
    }
}

impl<P: AsRef<Path> + Clone + Send> AnimeProvider for AniDbAnimeProvider<P> {
    type Iterator = Anidb;
    type Error = XmlError;

    fn old_anime_titles(&self) -> Result<Self::Iterator, Self::Error> {
        Anidb::new(self.old_dump_path.as_ref())
    }

    fn new_anime_titles(&self) -> Result<Self::Iterator, Self::Error> {
        Anidb::new(self.new_dump_path.as_ref())
    }

    fn should_reimport(&self, id: i32) -> bool {
        self.reimport_ids.contains(&id)
    }
}

// MARK: impl AnidbImportScheduler

impl AniDbImportScheduler {
    pub fn new(schedules: schedules::Schedules) -> Self {
        AniDbImportScheduler { schedules }
    }
}

impl ImportScheduler for AniDbImportScheduler {
    type Error = QueryError;

    fn add_title(&mut self, anime: &Anime) -> Result<(), Self::Error> {
        let schedule = SourceSchedule::new(anime.id, ExternalSource::AniDB);
        self.schedules.create_from_source(&schedule)
    }

    fn remove_title(&mut self, anime: &Anime) -> Result<(), Self::Error> {
        let schedule = SourceSchedule::new(anime.id, ExternalSource::AniDB);
        self.schedules.delete_from_source(&schedule)
    }
}

// MARK: impl ImportError

impl From<XmlError> for ImportError {
    fn from(e: XmlError) -> Self {
        ImportError::DataSourceFailed(format!("data source error: {}", e))
    }
}

impl<E: std::fmt::Debug> From<BlockingError<E>> for ImportError {
    fn from(e: BlockingError<E>) -> Self {
        match e {
            BlockingError::Error(e) => ImportError::DataSourceFailed(format!("{:?}", e)),
            BlockingError::Cancelled => ImportError::InternalError("blocking cancelled".to_string()),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::test_utils::import::*;
    use std::iter::FromIterator as _;

    #[test]
    fn test_import_no_diff() {
        let provider = FakeProvider::new(vec![], gen_anime([1, 2, 3, 4, 5]));
        let scheduler = FakeScheduler::empty();

        let mut importer = AnimeImporter::new(provider.clone(), scheduler.clone());
        importer.begin().unwrap();

        assert!(scheduler.removed.lock().unwrap().is_empty());
        assert_eq!(*scheduler.added.lock().unwrap(), provider.new);
    }

    #[test]
    fn test_import_diff_add() {
        let provider = FakeProvider::new(gen_anime([1, 3, 5]), gen_anime([1, 2, 3, 4, 5]));
        let scheduler = FakeScheduler::empty();

        let mut importer = AnimeImporter::new(provider.clone(), scheduler.clone());
        importer.begin().unwrap();

        assert!(scheduler.removed.lock().unwrap().is_empty());
        assert_eq!(*scheduler.added.lock().unwrap(), gen_anime([2, 4]));
    }

    #[test]
    fn test_import_diff_remove() {
        let provider = FakeProvider::new(gen_anime([1, 2, 3, 4, 5]), gen_anime([1, 3, 5]));
        let scheduler = FakeScheduler::empty();

        let mut importer = AnimeImporter::new(provider.clone(), scheduler.clone());
        importer.begin().unwrap();

        assert!(scheduler.added.lock().unwrap().is_empty());
        assert_eq!(*scheduler.removed.lock().unwrap(), gen_anime([2, 4]));
    }

    #[test]
    fn test_import_diff_add_remove() {
        let provider = FakeProvider::new(gen_anime([1, 3, 5]), gen_anime([2, 4, 5, 7]));
        let scheduler = FakeScheduler::empty();

        let mut importer = AnimeImporter::new(provider.clone(), scheduler.clone());
        importer.begin().unwrap();

        assert_eq!(*scheduler.removed.lock().unwrap(), gen_anime([1, 3]));
        assert_eq!(*scheduler.added.lock().unwrap(), gen_anime([2, 4, 7]));
    }

    #[test]
    fn test_generates_skip_ids() {
        let skip = vec![2, 5];
        let provider = FakeProvider::new(vec![], gen_anime([1, 2, 3, 4, 5]));
        let scheduler = FakeScheduler::empty_skipping(HashSet::from_iter(skip.clone()));

        let mut importer = AnimeImporter::new(provider, scheduler.clone());
        let skipped = importer.begin().unwrap();

        assert_eq!(skipped, HashSet::from_iter(skip));
        assert_eq!(*scheduler.added.lock().unwrap(), gen_anime([1, 3, 4]));
    }

    #[test]
    fn test_does_reimport() {
        let reimport = vec![2, 5];
        let provider = FakeProvider::new_reimporting(
            gen_anime(reimport.clone()),
            gen_anime([1, 2, 3, 4, 5]),
            HashSet::from_iter(reimport.clone()),
        );
        let scheduler = FakeScheduler::empty();

        let mut importer = AnimeImporter::new(provider.clone(), scheduler.clone());
        let skipped = importer.begin().unwrap();

        assert!(skipped.is_empty());
        assert_eq!(*scheduler.added.lock().unwrap(), provider.new);

        Ok(())
    }
}
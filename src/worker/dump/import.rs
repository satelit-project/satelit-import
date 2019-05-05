use futures::prelude::*;
use futures::try_ready;
use log::{trace, warn};
use tokio_threadpool::{blocking, BlockingError};

use std::cmp::Ordering;
use std::path::Path;

use crate::anidb::{AniDb, Anime, XmlError};
use crate::db::{self, entity::NewSchedule, schedules, ConnectionPool, QueryError, Table};
use crate::settings;

/// Creates AniDB dump importer configured with global app settings
///
/// Returned future will block your current task until it's ready
pub fn importer() -> impl Future<Item = (), Error = ImportError> {
    let settings = settings::shared().anidb();
    let provider = AniDbAnimeProvider::new(settings.old_dump_path(), settings.dump_path());

    let pool = db::connection_pool();
    let schedules = schedules::Schedules::new(pool);
    let scheduler = AniDbImportScheduler::new(schedules);

    DumpImporter::new(provider, scheduler)
}

/// Performs anime import from AniDB dump asynchronously
///
/// ## Note
/// This future will block your current task until it's ready due to moving into blocking
/// section of tokio thread pool. If this is not desired behavior, spawn a separate task and use
/// this future there. For more info see docs for `tokio_threadpool::blocking`
pub struct DumpImporter<P, S>(AnimeImporter<P, S>)
where
    P: AnimeProvider<Iterator = AniDb, Error = XmlError>,
    S: ImportScheduler<Error = QueryError>;

impl<P, S> DumpImporter<P, S>
where
    P: AnimeProvider<Iterator = AniDb, Error = XmlError>,
    S: ImportScheduler<Error = QueryError>,
{
    pub fn new(provider: P, scheduler: S) -> Self {
        let importer = AnimeImporter::new(provider, scheduler);
        DumpImporter(importer)
    }
}

impl<P, S> Future for DumpImporter<P, S>
where
    P: AnimeProvider<Iterator = AniDb, Error = XmlError>,
    S: ImportScheduler<Error = QueryError>,
{
    type Item = ();
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
}

impl<P, S> AnimeImporter<P, S>
where
    P: AnimeProvider<Error = XmlError>,
    S: ImportScheduler,
{
    /// Creates new instance with provided parameters
    pub fn new(provider: P, scheduler: S) -> Self {
        AnimeImporter {
            provider,
            scheduler,
        }
    }

    /// Starts importing anime titles by using id's to determine diff that should be processes.
    /// This method assumes that id's sorted in ascending order and no duplicates exists
    ///
    /// ## Note
    /// This method will block current thread until import is done
    pub fn begin(&mut self) -> Result<(), ImportError> {
        let mut iter_old = self.provider.old_anime_titles()?;
        let mut iter_new = self.provider.new_anime_titles()?;

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
                        old = iter_old.next();
                        new = iter_new.next();
                    }
                }
            }
        }

        Ok(())
    }

    fn add_title(&mut self, anime: &Anime) {
        match self.scheduler.add_title(anime) {
            Err(e) => warn!("adding schedule failed for id:{}: {}", anime.id, e),
            Ok(()) => trace!("added new schedule for id:{}", anime.id),
        }
    }

    fn remove_title(&mut self, anime: &Anime) {
        match self.scheduler.remove_title(anime) {
            Err(e) => warn!("removing schedule failed for id:{}: {}", anime.id, e),
            Ok(()) => trace!("removed old schedule for id:{}", anime.id),
        }
    }
}

/// Represents an error that may occur during anime import
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

/// Data source for anime records that should be imported
pub trait AnimeProvider: Clone + Send {
    /// Iterator for anime entities that should be processes. Entities should be sorted by id
    /// and returned in ascended order
    type Iterator: Iterator<Item = Anime>;

    /// If provider can't return an iterator this error type will be used to determine a cause of
    /// the error
    type Error: std::error::Error;

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

use log::{warn, trace};

use crate::anidb::{Anime, AniDb, XmlError};
use crate::db::{schedules, Table, ConnectionPool, QueryError, entity::NewSchedule};

use std::path::Path;
use std::cmp::Ordering;

/// Performs anime import with titles from `provider` and schedules changes in `scheduler`.
pub struct AnimeImporter<P, S>
    where P: AnimeProvider, S: ImportScheduler
{
    provider: P,
    scheduler: S,
}

impl<P, S> AnimeImporter<P, S>
    where P: AnimeProvider, S: ImportScheduler
{
    pub fn new(provider: P, scheduler: S) -> Self {
        AnimeImporter { provider, scheduler }
    }

    /// Starts anime import. This method will block current thread until import is done
    pub fn begin(mut self) -> Result<(), ImportError> {
        let mut iter_old = self.provider.old_anime_titles()?;
        let mut iter_new = self.provider.new_anime_titles()?;

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
                    },
                    Ordering::Greater => {
                        self.add_title(n);
                        new = iter_new.next();
                    },
                    Ordering::Equal => {
                        old = iter_old.next();
                        new = iter_new.next();
                    },
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
    DataSourceFailed(String)
}

impl<T: std::error::Error> From<T> for ImportError {
    fn from(e: T) -> Self {
        ImportError::DataSourceFailed(format!("data source error: {}", e))
    }
}

/// Data source for anime records that should be imported
pub trait AnimeProvider {
    /// Iterator for anime entities that should be processes. Entities should be sorted by id
    /// and returned in ascended order
    type Iterator: Iterator<Item = Anime>;

    /// If provider can't return an iterator this error type will be used to determine a cause of
    /// the error
    type Error: std::error::Error;

    /// Returns iterator for previously imported anime titles. It used to build a diff of changed
    /// anime entities and process them only. The iterator may return `None` at any time. In that
    /// case all titles returned from `new_anime_titles` iterator would be imported as new titles
    fn old_anime_titles(&self) -> Result<Self::Iterator, Self::Error>;

    /// Returns iterator for anime titles that should be imported. If non-empty iterator is returned
    /// from `old_anime_titles` then only diff will be processes
    fn new_anime_titles(&self) -> Result<Self::Iterator, Self::Error>;
}

/// Data source for anime entities from AniDB dumps
pub struct AniDbAnimeProvider<P: AsRef<Path>> {
    old_dump_path: P,
    new_dump_path: P,
}

impl<P: AsRef<Path>> AniDbAnimeProvider<P> {
    /// Creates instance with AniDB anime dumps
    ///
    /// * `old_dump_path` – path to previously imported dump
    /// * `new_dump_path` - path to dump that should be imported
    pub fn new(old_dump_path: P, new_dump_path: P) -> Self {
        AniDbAnimeProvider { old_dump_path, new_dump_path }
    }
}

impl<P: AsRef<Path>> AnimeProvider for AniDbAnimeProvider<P> {
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
pub trait ImportScheduler {
    type Error: std::error::Error;

    /// Adds new anime title to anime storage
    fn add_title(&mut self, anime: &Anime) -> Result<(), Self::Error>;

    /// Removes anime title from anime storage
    fn remove_title(&mut self, anime: &Anime) -> Result<(), Self::Error>;
}

/// Schedules for anime titles from AniDB dump
pub struct AniDbImportScheduler<P: ConnectionPool> {
    schedules: schedules::Schedules<P>,
}

impl<P: ConnectionPool> ImportScheduler for AniDbImportScheduler<P> {
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

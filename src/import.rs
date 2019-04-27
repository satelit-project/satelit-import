use crate::anidb::{Anime, AniDb, XmlError};
use std::path::Path;
use std::cmp::Ordering;

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

    pub fn begin(mut self) -> Result<(), ImportError> {
        let mut iter_old = self.provider.old_anime_titles()?;
        let mut iter_new = self.provider.new_anime_titles()?;

        let mut old = iter_old.next();
        let mut new = iter_new.next();

        while old.is_some() || new.is_some() {
            if old.is_none() && new.is_some() {
                self.scheduler.added_title(new.as_ref().unwrap());
                new = iter_new.next();
            } else if old.is_some() && new.is_none() {
                self.scheduler.removed_title(old.as_ref().unwrap());
                old = iter_old.next();
            } else {
                let o = old.as_ref().unwrap();
                let n = new.as_ref().unwrap();

                match o.id.cmp(&n.id) {
                    Ordering::Less => {
                        self.scheduler.removed_title(o);
                        old = iter_old.next();
                    },
                    Ordering::Greater => {
                        self.scheduler.added_title(n);
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
}

pub enum ImportError {
    DataSourceFailed(String)
}

impl<T: std::error::Error> From<T> for ImportError {
    fn from(e: T) -> Self {
        ImportError::DataSourceFailed(format!("data source error: {}", e))
    }
}

pub trait AnimeProvider {
    type Iterator: Iterator<Item = Anime>;
    type Error: std::error::Error;

    fn old_anime_titles(&self) -> Result<Self::Iterator, Self::Error>;
    fn new_anime_titles(&self) -> Result<Self::Iterator, Self::Error>;
}

pub struct AniDbAnimeProvider<'p> {
    old_dump_path: &'p Path,
    new_dump_path: &'p Path,
}

impl<'p> AniDbAnimeProvider<'p> {
    pub fn new(old_dump_path: &'p Path, new_dump_path: &'p Path) -> Self {
        AniDbAnimeProvider { old_dump_path, new_dump_path }
    }
}

impl AnimeProvider for AniDbAnimeProvider<'_> {
    type Iterator = AniDb;
    type Error = XmlError;

    fn old_anime_titles(&self) -> Result<Self::Iterator, Self::Error> {
        AniDb::new(self.old_dump_path)
    }

    fn new_anime_titles(&self) -> Result<Self::Iterator, Self::Error> {
        AniDb::new(self.new_dump_path)
    }
}

pub trait ImportScheduler {
    fn added_title(&mut self, anime: &Anime);
    fn removed_title(&mut self, anime: &Anime);
}

pub struct SqliteImportScheduler {

}

impl ImportScheduler for SqliteImportScheduler {
    fn added_title(&mut self, anime: &Anime) {
        unimplemented!()
    }

    fn removed_title(&mut self, anime: &Anime) {
        unimplemented!()
    }
}

#![cfg(test)]

use futures::prelude::*;

pub(crate) mod import {
    use crate::anidb::{importer::import::*, parser::*};
    use std::{
        collections::HashSet,
        sync::{Arc, Mutex},
        vec::IntoIter,
    };

    pub(crate) fn gen_anime<R: AsRef<[i32]>>(ids: R) -> Vec<Anime> {
        let mut anime = vec![];

        for id in ids.as_ref() {
            anime.push(Anime::new(*id, (*id).to_string(), vec![]));
        }

        anime
    }

    #[derive(Clone)]
    pub struct FakeProvider {
        pub old: Vec<Anime>,
        pub new: Vec<Anime>,
        pub reimport_ids: HashSet<i32>,
    }

    impl FakeProvider {
        pub fn new(old: Vec<Anime>, new: Vec<Anime>) -> Self {
            Self {
                old,
                new,
                reimport_ids: HashSet::new(),
            }
        }

        pub fn new_reimporting(
            old: Vec<Anime>,
            new: Vec<Anime>,
            reimport_ids: HashSet<i32>,
        ) -> Self {
            Self {
                old,
                new,
                reimport_ids,
            }
        }
    }

    impl AnimeProvider for FakeProvider {
        type Iterator = IntoIter<Anime>;
        type Error = XmlError;

        fn old_anime_titles(&self) -> Result<Self::Iterator, Self::Error> {
            Ok(self.old.clone().into_iter())
        }

        fn new_anime_titles(&self) -> Result<Self::Iterator, Self::Error> {
            Ok(self.new.clone().into_iter())
        }

        fn should_reimport(&self, id: i32) -> bool {
            self.reimport_ids.contains(&id)
        }
    }

    #[derive(Clone)]
    pub struct FakeScheduler {
        pub added: Arc<Mutex<Vec<Anime>>>,
        pub removed: Arc<Mutex<Vec<Anime>>>,
        pub skip_add: Arc<HashSet<i32>>,
    }

    impl FakeScheduler {
        pub fn new(added: Vec<Anime>, removed: Vec<Anime>) -> Self {
            FakeScheduler {
                added: Arc::new(Mutex::new(added)),
                removed: Arc::new(Mutex::new(removed)),
                skip_add: Arc::new(HashSet::new()),
            }
        }

        pub fn empty() -> Self {
            Self::new(vec![], vec![])
        }

        pub fn new_skipping(
            added: Vec<Anime>,
            removed: Vec<Anime>,
            skip_add: HashSet<i32>,
        ) -> Self {
            FakeScheduler {
                added: Arc::new(Mutex::new(added)),
                removed: Arc::new(Mutex::new(removed)),
                skip_add: Arc::new(skip_add),
            }
        }

        pub fn empty_skipping(skip_add: HashSet<i32>) -> Self {
            Self::new_skipping(vec![], vec![], skip_add)
        }
    }

    impl ImportScheduler for FakeScheduler {
        type Error = ImportError;

        fn add_title(&mut self, anime: &Anime) -> Result<(), Self::Error> {
            if self.skip_add.contains(&anime.id) {
                return Err(ImportError::InternalError("skipping".to_owned()));
            }

            let mut added = self.added.lock().unwrap();
            added.push(anime.clone());
            Ok(())
        }

        fn remove_title(&mut self, anime: &Anime) -> Result<(), Self::Error> {
            let mut removed = self.removed.lock().unwrap();
            removed.push(anime.clone());
            Ok(())
        }
    }
}

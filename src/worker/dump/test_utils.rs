#![cfg(test)]

pub(crate) mod import {
    use crate::anidb::*;
    use crate::worker::dump::import::*;
    use std::collections::HashSet;
    use std::sync::{Arc, Mutex};
    use std::vec::IntoIter;

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
            let skip_add = self.skip_add.as_ref();
            let should_skip = self.skip_add.contains(&anime.id);

            if should_skip {
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

pub(crate) mod download {
    use crate::worker::dump::download::*;
    use futures::stream::{self, IterOk};

    #[derive(Clone)]
    pub struct Chunk(pub [u8; 2]);

    impl AsRef<[u8]> for Chunk {
        fn as_ref(&self) -> &[u8] {
            &self.0
        }
    }

    pub struct FakeDownloader {
        pub content: Vec<Chunk>,
    }

    impl FileDownload for FakeDownloader {
        type Chunk = Chunk;
        type Bytes = IterOk<std::vec::IntoIter<Chunk>, DownloadError>;

        fn download(&self, _url: &str) -> Self::Bytes {
            stream::iter_ok(self.content.clone().into_iter())
        }
    }
}

pub(crate) mod extract {
    use std::ops::Deref;

    #[derive(Debug)]
    pub struct StringMut(String);

    pub trait ToMut {
        fn to_mut(&self) -> StringMut;
    }

    // String helpers implementations

    impl ToMut for String {
        fn to_mut(&self) -> StringMut {
            StringMut(self.clone())
        }
    }

    impl ToMut for str {
        fn to_mut(&self) -> StringMut {
            StringMut(self.to_owned())
        }
    }

    impl AsMut<[u8]> for StringMut {
        fn as_mut(&mut self) -> &mut [u8] {
            unsafe { self.0.as_bytes_mut() }
        }
    }

    impl From<Vec<u8>> for StringMut {
        fn from(bytes: Vec<u8>) -> Self {
            unsafe { StringMut(String::from_utf8_unchecked(bytes)) }
        }
    }

    impl Deref for StringMut {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::cmp::PartialEq for StringMut {
        fn eq(&self, other: &StringMut) -> bool {
            self.0 == other.0
        }
    }

    impl std::cmp::PartialEq<String> for StringMut {
        fn eq(&self, other: &String) -> bool {
            &self.0 == other
        }
    }

    impl std::cmp::PartialEq<&str> for StringMut {
        fn eq(&self, other: &&str) -> bool {
            &self.0 == other
        }
    }
}

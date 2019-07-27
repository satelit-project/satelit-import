pub mod copy;
pub mod download;
pub mod extract;
pub mod import;
pub mod respond;

mod test_utils;

pub use copy::copier;
pub use download::downloader;
pub use extract::extractor;
pub use import::importer;
pub use respond::responder;

use futures::prelude::*;
use futures::try_ready;
use log::{error, info, trace};

use std::collections::HashSet;
use std::error::Error;
use std::iter::FromIterator;
use std::path::Path;

use crate::db::ConnectionPool;
use crate::proto::scheduler::ImportIntent;
use crate::worker::Worker;

/// Creates new worker for importing AniDB dump configured with global app settings
pub fn worker(intent: ImportIntent) -> impl Worker {
    let settings = crate::settings::shared();
    let dump_url = intent.dump_url.clone();

    DumpImportWorker::new(
        intent,
        dump_url,
        settings.import().download_path(),
        settings.import().dump_path(),
        settings.import().dump_backup_path(),
        crate::db::connection_pool(),
    )
}

/// A worker that processes `ImportIntent`
///
/// It will download latest AniDB dump, make a backup of the previous dump, import new anime titles
/// and send import result to specified by `ImportIntent` destination
pub struct DumpImportWorker<U, S, P> {
    /// Import intent that describes the task
    intent: ImportIntent,

    /// URL of AniDB dump to download
    dump_url: U,

    /// Where to download AniDB dump archive
    download_path: S,

    /// Path for the extracted AniDB dump
    dump_path: S,

    /// Path where to copy previous AniDB dump
    backup_path: S,

    /// DB connection pool to import new entities
    connection_pool: P,
}

impl<U, S, P> DumpImportWorker<U, S, P> {
    pub fn new(
        intent: ImportIntent,
        dump_url: U,
        download_path: S,
        dump_path: S,
        backup_path: S,
        connection_pool: P,
    ) -> Self {
        Self {
            intent,
            dump_url,
            download_path,
            dump_path,
            backup_path,
            connection_pool,
        }
    }
}

impl<U, S, P> Worker for DumpImportWorker<U, S, P>
where
    U: AsRef<str> + Clone + Send + 'static,
    S: AsRef<Path> + Clone + Send + 'static,
    P: ConnectionPool + 'static,
{
    fn task(self: Box<Self>) -> Box<dyn Future<Item = (), Error = ()> + Send + 'static> {
        let download = download::downloader(self.dump_url, self.download_path.clone());
        let copy = copy::copier(self.dump_path.clone(), self.backup_path.clone());
        let extract = extract::extractor(self.download_path, self.dump_path.clone());
        let import = import::importer(
            self.backup_path,
            self.dump_path,
            HashSet::from_iter(self.intent.reimport_ids.iter().cloned()),
            self.connection_pool,
        );

        let intent = self.intent;
        let fut = DumpImporter::new(download, copy, extract, import)
            .then(move |result| responder(result, intent))
            .then(|result| match result {
                Ok(_) => {
                    info!("Import worker finished successfully");
                    futures::finished(())
                }
                Err(e) => {
                    error!("Import worker failed: {}", e);
                    futures::failed(())
                }
            });

        Box::new(fut)
    }
}

/// Future to download and import AniDB dump
pub struct DumpImporter<D, C, E, I> {
    /// Future to download new dump
    download: D,
    /// Future to backup previous dump
    copy: C,
    /// Future to extract new dump
    extract: E,
    /// Future to import changes in new dump
    import: I,
    /// Task state
    state: DumpImportState,
}

impl<D, C, E, I> DumpImporter<D, C, E, I>
where
    D: Future<Item = (), Error = download::DownloadError> + Send,
    C: Future<Item = (), Error = copy::CopyError> + Send,
    E: Future<Item = (), Error = extract::ExtractError> + Send,
    I: Future<Item = HashSet<i32>, Error = import::ImportError> + Send,
{
    pub fn new(download: D, copy: C, extract: E, import: I) -> Self {
        DumpImporter {
            download,
            copy,
            extract,
            import,
            state: DumpImportState::Downloading,
        }
    }

    fn poll_download(&mut self) -> Result<Async<()>, DumpImportError> {
        match self.download.poll() {
            Err(e) => {
                error!("failed to download anidb dump: {}", e);
                Err(Box::new(e))
            }
            Ok(v) => {
                if let Async::Ready(_) = v {
                    trace!("downloaded anidb dump");
                }

                Ok(v)
            }
        }
    }

    fn poll_copy(&mut self) -> Result<Async<()>, DumpImportError> {
        match self.copy.poll() {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    info!("no dump to backup before import");
                    return Ok(Async::Ready(()));
                }

                error!("failed to backup old dump, will not continue: {}", e);
                Err(Box::new(e))
            }
            Ok(v) => {
                if let Async::Ready(_) = v {
                    trace!("backed up old dump");
                }

                Ok(v)
            }
        }
    }

    fn poll_extract(&mut self) -> Result<Async<()>, DumpImportError> {
        match self.extract.poll() {
            Err(e) => {
                error!("failed to extract anidb dump: {}", e);
                Err(Box::new(e))
            }
            Ok(v) => {
                if let Async::Ready(_) = v {
                    trace!("extracted anidb dump");
                }

                Ok(v)
            }
        }
    }

    fn poll_import(&mut self) -> Result<Async<HashSet<i32>>, DumpImportError> {
        match self.import.poll() {
            Err(e) => {
                error!("failed to import anidb dump: {}", e);
                Err(Box::new(e))
            }
            Ok(v) => {
                if let Async::Ready(_) = v {
                    trace!("imported anidb dump");
                }

                Ok(v)
            }
        }
    }
}

impl<D, C, E, I> Future for DumpImporter<D, C, E, I>
where
    D: Future<Item = (), Error = download::DownloadError> + Send,
    C: Future<Item = (), Error = copy::CopyError> + Send,
    E: Future<Item = (), Error = extract::ExtractError> + Send,
    I: Future<Item = HashSet<i32>, Error = import::ImportError> + Send,
{
    type Item = HashSet<i32>;
    type Error = DumpImportError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        use DumpImportState::*;

        loop {
            match self.state {
                Downloading => {
                    try_ready!(self.poll_download());
                    self.state = Copying;
                }
                Copying => {
                    try_ready!(self.poll_copy());
                    self.state = Extracting;
                }
                Extracting => {
                    try_ready!(self.poll_extract());
                    self.state = Importing;
                }
                Importing => {
                    let skipped = try_ready!(self.poll_import());
                    return Ok(Async::Ready(skipped));
                }
            }
        }
    }
}

/// Represents task state
#[derive(Clone, Copy)]
enum DumpImportState {
    /// Downloading dump from AniDB
    Downloading,
    /// Backing up old dump
    Copying,
    /// Extracting dump archive
    Extracting,
    /// Importing dump entities to DB
    Importing,
}

/// Represents dump import task error as a whole
type DumpImportError = Box<dyn Error + Send + 'static>;

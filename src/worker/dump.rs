pub mod copy;
pub mod download;
pub mod extract;
pub mod import;

pub use copy::copier;
pub use download::downloader;
pub use extract::extractor;
pub use import::importer;

use futures::prelude::*;
use futures::try_ready;
use log::{error, info, trace};

/// Creates new task configured with global app settings
pub fn new_task() -> impl Future<Item = (), Error = ()> {
    let settings = crate::settings::shared();

    let download = download::downloader(
        settings.import().dump_url(),
        settings.import().download_path(),
    );

    let copy = copy::copier(
        settings.import().dump_path(),
        settings.import().old_dump_path(),
    );

    let extract = extract::extractor(
        settings.import().download_path(),
        settings.import().dump_path(),
        settings.import().chunk_size(),
    );

    let import = import::importer(
        settings.import().old_dump_path(),
        settings.import().dump_path(),
        crate::db::connection_pool(),
    );

    DumpImportTask {
        download,
        copy,
        extract,
        import,
        state: DumpImportState::Downloading,
    }
}

/// Task to download and import AniDB dump
pub struct DumpImportTask<D, C, E, I>
where
    D: Future<Item = (), Error = download::DownloadError>,
    C: Future<Item = (), Error = copy::CopyError>,
    E: Future<Item = (), Error = extract::ExtractError>,
    I: Future<Item = (), Error = import::ImportError>,
{
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

impl<D, C, E, I> DumpImportTask<D, C, E, I>
where
    D: Future<Item = (), Error = download::DownloadError>,
    C: Future<Item = (), Error = copy::CopyError>,
    E: Future<Item = (), Error = extract::ExtractError>,
    I: Future<Item = (), Error = import::ImportError>,
{
    fn poll_download(&mut self) -> Result<Async<()>, ()> {
        match self.download.poll() {
            Err(e) => {
                error!("failed to download anidb dump: {}", e);
                Err(())
            }
            Ok(v) => {
                if let Async::Ready(_) = v {
                    trace!("downloaded anidb dump");
                }

                Ok(v)
            }
        }
    }

    fn poll_copy(&mut self) -> Result<Async<()>, ()> {
        match self.copy.poll() {
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    info!("no dump to backup before import");
                    return Ok(Async::Ready(()));
                }

                error!("failed to backup old dump, will not continue: {}", e);
                Err(())
            }
            Ok(v) => {
                if let Async::Ready(_) = v {
                    trace!("backed up old dump");
                }

                Ok(v)
            }
        }
    }

    fn poll_extract(&mut self) -> Result<Async<()>, ()> {
        match self.extract.poll() {
            Err(e) => {
                error!("failed to extract anidb dump: {}", e);
                Err(())
            }
            Ok(v) => {
                if let Async::Ready(_) = v {
                    trace!("extracted anidb dump");
                }

                Ok(v)
            }
        }
    }

    fn poll_import(&mut self) -> Result<Async<()>, ()> {
        match self.import.poll() {
            Err(e) => {
                error!("failed to import anidb dump: {}", e);
                Err(())
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

impl<D, C, E, I> Future for DumpImportTask<D, C, E, I>
where
    D: Future<Item = (), Error = download::DownloadError>,
    C: Future<Item = (), Error = copy::CopyError>,
    E: Future<Item = (), Error = extract::ExtractError>,
    I: Future<Item = (), Error = import::ImportError>,
{
    type Item = ();
    type Error = ();

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
                    try_ready!(self.poll_import());
                    return Ok(Async::Ready(()));
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

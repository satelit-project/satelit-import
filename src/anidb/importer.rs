pub mod download;
pub mod extract;
pub mod import;

mod test_utils;

use tempfile;

use std::path::PathBuf;
use std::{collections::HashSet, error::Error, fmt, iter::FromIterator};

use crate::{
    db::ConnectionPool,
    proto::import::{ImportIntent, ImportIntentResult},
};

/// Represents dump import task error as a whole
#[derive(Debug)]
pub struct ImportError(Box<dyn Error + Send + 'static>);

/// Common file paths used during dump import.
#[derive(Debug)]
struct Paths {
    dir: tempfile::TempDir,
}

/// Imports AniDB database dump.
pub async fn import(
    intent: ImportIntent,
    db_pool: ConnectionPool,
) -> Result<ImportIntentResult, ImportError> {
    let paths = Paths::new()?;
    let has_old_dump = *&intent.old_index_url.len() > 0;

    let download_new = download::download_dump(&intent.new_index_url, paths.store_new());
    if has_old_dump {
        let download_old = download::download_dump(&intent.old_index_url, paths.store_old());
        futures::try_join!(download_old, download_new)?;
    } else {
        download_new.await?;
    }

    let extract_new = extract::extract_gzip(paths.store_new(), paths.extract_new());
    let old_path: Option<PathBuf>;
    if has_old_dump {
        let extract_old = extract::extract_gzip(paths.store_old(), paths.extract_old());
        futures::try_join!(extract_old, extract_new)?;
        old_path = Some(paths.extract_old());
    } else {
        extract_new.await?;
        old_path = None;
    }

    let ImportIntent {
        id, reimport_ids, ..
    } = intent;

    let skipped_ids = import::import(
        old_path,
        paths.extract_new(),
        HashSet::from_iter(reimport_ids.into_iter()),
        db_pool,
    )
    .await?;

    Ok(ImportIntentResult {
        id,
        skipped_ids: skipped_ids.into_iter().collect(),
    })
}

// MARK: impl Paths

impl Paths {
    fn new() -> std::io::Result<Self> {
        Ok(Paths { dir: tempfile::tempdir()? })
    }

    fn store_old(&self) -> PathBuf {
        self.path_with_file("archived.dump.old")
    }

    fn store_new(&self) -> PathBuf {
        self.path_with_file("archived.dump.new")
    }

    fn extract_old(&self) -> PathBuf {
        self.path_with_file("dump.old")
    }

    fn extract_new(&self) -> PathBuf {
        self.path_with_file("dump.new")
    }

    fn path_with_file(&self, name: &str) -> PathBuf {
        let mut path = self.dir.path().to_path_buf();
        path.push(name);
        path
    }
}

// MARK: impl ImportError

impl From<download::DownloadError> for ImportError {
    fn from(err: download::DownloadError) -> Self {
        ImportError(Box::new(err))
    }
}

impl From<extract::ExtractError> for ImportError {
    fn from(err: extract::ExtractError) -> Self {
        ImportError(Box::new(err))
    }
}

impl From<import::ImportError> for ImportError {
    fn from(err: import::ImportError) -> Self {
        ImportError(Box::new(err))
    }
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ImportError {}

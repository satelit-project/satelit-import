pub mod extract;
pub mod import;

mod test_utils;

use tempfile;
use tracing::{debug_span, info};
use tracing_futures::Instrument;

use std::{collections::HashSet, error::Error, fmt, iter::FromIterator, path::PathBuf};

use crate::{
    db::ConnectionPool,
    proto::import::{ImportIntent, ImportIntentResult},
    store::{IndexStore, StoreError},
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
    store: &IndexStore,
) -> Result<ImportIntentResult, ImportError> {
    let paths = Paths::new()?;
    let with_diff = intent.has_old_dump();

    download(&intent, &paths, store).in_current_span().await?;
    extract(&intent, &paths).in_current_span().await?;

    let ImportIntent {
        id, reimport_ids, ..
    } = intent;

    info!("starting index import");
    let skipped_ids = import::import(
        if with_diff {
            Some(paths.extract_old())
        } else {
            None
        },
        paths.extract_new(),
        HashSet::from_iter(reimport_ids.into_iter()),
        db_pool,
    )
    .in_current_span()
    .await?;

    Ok(ImportIntentResult {
        id,
        skipped_ids: skipped_ids.into_iter().collect(),
    })
}

async fn download(
    intent: &ImportIntent,
    paths: &Paths,
    store: &IndexStore,
) -> Result<(), ImportError> {
    let download_new = store
        .get(&intent.new_index_url, paths.store_new())
        .instrument(debug_span!("get::new"));

    if intent.has_old_dump() {
        let download_old = store
            .get(&intent.old_index_url, paths.store_old())
            .instrument(debug_span!("get::old"));

        info!("downloading old and new indexes");
        futures::try_join!(download_old, download_new)?;
    } else {
        info!("downloading new index");
        download_new.await?;
    }

    Ok(())
}

async fn extract(intent: &ImportIntent, paths: &Paths) -> Result<(), ImportError> {
    let extract_new = extract::extract_gzip(paths.store_new(), paths.extract_new())
        .instrument(debug_span!("gzip::new"));

    if intent.has_old_dump() {
        let extract_old = extract::extract_gzip(paths.store_old(), paths.extract_old())
            .instrument(debug_span!("gzip::old"));

        info!("extracting old and new indexes");
        futures::try_join!(extract_old, extract_new)?;
    } else {
        info!("extracting new index");
        extract_new.await?;
    };

    Ok(())
}

// MARK: impl Paths

impl Paths {
    fn new() -> std::io::Result<Self> {
        Ok(Paths {
            dir: tempfile::tempdir()?,
        })
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

impl From<StoreError> for ImportError {
    fn from(err: StoreError) -> Self {
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

// MARK: helpers

impl ImportIntent {
    fn has_old_dump(&self) -> bool {
        !self.old_index_url.is_empty()
    }
}

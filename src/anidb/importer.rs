pub mod download;
pub mod extract;
pub mod import;

mod test_utils;

use futures::prelude::*;

use std::{collections::HashSet, error::Error, iter::FromIterator};

use crate::{
    db::ConnectionPool,
    proto::import::{ImportIntent, ImportIntentResult},
    settings,
};

/// Represents dump import task error as a whole
type ImportError = Box<dyn Error + Send + 'static>;

/// Imports AniDB database dump.
pub async fn import(
    intent: ImportIntent,
    settings: settings::Import,
    db_pool: ConnectionPool,
) -> Result<ImportIntentResult, ImportError> {

    let download_old = download::download_dump(&intent.old_index_url, &settings.old_download_path);
    let download_new = download::download_dump(&intent.new_index_url, &settings.new_download_path);
    futures::try_join!(download_old, download_new)?;

    let extract_old = extract::extract(&settings.old_download_path, &settings.old_extract_path);
    let extract_new = extract::extract(&settings.new_download_path, &settings.new_extract_path);
    futures::try_join!(extract_old, extract_new)?;

    let ImportIntent { id, reimport_ids, .. } = intent;
    let settings::Import { old_extract_path, new_extract_path, .. } = settings;
    let skipped_ids = import::import(old_extract_path, new_extract_path, HashSet::from_iter(reimport_ids.into_iter()), db_pool).await?;

    Ok(ImportIntentResult { id, skipped_ids: skipped_ids.into_iter().collect() })
}

// MARK: impl ImportError

impl From<download::DownloadError> for ImportError {
    fn from(err: download::DownloadError) -> Self {
        Box::new(err)
    }
}

impl From<extract::ExtractError> for ImportError {
    fn from(err: extract::ExtractError) -> Self {
        Box::new(err)
    }
}

impl From<import::ImportError> for ImportError {
    fn from(err: import::ImportError) -> Self {
        Box::new(err)
    }
}

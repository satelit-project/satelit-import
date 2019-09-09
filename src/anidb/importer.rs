pub mod download;
pub mod extract;
pub mod import;

mod test_utils;

use futures::prelude::*;

use std::collections::HashSet;
use std::error::Error;
use std::iter::FromIterator;

use crate::db::{self, ConnectionPool};
use crate::proto::import::{ImportIntent, ImportIntentResult};
use crate::settings;

/// Represents dump import task error as a whole
type ImportError = Box<dyn Error + Send + 'static>;

/// Creates new future for importing AniDB dump configured with global app settings
pub fn importer(
    intent: ImportIntent,
) -> impl Future<Item = ImportIntentResult, Error = ImportError> {
    let settings = crate::settings::shared().import();
    let pool = db::connection_pool();

    let ImportIntent {
        id,
        new_index_url,
        old_index_url,
        source: _,
        reimport_ids,
    } = intent;

    download(old_index_url, new_index_url, settings)
        .and_then(move |_| extract(settings))
        .and_then(move |_| import(reimport_ids, pool, settings))
        .and_then(move |skipped_ids| {
            let result = ImportIntentResult { id, skipped_ids };
            Ok(result)
        })
}

fn download<U>(
    old_index_url: U,
    new_index_url: U,
    settings: &'static settings::Import,
) -> impl Future<Item = (), Error = ImportError>
where
    U: AsRef<str> + Send,
{
    let old_download_path = settings.old_download_path();
    let new_download_path = settings.new_download_path();

    download::downloader(old_index_url, old_download_path)
        .join(download::downloader(new_index_url, new_download_path))
        .map_err(|e| Box::new(e) as ImportError)
        .map(|_| ())
}

fn extract(settings: &'static settings::Import) -> impl Future<Item = (), Error = ImportError> {
    let old_download_path = settings.old_download_path();
    let old_extract_path = settings.old_extract_path();
    let new_download_path = settings.new_download_path();
    let new_extract_path = settings.new_extract_path();

    extract::extractor(old_download_path, old_extract_path)
        .join(extract::extractor(new_download_path, new_extract_path))
        .map_err(|e| Box::new(e) as ImportError)
        .map(|_| ())
}

fn import(
    reimport_ids: Vec<i32>,
    pool: ConnectionPool,
    settings: &'static settings::Import,
) -> impl Future<Item = Vec<i32>, Error = ImportError> {
    let old_extract_path = settings.old_extract_path();
    let new_extract_path = settings.new_extract_path();
    let reimport_ids = HashSet::from_iter(reimport_ids.into_iter());

    import::importer(old_extract_path, new_extract_path, reimport_ids, pool)
        .map_err(|e| Box::new(e) as ImportError)
        .map(|reimport| reimport.into_iter().collect())
}

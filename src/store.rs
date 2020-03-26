use bytes::BytesMut;
use prost::Message;
use s3::{self, bucket, credentials, region};

use crate::{
    proto::data::{Anime, Source},
    settings,
};

/// An error which may happen during store operations.
pub type StoreError = s3::error::S3Error;

/// Represents a remote anime storage.
#[derive(Debug)]
pub struct AnimeStore {
    bucket: bucket::Bucket,
}

// MARK: impl AnimeStore

impl AnimeStore {
    /// Creates and returns new store with given configuration.
    pub fn new(config: &settings::Storage) -> Result<Self, StoreError> {
        let region = region::Region::Custom {
            region: config.region().to_owned(),
            endpoint: config.host().to_owned(),
        };
        let creds = credentials::Credentials::new(
            Some(config.key().to_owned()),
            Some(config.secret().to_owned()),
            None,
            None,
        );
        let bucket = bucket::Bucket::new("anime", region, creds)?;

        Ok(AnimeStore { bucket })
    }

    /// Saves and uploads anime object to a remote store.
    pub async fn upload(&self, anime: &Anime, source: Source) -> Result<(), StoreError> {
        let mut buf = BytesMut::with_capacity(anime.encoded_len());
        anime
            .encode(&mut buf)
            .expect("not enough space for encoding");

        let path = storage_path(anime, source);
        self.bucket
            .put_object(&path, buf.as_ref(), "application/octet-stream")
            .await?;
        Ok(())
    }
}

// MARK: helpers

fn storage_path(anime: &Anime, source: Source) -> String {
    let prefix = match source {
        Source::Anidb => "anidb",
        Source::Unknown => "unknown",
    };

    let id = match source {
        Source::Anidb => anime
            .source
            .as_ref()
            .and_then(|s| s.anidb_ids.first())
            .map(|&s| s)
            .unwrap_or(0),
        Source::Unknown => 0,
    };

    format!("{}/scraped/{}.bin", prefix, id)
}

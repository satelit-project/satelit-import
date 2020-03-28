use bytes::{buf::ext::BufMutExt, BytesMut};
use prost::Message;
use s3::{self, bucket, credentials, region};
use tokio::{fs::File, io::AsyncWriteExt};

use std::{error, fmt, io, path};

use crate::{
    proto::data::{Anime, Source},
    settings,
};

/// An error which may happen during store operations.
#[derive(Debug)]
pub struct StoreError(s3::error::S3Error);

/// Represents a remote anime storage.
#[derive(Debug, Clone)]
pub struct AnimeStore {
    bucket: bucket::Bucket,
}

#[derive(Debug, Clone)]
pub struct IndexStore {
    bucket: bucket::Bucket,
}

// MARK: impl AnimeStore

impl AnimeStore {
    /// Creates and returns new store with given configuration.
    pub fn new(cfg: &settings::Storage) -> Result<Self, StoreError> {
        let bucket = get_bucket(cfg)?;
        Ok(AnimeStore { bucket })
    }

    /// Saves and uploads anime object to a remote store.
    pub async fn upload(&self, anime: &Anime, source: Source) -> Result<String, StoreError> {
        let mut buf = BytesMut::with_capacity(anime.encoded_len());
        anime
            .encode(&mut buf)
            .expect("not enough space for encoding");

        let path = storage_path(anime, source);
        self.bucket
            .put_object(&path, buf.as_ref(), "application/octet-stream")
            .await?;
        Ok(path)
    }
}

// MARK: impl IndexStore

impl IndexStore {
    /// Creates and returns new store with given configuration.
    pub fn new(cfg: &settings::Storage) -> Result<Self, StoreError> {
        let bucket = get_bucket(cfg)?;
        Ok(IndexStore { bucket })
    }

    /// Downloads anime index and saves it at given path.
    pub async fn get<P>(&self, path: &str, out: P) -> Result<(), StoreError>
    where
        P: AsRef<path::Path>,
    {
        let buf = BytesMut::default();
        let mut writer = buf.writer();

        self.bucket.get_object_stream(path, &mut writer).await?;

        let mut file = File::create(out.as_ref()).await?;
        let mut buf = writer.into_inner();
        file.write_buf(&mut buf).await?;

        Ok(())
    }
}

// MARK: impl StoreError

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for StoreError {}

impl From<s3::error::S3Error> for StoreError {
    fn from(err: s3::error::S3Error) -> Self {
        StoreError(err)
    }
}

impl From<io::Error> for StoreError {
    fn from(err: io::Error) -> Self {
        StoreError(err.into())
    }
}

// MARK: helpers

fn get_bucket(cfg: &settings::Storage) -> s3::error::Result<bucket::Bucket> {
    let host = if cfg.host().starts_with("localhost") || cfg.host().starts_with("127.0.0.1") {
        format!("http://{}", cfg.host())
    } else {
        cfg.host().to_owned()
    };

    let region = region::Region::Custom {
        region: cfg.region().to_owned(),
        endpoint: host,
    };
    let creds = credentials::Credentials::new(
        Some(cfg.key().to_owned()),
        Some(cfg.secret().to_owned()),
        None,
        None,
    );

    bucket::Bucket::new(cfg.bucket(), region, creds)
}

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
            .copied()
            .unwrap_or(0),
        Source::Unknown => 0,
    };

    format!("{}/scraped/{}.bin", prefix, id)
}

// MARK: tests

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::data::anime;

    #[test]
    fn filenames() {
        let anime = Anime {
            source: Some(anime::Source {
                anidb_ids: vec![1],
                mal_ids: vec![],
                ann_ids: vec![],
            }),
            r#type: 0,
            title: "Bleach".to_owned(),
            poster_url: "".to_owned(),
            episodes_count: 0,
            episodes: vec![],
            start_date: 0,
            end_date: 0,
            tags: vec![],
            rating: 0.0,
            description: "".to_owned(),
            src_created_at: 0,
            src_updated_at: 0,
        };

        let path = storage_path(&anime, Source::Anidb);
        assert_eq!(path, "anidb/scraped/1.bin");
    }
}

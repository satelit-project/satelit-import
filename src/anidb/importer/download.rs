use reqwest::{Client, ClientBuilder};
use tokio::{fs::File, prelude::*};

use std::{
    fmt::{self, Debug, Display},
    path::Path,
    time::Duration,
};

/// Downloads AniDB database dump.
///
/// # Arguments
///
/// * dump_url: URL of the database dump to download.
/// * dest_path: database dump path where to save it.
pub async fn download_dump<U, P>(dump_url: U, dest_path: P) -> Result<(), DownloadError>
where
    U: AsRef<str> + Send,
    P: AsRef<Path> + Send,
{
    let client = ClientBuilder::new()
        .gzip(true)
        .connect_timeout(Duration::new(60, 0)) // TODO: from config
        .build()?;

    let downloader = DumpDownloader::new(client, dump_url, dest_path);
    downloader.download().await
}

/// AniDB dump downloader.
#[derive(Debug)]
pub struct DumpDownloader<U, P> {
    /// Files downloading client.
    client: Client,

    /// URL where dump is hosted.
    dump_url: U,

    /// Path where to save dump.
    dest_path: P,
}

/// Represents an error that may happen during dump download.
pub enum DownloadError {
    /// Request or download has failed.
    Net(reqwest::Error),

    /// Failed to write dump on disk.
    Fs(tokio::io::Error),
}

// MARK: impl DumpDownloader

impl<U, P> DumpDownloader<U, P>
where
    U: AsRef<str>,
    P: AsRef<Path>,
{
    /// Creates new instance
    pub fn new(client: Client, dump_url: U, dest_path: P) -> Self {
        DumpDownloader {
            client,
            dump_url,
            dest_path,
        }
    }

    /// Asynchronously downloads dump at `dump_url` and saves it on disk at `dest_path`
    pub async fn download(&self) -> Result<(), DownloadError> {
        let file = File::create(self.dest_path.as_ref()).await?;
        let chunks = self.client.get(self.dump_url.as_ref()).send().await?;

        while let Some(chunk) = chunks.chunk().await? {
            file.write_all(&chunk).await?;
        }

        file.sync_data().await?;
        Ok(())
    }
}

// MARK: impl DownloadError

impl std::error::Error for DownloadError {}

impl Debug for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DownloadError::*;

        match self {
            Net(ref e) => <reqwest::Error as Debug>::fmt(e, f),
            Fs(ref e) => <std::io::Error as Debug>::fmt(e, f),
        }
    }
}

impl Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use DownloadError::*;

        match self {
            Net(ref e) => <reqwest::Error as Display>::fmt(e, f),
            Fs(ref e) => <std::io::Error as Display>::fmt(e, f),
        }
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        DownloadError::Net(e)
    }
}

impl From<tokio::io::Error> for DownloadError {
    fn from(e: tokio::io::Error) -> Self {
        DownloadError::Fs(e)
    }
}

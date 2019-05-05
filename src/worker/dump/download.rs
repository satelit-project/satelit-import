use futures::prelude::*;
use reqwest::r#async::{Client, ClientBuilder};
use reqwest::header::{HeaderMap, HeaderValue, self};
use reqwest::Method;
use tokio::fs::File;

use std::time::Duration;
use std::fmt::{Debug, Display, self};
use std::path::Path;

use crate::settings;

/// Creates downloader with configuration from global app settings
pub fn downloader() -> Result<DumpDownloader<impl FileDownload, &'static str, &'static str>, DownloadError> {
    const USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_14_4) AppleWebKit/605.1.15 \
(KHTML, like Gecko) Version/12.1 Safari/605.1.15";

    let settings = settings::Settings::shared().anidb();
    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static(USER_AGENT));

    let client = ClientBuilder::new()
        .default_headers(headers)
        .gzip(true)
        .connect_timeout(Duration::new(60, 0))
        .build()?;

    Ok(DumpDownloader {
        downloader: client,
        dump_url: settings.dump_url(),
        dest_path: settings.download_path(),
    })
}

/// AniDB dump downloader
pub struct DumpDownloader<D: FileDownload, U: AsRef<str>, P: AsRef<Path> + Clone + Send + 'static> {
    /// Files downloading client
    downloader: D,
    /// URL where dump is hosted
    dump_url: U,
    /// Path where to save dump
    dest_path: P,
}

impl<D: FileDownload, U: AsRef<str>, P: AsRef<Path> + Clone + Send + 'static> DumpDownloader<D, U, P> {
    /// Creates new instance
    pub fn new(downloader: D, dump_url: U, dest_path: P) -> Self {
        DumpDownloader { downloader, dump_url, dest_path }
    }

    /// Asynchronously downloads dump at `dump_url` and saves it on disk at `dest_path`
    pub fn download(&self) -> impl Future<Item = (), Error = DownloadError> {
        let dump = self.downloader.download(self.dump_url.as_ref());
        let file = File::create(self.dest_path.clone())
            .map_err(DownloadError::from);

        file.and_then(move |f| {
            dump.fold(f, |f, chunk| {
                tokio::io::write_all(f, chunk)
                    .map_err(DownloadError::from)
                    .map(|(f, _)| f)
            }).and_then(|_| Ok(()))
        })
    }
}

/// Asynchronous file downloading client
pub trait FileDownload {
    /// Type of chunks of data stream
    type Chunk: AsRef<[u8]>;
    /// Stream of data chunks
    type Bytes: Stream<Item=Self::Chunk, Error=DownloadError>;

    /// Asynchronously starts downloading file at specified `url`
    fn download(&self, url: &str) -> Self::Bytes;
}

impl FileDownload for Client {
    type Chunk = reqwest::r#async::Chunk;
    type Bytes = Box<dyn Stream<Item = Self::Chunk, Error = DownloadError>>;

    fn download(&self, url: &str) -> Self::Bytes {
        let bytes = self.request(Method::GET, url)
            .send()
            .into_stream()
            .take(1)
            .map(|r| r.into_body())
            .flatten()
            .map_err(|e| DownloadError::from(e));

        Box::new(bytes) // TODO: how to eliminate alloc?
    }
}

/// Represents an error that may happen during dump download
pub enum DownloadError {
    /// Request or download has failed
    Net(reqwest::Error),
    /// Failed to write dump on disk
    Fs(std::io::Error),
}

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

impl std::error::Error for DownloadError {}

impl From<reqwest::Error> for DownloadError {
    fn from(e: reqwest::Error) -> Self {
        DownloadError::Net(e)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(e: std::io::Error) -> Self {
        DownloadError::Fs(e)
    }
}

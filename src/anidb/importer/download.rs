use futures::prelude::*;
use reqwest::{
    r#async::{Client, ClientBuilder},
    Method,
};
use tokio::fs::File;

use std::{
    fmt::{self, Debug, Display},
    path::Path,
    time::Duration,
};

/// Creates downloader with configuration from global app settings
pub fn downloader<U, P>(
    dump_url: U,
    dest_path: P,
) -> impl Future<Item = (), Error = DownloadError> + Send
where
    U: AsRef<str> + Send,
    P: AsRef<Path> + Clone + Send + 'static,
{
    let client = ClientBuilder::new()
        .gzip(true)
        .connect_timeout(Duration::new(60, 0)) // TODO: from config
        .build();

    futures::future::result(client)
        .from_err()
        .and_then(move |client| DumpDownloader::new(client, dump_url, dest_path).download())
}

/// Asynchronous file downloading client
pub trait FileDownload: Send {
    /// Type of chunks of data stream
    type Chunk: AsRef<[u8]>;
    /// Stream of data chunks
    type Bytes: Stream<Item = Self::Chunk, Error = DownloadError>;

    /// Asynchronously starts downloading file at specified `url`
    fn download(&self, url: &str) -> Self::Bytes;
}

/// AniDB dump downloader
pub struct DumpDownloader<D, U, P> {
    /// Files downloading client
    downloader: D,
    /// URL where dump is hosted
    dump_url: U,
    /// Path where to save dump
    dest_path: P,
}

/// Represents an error that may happen during dump download
pub enum DownloadError {
    /// Request or download has failed
    Net(reqwest::Error),
    /// Failed to write dump on disk
    Fs(std::io::Error),
}

// MARK: impl DumpDownloader

impl<D, U, P> DumpDownloader<D, U, P>
where
    D: FileDownload,
    U: AsRef<str>,
    P: AsRef<Path> + Clone + Send + 'static,
{
    /// Creates new instance
    pub fn new(downloader: D, dump_url: U, dest_path: P) -> Self {
        DumpDownloader {
            downloader,
            dump_url,
            dest_path,
        }
    }

    /// Asynchronously downloads dump at `dump_url` and saves it on disk at `dest_path`
    pub fn download(&self) -> impl Future<Item = (), Error = DownloadError> {
        let dump = self.downloader.download(self.dump_url.as_ref());
        let file = File::create(self.dest_path.clone()).map_err(DownloadError::from);

        file.and_then(move |f| {
            dump.fold(f, |f, chunk| {
                tokio::io::write_all(f, chunk)
                    .map_err(DownloadError::Fs)
                    .map(|(f, _)| f)
            })
            .and_then(|_| Ok(()))
        })
    }
}

// MARK: impl FileDownload

impl FileDownload for Client {
    type Chunk = reqwest::r#async::Chunk;
    type Bytes = Box<dyn Stream<Item = Self::Chunk, Error = DownloadError> + Send>;

    fn download(&self, url: &str) -> Self::Bytes {
        let bytes = self
            .request(Method::GET, url)
            .send()
            .into_stream()
            .take(1)
            .map(|r| r.into_body())
            .flatten()
            .from_err();

        Box::new(bytes)
    }
}

// MARK: impl DownloadError

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

#[cfg(test)]
mod tests {
    use super::{
        super::test_utils::{download::*, tokio_run_aborting},
        *,
    };
    use tokio::prelude::*;

    #[test]
    fn test_download() -> Result<(), std::io::Error> {
        let mut dst = tempfile::Builder::new().tempfile()?;

        let chunks = vec![
            Chunk([1, 2]),
            Chunk([3, 4]),
            Chunk([5, 6]),
            Chunk([7, 8]),
            Chunk([9, 10]),
        ];

        let downloader = FakeDownloader {
            content: chunks.clone(),
        };

        let fut = DumpDownloader::new(downloader, "", dst.path().to_path_buf());
        tokio_run_aborting(
            fut.download()
                .map_err(|e| panic!("failed to save data: {}", e)),
        );

        let mut expected: Vec<u8> = vec![];
        let mut got = vec![];

        chunks.iter().for_each(|c| expected.extend(c.0.iter()));
        dst.read_to_end(&mut got)?;

        assert_eq!(expected, got);

        Ok(())
    }
}

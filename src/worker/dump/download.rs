use futures::prelude::*;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::r#async::{Client, ClientBuilder};
use reqwest::Method;
use tokio::fs::File;

use std::fmt::{self, Debug, Display};
use std::path::Path;
use std::time::Duration;

/// Creates downloader with configuration from global app settings
pub fn downloader<U, P>(
    dump_url: U,
    dest_path: P,
) -> impl Future<Item = (), Error = DownloadError> + Send
where
    U: AsRef<str> + Send,
    P: AsRef<Path> + Clone + Send + 'static,
{
    const USER_AGENT: &str =
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_14_4) AppleWebKit/605.1.15 \
         (KHTML, like Gecko) Version/12.1 Safari/605.1.15";

    let mut headers = HeaderMap::new();
    headers.insert(header::USER_AGENT, HeaderValue::from_static(USER_AGENT));

    let client = ClientBuilder::new()
        .default_headers(headers)
        .gzip(true)
        .connect_timeout(Duration::new(60, 0))
        .build();

    futures::future::result(client)
        .from_err()
        .and_then(move |client| DumpDownloader::new(client, dump_url, dest_path).download())
}

/// AniDB dump downloader
pub struct DumpDownloader<D, U, P>
where
    D: FileDownload,
    U: AsRef<str>,
    P: AsRef<Path> + Clone + Send + 'static,
{
    /// Files downloading client
    downloader: D,
    /// URL where dump is hosted
    dump_url: U,
    /// Path where to save dump
    dest_path: P,
}

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
                    .map_err(DownloadError::from)
                    .map(|(f, _)| f)
            })
            .and_then(|_| Ok(()))
        })
    }
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

#[cfg(test)]
mod tests {
    use super::super::test_utils::download::*;
    use super::*;
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
        tokio::run(
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

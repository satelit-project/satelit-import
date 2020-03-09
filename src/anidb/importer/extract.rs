use async_compression::futures::bufread::GzipDecoder;
use tokio::{fs::File, io::BufReader};
use tokio_util::compat::{FuturesAsyncReadCompatExt, Tokio02AsyncReadCompatExt};

use std::path::Path;

pub type ExtractError = std::io::Error;

/// Extracts gzip archive at `src_path` to `dst_path`.
pub async fn extract<P>(src_path: P, dst_path: P) -> Result<(), ExtractError>
where
    P: AsRef<Path> + Send,
{
    let extractor = GzipExtractor::new(src_path, dst_path);
    extractor.extract().await
}

/// Asynchronously extracts gzip archive.
pub struct GzipExtractor<P> {
    /// Path to gzip archive
    src_path: P,

    /// Path to where to extract the atchive.
    dest_path: P,
}

// MARK: impl GzipExtractor

impl<P: AsRef<Path> + Send> GzipExtractor<P> {
    /// Returns new instance for extracting from `src_path` to `dest_path`.
    pub fn new(src_path: P, dest_path: P) -> Self {
        GzipExtractor {
            src_path,
            dest_path,
        }
    }

    /// Asynchronously extracts gzip archive.
    pub async fn extract(&self) -> Result<(), ExtractError> {
        let src = File::open(&self.src_path);
        let dst = File::open(&self.dest_path);
        let (src_file, mut dst_file) = futures::try_join!(src, dst)?;

        let reader = BufReader::new(src_file);
        let mut decoder = GzipDecoder::new(reader.compat()).compat();
        tokio::io::copy(&mut decoder, &mut dst_file).await?;

        Ok(())
    }
}

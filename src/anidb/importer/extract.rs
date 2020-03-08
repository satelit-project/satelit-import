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

// #[cfg(test)]
// mod tests_gz {
//     use super::{super::test_utils::tokio_run_aborting, *};
//     use flate2::{write::GzEncoder, Compression};
//     use std::{fs::File, io::prelude::*};

//     #[test]
//     fn test_extraction() -> Result<(), std::io::Error> {
//         let src = tempfile::Builder::new().tempfile()?;
//         let dst = tempfile::Builder::new().tempfile()?;
//         let data = b"Hello world! Where are you? What are you doing?".to_vec();

//         compress_data(data.clone(), src.path())?;

//         let fut = extractor(src.path().to_path_buf(), dst.path().to_path_buf());
//         tokio_run_aborting(fut.map_err(|e| panic!("unexpected error on extract: {}", e)));

//         let mut got = vec![];
//         File::open(dst.path()).and_then(|mut f| f.read_to_end(&mut got))?;

//         assert_eq!(data, got);

//         Ok(())
//     }

//     fn compress_data<B, P>(mut data: B, path: P) -> Result<(), std::io::Error>
//     where
//         B: AsMut<[u8]>,
//         P: AsRef<Path>,
//     {
//         let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
//         encoder.write_all(data.as_mut())?;

//         let gz = encoder.finish()?;
//         File::create(path).and_then(|mut f| f.write_all(&gz))
//     }
// }

use flate2::bufread::GzDecoder;
use futures::prelude::*;
use futures::try_ready;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncWrite};

use std::io::BufReader;
use std::path::Path;

pub type ExtractError = std::io::Error;

/// Creates gzip extractor configured with global app settings
pub fn extractor<P>(src_path: P, dst_path: P) -> impl Future<Item = (), Error = ExtractError> + Send
where
    P: AsRef<Path> + Clone + Send + 'static,
{
    let extractor = GzipExtractor::new(src_path, dst_path);
    extractor.extract()
}

/// Asynchronously extracts single file from gzip archive
pub struct GzipExtractor<P> {
    /// Path to gzip archive
    src_path: P,
    /// Path to a file where archive should be extracted
    dest_path: P,
    /// Size of read buffer that should be filled before it will be written
    chunk_size: usize,
}

// TODO: replace with tokio::io::copy
/// Future that reads from a file and writes it's to another file
struct AsyncReadWrite<S: AsyncRead, D: AsyncWrite> {
    /// Source file
    src: S,
    /// Destination file
    dst: D,
    /// State of the future
    state: AsyncReadWriteState,
    /// Read buffer size that should be filled before moving to `Writing` state
    chunk_size: usize,
    /// Read buffer
    buf: Vec<u8>,
    /// Number of readed bytes from source file
    readed: usize,
    /// Number of written bytes to destination file
    written: usize,
}

/// State of `AsyncReadWrite` future
enum AsyncReadWriteState {
    /// Reading data from source file
    Reading,
    /// Writing data to destination file
    Writing,
    /// Flushing file data to fs
    Flushing,
}

// MARK: impl GzipExtractor

impl<P: AsRef<Path> + Clone + Send + 'static> GzipExtractor<P> {
    pub const DEFAULT_CHUNK_SIZE: usize = 1024;

    /// Returns new instance for extracting from `src_path` to `dest_path`. Read buffer size is
    /// set to `Self::DEFAULT_CHUNK_SIZE`
    pub fn new(src_path: P, dest_path: P) -> Self {
        GzipExtractor {
            src_path,
            dest_path,
            chunk_size: GzipExtractor::<P>::DEFAULT_CHUNK_SIZE,
        }
    }

    /// Sets read buffer size to `size`
    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size = size;
    }

    /// Asynchronously extracts archive
    pub fn extract(&self) -> impl Future<Item = (), Error = ExtractError> {
        let src = File::open(self.src_path.clone());
        let dest = File::create(self.dest_path.clone());
        let chunk_size = self.chunk_size;

        src.join(dest).and_then(move |(src, dest)| {
            let reader = BufReader::new(src);
            let decoder = GzDecoder::new(reader);

            AsyncReadWrite::new(decoder, dest, chunk_size)
        })
    }
}

// MARK: impl AsyncReadWrite

impl<S: AsyncRead, D: AsyncWrite> AsyncReadWrite<S, D> {
    fn new(src: S, dst: D, chunk_size: usize) -> Self {
        let buf = vec![0; chunk_size];
        AsyncReadWrite {
            src,
            dst,
            state: AsyncReadWriteState::Reading,
            chunk_size,
            buf,
            readed: 0,
            written: 0,
        }
    }
}

impl<S: AsyncRead, D: AsyncWrite> Future for AsyncReadWrite<S, D> {
    type Item = ();
    type Error = ExtractError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        use AsyncReadWriteState::*;

        let buf = self.buf.as_mut_slice();
        'g: loop {
            match self.state {
                Reading => {
                    while self.readed < self.chunk_size {
                        let bs = &mut buf[self.readed..self.chunk_size];
                        let n = try_ready!(self.src.poll_read(bs));

                        self.readed += n;

                        if n == 0 {
                            self.state = if self.readed == 0 { Flushing } else { Writing };
                            self.written = 0;
                            continue 'g;
                        }
                    }

                    self.written = 0;
                    self.state = Writing;
                }
                Writing => {
                    while self.written < self.readed {
                        let bs = &buf[self.written..self.readed];
                        let n = try_ready!(self.dst.poll_write(bs));

                        self.written += n;
                    }

                    self.readed = 0;
                    self.state = Reading;
                }
                Flushing => {
                    try_ready!(self.dst.poll_flush());
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests_gz {
    use super::super::test_utils::tokio_run_aborting;
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::fs::File;
    use std::io::prelude::*;

    #[test]
    fn test_extraction() -> Result<(), std::io::Error> {
        let src = tempfile::Builder::new().tempfile()?;
        let dst = tempfile::Builder::new().tempfile()?;
        let data = b"Hello world! Where are you? What are you doing?".to_vec();

        compress_data(data.clone(), src.path())?;

        let fut = extractor(src.path().to_path_buf(), dst.path().to_path_buf());
        tokio_run_aborting(fut.map_err(|e| panic!("unexpected error on extract: {}", e)));

        let mut got = vec![];
        File::open(dst.path()).and_then(|mut f| f.read_to_end(&mut got))?;

        assert_eq!(data, got);

        Ok(())
    }

    fn compress_data<B, P>(mut data: B, path: P) -> Result<(), std::io::Error>
    where
        B: AsMut<[u8]>,
        P: AsRef<Path>,
    {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data.as_mut())?;

        let gz = encoder.finish()?;
        File::create(path).and_then(|mut f| f.write_all(&gz))
    }
}

#[cfg(test)]
mod tests_rw {
    use super::super::test_utils::extract::*;
    use super::super::test_utils::tokio_run_aborting;
    use super::*;
    use std::io::{Read, Seek, SeekFrom, Write};

    /// Tests the case when data can be read into buffer all at once
    #[test]
    fn test_read_write_all() -> Result<(), ExtractError> {
        let content = "hello world";
        let (src, dst) = read_write_content(content.to_mut(), 32)?;

        assert_eq!(src, content);
        assert_eq!(src, dst);

        Ok(())
    }

    /// Test the case when data will not fit into read buffer all at once
    #[test]
    fn test_read_write_chunked() -> Result<(), ExtractError> {
        let content = "hello world ".repeat(20);
        let (src, dst) = read_write_content(content.to_mut(), 16)?;

        assert_eq!(src, content);
        assert_eq!(src, dst);

        Ok(())
    }

    /// Writes content of a file into another file and returns their content
    fn read_write_content<T>(mut content: T, chunk_size: usize) -> Result<(T, T), ExtractError>
    where
        T: AsMut<[u8]> + From<Vec<u8>>,
    {
        let mut tmp_src = tempfile::Builder::new().tempfile()?;
        let mut tmp_dst = tempfile::Builder::new().tempfile()?;

        tmp_src.write_all(content.as_mut())?;
        tmp_src.flush()?;
        tmp_src.seek(SeekFrom::Start(0))?;

        let src = tokio::fs::File::open(tmp_src.path().to_owned());
        let dst = tokio::fs::File::create(tmp_dst.path().to_owned());

        let task = src
            .join(dst)
            .and_then(move |(src, dst)| AsyncReadWrite::new(src, dst, chunk_size))
            .map_err(|e| panic!(format!("{}", e)));

        tokio_run_aborting(task);

        let mut expected = vec![];
        tmp_src.read_to_end(&mut expected)?;

        let mut got = vec![];
        tmp_dst.read_to_end(&mut got)?;

        Ok((T::from(expected), T::from(got)))
    }
}

use flate2::bufread::GzDecoder;
use futures::prelude::*;
use futures::try_ready;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncWrite};

use std::io::BufReader;
use std::path::Path;

pub type ExtractError = std::io::Error;

/// Creates gzip extractor configured with global app settings
pub fn extractor<P>(
    src_path: P,
    dst_path: P,
    chunk_size: usize,
) -> impl Future<Item = (), Error = ExtractError>
where
    P: AsRef<Path> + Clone + Send + 'static,
{
    let mut extractor = GzipExtractor::new(src_path, dst_path);
    extractor.set_chunk_size(chunk_size);
    extractor.extract()
}

/// Asynchronously extracts single file from gzip archive
pub struct GzipExtractor<P: AsRef<Path> + Clone + Send + 'static> {
    /// Path to gzip archive
    src_path: P,
    /// Path to a file where archive should be extracted
    dest_path: P,
    /// Size of read buffer that should be filled before it will be written
    chunk_size: usize,
}

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

/// State of `AsyncReadWrite` future
enum AsyncReadWriteState {
    /// Reading data from source file
    Reading,
    /// Writing data to destination file
    Writing,
    /// Flushing file data to fs
    Flushing,
}

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
            match &self.state {
                &Reading => {
                    while self.readed < self.chunk_size {
                        let bs = &mut buf[self.readed..self.chunk_size];
                        let n = try_ready!(self.src.poll_read(bs));

                        self.readed += n;

                        if n == 0 {
                            self.state = if self.readed == 0 { Flushing } else { Writing };
                            continue 'g;
                        }
                    }

                    self.written = 0;
                    self.state = Writing;
                }
                &Writing => {
                    while self.written < self.readed {
                        let bs = &buf[self.written..self.readed];
                        let n = try_ready!(self.dst.poll_write(bs));

                        self.written += n;
                    }

                    self.readed = 0;
                    self.state = Reading;
                }
                &Flushing => {
                    try_ready!(self.dst.poll_flush());
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests_rw {
    use super::*;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::ops::Deref;

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

        tokio::run(task);

        let mut expected = vec![];
        tmp_src.read_to_end(&mut expected)?;

        let mut got = vec![];
        tmp_dst.read_to_end(&mut got)?;

        Ok((T::from(expected), T::from(got)))
    }

    // String helpers

    #[derive(Debug)]
    struct StringMut(String);

    trait ToMut {
        fn to_mut(&self) -> StringMut;
    }

    // String helpers implementations

    impl ToMut for String {
        fn to_mut(&self) -> StringMut {
            StringMut(self.clone())
        }
    }

    impl ToMut for str {
        fn to_mut(&self) -> StringMut {
            StringMut(self.to_owned())
        }
    }

    impl AsMut<[u8]> for StringMut {
        fn as_mut(&mut self) -> &mut [u8] {
            unsafe { self.0.as_bytes_mut() }
        }
    }

    impl From<Vec<u8>> for StringMut {
        fn from(bytes: Vec<u8>) -> Self {
            unsafe { StringMut(String::from_utf8_unchecked(bytes)) }
        }
    }

    impl Deref for StringMut {
        type Target = str;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::cmp::PartialEq for StringMut {
        fn eq(&self, other: &StringMut) -> bool {
            self.0 == other.0
        }
    }

    impl std::cmp::PartialEq<String> for StringMut {
        fn eq(&self, other: &String) -> bool {
            &self.0 == other
        }
    }

    impl std::cmp::PartialEq<&str> for StringMut {
        fn eq(&self, other: &&str) -> bool {
            &self.0 == other
        }
    }
}

use futures::prelude::*;
use futures::try_ready;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::fs::{File};
use flate2::bufread::GzDecoder;

use std::path::Path;
use std::io::BufReader;

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
    pub const DEFAULT_CHUNK_SIZE: usize = 8192;

    /// Returns new instance for extracting from `src_path` to `dest_path`. Read buffer size is
    /// set to `Self::DEFAULT_CHUNK_SIZE`
    pub fn new(src_path: P, dest_path: P) -> Self {
        GzipExtractor {
            src_path,
            dest_path,
            chunk_size: GzipExtractor::<P>::DEFAULT_CHUNK_SIZE
        }
    }

    /// Sets read buffer size to `size`
    pub fn set_chunk_size(&mut self, size: usize) {
        self.chunk_size = size;
    }

    /// Asynchronously extracts archive
    pub fn extract(&self) -> impl Future<Item = (), Error = std::io::Error> {
        let src = File::open(self.src_path.clone());
        let dest = File::create(self.dest_path.clone());
        let chunk_size = self.chunk_size;

        src.join(dest)
            .and_then(move |(src, dest)| {
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
        let buf = Vec::<u8>::with_capacity(chunk_size);
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
    type Error = std::io::Error;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        use AsyncReadWriteState::*;

        let buf = self.buf.as_mut_slice();
        loop {
            match &self.state {
                &Reading => {
                    while self.readed < self.chunk_size {
                        let bs = &mut buf[self.readed..self.chunk_size];
                        let n = try_ready!(self.src.poll_read(bs));

                        self.readed += n;

                        if n == 0 {
                            self.state = Flushing;
                            continue;
                        }
                    }

                    self.written = 0;
                    self.state = Writing;
                },
                &Writing => {
                    while self.readed > 0 {
                        let bs = &buf[self.written..self.chunk_size];
                        let n = try_ready!(self.dst.poll_write(bs));

                        self.written += n;
                        self.readed -= n;
                    }

                    self.readed = 0;
                    self.state = Reading;
                },
                &Flushing => {
                    try_ready!(self.dst.poll_flush());
                    return Ok(Async::Ready(()));
                }
            }
        }
    }
}

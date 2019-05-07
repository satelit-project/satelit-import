use futures::prelude::*;
use tokio::fs;
use tokio::io;

use std::path::Path;

/// And io error that may occur during file copy
pub type CopyError = std::io::Error;

/// Creates a future that copies file at `src` to `dst`
pub fn copier<P>(src: P, dst: P) -> impl Future<Item = (), Error = CopyError>
where
    P: AsRef<Path> + Send + Clone + 'static,
{
    let dst_path = dst.clone();

    fs::File::open(src.clone())
        .and_then(move |src| {
            let dst = fs::File::create(dst_path);
            dst.and_then(move |dst| io::copy(src, dst))
        })
        .map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn test_copy() -> Result<(), std::io::Error> {
        let mut src = tempfile::Builder::new().tempfile()?;
        let mut dst = tempfile::Builder::new().tempfile()?;
        let content = b"Hello there".to_vec();

        src.write_all(&mut content.clone())?;

        let fut = copier(src.path().to_owned(), dst.path().to_owned())
            .map_err(|e| panic!("failed to copy data: {}", e));

        tokio::run(fut);

        let mut got = vec![];
        dst.read_to_end(&mut got)?;

        assert_eq!(content, got);

        Ok(())
    }

    #[test]
    fn test_no_copy_needed() -> Result<(), std::io::Error> {
        let src = tempfile::Builder::new().tempfile()?;
        let dst = tempfile::Builder::new().tempfile()?;

        let src_path = src.path().to_owned();
        let dst_path = dst.path().to_path_buf();
        let fut = copier(src_path.clone(), dst_path.clone());

        drop(src);
        drop(dst);

        tokio::run(futures::future::lazy(move || {
            fut.then(|res| {
                match res {
                    Ok(_) => panic!("expected to receive NotFound error"),
                    Err(e) => assert_eq!(e.kind(), std::io::ErrorKind::NotFound),
                }

                Ok(())
            })
        }));

        assert!(!src_path.exists(), "no files should be created");
        assert!(!dst_path.exists(), "no files should be created");

        Ok(())
    }
}

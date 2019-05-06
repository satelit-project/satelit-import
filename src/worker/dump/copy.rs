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
    use std::fs::{self, File};
    use std::io::{Read, Write};
    use std::path::PathBuf;

    #[test]
    fn test_copy() {
        let (src_path, dst_path) = create_file_paths("rw");
        let content = b"Hello there".to_vec();

        File::create(&src_path)
            .and_then(|mut f| f.write_all(&content))
            .expect("failed to write test data");

        let fut =
            copier(src_path, dst_path.clone()).map_err(|e| panic!("failed to copy data: {}", e));

        tokio::run(fut);

        let mut written_content = vec![];
        File::open(dst_path)
            .and_then(|mut f| f.read_to_end(&mut written_content))
            .expect("failed to read data from copied file");

        assert_eq!(content, written_content);
    }

    #[test]
    fn test_no_copy_needed() {
        let (src_path, dst_path) = create_file_paths("no");
        let fut = copier(src_path.clone(), dst_path.clone());

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
    }

    fn create_file_paths(suffix: &str) -> (PathBuf, PathBuf) {
        let mut src_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        src_path.push(format!("resources/tests/copy_me{}", suffix));

        let mut dst_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        dst_path.push(format!("resources/tests/copy_into_me{}", suffix));

        if src_path.exists() {
            fs::remove_file(&src_path).expect("failed to clean tests dir");
        }

        if dst_path.exists() {
            fs::remove_file(&dst_path).expect("failed to clean tests dir");
        }

        (src_path, dst_path)
    }
}

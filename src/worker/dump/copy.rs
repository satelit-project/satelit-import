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

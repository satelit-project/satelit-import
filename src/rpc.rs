pub mod import;
pub mod task;

use futures::future::poll_fn;
use futures::prelude::*;
use futures::sync::oneshot;

#[derive(Debug)]
pub enum BlockingError<E: std::error::Error> {
    Error(E),
    Cancelled,
    Unavailable,
}

pub fn blocking<F, I, E>(mut f: F) -> impl Future<Item = I, Error = BlockingError<E>>
where
    F: FnMut() -> Result<I, E> + Send + 'static,
    I: Send + 'static,
    E: std::error::Error + Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    tokio::spawn(futures::lazy(move || {
        poll_fn(move || tokio_threadpool::blocking(|| f())).then(move |result| {
            if tx.is_canceled() {
                return futures::future::err(());
            }

            let _ = match result {
                Ok(inner) => match inner {
                    Ok(item) => tx.send(Ok(item)),
                    Err(e) => tx.send(Err(BlockingError::Error(e))),
                },
                Err(_) => tx.send(Err(BlockingError::Unavailable)),
            };

            futures::future::ok(())
        })
    }));

    rx.then(|result| match result {
        Ok(inner) => match inner {
            Ok(item) => Ok(item),
            Err(e) => Err(e),
        },
        Err(_) => Err(BlockingError::Cancelled),
    })
}

impl<E: std::error::Error> std::fmt::Display for BlockingError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        use BlockingError::*;

        match self {
            Error(e) => write!(f, "{}\n", e),
            Cancelled => write!(f, "Channel has been closed\n"),
            Unavailable => write!(f, "Blocking thread pool is unavailable\n"),
        }
    }
}

impl<E: std::error::Error> std::error::Error for BlockingError<E> {}

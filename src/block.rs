use futures::prelude::*;
use futures::sync::oneshot;
use lazy_static::lazy_static;
use log::error;
use rayon::{ThreadPool, ThreadPoolBuilder};

lazy_static! {
    static ref THREAD_POOL: ThreadPool = {
        ThreadPoolBuilder::new()
            .build()
            .expect("failed to init thread pool")
    };
}

#[derive(Debug)]
pub enum BlockingError<E: std::fmt::Debug> {
    Error(E),
    Cancelled,
}

pub fn blocking<F, I, E>(f: F) -> impl Future<Item = I, Error = BlockingError<E>>
    where
        F: FnOnce() -> Result<I, E> + Send + 'static,
        I: Send + 'static,
        E: std::fmt::Debug + Send + 'static,
{
    let (tx, rx) = oneshot::channel();
    THREAD_POOL.spawn(move || {
        let result = f();

        if tx.is_canceled() {
            error!("blocking: receiver dropped");
            return;
        }

        let _ = match result {
            Ok(item) => tx.send(Ok(item)),
            Err(e) => tx.send(Err(BlockingError::Error(e))),
        };
    });

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
            Error(e) => writeln!(f, "{}", e),
            Cancelled => writeln!(f, "Channel has been closed"),
        }
    }
}

impl<E: std::error::Error> std::error::Error for BlockingError<E> {}

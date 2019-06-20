use futures::prelude::*;

pub mod dump;

pub trait Worker: Send + 'static {
    fn task(self) -> Box<dyn Future<Item = (), Error = ()>>;
}

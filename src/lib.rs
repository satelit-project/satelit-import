#[macro_use]
extern crate diesel; // TODO: remove when diesel will support 2018 edition's macro import

pub mod anidb;
pub mod block;
pub mod db;
pub mod proto;
pub mod rpc;
pub mod settings;
pub mod worker;

#[macro_use]
extern crate diesel; // TODO: remove when diesel will support 2018 edition's macro import

pub mod settings;
pub mod anidb;
pub mod db;
pub mod worker;

mod common;
mod db_tests;

use satelit_import::db;

pub fn connection_pool(id: &str) -> db::ConnectionPool {
    common::connection_pool("db_tests", id)
}

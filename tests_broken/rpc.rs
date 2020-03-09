mod common;
mod rpc_tests;

use satelit_import::{db, settings};

pub fn settings() -> settings::Settings {
    common::settings("rpc_tests")
}

pub fn connection_pool(id: &str) -> db::ConnectionPool {
    common::connection_pool("rpc_tests", id)
}

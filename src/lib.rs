#[macro_use]
extern crate diesel;

pub mod anidb;
pub mod block;
pub mod db;
pub mod proto;
pub mod proto_ext;
pub mod rpc;
pub mod settings;

use lazy_static::lazy_static;

use db::ConnectionPool;
use settings::Settings;

lazy_static! {
    static ref SHARED_SETTINGS: Settings = {
        use settings::Profile;

        let name = option_env!("SATELIT_IMPORT_CONFIG");
        let profile = match name {
            Some(name) => Profile::Named(name.to_string()),
            None => Profile::Default,
        };

        Settings::new(profile).expect("failed to read settings")
    };
}

lazy_static! {
    static ref SHARED_POOL: ConnectionPool = {
        db::new_connection_pool(SHARED_SETTINGS.db()).expect("failed to escablish db connection")
    };
}

/// Returns reference to global settings instance
pub fn shared_settings() -> Settings {
    SHARED_SETTINGS.clone()
}

/// Returns global connection pool with global settings
pub fn shared_db_pool() -> ConnectionPool {
    SHARED_POOL.clone()
}

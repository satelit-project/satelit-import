use config::{Config, ConfigError, File};
use serde::Deserialize;

use std::sync::Once;
use std::time::Duration;

/// Global settings used to configure app state
#[derive(Debug, Deserialize)]
pub struct Settings {
    db: Db,
    anidb: AniDb,
}

impl Settings {
    /// Returns reference to global settings instance
    pub fn shared() -> &'static Self {
        static mut SHARED: *const Settings = 0 as *const Settings;
        static ONCE: Once = Once::new();

        unsafe {
            ONCE.call_once(|| {
                let settings = Self::new().expect("failed to read settings");
                SHARED = Box::into_raw(Box::new(settings));
            });

            &*SHARED
        }
    }

    /// Returns database settings
    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Returns AniDB settings
    pub fn anidb(&self) -> &AniDb {
        &self.anidb
    }

    fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();
        s.merge(File::with_name("config/default"))?;
        s.try_into()
    }
}

/// Global database settings
#[derive(Debug, Deserialize)]
pub struct Db {
    path: String,
    max_connections: u32,
    connection_timeout: Duration,
}

impl Db {
    /// Returns path to database
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns number of maximum allowed db connections
    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    /// Returns timeout for a db connection
    pub fn connection_timeout(&self) -> Duration {
        self.connection_timeout
    }
}

/// Global AniDB settings
#[derive(Debug, Deserialize)]
pub struct AniDb {
    dump_url: String,
    last_dump: String,
    new_dump: String,
}

impl AniDb {
    /// Returns URL to latest AniDB dump
    pub fn dump_url(&self) -> &str {
        &self.dump_url
    }

    /// Return path to latest imported dump
    pub fn last_dump(&self) -> &str {
        &self.last_dump
    }

    /// Returns path to dump to be imported
    pub fn new_dump(&self) -> &str {
        &self.new_dump
    }
}

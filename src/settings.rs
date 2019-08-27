use config::{Config, ConfigError, File};
use serde::Deserialize;
use lazy_static::lazy_static;

use std::time::Duration;

lazy_static! {
    static ref SHARED: Settings = Settings::new().expect("failed to read settings");
}

/// Returns reference to global settings instance
pub fn shared() -> &'static Settings {
    &SHARED
}

/// Application settings
#[derive(Debug, Deserialize)]
pub struct Settings {
    db: Db,
    import: Import,
    ports: Ports,
}

/// Database settings
#[derive(Debug, Deserialize)]
pub struct Db {
    url: String,
    max_connections: u32,
    connection_timeout: u64,
}

/// Anime index import settings
#[derive(Debug, Deserialize)]
pub struct Import {
    new_download_path: String,
    old_download_path: String,
    new_extract_path: String,
    old_extract_path: String,
}

/// gRPC services ports
#[derive(Debug, Deserialize)]
pub struct Ports {
    import: i32,
    task: i32,
}

// MARK: impl Settings

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        let default = File::with_name("config/default.toml");
        s.merge(default)?;

        s.try_into()
    }

    /// Returns database settings
    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Returns dump import settings
    pub fn import(&self) -> &Import {
        &self.import
    }

    /// Returns gRPC services ports settings
    pub fn ports(&self) -> &Ports {
        &self.ports
    }
}

// MARK: impl Db

impl Db {
    /// Returns path to database
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns number of maximum allowed db connections
    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

    /// Returns timeout for a db connection
    pub fn connection_timeout(&self) -> Duration {
        Duration::new(self.connection_timeout, 0)
    }
}

// MARK: impl Import

impl Import {
    pub fn new_download_path(&self) -> &str {
        &self.new_download_path
    }

    pub fn old_download_path(&self) -> &str {
        &self.old_download_path
    }

    pub fn new_extract_path(&self) -> &str {
        &self.new_extract_path
    }

    pub fn old_extract_path(&self) -> &str {
        &self.old_extract_path
    }
}

// MARK: impl Ports

impl Ports {
    /// Port for `ImportService`
    pub fn import(&self) -> i32 {
        self.import
    }

    /// Port for `ScraperTasksService`
    pub fn task(&self) -> i32 {
        self.task
    }
}

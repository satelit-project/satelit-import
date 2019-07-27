use config::{Config, ConfigError, File};
use serde::Deserialize;

use std::sync::Once;
use std::time::Duration;

/// Returns reference to global settings instance
pub fn shared() -> &'static Settings {
    static mut SHARED: *const Settings = std::ptr::null();
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let settings = Settings::new(Profile::Default).expect("failed to read settings");
            SHARED = Box::into_raw(Box::new(settings));
        });

        &*SHARED
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Profile {
    Default,
    IntegrationTests,
}

impl Profile {
    fn files(&self) -> &[&str] {
        use Profile::*;

        match *self {
            Default => &["default"],
            IntegrationTests => &["default", "integration-tests"],
        }
    }
}

/// Global settings used to configure app state
#[derive(Debug, Deserialize)]
pub struct Settings {
    db: Db,
    import: Import,
}

impl Settings {
    /// Returns database settings
    pub fn db(&self) -> &Db {
        &self.db
    }

    /// Returns dump import settings
    pub fn import(&self) -> &Import {
        &self.import
    }

    pub fn new(profile: Profile) -> Result<Self, ConfigError> {
        let mut s = Config::new();

        for &file in profile.files() {
            s.merge(File::with_name(&format!("config/{}", file)))?;
        }

        s.try_into()
    }
}

/// Global database settings
#[derive(Debug, Deserialize)]
pub struct Db {
    path: String,
    max_connections: u32,
    connection_timeout: u64,
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
        Duration::new(self.connection_timeout, 0)
    }
}

/// Global anime import settings
#[derive(Debug, Deserialize)]
pub struct Import {
    dump_url: String,
    download_path: String,
    dump_backup_path: String,
    dump_path: String,
    reimport_track_path: String,
    chunk_size: usize,
}

impl Import {
    /// Returns URL to latest AniDB dump
    pub fn dump_url(&self) -> &str {
        &self.dump_url
    }

    /// Path where new dumps will be downloaded
    pub fn download_path(&self) -> &str {
        &self.download_path
    }

    /// Return path to backed up dump
    pub fn dump_backup_path(&self) -> &str {
        &self.dump_backup_path
    }

    /// Returns path to dump to be imported
    pub fn dump_path(&self) -> &str {
        &self.dump_path
    }

    /// Returns path to file with failed to import anime id's
    pub fn reimport_track_path(&self) -> &str {
        &self.reimport_track_path
    }

    /// Size of read buffer for gzip extraction
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }
}

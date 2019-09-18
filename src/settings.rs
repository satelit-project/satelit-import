use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

use std::time::Duration;

/// Settings profile
///
/// All profiles are based on `Default`, that is, all settings from `Default`
/// profile will be available, but may be overridden, will be inherited
/// by other profiles.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Default,
    Named(String),
}

/// Application settings
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// Database settings
    pub db: Db,
    /// Dump import settings
    pub import: Import,
    /// RPC services settings
    pub rpc: Rpc,
}

/// Database settings
#[derive(Debug, Clone, Deserialize)]
pub struct Db {
    /// Path to database
    pub url: String,
    /// Number of maximum simultaneous connections
    pub max_connections: u32,
    /// DB connection timeout
    pub connection_timeout: u64,
}

/// Anime index import settings
#[derive(Debug, Clone, Deserialize)]
pub struct Import {
    /// Path to download new anime dump
    pub new_download_path: String,
    /// Path to download old anime dump
    pub old_download_path: String,
    /// Path to extract new anime dump
    pub new_extract_path: String,
    /// Path to extract old anime dump
    pub old_extract_path: String,
}

/// Rpc services settings
#[derive(Debug, Clone, Deserialize)]
pub struct Rpc {
    /// URL to serve `ImportService`
    pub import: String,
    /// URL to serve `ScraperTasksService`
    pub task: String,
}

// MARK: impl Profile

impl Profile {
    fn files(&self) -> Vec<String> {
        let mut files = vec!["config/default.toml".to_string()];
        match self {
            Profile::Default => {}
            Profile::Named(name) => {
                let path = format!("config/{}.toml", name);
                files.push(path);
            }
        }

        files
    }
}

// MARK: impl Settings

impl Settings {
    pub fn new(profile: Profile) -> Result<Self, ConfigError> {
        let mut s = Config::new();

        for name in profile.files() {
            let file = File::with_name(&name);
            s.merge(file)?;
        }

        s.try_into()
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn import(&self) -> &Import {
        &self.import
    }

    pub fn rpc(&self) -> &Rpc {
        &self.rpc
    }
}

// MARK: impl Db

impl Db {
    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }

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

// MARK: impl Rpc

impl Rpc {
    pub fn import(&self) -> &str {
        &self.import
    }

    pub fn task(&self) -> &str {
        &self.task
    }
}

use config::{Config, ConfigError, File};
use serde::{Deserialize, Serialize};

use std::time::Duration;

/// Settings profile.
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

/// Application settings.
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// Database settings.
    db: Db,

    /// RPC services settings.
    rpc: Rpc,
}

/// Database settings.
#[derive(Debug, Clone, Deserialize)]
pub struct Db {
    /// Path to database.
    url: String,

    /// Number of maximum simultaneous connections.
    max_connections: u32,

    /// DB connection timeout.
    connection_timeout: u64,
}

/// Rpc services settings.
#[derive(Debug, Clone, Deserialize)]
pub struct Rpc {
    /// Port for serving RPC services.
    port: u32,
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

// MARK: impl Rpc

impl Rpc {
    pub fn port(&self) -> u32 {
        self.port
    }
}

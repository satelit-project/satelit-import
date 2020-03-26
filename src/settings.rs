mod template;

use config::{Config, ConfigError, File, FileFormat, FileSourceString};
use serde::{Deserialize, Serialize};

use std::time::Duration;

use template::TemplateConfig;

/// Settings profile.
///
/// All profiles are based on `Default`, that is, all settings from `Default`
/// profile will be available, but may be overridden, will be inherited
/// by other profiles.
#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    Default,
    Test(String),
}

/// Application settings.
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    /// Database settings.
    db: Db,

    /// RPC services settings.
    rpc: Rpc,

    /// External storage settings.
    storage: Option<Storage>,
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

/// Represents external S3-compatible storage settings.
#[derive(Debug, Clone, Deserialize)]
pub struct Storage {
    host: String,
    bucket: String,
    region: String,
    key: String,
    secret: String,
}

// MARK: impl Profile

impl Profile {
    fn files(&self) -> Result<Vec<File<FileSourceString>>, ConfigError> {
        let mut files = vec!["config/default.toml".to_string()];
        match self {
            Profile::Default => {}
            Profile::Test(name) => {
                let path = format!("config/tests/{}.toml", name);
                files.push(path);
            }
        }

        let rendered = files
            .into_iter()
            .map(|p| TemplateConfig::new(p).render())
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| ConfigError::Foreign(e))?
            .iter()
            .map(|r| File::from_str(&r, FileFormat::Toml))
            .collect();

        Ok(rendered)
    }
}

// MARK: impl Settings

impl Settings {
    pub fn new(profile: Profile) -> Result<Self, ConfigError> {
        let mut s = Config::new();
        for file in profile.files()? {
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

    pub fn storage(&self) -> Option<&Storage> {
        self.storage.as_ref()
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

// MARK: impl Storage

impl Storage {
    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn bucket(&self) -> &str {
        &self.bucket
    }

    pub fn region(&self) -> &str {
        &self.region
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }
}

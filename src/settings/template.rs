use serde::{Deserialize, Serialize};
use tinytemplate as tt;

use std::path::Path;
use std::fs;
use std::{io::Read, env};

/// Represents a template configuration file.
#[derive(Debug)]
pub struct TemplateConfig<P> {
    /// Path to the configuration file.
    path: P,

    /// Variables to substitute.
    env: Env,
}

/// Represents data to be substituted in a configuration file template.
#[derive(Debug, Serialize, Deserialize)]
pub struct Env {
    /// External storage configuration.
    storage: Option<Storage>,

    /// Database configuration.
    db: Option<Db>,
}

/// Represents external storage configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Storage {
    /// S3 host.
    host: String,

    /// S3 bucket name.
    bucket: String,

    /// AWS access key.
    key: String,

    /// AWS secret key.
    secret: String,
}

/// Represents database configuration.
#[derive(Debug, Serialize, Deserialize)]
pub struct Db {
    /// Database URL.
    url: String,
}

// MARK: impl ConfigFile

impl<P> TemplateConfig<P>
where
    P: AsRef<Path> + 'static
{
    /// Creates new configuration file.
    pub fn new(path: P) -> Self {
        Self::with_env(path, Env::default())
    }

    /// Creates new configuration file with custom environment.
    pub fn with_env(path: P, env: Env) -> Self {
        TemplateConfig { path, env }
    }

    /// Reads and renders configuration with environment data.
    pub fn render(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut tf = fs::File::open(&self.path)?;
        let mut raw = String::new();
        tf.read_to_string(&mut raw)?;

        let mut tmpl = tt::TinyTemplate::new();
        tmpl.add_template("cfg", &raw)?;

        Ok(tmpl.render("cfg", &self.env)?)
    }
}

// MARK: impl Env

impl Default for Env {
    fn default() -> Self {
        Env {
            storage: Storage::from_env(),
            db: Db::from_env(),
        }
    }
}

// MARK: impl Storage

impl Storage {
    fn from_env() -> Option<Self> {
        let host = env::var("DO_SPACES_HOST").ok()?;
        let bucket = env::var("DO_BUCKET").ok()?;
        let key = env::var("DO_SPACES_KEY").ok()?;
        let secret = env::var("DO_SPACES_SECRET").ok()?;

        Some(Storage { host, bucket, key, secret })
    }
}

// MARK: impl Db

impl Db {
    fn from_env() -> Option<Self> {
        let url = env::var("PG_DB_URL").ok()?;
        Some(Db { url })
    }
}

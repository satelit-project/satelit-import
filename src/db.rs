mod convert;
pub mod entity;
pub mod scheduled_tasks;
pub mod schedules;
pub mod schema;
pub mod tasks;

pub use diesel::r2d2::PoolError;
pub use diesel::result::Error as UnderlyingError;

use diesel::prelude::*;
use diesel::r2d2;
use lazy_static::lazy_static;

use std::fmt;

use crate::settings;

lazy_static! {
    static ref SHARED_POOL: ConnectionPool = {
        new_connection_pool(settings::shared().db()).expect("failed to escablish db connection")
    };
}

/// PostgresQL connection from connection pool
pub type PgPooledConnection = r2d2::PooledConnection<r2d2::ConnectionManager<PgConnection>>;

/// Database connection pool
#[derive(Clone)]
pub struct ConnectionPool(r2d2::Pool<r2d2::ConnectionManager<PgConnection>>);

/// Represents an error that may happen on querying db
#[derive(Debug)]
pub enum QueryError {
    /// Failed to acquire db connection from connection pool
    PoolFailed(PoolError),
    /// Failed to perform db query
    QueryFailed(UnderlyingError),
}

/// Returns global connection pool with global settings
pub fn connection_pool() -> ConnectionPool {
    SHARED_POOL.clone()
}

/// Creates new connection pool with global settings
///
/// There are should be only one connection pool per app
pub fn new_connection_pool(settings: &settings::Db) -> Result<ConnectionPool, PoolError> {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(settings.url());
    let pool = r2d2::Pool::builder()
        .max_size(settings.max_connections())
        .connection_timeout(settings.connection_timeout())
        .build(manager)?;

    pool.get()?
        .execute("PRAGMA foreign_keys = ON")
        .expect("Failed to enable foreign keys support");

    Ok(ConnectionPool(pool))
}

// MARK: impl ConnectionPool

impl ConnectionPool {
    pub fn get(&self) -> Result<PgPooledConnection, PoolError> {
        self.0.get()
    }
}

impl std::fmt::Debug for ConnectionPool {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "pg connection pool")
    }
}

// MARK: impl QueryError

impl From<PoolError> for QueryError {
    fn from(e: PoolError) -> Self {
        QueryError::PoolFailed(e)
    }
}

impl From<UnderlyingError> for QueryError {
    fn from(e: UnderlyingError) -> Self {
        QueryError::QueryFailed(e)
    }
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QueryError::*;

        match *self {
            PoolFailed(ref e) => <PoolError as fmt::Display>::fmt(&e, f),
            QueryFailed(ref e) => <UnderlyingError as fmt::Display>::fmt(&e, f),
        }
    }
}

impl std::error::Error for QueryError {}

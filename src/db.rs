mod convert;
pub mod entity;
pub mod queued_jobs;
pub mod schedules;
pub mod schema;
pub mod tasks;

pub use diesel::r2d2::PoolError;
pub use diesel::result::DatabaseErrorKind;
pub use diesel::result::Error as UnderlyingError;

use diesel::prelude::*;
use diesel::r2d2;

use std::fmt;

use crate::settings;

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

/// Creates new connection pool with global settings
///
/// There are should be only one connection pool per app
pub fn new_connection_pool(settings: &settings::Db) -> Result<ConnectionPool, PoolError> {
    let manager = r2d2::ConnectionManager::<PgConnection>::new(settings.url());
    let pool = r2d2::Pool::builder()
        .max_size(settings.max_connections())
        .connection_timeout(settings.connection_timeout())
        .build(manager)?;

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

impl QueryError {
    /// Returns error kind in case of database error
    ///
    /// If database returned an error when executing query then we can see what's
    /// exactly happened. If it's not a database error then `None` will be returned.
    pub fn database_error(&self) -> Option<&DatabaseErrorKind> {
        match self {
            QueryError::QueryFailed(UnderlyingError::DatabaseError(kind, _)) => Some(kind),
            _ => None,
        }
    }
}

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

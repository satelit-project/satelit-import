pub mod entity;
pub mod schema;
pub mod schedules;
mod convert;

pub use diesel::r2d2::PoolError;
pub use diesel::result::Error as UnderlyingError;

use diesel::prelude::*;
use diesel::r2d2;

use std::fmt;

use crate::settings;

/// Query that can be performed on database
pub trait Table<P: ConnectionPool> {
    fn execute<O, F>(&self, f: F) -> Result<O, QueryError>
        where F: Fn(&P::Connection) -> Result<O, QueryError>;
}

/// Represents an error that may happen on querying db
#[derive(Debug)]
pub enum QueryError {
    /// Failed to acquire db connection from connection pool
    PoolFailed(PoolError),
    /// Failed to perform db query
    QueryFailed(UnderlyingError),
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use QueryError::*;

        match self {
            &PoolFailed(ref e) => <PoolError as fmt::Display>::fmt(&e, f),
            &QueryFailed(ref e) => <UnderlyingError as fmt::Display>::fmt(&e, f),
        }
    }
}

impl std::error::Error for QueryError {}

/// Creates new connection pool with specified settings
///
/// There are should be only one connection pool per app
pub fn new_connection_pool() -> Result<impl ConnectionPool, PoolError> {
    let settings = settings::Settings::shared().db();
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(settings.path());
    let pool = r2d2::Pool::builder()
        .max_size(settings.max_connections())
        .connection_timeout(settings.connection_timeout())
        .build(manager)?;

    Ok(pool)
}

/// Connection pool for project's database
///
/// ## Note
///
/// `pool.clone()` should be used to pass connection pool around
pub trait ConnectionPool: Clone {
    type Connection: diesel::Connection<
        Backend=diesel::sqlite::Sqlite,
        TransactionManager=diesel::connection::AnsiTransactionManager
    >;

    fn get(&self) -> Result<Self::Connection, PoolError>;
}

impl ConnectionPool for r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>> {
    type Connection = r2d2::PooledConnection<r2d2::ConnectionManager<SqliteConnection>>;

    fn get(&self) -> Result<Self::Connection, PoolError> {
        r2d2::Pool::<r2d2::ConnectionManager<SqliteConnection>>::get(self)
    }
}

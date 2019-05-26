mod convert;
pub mod entity;
pub mod schedules;
pub mod schema;
pub mod tasks;

pub use diesel::r2d2::PoolError;
pub use diesel::result::Error as UnderlyingError;

use diesel::prelude::*;
use diesel::r2d2;

use std::fmt;
use std::sync::Once;

use crate::settings;

type SqlitePool = r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>;

/// Returns global connection pool with global settings
pub fn connection_pool() -> impl ConnectionPool {
    static mut SHARED: *const SqlitePool = 0 as *const SqlitePool;
    static ONCE: Once = Once::new();

    unsafe {
        ONCE.call_once(|| {
            let settings = settings::shared();
            let pool = new_r2d2_sqlite_pool(settings.db())
                .expect("failed to initialize db connection pool");
            SHARED = Box::into_raw(Box::new(pool))
        });

        (*SHARED).clone()
    }
}

/// Creates new connection pool with global settings
///
/// There are should be only one connection pool per app
pub fn new_connection_pool(settings: &settings::Db) -> Result<impl ConnectionPool, PoolError> {
    new_r2d2_sqlite_pool(settings)
}

/// Creates new connection pool with global settings
fn new_r2d2_sqlite_pool(settings: &settings::Db) -> Result<SqlitePool, PoolError> {
    let manager = r2d2::ConnectionManager::<SqliteConnection>::new(settings.path());
    let pool = r2d2::Pool::builder()
        .max_size(settings.max_connections())
        .connection_timeout(settings.connection_timeout())
        .build(manager)?;

    Ok(pool)
}

/// Represents a db table which can be queried
pub trait Table<P: ConnectionPool> {
    /// Returns a connection that should be used for DB access
    fn connection(&self) -> Result<P::Connection, PoolError>;

    /// Provides you with db connection to perform table query
    ///
    /// * f – closure to where db connection will be passes
    fn execute<O, F>(&self, f: F) -> Result<O, QueryError>
    where
        F: Fn(&P::Connection) -> Result<O, QueryError>,
    {
        f(&self.connection()?)
    }
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
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use QueryError::*;

        match self {
            &PoolFailed(ref e) => <PoolError as fmt::Display>::fmt(&e, f),
            &QueryFailed(ref e) => <UnderlyingError as fmt::Display>::fmt(&e, f),
        }
    }
}

impl std::error::Error for QueryError {}

/// Connection pool for project's database
///
/// ## Note
///
/// `pool.clone()` should be used to pass connection pool around
pub trait ConnectionPool: Clone {
    type Connection: diesel::Connection<
        Backend = diesel::sqlite::Sqlite,
        TransactionManager = diesel::connection::AnsiTransactionManager,
    >;

    fn get(&self) -> Result<Self::Connection, PoolError>;
}

impl ConnectionPool for r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>> {
    type Connection = r2d2::PooledConnection<r2d2::ConnectionManager<SqliteConnection>>;

    fn get(&self) -> Result<Self::Connection, PoolError> {
        r2d2::Pool::<r2d2::ConnectionManager<SqliteConnection>>::get(self)
    }
}

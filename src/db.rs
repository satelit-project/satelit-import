pub mod entity;
pub mod schema;
mod convert;

pub use diesel::r2d2::PoolError;

use diesel::prelude::*;
use diesel::r2d2;
use diesel::insert_into;
use diesel::query_dsl::RunQueryDsl;

use crate::settings;
use entity::NewSchedule;

/// Entity that represents *schedule* table in db
pub struct Schedules<P> where P: ConnectionPool {
    pool: P,
}

impl<P> Schedules<P> where P: ConnectionPool {
    /// Creates new instance with specified connection pool
    pub fn new(pool: P) -> Self {
        Schedules { pool }
    }
}

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
    type Connection: diesel::Connection;

    fn get(&self) -> Result<Self::Connection, PoolError>;
}

impl ConnectionPool for r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>> {
    type Connection = r2d2::PooledConnection<r2d2::ConnectionManager<SqliteConnection>>;

    fn get(&self) -> Result<Self::Connection, PoolError> {
        r2d2::Pool::<r2d2::ConnectionManager<SqliteConnection>>::get(self)
    }
}

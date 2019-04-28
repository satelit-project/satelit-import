mod schema;

pub use diesel::r2d2::PoolError;

use diesel::prelude::*;
use diesel::query_builder::*;
use diesel::sqlite::*;
use diesel::r2d2;

use crate::settings;

/// Connection pool for project's database
#[derive(Clone)]
pub struct Pool(r2d2::Pool<r2d2::ConnectionManager<SqliteConnection>>);

impl Pool {
    /// Creates new connection pool with specified settings
    ///
    /// There are should be only one connection pool per app
    pub fn new(settings: settings::Db) -> Result<Self, PoolError> {
        let manager = r2d2::ConnectionManager::<SqliteConnection>::new(settings.path());
        let pool = r2d2::Pool::builder()
            .max_size(settings.max_connections())
            .connection_timeout(settings.connection_timeout())
            .build(manager)?;

        Ok(Pool(pool))
    }

    pub fn get(&self) -> Result<PooledConnection, PoolError> {
        let conn = self.0.get()?;
        Ok(PooledConnection::new(conn))
    }
}

/// Sqlite connection that is managed by connection pool
type PooledSqliteConnection = r2d2::PooledConnection<r2d2::ConnectionManager<SqliteConnection>>;

/// Database connection that is managed by `Pool`
pub struct PooledConnection(PooledSqliteConnection);

impl PooledConnection {
    /// Creates new connection from private `PooledSqliteConnection`
    fn new(conn: PooledSqliteConnection) -> Self {
        PooledConnection(conn)
    }
}

impl diesel::connection::SimpleConnection for PooledConnection {
    fn batch_execute(&self, query: &str) -> QueryResult<()> {
        self.0.batch_execute(query)
    }
}

impl Connection for PooledConnection {
    type Backend = <PooledSqliteConnection as Connection>::Backend;
    type TransactionManager = <PooledSqliteConnection as Connection>::TransactionManager;

    fn establish(database_url: &str) -> ConnectionResult<Self> {
        match PooledSqliteConnection::establish(database_url) {
            Ok(conn) => Ok(PooledConnection::new(conn)),
            Err(e) => Err(e),
        }
    }

    fn execute(&self, query: &str) -> QueryResult<usize> {
        self.0.execute(query)
    }

    fn query_by_index<T, U>(&self, source: T) -> QueryResult<Vec<U>>
        where
            T: AsQuery,
            T::Query: QueryFragment<Self::Backend> + QueryId,
            Self::Backend: diesel::sql_types::HasSqlType<T::SqlType>,
            U: Queryable<T::SqlType, Self::Backend>,
    {
        self.0.query_by_index(source)
    }

    fn query_by_name<T, U>(&self, source: &T) -> QueryResult<Vec<U>>
        where
            T: QueryFragment<Self::Backend> + QueryId,
            U: diesel::deserialize::QueryableByName<Self::Backend>,
    {
        self.0.query_by_name(source)
    }

    fn execute_returning_count<T>(&self, source: &T) -> QueryResult<usize>
        where
            T: QueryFragment<Self::Backend> + QueryId,
    {
        self.0.execute_returning_count(source)
    }

    fn transaction_manager(&self) -> &Self::TransactionManager {
        self.0.transaction_manager()
    }
}

/// Entity that represents *schedule* table in db
pub struct Schedule {
    pool: Pool,
}

impl Schedule {
    /// Creates new instance with specifies connection pool
    pub fn new(pool: Pool) -> Self {
        Schedule { pool }
    }
}


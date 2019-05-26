use diesel::prelude::*;

use super::entity::Task;
use super::schema::tasks;
use super::{ConnectionPool, PoolError, QueryError, Table};

/// Represents *tasks* table that contains all created scrapping tasks
#[derive(Clone)]
pub struct Tasks<P: ConnectionPool> {
    pool: P,
}

impl<P: ConnectionPool> Tasks<P> {
    /// Creates new table instance
    pub fn new(pool: P) -> Self {
        Self { pool }
    }

    /// Inserts new task into DB
    pub fn insert(&self, task: &Task) -> Result<(), QueryError> {
        use self::tasks::dsl::*;

        let conn = self.connection()?;
        diesel::insert_into(tasks).values(task).execute(&conn)?;

        Ok(())
    }
}

impl<P: ConnectionPool> Table<P> for Tasks<P> {
    fn connection(&self) -> Result<<P as ConnectionPool>::Connection, PoolError> {
        self.pool.get()
    }
}

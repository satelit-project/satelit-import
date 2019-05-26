use diesel::prelude::*;

use super::entity::{ScheduledTask, Task};
use super::schema::scheduled_tasks;
use super::schema::schedules;
use super::{ConnectionPool, PoolError, QueryError, Table};

/// Represents *scheduled_tasks* table that contains mapping between a task and schedule
#[derive(Clone)]
pub struct ScheduledTasks<P: ConnectionPool> {
    pool: P,
}

impl<P: ConnectionPool> ScheduledTasks<P> {
    /// Creates new table instance
    pub fn new(pool: P) -> Self {
        Self { pool }
    }

    pub fn create(&self, task: &Task, count: i32) -> Result<(), QueryError> {
        use self::scheduled_tasks::dsl::*;
        use self::schedules::dsl as sdsl;

        let conn = self.connection()?;
        // TODO: insert

        Ok(())
    }
}

impl<P: ConnectionPool> Table<P> for ScheduledTasks<P> {
    fn connection(&self) -> Result<<P as ConnectionPool>::Connection, PoolError> {
        self.pool.get()
    }
}

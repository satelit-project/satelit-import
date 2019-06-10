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

    /// Registers new task in DB
    pub fn register(&self, task: &Task) -> Result<(), QueryError> {
        use self::tasks::dsl::*;

        let conn = self.connection()?;
        diesel::insert_into(tasks).values(task).execute(&conn)?;

        Ok(())
    }

    /// Returns a task for specified task `id`
    pub fn for_id(&self, task_id: &str) -> Result<Task, QueryError> {
        use self::tasks::dsl::*;

        let conn = self.connection()?;
        let result = tasks.find(task_id).get_result::<Task>(&conn)?;

        Ok(result)
    }

    /// Removes specified task's `id` and marks associated schedules entities as finished processing  
    pub fn remove(&self, task_id: &str) -> Result<(), QueryError> {
        use self::tasks::dsl::*;

        let conn = self.connection()?;
        diesel::delete(tasks.filter(id.eq(task_id))).execute(&conn)?;

        Ok(())
    }
}

impl<P: ConnectionPool> Table<P> for Tasks<P> {
    fn connection(&self) -> Result<<P as ConnectionPool>::Connection, PoolError> {
        self.pool.get()
    }
}

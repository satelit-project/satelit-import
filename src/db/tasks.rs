use diesel::prelude::*;

use super::entity::Uuid;
use super::entity::{ExternalSource, Task};
use super::schema::{queued_jobs, tasks};
use super::{ConnectionPool, QueryError};

/// Represents *tasks* table that contains all created scrapping tasks
#[derive(Clone)]
pub struct Tasks {
    pool: ConnectionPool,
}

impl Tasks {
    /// Creates new table instance
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Registers new scraping task for provided source
    pub fn register(&self, schedule_source: &ExternalSource) -> Result<Task, QueryError> {
        use self::tasks::dsl::*;

        let conn = self.pool.get()?;
        let task: Task = diesel::insert_into(tasks)
            .values(source.eq(schedule_source))
            .get_result(&conn)?;

        Ok(task)
    }

    /// Removes queued jobs for a task with specified id
    pub fn finish(&self, task_id: &Uuid) -> Result<(), QueryError> {
        use self::queued_jobs::dsl;

        let conn = self.pool.get()?;
        diesel::delete(dsl::queued_jobs.filter(dsl::task_id.eq(task_id))).execute(&conn)?;

        Ok(())
    }
}

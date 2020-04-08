use diesel::prelude::*;

use super::{
    entity::{ExternalSource, Task, Uuid},
    schema::{queued_jobs, tasks},
    ConnectionPool, QueryError,
};

/// Represents *tasks* table that contains all created scrapping tasks.
#[derive(Clone)]
pub struct Tasks {
    pool: ConnectionPool,
}

impl Tasks {
    /// Creates new table instance.
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Registers new scraping task for provided source.
    pub fn register(&self, schedule_source: ExternalSource) -> Result<Task, QueryError> {
        use self::tasks::dsl::*;

        let conn = self.pool.get()?;
        let task: Task = diesel::insert_into(tasks)
            .values(source.eq(schedule_source))
            .get_result(&conn)?;

        Ok(task)
    }

    /// Returns all unfinished tasks.
    pub fn unfinished(&self) -> Result<Vec<Task>, QueryError> {
        use self::tasks::dsl;

        let conn = self.pool.get()?;
        let tasks = dsl::tasks
            .filter(dsl::finished.eq(false))
            .load::<Task>(&conn)?;

        Ok(tasks)
    }

    /// Removes queued jobs for a task with specified id.
    pub fn finish(&self, task_id: &Uuid) -> Result<(), QueryError> {
        use self::{queued_jobs::dsl as q, tasks::dsl as t};

        let conn = self.pool.get()?;
        diesel::delete(q::queued_jobs.filter(q::task_id.eq(task_id))).execute(&conn)?;
        diesel::update(t::tasks.find(task_id))
            .set(t::finished.eq(true))
            .execute(&conn)?;

        Ok(())
    }
}

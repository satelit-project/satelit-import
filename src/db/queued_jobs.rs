use diesel::prelude::*;

use super::{
    entity::{QueuedJob, Schedule, Uuid},
    schema::{queued_jobs, schedules},
    ConnectionPool, QueryError,
};

/// Represents *queued_jobs* table that contains mapping between a task and schedule
#[derive(Clone)]
pub struct QueuedJobs {
    pool: ConnectionPool,
}

impl QueuedJobs {
    /// Creates new table instance
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Binds `task` with provided id with pending schedules from `schedules` table
    pub fn bind(&self, task_id: &Uuid, count: i32) -> Result<(), QueryError> {
        let conn = self.pool.get()?;

        let sql = r#"
        select queued_jobs_bind_schedules_for_task($1, $2)
        "#;

        diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(task_id)
            .bind::<diesel::sql_types::Integer, _>(count)
            .execute(&conn)?;

        Ok(())
    }

    /// Returns all scheduled jobs associated with `task`
    pub fn jobs_for_task_id(
        &self,
        task_id: &Uuid,
    ) -> Result<Vec<(QueuedJob, Schedule)>, QueryError> {
        use self::queued_jobs::dsl;

        let conn = self.pool.get()?;
        let result = dsl::queued_jobs
            .filter(dsl::task_id.eq(task_id))
            .inner_join(self::schedules::table)
            .load::<(QueuedJob, Schedule)>(&conn)?;

        Ok(result)
    }

    /// Removes queued job with specified ID and returns it
    pub fn pop(&self, job_id: &Uuid) -> Result<QueuedJob, QueryError> {
        use self::queued_jobs::dsl::*;

        let conn = self.pool.get()?;
        let job = diesel::delete(queued_jobs.find(job_id))
            .returning(queued_jobs::all_columns())
            .get_result(&conn)?;

        Ok(job)
    }
}

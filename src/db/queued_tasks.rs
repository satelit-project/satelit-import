use diesel::prelude::*;
use uuid::Uuid;

use super::entity::{Schedule, QueuedTask};
use super::schema::{queued_tasks, schedules};
use super::{ConnectionPool, QueryError};

/// Represents *queued_tasks* table that contains mapping between a task and schedule
#[derive(Clone)]
pub struct QueuedTasks {
    pool: ConnectionPool,
}

impl QueuedTasks {
    /// Creates new table instance
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Binds `task` with provided id with pending schedules from `schedules` table
    pub fn bind(&self, task_id: &Uuid, count: i32) -> Result<(), QueryError> {
        let conn = self.pool.get()?;

        let sql = r#"
        insert into queued_tasks (task_id, schedule_id)
        select ?, schedules.id from schedules
        where schedules.state = 0 and schedules.next_update_at <= now()
        order by schedules.priority desc 
        limit ?;
        "#;

        diesel::sql_query(sql)
            .bind::<diesel::sql_types::Uuid, _>(task_id)
            .bind::<diesel::sql_types::Integer, _>(count)
            .execute(&conn)?;

        Ok(())
    }

    /// Returns all scheduled tasks associated with `task`
    pub fn for_task_id(&self, task_id: &Uuid) -> Result<Vec<(QueuedTask, Schedule)>, QueryError> {
        use self::queued_tasks::dsl::*;

        let conn = self.pool.get()?;
        let result = queued_tasks
            .filter(task_id.eq(task_id))
            .inner_join(self::schedules::table)
            .load::<(QueuedTask, Schedule)>(&conn)?;

        Ok(result)
    }

    pub fn complete_for_schedule(&self, task_id: &Uuid, schedule_id: i32) -> Result<(), QueryError> {
        use self::queued_tasks::dsl;

        let conn = self.pool.get()?;
        let target = dsl::queued_tasks
            .filter(dsl::task_id.eq(task_id))
            .filter(dsl::schedule_id.eq(schedule_id));
        diesel::delete(target).execute(&conn)?;

        Ok(())
    }
}

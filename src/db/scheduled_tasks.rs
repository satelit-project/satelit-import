use diesel::prelude::*;

use super::entity::{Schedule, ScheduledTask, Task};
use super::schema::{scheduled_tasks, schedules};
use super::{ConnectionPool, QueryError};

/// Represents *scheduled_tasks* table that contains mapping between a task and schedule
#[derive(Clone)]
pub struct ScheduledTasks {
    pool: ConnectionPool,
}

impl ScheduledTasks {
    /// Creates new table instance
    pub fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }

    /// Binds provided `task` with pending schedules from `schedules` table
    ///
    /// It will choose the first `count` anime titles with `Pending` state and sorted
    /// by priority. You can retrieve  
    pub fn create(&self, task: &Task, count: i32) -> Result<(), QueryError> {
        let conn = self.pool.get()?;

        let sql = r#"
        insert into scheduled_tasks (task_id, schedule_id)
        select ?, schedules.id from schedules
        where schedules.state = 0
        order by schedules.priority desc 
        limit ?;
        "#;

        diesel::sql_query(sql)
            .bind::<diesel::sql_types::Text, _>(&task.id)
            .bind::<diesel::sql_types::Integer, _>(count)
            .execute(&conn)?;

        Ok(())
    }

    /// Returns all scheduled tasks associated with `task`
    pub fn for_task(&self, task: &Task) -> Result<Vec<(ScheduledTask, Schedule)>, QueryError> {
        use self::scheduled_tasks::dsl::*;

        let conn = self.pool.get()?;
        let result = scheduled_tasks
            .filter(task_id.eq(&task.id))
            .inner_join(self::schedules::table)
            .load::<(ScheduledTask, Schedule)>(&conn)?;

        Ok(result)
    }

    pub fn complete_for_schedule(&self, task_id: &str, schedule_id: i32) -> Result<(), QueryError> {
        use self::scheduled_tasks::dsl;

        let conn = self.pool.get()?;
        let target = dsl::scheduled_tasks
            .filter(dsl::task_id.eq(task_id))
            .filter(dsl::schedule_id.eq(schedule_id));
        diesel::delete(target).execute(&conn)?;

        Ok(())
    }
}

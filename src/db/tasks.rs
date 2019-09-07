use diesel::prelude::*;

use super::entity::{Task, ExternalSource};
use super::schema::tasks;
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
            .values(source.eq(schedule_source)).get_result(&conn)?;

        Ok(task)
    }
}

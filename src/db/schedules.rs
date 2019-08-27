use diesel::prelude::*;

use super::entity::{SourceSchedule, UpdatedSchedule};
use super::{ConnectionPool, QueryError};

/// Entity that represents *schedule* table in db
#[derive(Clone)]
pub struct Schedules {
    /// Db connection pool
    pool: ConnectionPool,
}

impl Schedules {
    pub fn new(pool: ConnectionPool) -> Self {
        Schedules { pool }
    }

    pub fn create_from_source(&self, src: &SourceSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.pool.get()?;
        diesel::insert_into(schedules).values(src)
            .on_conflict((sourced_id, source))
            .do_update()
            .execute(&conn)?;

        Ok(())
    }

    pub fn delete_from_source(&self, src: &SourceSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.pool.get()?;
        let target = schedules
            .filter(sourced_id.eq(src.sourced_id))
            .filter(source.eq(src.source));
        diesel::delete(target).execute(&conn)?;

        Ok(())
    }

    pub fn update_for_id(
        &self,
        schedule_id: i32,
        updated: &UpdatedSchedule,
    ) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.connection()?;
        let target = schedules.find(schedule_id);
        diesel::update(target).set(updated).execute(&conn)?;

        Ok(())
    }
}

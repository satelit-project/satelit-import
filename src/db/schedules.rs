use diesel::prelude::*;

use super::{
    entity::{NewSchedule, UpdatedSchedule},
    ConnectionPool, QueryError,
};

/// Entity that represents *schedule* table in db
#[derive(Debug, Clone)]
pub struct Schedules {
    /// Db connection pool
    pool: ConnectionPool,
}

impl Schedules {
    pub fn new(pool: ConnectionPool) -> Self {
        Schedules { pool }
    }

    pub fn put(&self, src: &NewSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.pool.get()?;
        diesel::insert_into(schedules)
            .values(src)
            .on_conflict((external_id, source))
            .do_nothing()
            .execute(&conn)?;

        Ok(())
    }

    pub fn pop(&self, src: &NewSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.pool.get()?;
        let target = schedules
            .filter(external_id.eq(src.external_id))
            .filter(source.eq(src.source));
        diesel::delete(target).execute(&conn)?;

        Ok(())
    }

    pub fn update(&self, schedule_id: i32, updated: &UpdatedSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.pool.get()?;
        diesel::update(schedules.find(schedule_id))
            .set(updated)
            .execute(&conn)?;

        Ok(())
    }
}

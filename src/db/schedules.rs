use diesel::prelude::*;

use super::entity::{NewSchedule, ExternalSource, UpdatedSchedule};
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

    pub fn put(&self, src: &NewSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.pool.get()?;
        diesel::insert_into(schedules).values(src)
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
        diesel::delete(target).execute(&conn)?; // TODO: test deleting non-existent row

        Ok(())
    }

    pub fn update(
        &self,
        schedule_id: i32,
        schedule_source: ExternalSource,
        updated: &UpdatedSchedule,
    ) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.pool.get()?;
        let target = schedules
            .filter(external_id.eq(schedule_id))
            .filter(source.eq(schedule_source));
        diesel::update(target).set(updated).execute(&conn)?;

        Ok(())
    }
}

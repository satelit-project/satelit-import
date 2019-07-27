use diesel::prelude::*;

use super::entity::{SourceSchedule, UpdatedSchedule};
use super::{ConnectionPool, PoolError, QueryError, Table};

/// Entity that represents *schedule* table in db
#[derive(Clone)]
pub struct Schedules<P> {
    /// Db connection pool
    pool: P,
}

impl<P: ConnectionPool> Schedules<P> {
    pub fn new(pool: P) -> Self {
        Schedules { pool }
    }

    pub fn create_from_source(&self, src: &SourceSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.connection()?;
        diesel::replace_into(schedules).values(src).execute(&conn)?;

        Ok(())
    }

    pub fn delete_from_source(&self, src: &SourceSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.connection()?;
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

impl<P: ConnectionPool> Table<P> for Schedules<P> {
    fn connection(&self) -> Result<<P as ConnectionPool>::Connection, PoolError> {
        self.pool.get()
    }
}

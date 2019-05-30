use diesel::prelude::*;

use super::entity::{Schedule, SourceSchedule, UpdatedSchedule};
use super::{ConnectionPool, PoolError, QueryError, Table};

/// Entity that represents *schedule* table in db
#[derive(Clone)]
pub struct Schedules<P: ConnectionPool> {
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
        diesel::insert_into(schedules).values(src).execute(&conn)?;

        Ok(())
    }

    pub fn delete_from_source(&self, src: &SourceSchedule) -> Result<(), QueryError> {
        use crate::db::schema::schedules::dsl::*;

        let conn = self.connection()?;
        let target = schedules
            .filter(source_id.eq(src.source_id))
            .filter(source.eq(src.source));
        diesel::delete(target).execute(&conn)?;

        Ok(())
    }

    pub fn update(&self, update: &UpdatedSchedule) -> Result<(), QueryError> {
        Ok(())
    }
}

impl<P: ConnectionPool> Table<P> for Schedules<P> {
    fn connection(&self) -> Result<<P as ConnectionPool>::Connection, PoolError> {
        self.pool.get()
    }
}

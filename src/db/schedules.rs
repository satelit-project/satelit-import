use super::{ConnectionPool, Table, QueryError};

/// Entity that represents *schedule* table in db
pub struct Schedules<P: ConnectionPool> {
    /// Db connection pool
    pool: P,
}

impl<P: ConnectionPool> Schedules<P> {
    pub fn new(pool: P) -> Self {
        Schedules { pool }
    }
}

impl<P: ConnectionPool> Table<P> for Schedules<P> {
    fn execute<O, F>(&self, f: F) -> Result<O, QueryError>
        where F: Fn(&P::Connection) -> Result<O, QueryError>
    {
        f(&self.pool.get()?)
    }
}

#[macro_export]
macro_rules! schedules_insert {
    ( $sched:expr, $value:expr ) => {
        {
            use diesel::prelude::*;
            use crate::db::schema::schedules::dsl::*;

            $sched.execute(|conn| {
                diesel::insert_into(schedules)
                    .values($value)
                    .execute(conn)?;

                Ok(())
            })
        }
    };
}

#[macro_export]
macro_rules! schedules_delete {
    ( $sched:expr, anidb_id($id:expr) ) => {
        {
            use diesel::prelude::*;
            use crate::db::schema::schedules::dsl::*;

            $sched.execute(|conn| {
                diesel::delete(schedules.filter(anidb_id.eq($id)))
                    .execute(conn)?;

                Ok(())
            })
        }
    };
}

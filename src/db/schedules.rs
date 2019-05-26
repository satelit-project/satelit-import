use super::{ConnectionPool, PoolError, Table};

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
}

impl<P: ConnectionPool> Table<P> for Schedules<P> {
    fn connection(&self) -> Result<<P as ConnectionPool>::Connection, PoolError> {
        self.pool.get()
    }
}

#[macro_export]
macro_rules! schedules_insert {
    ( $sched:expr, $value:expr ) => {{
        use crate::db::schema::schedules::dsl::*;
        use diesel::prelude::*;

        $sched.execute(|conn| {
            diesel::insert_into(schedules)
                .values($value)
                .execute(conn)?;

            Ok(())
        })
    }};
}

#[macro_export]
macro_rules! schedules_delete {
    ( $sched:expr, anidb_id($id:expr) ) => {{
        use crate::db::schema::schedules::dsl::*;
        use diesel::prelude::*;

        $sched.execute(|conn| {
            diesel::delete(schedules.filter(anidb_id.eq($id))).execute(conn)?;

            Ok(())
        })
    }};
}

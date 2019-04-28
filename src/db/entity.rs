use diesel::deserialize::FromSql;
use diesel::serialize::ToSql;
use diesel::sql_types::Integer;
use diesel::sqlite::Sqlite;
use diesel::backend::Backend;

use std::io::Write;

/// Represents scheduled anidb item import
#[derive(Queryable)]
pub struct Schedule {
    pub id: i32,
    pub anidb_id: i32,
    pub state: ScheduleState,
    pub data_mask1: u32,
    pub created_at: f64,
    pub updated_at: f64,
}

/// Represents state of a schedule
#[sql_type = "Integer"]
#[derive(Debug, Clone, Copy, PartialEq, FromSqlRow, AsExpression)]
pub enum ScheduleState {
    Pending = 0,
    Processing = 1,
    Finished = 2
}

// Conversion from ScheduleState to Integer (SQLite)
impl ToSql<Integer, Sqlite> for ScheduleState {
    fn to_sql<W: Write>(&self, out: &mut diesel::serialize::Output<W, Sqlite>) -> diesel::serialize::Result {
        ToSql::<Integer, Sqlite>::to_sql(&(*self as i32), out)
    }
}

// Conversion from Integer (SQLite) to ScheduleState
impl FromSql<Integer, Sqlite> for ScheduleState {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> diesel::deserialize::Result<Self> {
        use ScheduleState::*;

        let value: i32 = FromSql::<Integer, Sqlite>::from_sql(bytes)?;
        match value {
            0 => Ok(Pending),
            1 => Ok(Processing),
            2 => Ok(Finished),
            _ => Err(format!("Unrecognized ScheduleState raw value: {}", value).into())
        }
    }
}

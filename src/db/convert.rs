use diesel::deserialize::FromSql;
use diesel::serialize::ToSql;
use diesel::sql_types::Integer;
use diesel::sqlite::Sqlite;
use diesel::backend::Backend;

use std::io::Write;

use super::entity::*;

// Conversion from ScheduleState
impl ToSql<Integer, Sqlite> for ScheduleState {
    fn to_sql<W: Write>(&self, out: &mut diesel::serialize::Output<'_, W, Sqlite>) -> diesel::serialize::Result {
        ToSql::<Integer, Sqlite>::to_sql(&(*self as i32), out)
    }
}

// Conversion to ScheduleState
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

// Conversion from SchedulePriority
impl ToSql<Integer, Sqlite> for SchedulePriority {
    fn to_sql<W: Write>(&self, out: &mut diesel::serialize::Output<'_, W, Sqlite>) -> diesel::serialize::Result {
        ToSql::<Integer, Sqlite>::to_sql(&(*self as i32), out)
    }
}

// Conversion to SchedulePriority
impl FromSql<Integer, Sqlite> for SchedulePriority {
    fn from_sql(bytes: Option<&<Sqlite as Backend>::RawValue>) -> diesel::deserialize::Result<Self> {
        use SchedulePriority::*;

        let value: i32 = FromSql::<Integer, Sqlite>::from_sql(bytes)?;
        match value {
            0 => Ok(Idle),
            1_000_000 => Ok(New),
            _ => Err(format!("Unrecognized SchedulePriority raw value: {}", value).into())
        }
    }
}

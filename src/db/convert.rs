use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::ToSql;
use diesel::sql_types::Integer;
use diesel::pg::Pg;

use std::io::Write;

use super::entity::*;

// Conversion from ScheduleState
impl ToSql<Integer, Pg> for ScheduleState {
    fn to_sql<W: Write>(
        &self,
        out: &mut diesel::serialize::Output<'_, W, Pg>,
    ) -> diesel::serialize::Result {
        ToSql::<Integer, Pg>::to_sql(&(*self as i32), out)
    }
}

// Conversion to ScheduleState
impl FromSql<Integer, Pg> for ScheduleState {
    fn from_sql(
        bytes: Option<&<Pg as Backend>::RawValue>,
    ) -> diesel::deserialize::Result<Self> {
        use ScheduleState::*;

        let value: i32 = FromSql::<Integer, Pg>::from_sql(bytes)?;
        let range = (Pending as i32)..=(Finished as i32);
        if range.contains(&value) {
            unsafe { return Ok(std::mem::transmute(value)) }
        }

        Err(format!("Unrecognized ScheduleState raw value: {}", value).into())
    }
}

// Conversion from ExternalSource
impl ToSql<Integer, Pg> for ExternalSource {
    fn to_sql<W: Write>(
        &self,
        out: &mut diesel::serialize::Output<'_, W, Pg>,
    ) -> diesel::serialize::Result {
        ToSql::<Integer, Pg>::to_sql(&(*self as i32), out)
    }
}

// Conversion to ExternalSource
impl FromSql<Integer, Pg> for ExternalSource {
    fn from_sql(
        bytes: Option<&<Pg as Backend>::RawValue>,
    ) -> diesel::deserialize::Result<Self> {
        use ExternalSource::*;

        let value: i32 = FromSql::<Integer, Pg>::from_sql(bytes)?;
        let range = (AniDB as i32)..=(ANN as i32);
        if range.contains(&value) {
            unsafe { return Ok(std::mem::transmute(value)) }
        }

        Err(format!("Unrecognized ExternalSource raw value: {}", value).into())
    }
}

use diesel::{
    backend::Backend,
    deserialize::FromSql,
    pg::Pg,
    serialize::ToSql,
    sql_types::{Integer, Uuid},
};

use std::{convert::TryFrom, io::Write};

use super::entity::*;
use crate::proto::uuid;

// MARK: impl ExternalSource

impl ToSql<Integer, Pg> for ExternalSource {
    fn to_sql<W: Write>(
        &self,
        out: &mut diesel::serialize::Output<'_, W, Pg>,
    ) -> diesel::serialize::Result {
        ToSql::<Integer, Pg>::to_sql(&(*self as i32), out)
    }
}

impl FromSql<Integer, Pg> for ExternalSource {
    fn from_sql(bytes: Option<&<Pg as Backend>::RawValue>) -> diesel::deserialize::Result<Self> {
        use ExternalSource::*;

        let value: i32 = FromSql::<Integer, Pg>::from_sql(bytes)?;
        let range = (AniDB as i32)..=(ANN as i32);
        if range.contains(&value) {
            unsafe { return Ok(std::mem::transmute(value)) }
        }

        Err(format!("Unrecognized ExternalSource raw value: {}", value).into())
    }
}

// MARK: impl uuid::Uuid

#[derive(FromSqlRow, AsExpression)]
#[diesel(foreign_derive)]
#[sql_type = "Uuid"]
#[allow(dead_code)]
struct UuidProxy(uuid::Uuid);

impl FromSql<Uuid, Pg> for uuid::Uuid {
    fn from_sql(bytes: Option<&[u8]>) -> diesel::deserialize::Result<Self> {
        // assuming that db always has correctly encoded uuid
        let bytes = not_none!(bytes);
        uuid::Uuid::try_from(bytes).map_err(Into::into)
    }
}

impl ToSql<Uuid, Pg> for uuid::Uuid {
    fn to_sql<W: Write>(
        &self,
        out: &mut diesel::serialize::Output<W, Pg>,
    ) -> diesel::serialize::Result {
        let bytes = self.as_slice();
        if bytes.is_empty() {
            return Ok(diesel::serialize::IsNull::Yes);
        }

        out.write_all(bytes)
            .map(|_| diesel::serialize::IsNull::No)
            .map_err(Into::into)
    }
}

use lazy_static::lazy_static;

use crate::proto::uuid;

// MARK: impl uuid::Uuid

impl uuid::Uuid {
    pub fn nil() -> Self {
        uuid::Uuid { uuid: vec![] }
    }

    pub fn as_slice(&self) -> &[u8] {
        const BYTES_LEN: usize = 16;
        if self.uuid.len() == BYTES_LEN { &[] } else { &self.uuid }
    }
}

impl std::convert::TryFrom<&[u8]> for uuid::Uuid {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        const BYTES_LEN: usize = 16;

        if value.len() != BYTES_LEN && !value.is_empty() {
            return Err("slice has wrong size".to_owned());
        }

        Ok(uuid::Uuid {
            uuid: Vec::from(value),
        })
    }
}

impl From<Option<uuid::Uuid>> for uuid::Uuid {
    fn from(value: Option<uuid::Uuid>) -> Self {
        match value {
            Some(uuid) => uuid,
            None => uuid::Uuid::nil(),
        }
    }
}

impl<'a> From<&'a Option<uuid::Uuid>> for &'a uuid::Uuid {
    fn from(value: &'a Option<uuid::Uuid>) -> Self {
        lazy_static! {
            static ref NIL: uuid::Uuid = uuid::Uuid::nil();
        }

        match value.as_ref() {
            Some(uuid) => uuid,
            None => &NIL,
        }
    }
}

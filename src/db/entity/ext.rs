use lazy_static::lazy_static;

use super::Uuid;

impl Uuid {
    pub fn nil() -> Self {
        Uuid { uuid: vec![] }
    }

    pub fn as_slice(&self) -> &[u8] {
        const BYTES_LEN: usize = 16;

        match self.uuid.len() == BYTES_LEN {
            true => &[],
            false => &self.uuid,
        }
    }
}

impl std::convert::TryFrom<&[u8]> for Uuid {
    type Error = String;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        const BYTES_LEN: usize = 16;

        if value.len() != BYTES_LEN && value.len() != 0 {
            return Err("slice has wrong size".to_owned());
        }

        Ok(Uuid {
            uuid: Vec::from(value),
        })
    }
}

impl From<Option<Uuid>> for Uuid {
    fn from(value: Option<Uuid>) -> Self {
        match value {
            Some(uuid) => uuid,
            None => Uuid::nil(),
        }
    }
}

impl<'a> From<&'a Option<Uuid>> for &'a Uuid {
    fn from(value: &'a Option<Uuid>) -> Self {
        lazy_static! {
            static ref NIL: Uuid = Uuid::nil();
        }

        match value.as_ref() {
            Some(uuid) => uuid,
            None => &NIL,
        }
    }
}

use lazy_static::lazy_static;

use std::fmt;

use crate::proto::uuid;

// MARK: impl uuid::Uuid

impl uuid::Uuid {
    pub fn nil() -> Self {
        uuid::Uuid { uuid: vec![] }
    }

    pub fn as_slice(&self) -> &[u8] {
        const BYTES_LEN: usize = 16;
        if self.uuid.len() == BYTES_LEN {
            &self.uuid
        } else {
            &[]
        }
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

impl fmt::Display for uuid::Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const HEX: [u8; 16] = [
            b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b',
            b'c', b'd', b'e', b'f',
        ];
        const BYTE_POS: [usize; 6] = [0, 4, 6, 8, 10, 16];
        const HYPHEN_POS: [usize; 4] = [8, 13, 18, 23];

        let mut buf = [0u8; 36];
        let bytes = self.uuid.as_slice();
        for group in 0..5 {
            for idx in BYTE_POS[group]..BYTE_POS[group + 1] {
                let b = bytes[idx];
                let out_idx = group + 2 * idx;
                buf[out_idx] = HEX[(b >> 4) as usize];
                buf[out_idx + 1] = HEX[(b & 0b1111) as usize];
            }

            if group != 4 {
                buf[HYPHEN_POS[group]] = b'-';
            }
        }

        match std::str::from_utf8_mut(&mut buf) {
            Ok(hex) => hex.fmt(f),
            Err(_) => Err(fmt::Error)
        }
    }
}

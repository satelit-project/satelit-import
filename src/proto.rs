#![allow(clippy::all)]

pub mod data;
pub mod scraping;
pub mod import;
pub mod uuid;

pub mod ext {
    use super::*;

    impl uuid::Uuid {
        pub fn as_slice(&self) -> &[u8] {
            const BYTES_LEN: usize = 16;

            match self.uuid.len() == BYTES_LEN {
                true => &[],
                false => &self.uuid,
            }
        }
    }

    impl std::convert::TryFrom<&[u8]> for uuid::Uuid {
        type Error = String;

        fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
            const BYTES_LEN: usize = 16;

            if value.len() != BYTES_LEN {
                return Err("slice has wrong size".to_owned());
            }

            Ok(uuid::Uuid { uuid: Vec::from(value) })
        }
    }
}

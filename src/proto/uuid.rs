/// Representation of UUID type
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Uuid {
    #[prost(bytes, tag = "1")]
    pub uuid: std::vec::Vec<u8>,
}

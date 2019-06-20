/// Asks to import anime titles index and schedule new titles for scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntent {
    /// Intent ID
    #[prost(string, tag="1")]
    pub id: std::string::String,
    /// Represents an external DB from where anime titles index should be imported
    #[prost(enumeration="import_intent::Source", tag="2")]
    pub source: i32,
    /// Identifiers of anime titles that should be re-imported
    #[prost(sint32, repeated, tag="3")]
    pub reimport_ids: ::std::vec::Vec<i32>,
    /// URL to send request with import result
    #[prost(string, tag="4")]
    pub callback_url: std::string::String,
}
pub mod import_intent {
    /// External DB
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Source {
        Anidb = 0,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntentResult {
    /// Intent ID
    #[prost(string, tag="1")]
    pub id: std::string::String,
    /// If import succeeded then `true`, `false` otherwise
    #[prost(bool, tag="2")]
    pub succeeded: bool,
    /// IDs of anime titles that was not imported
    #[prost(sint32, repeated, tag="3")]
    pub skipped_ids: ::std::vec::Vec<i32>,
    /// Description of the error if import failed
    #[prost(string, tag="4")]
    pub error_description: std::string::String,
}

/// Asks to import anime titles index and schedule new titles for scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImportIntent {
    /// Represents an external DB from where anime titles index should be imported
    #[prost(enumeration="import_intent::Source", tag="1")]
    pub source: i32,
    /// URL to send request with import result
    #[prost(string, tag="2")]
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
    /// If import succeeded then `true`, `false` otherwise
    #[prost(bool, tag="1")]
    pub succeeded: bool,
    /// Description of the error if import failed
    #[prost(string, tag="2")]
    pub error_description: std::string::String,
}

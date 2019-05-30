/// Represents a task for anime pages scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Task {
    /// Task ID
    #[prost(string, tag = "1")]
    pub id: std::string::String,
    /// Type of an anime info source to parse from
    #[prost(enumeration = "task::Source", tag = "2")]
    pub source: i32,
    /// Schedule IDs for each anime ID
    #[prost(sint32, repeated, tag = "3")]
    pub schedule_ids: ::std::vec::Vec<i32>,
    /// Anime ID's to scrape
    #[prost(sint32, repeated, tag = "4")]
    pub anime_ids: ::std::vec::Vec<i32>,
}
pub mod task {
    /// External DB where an anime info resides
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Source {
        Unknown = 0,
        Anidb = 1,
    }
}
/// Intermediate result of a parse task
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskYield {
    /// ID of the related task
    #[prost(string, tag = "1")]
    pub task_id: std::string::String,
    /// ID of the schedule
    #[prost(sint32, tag = "2")]
    pub schedule_id: i32,
    /// Parsed anime entity
    #[prost(message, optional, tag = "3")]
    pub anime: ::std::option::Option<super::anime::Anime>,
}
/// Signals that a task has been finished
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskFinish {
    /// ID of the related task
    #[prost(string, tag = "1")]
    pub task_id: std::string::String,
}

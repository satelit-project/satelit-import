/// Represents a task for anime pages scraping
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Task {
    /// Task id
    #[prost(string, tag="1")]
    pub id: std::string::String,
    /// Type of an anime info source to parse from
    #[prost(enumeration="task::Source", tag="2")]
    pub source: i32,
    /// Anime ids
    #[prost(sint32, repeated, tag="3")]
    pub anime_ids: ::std::vec::Vec<i32>,
}
pub mod task {
    /// External DB where an anime info resides
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Source {
        Anidb = 0,
    }
}
/// Intermediate result of a parse task
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskYield {
    /// ID of the related task
    #[prost(string, tag="1")]
    pub task_id: std::string::String,
    /// Parsed anime entities
    #[prost(message, repeated, tag="2")]
    pub anime: ::std::vec::Vec<super::anime::Anime>,
}
/// Signals that a task has been finished
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TaskFinish {
    /// ID of the related task
    #[prost(string, tag="1")]
    pub task_id: std::string::String,
}

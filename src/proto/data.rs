/// External data source
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Source {
    Unknown = 0,
    Anidb = 1,
}
/// Anime episode
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Episode {
    /// Episode type
    #[prost(enumeration = "episode::Type", tag = "1")]
    pub r#type: i32,
    /// Episode number
    #[prost(sint32, tag = "2")]
    pub number: i32,
    /// Episode name
    #[prost(string, tag = "3")]
    pub name: std::string::String,
    /// Episode duration in seconds
    #[prost(double, tag = "4")]
    pub duration: f64,
    /// Timestamp of the episode air date (unix time)
    #[prost(sint64, tag = "5")]
    pub air_date: i64,
}
pub mod episode {
    /// Type of an anime episode
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unknown = 0,
        Regular = 1,
        Special = 2,
    }
}
/// Anime title
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Anime {
    /// Anime ids in external databases
    #[prost(message, optional, tag = "1")]
    pub source: ::std::option::Option<anime::Source>,
    /// Anime type
    #[prost(enumeration = "anime::Type", tag = "2")]
    pub r#type: i32,
    /// Canonical anime title in romaji
    #[prost(string, tag = "3")]
    pub title: std::string::String,
    /// URL of the anime poster
    #[prost(string, tag = "4")]
    pub poster_url: std::string::String,
    /// Number of the anime episodes
    #[prost(sint32, tag = "5")]
    pub episodes_count: i32,
    /// Known anime episodes info
    #[prost(message, repeated, tag = "6")]
    pub episodes: ::std::vec::Vec<Episode>,
    /// Timestamp of the anime start air date (unix)
    #[prost(sint64, tag = "7")]
    pub start_date: i64,
    /// Timestamp of the anime end air date (unix)
    #[prost(sint64, tag = "8")]
    pub end_date: i64,
    /// Anime tags (same as genre in some external sources)
    #[prost(message, repeated, tag = "9")]
    pub tags: ::std::vec::Vec<anime::Tag>,
    /// Anime rating
    #[prost(double, tag = "10")]
    pub rating: f64,
    /// Anime description
    #[prost(string, tag = "11")]
    pub description: std::string::String,
    /// Creation date in external DB
    #[prost(sint64, tag = "12")]
    pub src_created_at: i64,
    /// Last update date in external DB
    #[prost(sint64, tag = "13")]
    pub src_updated_at: i64,
}
pub mod anime {
    /// External DB location
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Source {
        /// AniDB id
        #[prost(sint32, repeated, tag = "1")]
        pub anidb_ids: ::std::vec::Vec<i32>,
        /// MyAnimeList id
        #[prost(sint32, repeated, tag = "2")]
        pub mal_ids: ::std::vec::Vec<i32>,
        /// AnimeNewsNetwork id
        #[prost(sint32, repeated, tag = "3")]
        pub ann_ids: ::std::vec::Vec<i32>,
    }
    /// Anime tag
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct Tag {
        /// Tag name
        #[prost(string, tag = "1")]
        pub name: std::string::String,
        /// Tag description
        #[prost(string, tag = "2")]
        pub description: std::string::String,
        /// Tag id in external db
        #[prost(oneof = "tag::Source", tags = "10")]
        pub source: ::std::option::Option<tag::Source>,
    }
    pub mod tag {
        /// Tag id in external db
        #[derive(Clone, PartialEq, ::prost::Oneof)]
        pub enum Source {
            #[prost(sint32, tag = "10")]
            AnidbId(i32),
        }
    }
    /// Type of an anime title
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Type {
        Unknown = 0,
        TvSeries = 1,
        Ova = 2,
        Ona = 3,
        Movie = 4,
        Special = 5,
    }
}

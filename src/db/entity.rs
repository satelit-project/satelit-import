use diesel::sql_types::Integer;

use super::schema::scheduled_tasks;
use super::schema::schedules;
use super::schema::tasks;

/// Represents scheduled anidb item import
#[derive(Queryable)]
pub struct Schedule {
    pub id: i32,
    pub sourced_id: i32,
    pub source: ExternalSource,
    pub state: ScheduleState,
    pub priority: SchedulePriority,
    pub has_poster: bool,
    pub has_air_date: bool,
    pub has_type: bool,
    pub has_anidb_id: bool,
    pub has_mal_id: bool,
    pub has_ann_id: bool,
    pub has_tags: bool,
    pub has_episode_count: bool,
    pub has_all_episodes: bool,
    pub has_rating: bool,
    pub has_description: bool,
    pub created_at: f64,
    pub updated_at: f64,
}

/// Represents state of a schedule
#[sql_type = "Integer"]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, FromSqlRow, AsExpression)]
pub enum ScheduleState {
    Pending = 0,
    Processing = 1,
    Finished = 2,
}

/// Represents scraping priority of a schedule
#[sql_type = "Integer"]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, FromSqlRow, AsExpression)]
pub enum SchedulePriority {
    /// Lowest priority meaning that the item should be scraped if no more work is available
    Idle = 0,

    /// External sources like AniDB or MAL are missing
    NeedExternalSources = 500,

    /// Rating is missing
    NeedRating = 700,

    /// Episodes are missing
    NeedEpisodes = 750,

    /// Description is missing
    NeedDescription = 800,

    /// Tags are missing
    NeedTags = 850,

    /// Poster is missing
    NeedPoster = 900,

    /// Air date, type or episodes count is missing
    NeedAiringDetails = 950,

    /// Newly added item that should be scraped asap
    New = 1_000,
}

#[derive(Insertable)]
#[table_name = "schedules"]
pub struct SourceSchedule {
    pub sourced_id: i32,
    pub source: ExternalSource,
    pub has_anidb_id: bool,
    pub has_mal_id: bool,
    pub has_ann_id: bool,
}

impl SourceSchedule {
    pub fn new(sourced_id: i32, source: ExternalSource) -> Self {
        let mut new = Self {
            sourced_id,
            source,
            has_anidb_id: false,
            has_mal_id: false,
            has_ann_id: false,
        };
        match source {
            ExternalSource::AniDB => new.has_anidb_id = true,
            ExternalSource::MAL => new.has_mal_id = true,
            ExternalSource::ANN => new.has_ann_id = true,
        }

        new
    }
}

#[derive(AsChangeset)]
#[table_name = "schedules"]
pub struct UpdatedSchedule {
    pub priority: SchedulePriority,
    pub has_poster: bool,
    pub has_air_date: bool,
    pub has_type: bool,
    pub has_anidb_id: bool,
    pub has_mal_id: bool,
    pub has_ann_id: bool,
    pub has_tags: bool,
    pub has_episode_count: bool,
    pub has_all_episodes: bool,
    pub has_rating: bool,
    pub has_description: bool,
}

impl Default for UpdatedSchedule {
    fn default() -> Self {
        Self {
            priority: SchedulePriority::Idle,
            has_poster: false,
            has_air_date: false,
            has_type: false,
            has_anidb_id: false,
            has_mal_id: false,
            has_ann_id: false,
            has_tags: false,
            has_episode_count: false,
            has_all_episodes: false,
            has_rating: false,
            has_description: false,
        }
    }
}

#[sql_type = "Integer"]
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, FromSqlRow, AsExpression)]
pub enum ExternalSource {
    AniDB = 0,
    MAL = 1,
    ANN = 2,
}

#[derive(Queryable, Insertable)]
pub struct Task {
    pub id: String,
    pub source: ExternalSource,
}

impl Task {
    pub fn new(id: String, source: ExternalSource) -> Self {
        Self { id, source }
    }
}

#[derive(Queryable, QueryableByName)]
#[table_name = "scheduled_tasks"]
pub struct ScheduledTask {
    pub id: i32,
    pub task_id: String,
    pub schedule_id: i32,
}

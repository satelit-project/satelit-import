use diesel::sql_types::Integer;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::schema::queued_tasks;
use super::schema::schedules;

/// Represents scheduled anidb item import
#[derive(Queryable)]
pub struct Schedule {
    pub id: i32,
    pub external_id: i32,
    pub source: ExternalSource,
    pub state: ScheduleState,
    pub priority: SchedulePriority,
    pub update_count: i32,
    pub has_poster: bool,
    pub has_air_date: bool,
    pub has_type: bool,
    pub has_anidb_id: bool,
    pub has_mal_id: bool,
    pub has_ann_id: bool,
    pub has_tags: bool,
    pub has_ep_count: bool,
    pub has_all_eps: bool,
    pub has_rating: bool,
    pub has_description: bool,
    pub src_created_at: Option<DateTime<Utc>>,
    pub src_updated_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
pub struct NewSchedule {
    pub external_id: i32,
    pub source: ExternalSource,
    pub has_anidb_id: bool,
    pub has_mal_id: bool,
    pub has_ann_id: bool,
}

impl NewSchedule {
    pub fn new(external_id: i32, source: ExternalSource) -> Self {
        let mut new = Self {
            external_id,
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
    pub has_ep_count: bool,
    pub has_all_eps: bool,
    pub has_rating: bool,
    pub has_description: bool,
    pub src_created_at: Option<DateTime<Utc>>,
    pub src_updated_at: Option<DateTime<Utc>>,
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
            has_ep_count: false,
            has_all_eps: false,
            has_rating: false,
            has_description: false,
            src_created_at: None,
            src_updated_at: None,
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

#[derive(Queryable)]
pub struct Task {
    pub id: Uuid,
    pub source: ExternalSource,
    pub external_ids: Vec<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Queryable, QueryableByName)]
#[table_name = "queued_tasks"]
pub struct QueuedTask {
    pub id: Uuid,
    pub task_id: Uuid,
    pub schedule_id: i32,
}

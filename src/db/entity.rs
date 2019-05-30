use diesel::sql_types::Integer;

use super::schema::scheduled_tasks;
use super::schema::schedules;
use super::schema::tasks;

/// Represents scheduled anidb item import
#[derive(Queryable)]
pub struct Schedule {
    pub id: i32,
    pub source_id: i32,
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
    pub has_episodes: bool,
    pub has_rating: bool,
    pub has_description: bool,
    pub created_at: f64,
    pub updated_at: f64,
}

/// Represents state of a schedule
#[sql_type = "Integer"]
#[derive(Debug, Clone, Copy, PartialEq, FromSqlRow, AsExpression)]
pub enum ScheduleState {
    Pending = 0,
    Processing = 1,
    Finished = 2,
}

/// Represents scraping priority of a schedule
#[sql_type = "Integer"]
#[derive(Debug, Clone, Copy, PartialEq, FromSqlRow, AsExpression)]
pub enum SchedulePriority {
    /// Lowest priority meaning that the item should be scraped if no more work is available
    Idle = 0,

    /// Newly added item that should be scraped asap
    New = 1_000,
}

#[derive(Insertable)]
#[table_name = "schedules"]
pub struct SourceSchedule {
    pub source_id: i32,
    pub source: ExternalSource,
    pub has_anidb_id: bool,
    pub has_mal_id: bool,
    pub has_ann_id: bool,
}

impl SourceSchedule {
    pub fn new(source_id: i32, source: ExternalSource) -> Self {
        let mut new = Self {
            source_id,
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
    pub has_episodes: bool,
    pub has_rating: bool,
    pub has_description: bool,
}

#[sql_type = "Integer"]
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

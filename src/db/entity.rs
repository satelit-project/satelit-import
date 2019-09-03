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
    pub priority: i32,
    pub next_update_at: DateTime<Utc>,
    pub update_count: i32,
    pub has_poster: bool,
    pub has_start_air_date: bool,
    pub has_end_air_date: bool,
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
    pub next_update_at: DateTime<Utc>,
    pub has_poster: bool,
    pub has_start_air_date: bool,
    pub has_end_air_date: bool,
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
            next_update_at: Utc::now(),
            has_poster: false,
            has_start_air_date: false,
            has_end_air_date: false,
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

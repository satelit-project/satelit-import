use diesel::sql_types::Integer;

use super::schema::schedules;

/// Represents scheduled anidb item import
#[derive(Queryable)]
pub struct Schedule {
    pub id: i32,
    pub anidb_id: i32,
    pub state: ScheduleState,
    pub priority: SchedulePriority,
    pub has_poster: bool,
    pub has_air_date: bool,
    pub has_type: bool,
    pub has_mal_id: bool,
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
    Finished = 2
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
pub struct NewSchedule {
    pub anidb_id: i32,
}

impl NewSchedule {
    pub fn new(anidb_id: i32) -> Self {
        NewSchedule { anidb_id }
    }
}

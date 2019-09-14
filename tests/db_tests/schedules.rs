use diesel::prelude::*;

use satelit_import::db::schema::schedules::dsl;
use satelit_import::db::{ConnectionPool, QueryError};
use satelit_import::db::schedules::Schedules;
use satelit_import::db::entity::*;

#[test]
fn test_put_new() -> Result<(), QueryError> {
    let pool = make_pool();
    let conn = pool.get()?;
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(1, ExternalSource::AniDB);
    table.put(&new)?;

    let schedule: Schedule = dsl::schedules
        .filter(dsl::external_id.eq(new.external_id))
        .filter(dsl::source.eq(new.source))
        .get_result(&conn)?;

    let mut expected = default_schedule();
    merge_db_schedule(&schedule, &mut expected);
    merge_new_schedule(&new, &mut expected);

    assert_eq!(expected, schedule);

    delete_new_schedule(&pool, &new)?;
    Ok(())
}

#[test]
fn test_put_twice() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(2, ExternalSource::AniDB);
    table.put(&new)?;
    assert_eq!(count_new_schedules(&pool, &new)?, 1);

    table.put(&new)?;
    assert_eq!(count_new_schedules(&pool, &new)?, 1);

    delete_new_schedule(&pool, &new)?;
    Ok(())
}

#[test]
fn test_put_twice_diff_id() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new1 = NewSchedule::new(3, ExternalSource::AniDB);
    table.put(&new1)?;
    assert_eq!(count_new_schedules(&pool, &new1)?, 1);

    let new2 = NewSchedule::new(4, ExternalSource::AniDB);
    table.put(&new2)?;
    assert_eq!(count_new_schedules(&pool, &new2)?, 1);

    delete_new_schedule(&pool, &new1)?;
    delete_new_schedule(&pool, &new2)?;
    Ok(())
}

#[test]
fn test_put_twice_diff_source() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new1 = NewSchedule::new(5, ExternalSource::AniDB);
    table.put(&new1)?;
    assert_eq!(count_new_schedules(&pool, &new1)?, 1);

    let new2 = NewSchedule::new(5, ExternalSource::MAL);
    table.put(&new2)?;
    assert_eq!(count_new_schedules(&pool, &new2)?, 1);

    delete_new_schedule(&pool, &new1)?;
    delete_new_schedule(&pool, &new2)?;
    Ok(())
}

#[test]
fn test_pop_schedule() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(100, ExternalSource::ANN);
    table.put(&new)?;
    assert_eq!(count_new_schedules(&pool, &new)?, 1);

    table.pop(&new)?;
    assert_eq!(count_new_schedules(&pool, &new)?, 0);

    Ok(())
}

#[test]
fn test_nonexistent_schedule() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(101, ExternalSource::ANN);
    table.pop(&new)?;
    table.pop(&new)?;
    assert_eq!(count_new_schedules(&pool, &new)?, 0);

    Ok(())
}

// MARK: db

fn make_pool() -> ConnectionPool {
    crate::connection_pool("schedules")
}

fn count_new_schedules(pool: &ConnectionPool, new: &NewSchedule) -> Result<i64, QueryError> {
    let conn = pool.get()?;

    let count = dsl::schedules
        .filter(dsl::external_id.eq(new.external_id))
        .filter(dsl::source.eq(new.source))
        .count()
        .get_result(&conn)?;

    Ok(count)
}

fn delete_new_schedule(pool: &ConnectionPool, new: &NewSchedule) -> Result<(), QueryError> {
    let conn = pool.get()?;

    diesel::delete(dsl::schedules
        .filter(dsl::external_id.eq(new.external_id))
        .filter(dsl::source.eq(new.source)))
        .execute(&conn)?;

    Ok(())
}

// MARK: schedule

fn default_schedule() -> Schedule {
    Schedule {
        id: 0,
        external_id: 0,
        source: ExternalSource::AniDB,
        state: ScheduleState::Pending,
        priority: 1000,
        next_update_at: None,
        update_count: 0,
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
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn merge_db_schedule(source: &Schedule, out: &mut Schedule) {
    out.id = source.id;
    out.created_at = source.created_at;
    out.updated_at = source.updated_at;
}

fn merge_new_schedule(new: &NewSchedule, out: &mut Schedule) {
    out.external_id = new.external_id;
    out.source = new.source;
    out.has_anidb_id = new.has_anidb_id;
    out.has_mal_id = new.has_mal_id;
    out.has_ann_id = new.has_ann_id;
}

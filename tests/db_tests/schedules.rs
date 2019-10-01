use chrono::prelude::*;

use satelit_import::db::entity::*;
use satelit_import::db::schedules::Schedules;
use satelit_import::db::{ConnectionPool, QueryError};

use super::count_schedules_by_new;
use super::delete_schedule_by_new;
use super::fetch_schedule_from_new;

// MARK: put tests

#[test]
fn test_put_new() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(1, ExternalSource::AniDB);
    table.put(&new)?;

    let schedule = fetch_schedule_from_new(&pool, &new)?;
    let mut expected = default_schedule();
    merge_db_schedule(&schedule, &mut expected);
    merge_new_schedule(&new, &mut expected);

    assert_eq!(expected, schedule);

    delete_schedule_by_new(&pool, &new)?;
    Ok(())
}

#[test]
fn test_put_twice() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(2, ExternalSource::AniDB);
    table.put(&new)?;
    assert_eq!(count_schedules_by_new(&pool, &new)?, 1);

    table.put(&new)?;
    assert_eq!(count_schedules_by_new(&pool, &new)?, 1);

    delete_schedule_by_new(&pool, &new)?;
    Ok(())
}

#[test]
fn test_put_twice_diff_id() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new1 = NewSchedule::new(3, ExternalSource::AniDB);
    table.put(&new1)?;
    assert_eq!(count_schedules_by_new(&pool, &new1)?, 1);

    let new2 = NewSchedule::new(4, ExternalSource::AniDB);
    table.put(&new2)?;
    assert_eq!(count_schedules_by_new(&pool, &new2)?, 1);

    delete_schedule_by_new(&pool, &new1)?;
    delete_schedule_by_new(&pool, &new2)?;
    Ok(())
}

#[test]
fn test_put_twice_diff_source() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new1 = NewSchedule::new(5, ExternalSource::AniDB);
    table.put(&new1)?;
    assert_eq!(count_schedules_by_new(&pool, &new1)?, 1);

    let new2 = NewSchedule::new(5, ExternalSource::MAL);
    table.put(&new2)?;
    assert_eq!(count_schedules_by_new(&pool, &new2)?, 1);

    delete_schedule_by_new(&pool, &new1)?;
    delete_schedule_by_new(&pool, &new2)?;
    Ok(())
}

// MARK: pop tests

#[test]
fn test_pop_schedule() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(100, ExternalSource::ANN);
    table.put(&new)?;
    assert_eq!(count_schedules_by_new(&pool, &new)?, 1);

    table.pop(&new)?;
    assert_eq!(count_schedules_by_new(&pool, &new)?, 0);

    Ok(())
}

#[test]
fn test_pop_nonexistent_schedule() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(101, ExternalSource::ANN);
    table.pop(&new)?;
    table.pop(&new)?;
    assert_eq!(count_schedules_by_new(&pool, &new)?, 0);

    Ok(())
}

// MARK: update tests

#[test]
fn test_update_schedule() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    let new = NewSchedule::new(300, ExternalSource::MAL);
    table.put(&new)?;

    let schedule = fetch_schedule_from_new(&pool, &new)?;
    let mut expected = default_schedule();
    merge_db_schedule(&schedule, &mut expected);
    merge_new_schedule(&new, &mut expected);
    merge_updated_schedule(&default_update(&new), &mut expected);
    // default values should be equal after applying `UpdatedSchedule::default`
    assert_eq!(schedule, expected);

    let update = full_update();
    table.update(schedule.id, &update)?;

    let updated_schedule = fetch_schedule_from_new(&pool, &new)?;
    merge_db_schedule(&updated_schedule, &mut expected);
    merge_updated_schedule(&update, &mut expected);
    assert_eq!(updated_schedule, expected);
    assert_eq!(schedule.created_at, updated_schedule.created_at);
    assert_ne!(schedule.updated_at, updated_schedule.updated_at);

    delete_schedule_by_new(&pool, &new)?;
    Ok(())
}

#[test]
fn test_update_nonexistent() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Schedules::new(pool.clone());

    // shouldn't be any errors
    let update = full_update();
    table.update(1_000_000, &update)?;

    Ok(())
}

// MARK: db

fn make_pool() -> ConnectionPool {
    crate::connection_pool("schedules-tests")
}

// MARK: schedule

fn default_schedule() -> Schedule {
    Schedule {
        id: 0,
        external_id: 0,
        source: ExternalSource::AniDB,
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
        created_at: chrono::Utc::now().trunc_subsecs(0),
        updated_at: chrono::Utc::now().trunc_subsecs(0),
    }
}

fn default_update(identity: &NewSchedule) -> UpdatedSchedule {
    let mut update = UpdatedSchedule::default();
    update.has_anidb_id = identity.has_anidb_id;
    update.has_mal_id = identity.has_mal_id;
    update.has_ann_id = identity.has_anidb_id;
    update
}

fn full_update() -> UpdatedSchedule {
    UpdatedSchedule {
        next_update_at: Some(chrono::Utc::now().trunc_subsecs(0)),
        has_poster: true,
        has_start_air_date: true,
        has_end_air_date: true,
        has_type: true,
        has_anidb_id: true,
        has_mal_id: true,
        has_ann_id: true,
        has_tags: true,
        has_ep_count: true,
        has_all_eps: true,
        has_rating: true,
        has_description: true,
        src_created_at: Some(chrono::Utc::now().trunc_subsecs(0)),
        src_updated_at: Some(chrono::Utc::now().trunc_subsecs(0)),
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

fn merge_updated_schedule(updated: &UpdatedSchedule, out: &mut Schedule) {
    out.next_update_at = updated.next_update_at;
    out.has_poster = updated.has_poster;
    out.has_start_air_date = updated.has_start_air_date;
    out.has_end_air_date = updated.has_end_air_date;
    out.has_type = updated.has_type;
    out.has_anidb_id = updated.has_anidb_id;
    out.has_mal_id = updated.has_mal_id;
    out.has_ann_id = updated.has_ann_id;
    out.has_tags = updated.has_tags;
    out.has_ep_count = updated.has_ep_count;
    out.has_all_eps = updated.has_all_eps;
    out.has_rating = updated.has_rating;
    out.has_description = updated.has_description;
    out.src_created_at = updated.src_created_at;
    out.src_updated_at = updated.src_updated_at;
}

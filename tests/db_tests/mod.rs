pub mod schedules;
pub mod tasks;
pub mod queued_jobs;

use diesel::prelude::*;

use satelit_import::db::entity::*;
use satelit_import::db::{ConnectionPool, QueryError};

// MARK: count

fn count_tasks(pool: &ConnectionPool, task: &Task) -> Result<i64, QueryError> {
    use satelit_import::db::schema::tasks::dsl;

    let conn = pool.get()?;
    let count = dsl::tasks
        .filter(dsl::id.eq(&task.id))
        .count()
        .get_result(&conn)?;

    Ok(count)
}

fn count_jobs(pool: &ConnectionPool, task: &Task) -> Result<i64, QueryError> {
    use satelit_import::db::schema::queued_jobs::dsl;

    let conn = pool.get()?;
    let count = dsl::queued_jobs
        .filter(dsl::task_id.eq(&task.id))
        .count()
        .get_result(&conn)?;

    Ok(count)
}

fn count_schedules_by_new(pool: &ConnectionPool, new: &NewSchedule) -> Result<i64, QueryError> {
    use satelit_import::db::schema::schedules::dsl;

    let conn = pool.get()?;
    let count = dsl::schedules
        .filter(dsl::external_id.eq(new.external_id))
        .filter(dsl::source.eq(new.source))
        .count()
        .get_result(&conn)?;

    Ok(count)
}

// MARK: add

fn add_schedule(
    pool: &ConnectionPool,
    external_id: i32,
    source: ExternalSource,
) -> Result<Schedule, QueryError> {
    use satelit_import::db::schedules::Schedules;

    let table = Schedules::new(pool.clone());
    let new = NewSchedule::new(external_id, source);
    table.put(&new)?;

    let schedule = fetch_schedule_from_new(&pool, &new)?;
    let mut update = UpdatedSchedule::default();
    update.next_update_at = Some(chrono::Utc::now());
    table.update(schedule.id, &update)?;

    fetch_schedule_from_new(&pool, &new)
}

// MARK: fetch

fn fetch_task_by_id(pool: &ConnectionPool, task_id: &Uuid) -> Result<Task, QueryError> {
    use satelit_import::db::schema::tasks::dsl;

    let conn = pool.get()?;
    let task: Task = dsl::tasks.find(&task_id).get_result(&conn)?;

    Ok(task)
}

fn fetch_queued_schedules(pool: &ConnectionPool, task: &Task) -> Result<Vec<Schedule>, QueryError> {
    use satelit_import::db::schema::queued_jobs::dsl;
    use satelit_import::db::schema::schedules;

    let conn = pool.get()?;
    let schedules = dsl::queued_jobs
        .filter(dsl::task_id.eq(&task.id))
        .inner_join(schedules::table)
        .select(schedules::all_columns)
        .get_results(&conn)?;

    Ok(schedules)
}

fn fetch_schedule_by_id(pool: &ConnectionPool, schedule_id: i32) -> Result<Schedule, QueryError> {
    use satelit_import::db::schema::schedules::dsl;

    let conn = pool.get()?;
    let schedule = dsl::schedules
        .find(schedule_id)
        .get_result(&conn)?;

    Ok(schedule)
}

fn fetch_schedule_from_new(pool: &ConnectionPool, new: &NewSchedule) -> Result<Schedule, QueryError> {
    use satelit_import::db::schema::schedules::dsl;

    let conn = pool.get()?;
    let schedule = dsl::schedules
        .filter(dsl::external_id.eq(new.external_id))
        .filter(dsl::source.eq(new.source))
        .get_result(&conn)?;

    Ok(schedule)
}

// MARK: delete

fn delete_task(pool: &ConnectionPool, task: &Task) -> Result<(), QueryError> {
    use satelit_import::db::schema::tasks::dsl;

    let conn = pool.get()?;
    diesel::delete(dsl::tasks.find(&task.id)).execute(&conn)?;

    Ok(())
}

fn delete_task_jobs(pool: &ConnectionPool, task: &Task) -> Result<(), QueryError> {
    use satelit_import::db::schema::queued_jobs::dsl;

    let conn = pool.get()?;
    diesel::delete(dsl::queued_jobs.filter(dsl::task_id.eq(&task.id))).execute(&conn)?;

    Ok(())
}

// DO NOT DELETE SCHEDULES IF YOU EVER BIND JOBS TO SCRAPING TASKS!!!!!
fn delete_schedules_by_ids(pool: &ConnectionPool, schedules: &[i32]) -> Result<(), QueryError> {
    use satelit_import::db::schema::schedules::dsl;

    if schedules.is_empty() {
        return Ok(());
    }

    let conn = pool.get()?;
    let target = dsl::id.eq(diesel::pg::expression::dsl::any(schedules));
    diesel::delete(dsl::schedules.filter(target)).execute(&conn)?;

    Ok(())
}

fn delete_schedule_by_new(pool: &ConnectionPool, new: &NewSchedule) -> Result<(), QueryError> {
    use satelit_import::db::schema::schedules::dsl;

    let conn = pool.get()?;
    diesel::delete(
        dsl::schedules
            .filter(dsl::external_id.eq(new.external_id))
            .filter(dsl::source.eq(new.source)),
    )
        .execute(&conn)?;

    Ok(())
}

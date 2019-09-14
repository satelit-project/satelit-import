use diesel::prelude::*;

use satelit_import::db::entity::*;
use satelit_import::db::{ConnectionPool, QueryError};
use satelit_import::db::tasks::Tasks;
use satelit_import::db::queued_jobs::QueuedJobs;

#[test]
fn test_register_task() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Tasks::new(pool.clone());

    let task = table.register(ExternalSource::AniDB)?;
    assert_eq!(count_tasks(&pool, &task)?, 1);

    delete_task(&pool, &task)?;
    Ok(())
}

#[test]
fn test_finish_task() -> Result<(), QueryError> {
    let pool = make_pool();
    let table = Tasks::new(pool.clone());

    let task = table.register(ExternalSource::AniDB)?;
    table.finish(&task.id)?;
    assert_eq!(count_tasks(&pool, &task)?, 1);

    Ok(())
}

#[test]
fn test_schedule_ids_after_bind() -> Result<(), QueryError> {
    let pool = make_pool();
    let tasks_table = Tasks::new(pool.clone());
    let queue_table = QueuedJobs::new(pool.clone());

    let mut schedules = vec![];
    schedules.push(add_schedule(&pool, 1, ExternalSource::AniDB)?.id);
    schedules.push(add_schedule(&pool, 2, ExternalSource::AniDB)?.id);
    schedules.push(add_schedule(&pool, 3, ExternalSource::AniDB)?.id);

    let task = tasks_table.register(ExternalSource::AniDB)?;
    queue_table.bind(&task.id, schedules.len() as i32)?;

    let mut task = fetch_task(&pool, &task.id)?;
    task.schedule_ids.sort();

    assert_eq!(task.schedule_ids, schedules);

    delete_task_jobs(&pool, &task)?;
    delete_task(&pool, &task)?;
    Ok(())
}

#[test]
fn test_queue_after_task_finish() -> Result<(), QueryError> {
    let pool = make_pool();
    let tasks_table = Tasks::new(pool.clone());
    let queue_table = QueuedJobs::new(pool.clone());

    let mut schedules = vec![];
    schedules.push(add_schedule(&pool, 4, ExternalSource::AniDB)?.id);
    schedules.push(add_schedule(&pool, 5, ExternalSource::AniDB)?.id);
    schedules.push(add_schedule(&pool, 6, ExternalSource::AniDB)?.id);

    let task = tasks_table.register(ExternalSource::AniDB)?;
    queue_table.bind(&task.id, schedules.len() as i32)?;

    tasks_table.finish(&task.id)?;
    let mut finished_task = fetch_task(&pool, &task.id)?;
    finished_task.schedule_ids.sort();

    assert_eq!(task.id, finished_task.id);
    assert_ne!(task.updated_at, finished_task.updated_at);
    assert_eq!(finished_task.schedule_ids, schedules);
    assert_eq!(count_jobs(&pool, &finished_task)?, 0);

    Ok(())
}

// MARK: db

fn make_pool() -> ConnectionPool {
    crate::connection_pool("tasks-tests")
}

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

fn fetch_task(pool: &ConnectionPool, task_id: &Uuid) -> Result<Task, QueryError> {
    use satelit_import::db::schema::tasks::dsl;

    let conn = pool.get()?;
    let task: Task = dsl::tasks.find(&task_id)
        .get_result(&conn)?;

    Ok(task)
}

fn delete_task(pool: &ConnectionPool, task: &Task) -> Result<(), QueryError> {
    use satelit_import::db::schema::tasks::dsl;

    let conn = pool.get()?;
    diesel::delete(dsl::tasks.find(&task.id))
        .execute(&conn)?;

    Ok(())
}

fn delete_task_jobs(pool: &ConnectionPool, task: &Task) -> Result<(), QueryError> {
    use satelit_import::db::schema::queued_jobs::dsl;

    let conn = pool.get()?;
    diesel::delete(dsl::queued_jobs
        .filter(dsl::task_id.eq(&task.id))
    )
    .execute(&conn)?;

    Ok(())
}

fn add_schedule(pool: &ConnectionPool, external_id: i32, source: ExternalSource) -> Result<Schedule, QueryError> {
    use satelit_import::db::schedules::Schedules;

    let table = Schedules::new(pool.clone());
    let new = NewSchedule::new(external_id, source);
    table.put(&new)?;

    let schedule = get_schedule_from_new(&pool, &new)?;
    let mut update = UpdatedSchedule::default();
    update.next_update_at = Some(chrono::Utc::now());
    table.update(schedule.id, &update)?;

    get_schedule_from_new(&pool, &new)
}

fn get_schedule_from_new(pool: &ConnectionPool, new: &NewSchedule) -> Result<Schedule, QueryError> {
    use satelit_import::db::schema::schedules::dsl;

    let conn = pool.get()?;
    let schedule: Schedule = dsl::schedules
        .filter(dsl::external_id.eq(new.external_id))
        .filter(dsl::source.eq(new.source))
        .get_result(&conn)?;

    Ok(schedule)
}

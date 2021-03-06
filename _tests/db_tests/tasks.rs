use satelit_import::db::{
    entity::*, queued_jobs::QueuedJobs, tasks::Tasks, ConnectionPool, QueryError,
};

use super::{
    add_schedule, count_jobs, count_tasks, delete_task, delete_task_jobs, fetch_queued_schedules,
    fetch_task_by_id,
};

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

    delete_task(&pool, &task)?;
    Ok(())
}

#[test]
fn test_schedule_ids_after_bind() -> Result<(), QueryError> {
    let pool = make_pool();
    let tasks_table = Tasks::new(pool.clone());
    let queue_table = QueuedJobs::new(pool.clone());

    add_schedule(&pool, 1, ExternalSource::AniDB)?;
    add_schedule(&pool, 2, ExternalSource::AniDB)?;
    add_schedule(&pool, 3, ExternalSource::AniDB)?;

    let task = tasks_table.register(ExternalSource::AniDB)?;
    queue_table.bind(&task.id, 3)?;

    let mut task = fetch_task_by_id(&pool, &task.id)?;
    task.schedule_ids.sort();

    let mut schedules: Vec<_> = fetch_queued_schedules(&pool, &task)?
        .into_iter()
        .map(|s| s.id)
        .collect();
    schedules.sort();

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

    add_schedule(&pool, 4, ExternalSource::AniDB)?;
    add_schedule(&pool, 5, ExternalSource::AniDB)?;
    add_schedule(&pool, 6, ExternalSource::AniDB)?;

    let task = tasks_table.register(ExternalSource::AniDB)?;
    queue_table.bind(&task.id, 3)?;

    let mut schedules: Vec<_> = fetch_queued_schedules(&pool, &task)?
        .into_iter()
        .map(|s| s.id)
        .collect();
    schedules.sort();

    tasks_table.finish(&task.id)?;
    let mut finished_task = fetch_task_by_id(&pool, &task.id)?;
    finished_task.schedule_ids.sort();

    assert_eq!(task.id, finished_task.id);
    assert_ne!(task.updated_at, finished_task.updated_at);
    assert_eq!(finished_task.schedule_ids, schedules);
    assert_eq!(count_jobs(&pool, &finished_task)?, 0);

    delete_task(&pool, &finished_task)?;
    Ok(())
}

// MARK: db

fn make_pool() -> ConnectionPool {
    crate::connection_pool("tasks-tests")
}

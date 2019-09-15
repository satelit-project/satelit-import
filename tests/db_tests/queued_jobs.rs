use satelit_import::db::entity::*;
use satelit_import::db::{ConnectionPool, QueryError};
use satelit_import::db::queued_jobs::QueuedJobs;
use satelit_import::db::tasks::Tasks;

use super::count_jobs;
use super::add_schedule;
use super::{fetch_queued_schedules, fetch_schedule_by_id, fetch_task_by_id};
use super::delete_task;

#[test]
fn test_job_binding() -> Result<(), QueryError> {
    let pool = make_pool();
    let tasks_table = Tasks::new(pool.clone());
    let queue_table = QueuedJobs::new(pool.clone());

    add_schedule(&pool, 1, ExternalSource::AniDB)?;
    add_schedule(&pool, 2, ExternalSource::AniDB)?;
    add_schedule(&pool, 3, ExternalSource::AniDB)?;

    let task = tasks_table.register(ExternalSource::AniDB)?;
    queue_table.bind(&task.id, 3)?;
    assert_eq!(count_jobs(&pool, &task)?, 3);

    let queued = fetch_queued_schedules(&pool, &task)?;
    queued.iter().for_each(|s| assert_eq!(s.state, ScheduleState::Processing));

    tasks_table.finish(&task.id)?;
    assert_eq!(count_jobs(&pool, &task)?, 0);

    let schedule_ids: Vec<_> = queued.into_iter().map(|s| s.id).collect();
    delete_task(&pool, &task)?;
    delete_schedules_by_ids(&pool, &schedule_ids)?;
    Ok(())
}

#[test]
fn test_job_binding_limit() -> Result<(), QueryError> {
    let pool = make_pool();
    let tasks_table = Tasks::new(pool.clone());
    let queue_table = QueuedJobs::new(pool.clone());

    add_schedule(&pool, 10, ExternalSource::AniDB)?;
    add_schedule(&pool, 20, ExternalSource::AniDB)?;
    add_schedule(&pool, 30, ExternalSource::AniDB)?;

    let task = tasks_table.register(ExternalSource::AniDB)?;
    queue_table.bind(&task.id, 2)?;
    assert_eq!(count_jobs(&pool, &task)?, 2);

    let task = fetch_task_by_id(&pool, &task.id)?;
    tasks_table.finish(&task.id)?;

    delete_task(&pool, &task)?;
    delete_schedules_by_ids(&pool, &task.schedule_ids)?;
    Ok(())
}

#[test]
fn test_fetching_by_task_id() -> Result<(), QueryError> {
    let pool = make_pool();
    let tasks_table = Tasks::new(pool.clone());
    let queue_table = QueuedJobs::new(pool.clone());

    add_schedule(&pool, 100, ExternalSource::AniDB)?;
    add_schedule(&pool, 101, ExternalSource::AniDB)?;
    add_schedule(&pool, 102, ExternalSource::AniDB)?;

    let task = tasks_table.register(ExternalSource::AniDB)?;
    queue_table.bind(&task.id, 3)?;

    let task = fetch_task_by_id(&pool, &task.id)?;
    let queued = queue_table.for_task_id(&task.id)?;
    for (job, schedule) in queued.iter() {
        assert_eq!(task.id, job.task_id);
        assert_eq!(job.schedule_id, schedule.id);
        assert!(task.schedule_ids.contains(&schedule.id));
    }

    tasks_table.finish(&task.id)?;

    delete_task(&pool, &task)?;
    delete_schedules_by_ids(&pool, &task.schedule_ids)?;
    Ok(())
}

#[test]
fn test_pop_job() -> Result<(), QueryError> {
    Ok(())
}

// MARK: db

fn make_pool() -> ConnectionPool {
    crate::connection_pool("queued-jobs-tests")
}

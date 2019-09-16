use satelit_import::db::entity::*;
use satelit_import::db::queued_jobs::QueuedJobs;
use satelit_import::db::tasks::Tasks;
use satelit_import::db::{ConnectionPool, QueryError};

use super::add_schedule;
use super::count_jobs;
use super::{delete_schedules_by_ids, delete_task};
use super::{fetch_queued_schedules, fetch_task_by_id};

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

    tasks_table.finish(&task.id)?;
    assert_eq!(count_jobs(&pool, &task)?, 0);

    delete_task(&pool, &task)?;
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
    let queued = queue_table.jobs_for_task_id(&task.id)?;
    for (job, schedule) in queued.iter() {
        assert_eq!(task.id, job.task_id);
        assert_eq!(job.schedule_id, schedule.id);
        assert!(task.schedule_ids.contains(&schedule.id));
    }

    tasks_table.finish(&task.id)?;

    delete_task(&pool, &task)?;
    Ok(())
}

#[test]
fn test_pop_job() -> Result<(), QueryError> {
    let pool = make_pool();
    let tasks_table = Tasks::new(pool.clone());
    let queue_table = QueuedJobs::new(pool.clone());

    add_schedule(&pool, 200, ExternalSource::AniDB)?;
    add_schedule(&pool, 201, ExternalSource::AniDB)?;

    let task = tasks_table.register(ExternalSource::AniDB)?;
    queue_table.bind(&task.id, 2)?;

    let mut jobs = queue_table.jobs_for_task_id(&task.id)?;
    queue_table.pop(&jobs[0].0.id)?;
    jobs.remove(0);
    assert_eq!(jobs, queue_table.jobs_for_task_id(&task.id)?);

    queue_table.pop(&jobs[0].0.id)?;
    jobs.remove(0);
    assert_eq!(jobs, queue_table.jobs_for_task_id(&task.id)?);

    delete_task(&pool, &task)?;
    Ok(())
}

#[test]
fn test_fk_rules() -> Result<(), QueryError> {
    let pool = make_pool();
    let tasks_table = Tasks::new(pool.clone());
    let queue_table = QueuedJobs::new(pool.clone());

    add_schedule(&pool, 1, ExternalSource::ANN)?;
    add_schedule(&pool, 2, ExternalSource::ANN)?;
    add_schedule(&pool, 3, ExternalSource::ANN)?;
    add_schedule(&pool, 4, ExternalSource::ANN)?;

    let task = tasks_table.register(ExternalSource::ANN)?;
    queue_table.bind(&task.id, 3)?;
    assert_eq!(count_jobs(&pool, &task)?, 3);

    let mut task = fetch_task_by_id(&pool, &task.id)?;
    task.schedule_ids.sort();

    let remove_range = 0..2;
    delete_schedules_by_ids(&pool, &task.schedule_ids[remove_range.clone()])?;
    remove_range.rev().for_each(|i| {
        let _ = task.schedule_ids.remove(i);
    });

    let mut remaining: Vec<_> = fetch_queued_schedules(&pool, &task)?
        .into_iter()
        .map(|s| s.id)
        .collect();
    remaining.sort();

    // jobs should be deleted automatically when it's schedule is deleted
    assert_eq!(task.schedule_ids, remaining);

    // jobs should be deleted automatically when it's task deleted
    delete_task(&pool, &task)?;
    assert_eq!(count_jobs(&pool, &task)?, 0);

    Ok(())
}

// MARK: db

fn make_pool() -> ConnectionPool {
    crate::connection_pool("queued-jobs-tests")
}

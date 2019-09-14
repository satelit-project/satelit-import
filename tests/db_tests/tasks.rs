use diesel::prelude::*;

use satelit_import::db::entity::*;
use satelit_import::db::{ConnectionPool, QueryError};
use satelit_import::db::tasks::Tasks;
use satelit_import::db::queued_jobs::QueuedJobs;

#[test]
fn test_register_task() -> Result<(), QueryError> {
    use satelit_import::db::schema::{tasks, queued_jobs};

    let pool = make_pool();
    let table = Tasks::new(pool.clone());

    let task = table.register(ExternalSource::AniDB)?;
    assert_eq!(count_tasks(&pool, &task)?, 1);

    delete_task(&pool, &task)?;
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

fn delete_task(pool: &ConnectionPool, task: &Task) -> Result<(), QueryError> {
    use satelit_import::db::schema::tasks::dsl;

    let conn = pool.get()?;

    diesel::delete(dsl::tasks.find(&task.id))
        .execute(&conn)?;

    Ok(())
}

use actix_web::{middleware, App, HttpServer};
use env_logger;

use satelit_import::api::{import::ImportService, task::TasksService};
use satelit_import::db;
use satelit_import::worker;

fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().filter_or("SATELIT_LOG", "info"));

    worker::start_worker_thread();

    let server = HttpServer::new(|| {
        let conn_pool = db::connection_pool();

        // TasksService
        let tasks = db::tasks::Tasks::new(conn_pool.clone());
        let schedules = db::schedules::Schedules::new(conn_pool.clone());
        let scheduled_tasks = db::scheduled_tasks::ScheduledTasks::new(conn_pool.clone());
        let tasks_service = TasksService::new(tasks, schedules, scheduled_tasks);

        // ImportService
        let import_service = ImportService::default();

        // App
        App::new()
            .wrap(middleware::Logger::default())
            .service(tasks_service)
            .service(import_service)
    });

    server.bind("127.0.0.1:8080")?.run()?;

    worker::shutdown_worker_thread();

    Ok(())
}

use futures::prelude::*;
use log::error;

use satelit_import::db;
use satelit_import::rpc;
use satelit_import::settings;
use satelit_import::worker;

fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().filter_or("SATELIT_LOG", "info"));
    worker::start_worker_thread();

    let mut rt = tokio::runtime::Runtime::new()?;
    serve_services(&mut rt);

    rt.shutdown_on_idle().wait().unwrap();
    worker::shutdown_worker_thread();

    Ok(())
}

fn serve_services(rt: &mut tokio::runtime::Runtime) {
    let builder = rpc::ServicesBuilder::new(db::connection_pool());
    let port_settings = settings::shared().ports();

    let import = satelit_import::serve_service!(builder.import_service(), port_settings.import());
    let tasks = satelit_import::serve_service!(builder.tasks_service(), port_settings.task());

    rt.spawn(import);
    rt.spawn(tasks);
}

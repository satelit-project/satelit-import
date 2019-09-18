use futures::prelude::*;
use log::error;

use satelit_import::{shared_db_pool, shared_settings};
use satelit_import::rpc;

fn main() -> std::io::Result<()> {
    //    env_logger::init_from_env(env_logger::Env::new().filter_or("SATELIT_LOG", "info"));

    let mut rt = tokio::runtime::Runtime::new()?;
    serve_services(&mut rt);

    rt.shutdown_on_idle().wait().unwrap();

    Ok(())
}

fn serve_services(rt: &mut tokio::runtime::Runtime) {
    let builder = rpc::ServicesBuilder::new(shared_settings(), shared_db_pool());
    let rpc_urls = shared_settings().rpc;

    let import = satelit_import::serve_service!(builder.import_service(), rpc_urls.import());
    let tasks = satelit_import::serve_service!(builder.tasks_service(), rpc_urls.task());

    rt.spawn(import);
    rt.spawn(tasks);
}

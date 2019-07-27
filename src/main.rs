use futures::prelude::*;
use tokio::net;
use log::error;

use satelit_import::db;
use satelit_import::proto;
use satelit_import::rpc;
use satelit_import::settings;
use satelit_import::worker;

fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().filter_or("SATELIT_LOG", "info"));

    worker::start_worker_thread();

    let mut rt = tokio::runtime::Runtime::new()?;
    rt.spawn(import_service());
    rt.spawn(tasks_service());
    rt.shutdown_on_idle().wait().unwrap();

    worker::shutdown_worker_thread();

    Ok(())
}

fn import_service() -> impl Future<Item = (), Error = ()> {
    let service = rpc::import::ImportService;
    let factory = proto::scheduler::server::ImportServiceServer::new(service);

    serve_service(factory, settings::shared().ports().import())
}

fn tasks_service() -> impl Future<Item = (), Error = ()> {
    let conn_pool = db::connection_pool();
    let tasks = db::tasks::Tasks::new(conn_pool.clone());
    let schedules = db::schedules::Schedules::new(conn_pool.clone());
    let scheduled_tasks = db::scheduled_tasks::ScheduledTasks::new(conn_pool.clone());

    let service = rpc::task::ScraperTasksService::new(tasks, schedules, scheduled_tasks);
    let factory = proto::scraper::server::ScraperTasksServiceServer::new(service);

    serve_service(factory, settings::shared().ports().task())
}

fn serve_service<S, B>(factory: S, port: i32) -> impl Future<Item = (), Error = ()>
    where
        S: tower_util::MakeService<(), http::Request<tower_hyper::Body>, Response = http::Response<B>> + Send + 'static,
        S::MakeError: Into<Box<dyn std::error::Error + Send + Sync>> + std::fmt::Debug,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        S::Future: Send,
        S::Service: tower_service::Service<http::Request<tower_hyper::Body>> + Send,
        <S::Service as tower_service::Service<http::Request<tower_hyper::Body>>>::Future: Send + 'static,
        B: http_body::Body + Send + 'static,
        B::Data: Send + 'static,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
{
    let mut server = tower_hyper::Server::new(factory);
    let http = tower_hyper::server::Http::new().http2_only(true).clone();

    let addr = format!("127.0.0.1:{}", port).parse().unwrap();
    let bind = net::TcpListener::bind(&addr).expect("failed to bind TcpListener");

    bind.incoming().for_each(move |sock| {
        if let Err(e) = sock.set_nodelay(true) {
            return Err(e);
        }

        let serve = server.serve_with(sock, http.clone());
        tokio::spawn(serve.map_err(|e| error!("h2 error: {:?}", e)));

        Ok(())
    }).map_err(|e| error!("tcp error: {:?}", e))
}

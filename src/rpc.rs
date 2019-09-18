pub mod import;
pub mod task;

use crate::Settings;
use crate::db::{self, ConnectionPool};
use crate::proto::import::server::ImportServiceServer;
use crate::proto::scraping::server::ScraperTasksServiceServer;

use import::ImportService;
use task::ScraperTasksService;

/// Builder for server-side gRPC services
pub struct ServicesBuilder {
    settings: Settings,
    db_pool: ConnectionPool,
}

impl ServicesBuilder {
    /// Creates new builder instance
    ///
    /// # Arguments
    /// * `settings` - app settings
    /// * `db_pool` – db connection pool
    pub fn new(settings: Settings, db_pool: ConnectionPool) -> Self {
        ServicesBuilder { settings, db_pool }
    }

    /// Creates and returns an `ImportService` gRPC service
    pub fn import_service(&self) -> ImportServiceServer<ImportService> {
        let service = ImportService::new(self.settings.import().clone(), self.db_pool.clone());
        ImportServiceServer::new(service)
    }

    /// Creates and returns an `ScraperTasksService` gRPC service
    pub fn tasks_service(&self) -> ScraperTasksServiceServer<ScraperTasksService> {
        let tasks = db::tasks::Tasks::new(self.db_pool.clone());
        let schedules = db::schedules::Schedules::new(self.db_pool.clone());
        let scheduled_tasks = db::queued_jobs::QueuedJobs::new(self.db_pool.clone());

        let service = ScraperTasksService::new(tasks, schedules, scheduled_tasks);
        ScraperTasksServiceServer::new(service)
    }
}

/// Returns a future that serves server-side gRPC service
#[macro_export]
macro_rules! serve_service {
    ($service:expr, $addr:expr) => {{
        use tokio::net;

        let mut server = tower_hyper::Server::new($service);
        let http = tower_hyper::server::Http::new().http2_only(true).clone();

        let addr = ($addr).parse().unwrap();
        let bind = net::TcpListener::bind(&addr).expect("failed to bind TcpListener");

        bind.incoming()
            .for_each(move |sock| {
                if let Err(e) = sock.set_nodelay(true) {
                    return Err(e);
                }

                let serve = server.serve_with(sock, http.clone());
                tokio::spawn(serve.map_err(|e| error!("h2 error: {:?}", e)));

                Ok(())
            })
            .map_err(|e| error!("tcp error: {:?}", e))
    }};
}

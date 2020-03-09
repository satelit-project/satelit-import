pub mod import;
pub mod task;

use crate::{
    db::{self, ConnectionPool},
    proto::{
        import::import_service_server::ImportServiceServer,
        scraping::scraper_tasks_service_server::ScraperTasksServiceServer,
    },
    settings::Settings,
};

use import::ImportService;
use task::ScraperTasksService;

/// Builder for server-side gRPC services.
pub struct ServicesBuilder {
    settings: Settings,
    db_pool: ConnectionPool,
}

impl ServicesBuilder {
    /// Creates new builder instance.
    ///
    /// # Arguments
    ///
    /// * `settings` - app settings.
    /// * `db_pool` – db connection pool.
    pub fn new(settings: Settings, db_pool: ConnectionPool) -> Self {
        ServicesBuilder { settings, db_pool }
    }

    /// Creates and returns an `ImportService` gRPC service.
    pub fn import_service(&self) -> ImportServiceServer<ImportService> {
        let service = ImportService::new(self.settings.import().clone(), self.db_pool.clone());
        ImportServiceServer::new(service)
    }

    /// Creates and returns an `ScraperTasksService` gRPC service.
    pub fn tasks_service(&self) -> ScraperTasksServiceServer<ScraperTasksService> {
        let tasks = db::tasks::Tasks::new(self.db_pool.clone());
        let schedules = db::schedules::Schedules::new(self.db_pool.clone());
        let scheduled_tasks = db::queued_jobs::QueuedJobs::new(self.db_pool.clone());

        let service = ScraperTasksService::new(tasks, schedules, scheduled_tasks);
        ScraperTasksServiceServer::new(service)
    }
}

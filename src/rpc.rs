pub mod import;
pub mod task;

use std::error;

use crate::{
    db::{self, ConnectionPool},
    proto::{
        import::import_service_server::ImportServiceServer,
        scraping::scraper_tasks_service_server::ScraperTasksServiceServer,
    },
    settings::Settings,
    store::{AnimeStore, IndexStore},
};
use import::ImportService;
use task::ScraperTasksService;

/// Builder for server-side gRPC services.
#[derive(Debug)]
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
    pub fn import_service(
        &self,
    ) -> Result<ImportServiceServer<ImportService>, Box<dyn error::Error>> {
        let store = IndexStore::new(self.settings.storage())?;
        let service = ImportService::new(self.db_pool.clone(), store);
        Ok(ImportServiceServer::new(service))
    }

    /// Creates and returns an `ScraperTasksService` gRPC service.
    pub fn tasks_service(
        &self,
        cleanup: bool,
    ) -> Result<ScraperTasksServiceServer<ScraperTasksService>, Box<dyn error::Error>> {
        let tasks = db::tasks::Tasks::new(self.db_pool.clone());
        let schedules = db::schedules::Schedules::new(self.db_pool.clone());
        let scheduled_tasks = db::queued_jobs::QueuedJobs::new(self.db_pool.clone());
        let store = AnimeStore::new(self.settings.storage())?;

        let service = ScraperTasksService::new(tasks, schedules, scheduled_tasks, store);
        if cleanup {
            service.cleanup_tasks()?;
        }

        Ok(ScraperTasksServiceServer::new(service))
    }
}

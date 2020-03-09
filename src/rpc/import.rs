use tonic::{Request, Response, Status};

use std::{
    string::ToString,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use crate::{
    anidb::importer,
    db::ConnectionPool,
    proto::import::{import_service_server, ImportIntent, ImportIntentResult},
    settings,
};

/// RPC service for importing AniDB database dumps on demand.
#[derive(Debug, Clone)]
pub struct ImportService {
    /// Import service settings.
    settings: settings::Import,

    /// Database connection pool.
    db_pool: ConnectionPool,

    /// Flag to indicate if import is already in-progress.
    is_importing: Arc<AtomicBool>,
}

// MARK: impl ImportService

impl ImportService {
    pub fn new(settings: settings::Import, db_pool: ConnectionPool) -> Self {
        let is_importing = Arc::new(AtomicBool::new(false));
        Self {
            settings,
            db_pool,
            is_importing,
        }
    }
}

#[tonic::async_trait]
impl import_service_server::ImportService for ImportService {
    async fn start_import(
        &self,
        request: Request<ImportIntent>,
    ) -> Result<Response<ImportIntentResult>, Status> {
        let flag = self.is_importing.clone();
        if flag.compare_and_swap(false, true, Ordering::SeqCst) {
            let status = Status::already_exists("import is already in progress");
            return Err(status);
        }

        let intent = request.into_inner();
        let result = importer::import(intent, self.settings.clone(), self.db_pool.clone()).await;

        flag.store(false, Ordering::SeqCst);
        match result {
            Ok(r) => Ok(Response::new(r)),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

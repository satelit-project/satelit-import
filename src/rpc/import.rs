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
    store::IndexStore,
};

/// RPC service for importing AniDB database dumps on demand.
#[derive(Debug, Clone)]
pub struct ImportService {
    /// Database connection pool.
    db_pool: ConnectionPool,

    /// Index files storage.
    store: IndexStore,

    /// Flag to indicate if import is already in-progress.
    is_importing: Arc<AtomicBool>,
}

// MARK: impl ImportService

impl ImportService {
    pub fn new(db_pool: ConnectionPool, store: IndexStore) -> Self {
        let is_importing = Arc::new(AtomicBool::new(false));
        Self {
            db_pool,
            store,
            is_importing,
        }
    }
}

#[tonic::async_trait]
impl import_service_server::ImportService for ImportService {
    /// Initiates AniDB database dump import.
    #[tracing::instrument(skip(self))]
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
        let result = importer::import(intent, self.db_pool.clone(), &self.store).await;

        flag.store(false, Ordering::SeqCst);
        match result {
            Ok(r) => Ok(Response::new(r)),
            Err(e) => Err(Status::internal(e.to_string())),
        }
    }
}

use tonic::{Request, Response, Status};
use tracing::{debug, error, info, info_span};
use tracing_futures::Instrument;

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
    async fn start_import(
        &self,
        request: Request<ImportIntent>,
    ) -> Result<Response<ImportIntentResult>, Status> {
        let intent = request.into_inner();
        let span = match intent.id.as_ref() {
            Some(id) => info_span!("rpc::import::start_import", id = display(id)),
            None => return Err(Status::invalid_argument("import intent id expected")),
        };
        let _enter = span.enter();

        let flag = self.is_importing.clone();
        if flag.compare_and_swap(false, true, Ordering::SeqCst) {
            debug!("import already in progress, skipping...");
            let status = Status::already_exists("import is already in progress");
            return Err(status);
        }

        info!("starting import for {}", intent.source);
        debug!(
            "old: {}, new: {}",
            intent.new_index_url, intent.old_index_url
        );
        let result = importer::import(intent, self.db_pool.clone(), &self.store)
            .instrument(info_span!("importer::import"))
            .await;

        flag.store(false, Ordering::SeqCst);
        match result {
            Ok(r) => {
                info!("import succeeded, skipped: {:?}", &r.skipped_ids);
                Ok(Response::new(r))
            }
            Err(e) => {
                let msg = e.to_string();
                error!("import failed: {}", &msg);
                Err(Status::internal(msg))
            }
        }
    }
}

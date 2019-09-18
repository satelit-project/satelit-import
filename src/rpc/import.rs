use futures::prelude::*;
use tower_grpc::{Code, Request, Response, Status};

use crate::settings;
use crate::db::ConnectionPool;
use crate::anidb::importer;
use crate::proto::import::{server, ImportIntent, ImportIntentResult};

#[derive(Clone)]
pub struct ImportService {
    settings: settings::Import,
    db_pool: ConnectionPool,
}

impl ImportService {
    pub fn new(settings: settings::Import, db_pool: ConnectionPool) -> Self {
        Self { settings, db_pool }
    }
}

impl server::ImportService for ImportService {
    type StartImportFuture =
        Box<dyn Future<Item = Response<ImportIntentResult>, Error = Status> + Send>;

    fn start_import(&mut self, request: Request<ImportIntent>) -> Self::StartImportFuture {
        let intent = request.into_inner();
        let fut = importer::importer(intent, self.settings.clone(), self.db_pool.clone())
            .and_then(|result| {
                let response = Response::new(result);
                Ok(response)
            })
            .map_err(|e| Status::new(Code::Internal, e.to_string()));

        Box::new(fut)
    }
}

use futures::prelude::*;
use futures::future::FutureResult;
use tower_grpc::{Code, Request, Response, Status};

use crate::proto::import::{server, ImportIntent, ImportIntentResult};
use crate::anidb::importer;

#[derive(Clone)]
pub struct ImportService;

impl server::ImportService for ImportService {
    type StartImportFuture = FutureResult<Response<ImportIntentResult>, Status>;

    fn start_import(&mut self, request: Request<ImportIntent>) -> Self::StartImportFuture {
        let intent = request.into_inner();
        importer::importer(intent)
            .and_then(|result| {
                let response = Response::new(result);
                Ok(response)
            })
            .map_err(|e| {
                Status::new(Code::Internal, e.to_string())
            })
    }
}

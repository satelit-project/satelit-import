use futures::future::{self, FutureResult};
use log::info;
use tower_grpc::{Code, Request, Response, Status};

use crate::proto::scheduler::server;
use crate::proto::scheduler::ImportIntent;
use crate::worker::{self, dump};

#[derive(Clone)]
pub struct ImportService;

impl server::ImportService for ImportService {
    type StartImportFuture = FutureResult<Response<()>, Status>;

    fn start_import(&mut self, request: Request<ImportIntent>) -> Self::StartImportFuture {
        let intent = request.into_inner();
        match begin_anidb_import(intent) {
            Ok(_) => future::ok(Response::new(())),
            Err(e) => future::err(Status::new(Code::Internal, e.to_string())),
        }
    }
}

fn begin_anidb_import(intent: ImportIntent) -> Result<(), impl std::error::Error> {
    info!("Starting AniDB import");

    let importer = dump::worker(intent);
    worker::spawn_worker(importer)
}

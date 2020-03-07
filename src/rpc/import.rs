use futures::prelude::*;
use tonic::{Request, Response, Status};

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{
    anidb::importer,
    db::ConnectionPool,
    proto::import::{ImportIntent, ImportIntentResult, import_service_server},
    settings,
};

#[derive(Clone)]
pub struct ImportService {
    settings: settings::Import,
    db_pool: ConnectionPool,
    is_importing: Arc<AtomicBool>,
}

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

// #[tonic::async_trait]
// impl import_service_server::ImportService for ImportService {
//     async fn start_import(
//         &self,
//         request: Request<ImportIntent>,
//     ) -> Result<Response<ImportIntentResult>, Status> {
//         let flag = self.is_importing.clone();
//         if flag.compare_and_swap(false, true, Ordering::SeqCst) {
//             let status = Status::already_exists("import is already in progress");
//             return Err(status);
//         }

//         let intent = request.into_inner();
//     }

//     // fn start_import(&mut self, request: Request<ImportIntent>) -> Self::StartImportFuture {
//     //     let flag = self.is_importing.clone();
//     //     let failed = flag.compare_and_swap(false, true, Ordering::SeqCst);
//     //     if failed {
//     //         let status = Status::new(Code::ResourceExhausted, "import is already in progress");
//     //         return Box::new(futures::failed(status));
//     //     }

//     //     let intent = request.into_inner();
//     //     let fut = importer::importer(intent, self.settings.clone(), self.db_pool.clone())
//     //         .and_then(move |result| {
//     //             flag.store(false, Ordering::SeqCst);
//     //             let response = Response::new(result);
//     //             Ok(response)
//     //         })
//     //         .map_err(|e| Status::new(Code::Internal, e.to_string()));

//     //     Box::new(fut)
//     // }
// }

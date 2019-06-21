use actix_protobuf::ProtoBuf;
use actix_web::dev::{AppService, HttpServiceFactory};
use actix_web::web;
use actix_web::HttpResponse;
use futures::prelude::*;
use log::{error, info};

use crate::proto::scheduler::intent::{self, ImportIntent};
use crate::worker::{self, dump};

use std::error::Error;

pub struct ImportService;

impl ImportService {
    pub fn new() -> Self {
        Self
    }
}

impl HttpServiceFactory for ImportService {
    fn register(self, config: &mut AppService) {
        let service = web::scope("/import")
            .service(web::resource("/").route(web::post()).to_async(begin_import));
        service.register(config);
    }
}

fn begin_import(
    proto: ProtoBuf<ImportIntent>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    use intent::import_intent::Source as IntentSource;

    let _source = match IntentSource::from_i32(proto.source) {
        Some(s) => s,
        None => {
            info!(
                "received 'ImportIntent' with unsupported source type: {}",
                proto.source
            );
            return futures::finished(HttpResponse::BadRequest().into());
        }
    };

    let ProtoBuf(intent) = proto;
    match begin_anidb_import(intent) {
        Ok(()) => futures::finished(HttpResponse::Ok().into()),
        Err(e) => {
            error!("failed to spawn worker: {}", e);
            futures::finished(HttpResponse::InternalServerError().into())
        }
    }
}

fn begin_anidb_import(intent: ImportIntent) -> Result<(), impl Error> {
    let importer = dump::worker(intent);
    worker::spawn_worker(importer)
}

use actix_protobuf::ProtoBuf;
use actix_web::dev::{AppService, HttpServiceFactory};
use actix_web::web::{self, Data};
use actix_web::HttpResponse;
use futures::prelude::*;
use log::info;

use std::sync::mpsc::Sender;

use crate::proto::scheduler::intent::{self, ImportIntent};
use crate::worker::Worker;

pub struct ImportService {
    worker_sender: Sender<Box<dyn Worker>>,
}

impl HttpServiceFactory for ImportService {
    fn register(self, config: &mut AppService) {
        let service = web::scope("/import")
            .data(Data::new(self.worker_sender))
            .service(web::resource("/").route(web::post()).to_async(begin_import));
        service.register(config);
    }
}

fn begin_import(
    proto: ProtoBuf<ImportIntent>,
    sender: Data<Sender<Box<dyn Worker>>>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    use futures::future::ok as fut_ok;
    use intent::import_intent::Source as IntentSource;

    let _source = match IntentSource::from_i32(proto.source) {
        Some(s) => s,
        None => {
            info!(
                "received 'ImportIntent' with unsupported source type: {}",
                proto.source
            );
            return fut_ok(HttpResponse::BadRequest().into());
        }
    };

    let ProtoBuf(intent) = proto;
    begin_anidb_import(intent, sender.clone());
    fut_ok(HttpResponse::Ok().into())
}

fn begin_anidb_import(intent: ImportIntent, sender: Data<Sender<Box<dyn Worker>>>) {}

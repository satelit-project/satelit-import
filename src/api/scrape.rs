use actix_web::dev::{AppService, HttpServiceFactory};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use futures::Future;

use crate::db::entity::{ExternalSource, Task};
use crate::db::tasks::Tasks;
use crate::db::ConnectionPool;

pub struct TasksService<P: ConnectionPool + 'static> {
    tasks: Tasks<P>,
}

impl<P: ConnectionPool + 'static> HttpServiceFactory for TasksService<P> {
    fn register(self, config: &mut AppService) {
        let service = web::scope("/task")
            .data(web::Data::new(self.tasks))
            .service(web::resource("/").route(web::post().to_async(create_task::<P>)));

        service.register(config);
    }
}

fn create_task<P: ConnectionPool + 'static>(
    req: HttpRequest,
    table: web::Data<Tasks<P>>,
) -> impl Future<Item = HttpResponse, Error = Error> {
    web::block(move || {
        let id = uuid::Uuid::new_v4().to_string();
        let task = Task::new(id, ExternalSource::AniDB);

        table.insert(&task)
    })
    .then(|res| HttpResponse::Ok())
}

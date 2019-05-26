use actix_protobuf::ProtoBufResponseBuilder;
use actix_web::dev::{AppService, HttpServiceFactory};
use actix_web::error::BlockingError;
use actix_web::{web, HttpResponse};
use futures::Future;
use log::error;

use crate::db::entity::{ExternalSource, Task};
use crate::db::scheduled_tasks::ScheduledTasks;
use crate::db::tasks::Tasks;
use crate::db::{ConnectionPool, QueryError};
use crate::proto::scrape::task;

pub struct TasksService<P: ConnectionPool + 'static> {
    tasks: Tasks<P>,
    scheduled_tasks: ScheduledTasks<P>,
}

impl<P: ConnectionPool + 'static> TasksService<P> {
    pub fn new(tasks: Tasks<P>, scheduled_tasks: ScheduledTasks<P>) -> Self {
        Self {
            tasks,
            scheduled_tasks,
        }
    }
}

impl<P: ConnectionPool + 'static> HttpServiceFactory for TasksService<P> {
    fn register(self, config: &mut AppService) {
        let service = web::scope("/task")
            .data(web::Data::new(self.tasks))
            .data(web::Data::new(self.scheduled_tasks))
            .service(web::resource("/").route(web::post().to_async(create_task::<P>)));

        service.register(config);
    }
}

fn create_task<P: ConnectionPool + 'static>(
    tasks: web::Data<Tasks<P>>,
    scheduled_tasks: web::Data<ScheduledTasks<P>>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    web::block(move || {
        let id = uuid::Uuid::new_v4().to_string();
        let task = Task::new(id, ExternalSource::AniDB);

        tasks.register(&task)?;
        let scheduled = scheduled_tasks.for_task(&task)?;
        let ids = scheduled.iter().map(|s| s.schedule_id);

        Ok(task::Task {
            id: task.id,
            source: task.source as i32,
            anime_ids: ids.collect(),
        })
    })
    .then(
        |res: Result<task::Task, BlockingError<QueryError>>| match res {
            Ok(task) => HttpResponse::Ok().protobuf(task),
            Err(e) => {
                error!("Failed to create new scrape task: {}", e);
                Ok(HttpResponse::InternalServerError().into())
            }
        },
    )
}

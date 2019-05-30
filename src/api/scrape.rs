use actix_protobuf::{ProtoBuf, ProtoBufResponseBuilder};
use actix_web::dev::{AppService, HttpServiceFactory};
use actix_web::error::BlockingError;
use actix_web::{web, web::Data, HttpResponse};
use futures::Future;
use log::error;

use crate::db::entity::{ExternalSource, Task};
use crate::db::scheduled_tasks::ScheduledTasks;
use crate::db::tasks::Tasks;
use crate::db::{ConnectionPool, QueryError};
use crate::proto::scrape::{anime, task};

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
            .data(Data::new(self.tasks))
            .data(Data::new(self.scheduled_tasks))
            .service(web::resource("/").route(web::post().to_async(create_task::<P>)));

        service.register(config);
    }
}

fn create_task<P: ConnectionPool + 'static>(
    tasks: Data<Tasks<P>>,
    scheduled_tasks: Data<ScheduledTasks<P>>,
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
                Ok(HttpResponse::InternalServerError().into()) // TODO: error proto
            }
        },
    )
}

//fn task_yield<P: ConnectionPool + 'static>(
//    proto: ProtoBuf<task::TaskYield>,
//    tasks: Data<Tasks<P>>,
//    scheduled_tasks: Data<ScheduledTasks<P>>,
//) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
//    web::block(move || {
//        // TODO: pass parsed stuff to main service
//
//        let task = tasks.for_id(&proto.task_id)?;
//        let mut not_found = vec![];
//
//        for anime in proto.anime {
//            if let Some(source) = anime.source {
//
//            }
//
//            if let Some(id) = anime.id_for_source(task.source) {
//                scheduled_tasks.complete(id)?;
//            } else {
//                not_found.push(anime);
//            }
//        }
//    })
//}

//impl anime::Anime {
//    fn id_for_source(&self, source: ExternalSource) -> Option<i32> {
//        use task::task::Source::*;
//
//        let my_sources = self.source?;
//        let id = match source {
//            Anidb => my_sources.anidb_id,
//        };
//
//        if id == 0 {
//            None
//        } else {
//            Some(id)
//        }
//    }
//}

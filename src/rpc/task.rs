mod update;

use futures::prelude::*;
use log::{error, warn};
use tower_grpc::{Code, Request, Response, Status};

use std::convert::{TryFrom, TryInto};
use std::sync::Arc;

use crate::db::entity::{ExternalSource, Task};
use crate::db::queued_tasks::QueuedTasks;
use crate::db::schedules::Schedules;
use crate::db::tasks::Tasks;
use crate::db::QueryError;

use crate::block::{blocking, BlockingError};
use crate::proto::data;
use crate::proto::scraping::{self, server};

#[derive(Clone)]
pub struct ScraperTasksService {
    state: Arc<State>, // TODO: will Rc be enough?
}

#[derive(Clone)]
struct State {
    tasks: Tasks,
    schedules: Schedules,
    queued_tasks: QueuedTasks,
}

impl ScraperTasksService {
    pub fn new(
        tasks: Tasks,
        schedules: Schedules,
        queued_tasks: QueuedTasks,
    ) -> Self {
        let state = State {
            tasks,
            schedules,
            queued_tasks,
        };

        Self {
            state: Arc::new(state),
        }
    }
}

impl server::ScraperTasksService for ScraperTasksService {
    type CreateTaskFuture = Box<dyn Future<Item = Response<scraping::Task>, Error = Status> + Send>;
    type YieldResultFuture = Box<dyn Future<Item = Response<()>, Error = Status> + Send>;
    type CompleteTaskFuture = Box<dyn Future<Item = Response<()>, Error = Status> + Send>;

    fn create_task(&mut self, request: Request<scraping::TaskCreate>) -> Self::CreateTaskFuture {
        let state = self.state.clone();

        let response = blocking(move || {
            let data = request.get_ref();
            make_task(&state, data)
        })
        .then(|result| match result {
            Ok(task) => Ok(Response::new(task)),
            Err(e) => {
                error!("Failed to create new scrape task: {}", e);
                Err(e.into())
            }
        });

        Box::new(response)
    }

    fn yield_result(&mut self, request: Request<scraping::TaskYield>) -> Self::YieldResultFuture {
        let state = self.state.clone();

        let response = blocking(move || {
            let data = request.get_ref();
            update_task(&state, data)
        })
        .then(|result| match result {
            Ok(_) => Ok(Response::new(())),
            Err(e) => {
                error!("Failed to update yielded entity: {}", e);
                Err(e.into())
            }
        });

        Box::new(response)
    }

    fn complete_task(&mut self, request: Request<scraping::TaskFinish>) -> Self::CompleteTaskFuture {
        let state = self.state.clone();

        let response = blocking(move || {
            let data = request.get_ref();
            state.tasks.remove(&data.task_id)
        })
        .then(|result| match result {
            Ok(()) => Ok(Response::new(())),
            Err(e) => {
                error!("Failed to finish task: {}", e);
                Err(e.into())
            }
        });

        Box::new(response)
    }
}

fn make_task(state: &State, options: &scraping::TaskCreate) -> Result<scraping::Task, Status> {
    let source = data::Source::from_i32(options.source).unwrap_or(data::Source::Unknown);
    let source: ExternalSource = source.try_into()?;
    let task = state.tasks.register(&source)?;

    state.queued_tasks.bind(&task.id, options.limit)?;

    let queued = state.queued_tasks.for_task_id(&task.id)?;
    let mut anime_ids = vec![];
    let mut schedule_ids = vec![];

    for (_, schedule) in queued {
        anime_ids.push(schedule.external_id);
        schedule_ids.push(schedule.id);
    }

    Ok(scraping::Task {
        id: task.id.to_string(),
        source: task.source as i32,
        schedule_ids,
        anime_ids,
    })
}

fn update_task(state: &State, data: &scraping::TaskYield) -> Result<(), Status> {
    let anime = match data.anime {
        Some(ref a) => a,
        None => {
            warn!(
                "anime entity is missing, won't update task, 'task_id': {}, 'schedule_id': {}",
                data.task_id, data.schedule_id
            );

            return Err(Status::new(
                Code::InvalidArgument,
                "Anime entity is missing",
            ));
        }
    };

    let update = update::make_update(anime);
    state.schedules.update(data.schedule_id, data.sou)
    state.schedules.update_for_id(data.schedule_id, &update)?;
    state
        .scheduled_tasks
        .complete_for_schedule(&data.task_id, data.schedule_id)?;

    Ok(())
}

impl From<QueryError> for Status {
    fn from(e: QueryError) -> Self {
        Status::new(Code::Internal, e.to_string())
    }
}

impl From<BlockingError<QueryError>> for Status {
    fn from(e: BlockingError<QueryError>) -> Self {
        use BlockingError::*;

        match e {
            Error(e) => Status::new(Code::Internal, e.to_string()),
            Cancelled => Status::new(Code::Cancelled, "Job was cancelled"),
        }
    }
}

impl From<BlockingError<Status>> for Status {
    fn from(e: BlockingError<Status>) -> Self {
        use BlockingError::*;

        match e {
            Error(status) => status,
            Cancelled => Status::new(Code::Cancelled, "Job was cancelled"),
        }
    }
}

impl TryFrom<data::Source> for ExternalSource {
    type Error = Status;

    fn try_from(value: data::Source) -> Result<Self, Self::Error> {
        match value {
            data::Source::Unknown => Err(Status::new(
                Code::InvalidArgument,
                "scraping source is not supported",
            )),
            data::Source::Anidb => Ok(ExternalSource::AniDB),
        }
    }
}

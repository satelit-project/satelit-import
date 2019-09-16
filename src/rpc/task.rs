mod update;

use futures::prelude::*;
use log::{error, warn};
use tower_grpc::{Code, Request, Response, Status};

use std::convert::{TryFrom, TryInto};
use std::sync::Arc;

use crate::db::entity::ExternalSource;
use crate::db::queued_jobs::QueuedJobs;
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
    queued_jobs: QueuedJobs,
}

impl ScraperTasksService {
    pub fn new(tasks: Tasks, schedules: Schedules, queued_jobs: QueuedJobs) -> Self {
        let state = State {
            tasks,
            schedules,
            queued_jobs,
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

    fn complete_task(
        &mut self,
        request: Request<scraping::TaskFinish>,
    ) -> Self::CompleteTaskFuture {
        let state = self.state.clone();

        let response = blocking(move || {
            let data = request.into_inner();
            state.tasks.finish(&data.task_id.into())
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
    let task = state.tasks.register(source)?;

    state.queued_jobs.bind(&task.id, options.limit)?;

    let queued = state.queued_jobs.jobs_for_task_id(&task.id)?;
    let mut jobs = vec![];

    for (job, schedule) in queued {
        jobs.push(scraping::Job {
            id: Some(job.id),
            anime_id: schedule.external_id,
        });
    }

    Ok(scraping::Task {
        id: Some(task.id),
        source: options.source,
        jobs,
    })
}

fn update_task(state: &State, data: &scraping::TaskYield) -> Result<(), Status> {
    let anime = match data.anime {
        Some(ref a) => a,
        None => {
            warn!(
                "anime entity is missing, won't update task, 'task_id': {:?}",
                data.task_id
            );

            return Err(Status::new(
                Code::InvalidArgument,
                "Anime entity is missing",
            ));
        }
    };

    let update = update::make_update(anime);
    let job = state.queued_jobs.pop((&data.job_id).into())?;
    state.schedules.update(job.schedule_id, &update)?;

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

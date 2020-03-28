mod update;

use futures::prelude::*;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

use std::{
    convert::{TryFrom, TryInto},
    sync::Arc,
};

use crate::{
    db::{
        entity::ExternalSource, queued_jobs::QueuedJobs, schedules::Schedules, tasks::Tasks,
        QueryError,
    },
    proto::{
        data,
        scraping::{self, scraper_tasks_service_server},
    },
    store::{AnimeStore, StoreError},
};

/// Service to manage import tasks for scraper service.
#[derive(Clone)]
pub struct ScraperTasksService {
    /// Service state.
    state: Arc<State>,
}

/// Tasks service state.
#[derive(Clone)]
struct State {
    /// Scrape tasks storage.
    tasks: Tasks,

    /// Storage for anime entities waiting for scraping.
    schedules: Schedules,

    /// Storage for queued task jobs.
    queued_jobs: QueuedJobs,

    /// External anime storage.
    store: AnimeStore,
}

// MARK: impl ScraperTasksService

impl ScraperTasksService {
    pub fn new(
        tasks: Tasks,
        schedules: Schedules,
        queued_jobs: QueuedJobs,
        store: AnimeStore,
    ) -> Self {
        let state = State {
            tasks,
            schedules,
            queued_jobs,
            store,
        };

        Self {
            state: Arc::new(state),
        }
    }
}

#[tonic::async_trait]
impl scraper_tasks_service_server::ScraperTasksService for ScraperTasksService {
    /// Creates new task for a scraping service.
    #[tracing::instrument(skip(self))]
    async fn create_task(
        &self,
        request: Request<scraping::TaskCreate>,
    ) -> Result<Response<scraping::Task>, Status> {
        info!("creating new scraping task");

        let state = self.state.clone();
        let result = blocking(move || {
            let data = request.get_ref();
            make_task(&state, data)
        })
        .await?;

        match result {
            Ok(task) => {
                info!("created new task");
                Ok(Response::new(task))
            }
            Err(status) => {
                error!("failed to create task: {}", &status);
                Err(status)
            }
        }
    }

    /// Finishes a job accossiated with a task.
    #[tracing::instrument(skip(self))]
    async fn yield_result(
        &self,
        request: Request<scraping::TaskYield>,
    ) -> Result<Response<()>, Status> {
        info!("updating task");

        let state = self.state.clone();
        let data = request.into_inner();

        if let Some(anime) = data.anime.as_ref() {
            let path = self.state.store.upload(anime, data::Source::Anidb).await?;
            info!("uploaded anime: {}", path);
        }

        let result = blocking(move || update_task(&state, &data)).await?;

        match result {
            Ok(_) => Ok(Response::new(())),
            Err(status) => {
                error!("failed to update task: {}", &status);
                Err(status)
            }
        }
    }

    /// Finishes scraping task and all it's assocciated jobs.
    #[tracing::instrument(skip(self))]
    async fn complete_task(
        &self,
        request: Request<scraping::TaskFinish>,
    ) -> Result<Response<()>, Status> {
        info!("finalizing task");

        let state = self.state.clone();
        blocking(move || {
            let data: scraping::TaskFinish = request.into_inner();
            let task_id = match data.task_id {
                Some(task_id) => task_id,
                None => return Err(Status::invalid_argument("task_id is required")),
            };

            match state.tasks.finish(&task_id) {
                Ok(_) => Ok(()),
                Err(e) => {
                    error!("failed to finish task: {}", e);
                    Err(Status::from(e))
                }
            }
        })
        .await??;

        Ok(Response::new(()))
    }
}

// MARK: tasks

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

            return Err(Status::invalid_argument("anime entity is missing"));
        }
    };

    let update = update::make_update(anime);
    let job = state.queued_jobs.pop((&data.job_id).into())?;
    state.schedules.update(job.schedule_id, &update)?;

    Ok(())
}

// MARK: blocking

async fn blocking<F, R>(f: F) -> Result<R, Status>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .map_err(|err| Status::internal(err.to_string()))
        .await
}

// MARK: impl ExternalSource

impl TryFrom<data::Source> for ExternalSource {
    type Error = Status;

    fn try_from(value: data::Source) -> Result<Self, Self::Error> {
        match value {
            data::Source::Unknown => {
                Err(Status::invalid_argument("scraping source is not supported"))
            }
            data::Source::Anidb => Ok(ExternalSource::AniDB),
        }
    }
}

// MARK: impl Status

impl From<QueryError> for Status {
    fn from(err: QueryError) -> Self {
        Status::internal(err.to_string())
    }
}

impl From<StoreError> for Status {
    fn from(err: StoreError) -> Self {
        Status::internal(err.to_string())
    }
}

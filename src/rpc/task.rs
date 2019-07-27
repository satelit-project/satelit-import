use futures::prelude::*;
use log::{error, warn};
use tower_grpc::{Code, Request, Response, Status};

use std::convert::{TryFrom, TryInto};

use crate::db::entity::{ExternalSource, SchedulePriority, Task, UpdatedSchedule};
use crate::db::scheduled_tasks::ScheduledTasks;
use crate::db::schedules::Schedules;
use crate::db::tasks::Tasks;
use crate::db::{ConnectionPool, QueryError};

use crate::proto::data;
use crate::proto::scraper::{self, server};
use crate::rpc::{blocking, BlockingError};

#[derive(Clone)]
pub struct ScraperTasksService<P: ConnectionPool + 'static> {
    tasks: Tasks<P>,
    schedules: Schedules<P>,
    scheduled_tasks: ScheduledTasks<P>,
}

impl<P> server::ScraperTasksService for ScraperTasksService<P>
where
    P: ConnectionPool + 'static,
{
    type CreateTaskFuture = Box<dyn Future<Item = Response<scraper::Task>, Error = Status>>;
    type YieldResultFuture = Box<dyn Future<Item = Response<()>, Error = Status>>;
    type CompleteTaskFuture = Box<dyn Future<Item = Response<()>, Error = Status>>;

    fn create_task(&mut self, request: Request<scraper::TaskCreate>) -> Self::CreateTaskFuture {
        let tasks = self.tasks.clone();
        let scheduled_tasks = self.scheduled_tasks.clone();

        let response = blocking(move || {
            let data = request.get_ref();
            make_task(&tasks, &scheduled_tasks, data)
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

    fn yield_result(&mut self, request: Request<scraper::TaskYield>) -> Self::YieldResultFuture {
        let schedules = self.schedules.clone();
        let scheduled_tasks = self.scheduled_tasks.clone();

        let response = blocking(move || {
            let data = request.get_ref();
            update_task(&schedules, &scheduled_tasks, data)
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

    fn complete_task(&mut self, request: Request<scraper::TaskFinish>) -> Self::CompleteTaskFuture {
        let tasks = self.tasks.clone();

        let response = blocking(move || {
            let data = request.get_ref();
            tasks.remove(&data.task_id)
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

fn make_task<P>(
    tasks: &Tasks<P>,
    scheduled_tasks: &ScheduledTasks<P>,
    options: &scraper::TaskCreate,
) -> Result<scraper::Task, Status>
where
    P: ConnectionPool + 'static,
{
    let source = data::Source::from_i32(options.source).unwrap_or(data::Source::Unknown);

    let id = uuid::Uuid::new_v4().to_string();
    let task = Task::new(id, source.try_into()?);

    // TODO: Do not retrieve entities that has been scraped in < 1 week
    tasks.register(&task)?;
    scheduled_tasks.create(&task, options.limit)?;
    let scheduled = scheduled_tasks.for_task(&task)?;
    let mut anime_ids = vec![];
    let mut schedule_ids = vec![];

    for (_, schedule) in scheduled {
        anime_ids.push(schedule.sourced_id);
        schedule_ids.push(schedule.id);
    }

    Ok(scraper::Task {
        id: task.id,
        source: task.source as i32,
        schedule_ids,
        anime_ids,
    })
}

fn update_task<P>(
    schedules: &Schedules<P>,
    scheduled_tasks: &ScheduledTasks<P>,
    data: &scraper::TaskYield,
) -> Result<(), Status>
where
    P: ConnectionPool + 'static,
{
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

    let update = update_for_anime(anime);
    schedules.update_for_id(data.schedule_id, &update)?;
    scheduled_tasks.complete_for_schedule(&data.task_id, data.schedule_id)?;

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
            Unavailable => Status::new(Code::Internal, "Worker thread is unavailable"),
        }
    }
}

impl From<BlockingError<Status>> for Status {
    fn from(e: BlockingError<Status>) -> Self {
        use BlockingError::*;

        match e {
            Error(status) => status,
            Cancelled => Status::new(Code::Cancelled, "Job was cancelled"),
            Unavailable => Status::new(Code::Internal, "Worker thread is unavailable"),
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

// Helpers

fn update_for_anime(anime: &data::Anime) -> UpdatedSchedule {
    use data::anime::Type as AnimeType;
    use data::episode::Type as EpisodeType;

    let mut schedule = UpdatedSchedule::default();
    schedule.has_poster = !anime.poster_url.is_empty();
    schedule.has_air_date = anime.start_date != 0 && anime.end_date != 0;

    let anime_type = AnimeType::from_i32(anime.r#type).unwrap_or(AnimeType::Unknown);
    schedule.has_type = anime_type != AnimeType::Unknown;

    schedule.has_anidb_id = anime
        .source
        .as_ref()
        .map_or(false, |s| !s.anidb_ids.is_empty());
    schedule.has_mal_id = anime
        .source
        .as_ref()
        .map_or(false, |s| !s.mal_ids.is_empty());
    schedule.has_ann_id = anime
        .source
        .as_ref()
        .map_or(false, |s| !s.ann_ids.is_empty());

    schedule.has_tags = !anime.tags.is_empty();
    schedule.has_episode_count = anime.episodes_count != 0;

    let unknown_eps_count = anime
        .episodes
        .iter()
        .filter(|&e| {
            let ep_type = EpisodeType::from_i32(e.r#type).unwrap_or(EpisodeType::Unknown);
            ep_type == EpisodeType::Unknown
                || e.air_date == 0
                || e.duration == 0.0
                || e.name.is_empty()
        })
        .count();

    schedule.has_all_episodes = unknown_eps_count == 0 && !anime.episodes.is_empty();
    schedule.has_rating = anime.rating != 0.0;
    schedule.has_description = !anime.description.is_empty();
    schedule.priority = priority_for_schedule(&schedule);

    schedule
}

fn priority_for_schedule(schedule: &UpdatedSchedule) -> SchedulePriority {
    if !schedule.has_air_date || !schedule.has_type || !schedule.has_episode_count {
        return SchedulePriority::NeedAiringDetails;
    }

    if !schedule.has_poster {
        return SchedulePriority::NeedPoster;
    }

    if !schedule.has_tags {
        return SchedulePriority::NeedTags;
    }

    if !schedule.has_description {
        return SchedulePriority::NeedDescription;
    }

    if !schedule.has_all_episodes {
        return SchedulePriority::NeedEpisodes;
    }

    if !schedule.has_rating {
        return SchedulePriority::NeedRating;
    }

    if !schedule.has_anidb_id || !schedule.has_mal_id || !schedule.has_ann_id {
        return SchedulePriority::NeedExternalSources;
    }

    SchedulePriority::Idle
}

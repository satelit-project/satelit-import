use actix_protobuf::{ProtoBuf, ProtoBufResponseBuilder};
use actix_web::dev::{AppService, HttpServiceFactory};
use actix_web::error::BlockingError;
use actix_web::{web, web::Data, HttpResponse};
use futures::Future;
use log::{error, info};

use crate::db::entity::{ExternalSource, SchedulePriority, Task, UpdatedSchedule};
use crate::db::scheduled_tasks::ScheduledTasks;
use crate::db::schedules::Schedules;
use crate::db::tasks::Tasks;
use crate::db::{ConnectionPool, QueryError};
use crate::proto::scraper::{anime, task};

/// Service for scraper's tasks manipulation
pub struct TasksService<P: ConnectionPool + 'static> {
    tasks: Tasks<P>,
    schedules: Schedules<P>,
    scheduled_tasks: ScheduledTasks<P>,
}

impl<P: ConnectionPool + 'static> TasksService<P> {
    pub fn new(
        tasks: Tasks<P>,
        schedules: Schedules<P>,
        scheduled_tasks: ScheduledTasks<P>,
    ) -> Self {
        Self {
            tasks,
            schedules,
            scheduled_tasks,
        }
    }
}

impl<P: ConnectionPool + 'static> HttpServiceFactory for TasksService<P> {
    fn register(self, config: &mut AppService) {
        let service = web::scope("/scraper/task")
            .data(Data::new(self.tasks))
            .data(Data::new(self.schedules))
            .data(Data::new(self.scheduled_tasks))
            .service(web::resource("/").route(web::post().to_async(create_task::<P>)))
            .service(web::resource("/{task_id}/yield").route(web::post().to_async(task_yield::<P>)))
            .service(
                web::resource("/{task_id}/finish").route(web::post().to_async(task_finish::<P>)),
            );

        service.register(config);
    }
}

/// Registers new scrape task and returns schedules ID's that should be scraped
fn create_task<P: ConnectionPool + 'static>(
    tasks: Data<Tasks<P>>,
    scheduled_tasks: Data<ScheduledTasks<P>>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    web::block(move || {
        let id = uuid::Uuid::new_v4().to_string();
        let task = Task::new(id, ExternalSource::AniDB);

        tasks.register(&task)?;
        scheduled_tasks.create(&task, 10)?; // TODO: should be parameter
        let scheduled = scheduled_tasks.for_task(&task)?;
        let mut anime_ids = vec![];
        let mut schedule_ids = vec![];

        for (_, schedule) in scheduled {
            anime_ids.push(schedule.source_id);
            schedule_ids.push(schedule.id);
        }

        Ok(task::Task {
            id: task.id,
            source: task.source as i32,
            schedule_ids,
            anime_ids,
        })
    })
    .then(
        |res: Result<task::Task, BlockingError<QueryError>>| match res {
            Ok(task) => HttpResponse::Ok().protobuf(task),
            Err(e) => {
                error!("failed to create new scrape task: {}", e);
                Ok(HttpResponse::InternalServerError().into())
            }
        },
    )
}

/// Represents result of the `/yield` response
enum TaskYieldResult {
    /// Anime entity is missed
    AnimeMissed,

    /// Request successful
    Ok,
}

/// Updates associated `Schedule` with new scraped anime data and pushes changes to main service
fn task_yield<P: ConnectionPool + 'static>(
    proto: ProtoBuf<task::TaskYield>,
    schedules: Data<Schedules<P>>,
    scheduled_tasks: Data<ScheduledTasks<P>>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    web::block(move || {
        // TODO: pass parsed stuff to main service

        let anime = match proto.anime {
            Some(ref a) => a,
            None => {
                info!(
                    "received 'TaskYield' without anime entity, 'task_id': {}, 'schedule_id': {}",
                    proto.task_id, proto.schedule_id
                );

                return Ok(TaskYieldResult::AnimeMissed);
            }
        };

        let update = update_for_anime(anime);
        schedules.update_for_id(proto.schedule_id, &update)?;
        scheduled_tasks.complete_for_schedule(&proto.task_id, proto.schedule_id)?;

        Ok(TaskYieldResult::Ok)
    })
    .then(
        |result: Result<TaskYieldResult, BlockingError<QueryError>>| {
            let result = match result {
                Ok(v) => v,
                Err(e) => {
                    error!("failed to update yielded entity: {}", e);
                    return Ok(HttpResponse::InternalServerError().into());
                }
            };

            match result {
                TaskYieldResult::Ok => Ok(HttpResponse::Ok().into()),
                TaskYieldResult::AnimeMissed => Ok(HttpResponse::BadRequest().into()),
            }
        },
    )
}

/// Removes all scheduled tasks associated with provided task
/// and updates schedules to be in Finished state
fn task_finish<P: ConnectionPool + 'static>(
    proto: ProtoBuf<task::TaskFinish>,
    tasks: Data<Tasks<P>>,
) -> impl Future<Item = HttpResponse, Error = actix_web::Error> {
    web::block(move || tasks.remove(&proto.task_id)).then(|result| match result {
        Ok(()) => Ok(HttpResponse::Ok().into()),
        Err(e) => {
            error!("failed to finish task: {}", e);
            Ok(HttpResponse::InternalServerError().into())
        }
    })
}

// Helpers

fn update_for_anime(anime: &anime::Anime) -> UpdatedSchedule {
    use anime::anime::Type as AnimeType;
    use anime::episode::Type as EpisodeType;

    let mut schedule = UpdatedSchedule::default();
    schedule.has_poster = !anime.poster_url.is_empty();
    schedule.has_air_date = anime.start_date != 0.0 && anime.end_date != 0.0;

    let anime_type = AnimeType::from_i32(anime.r#type).unwrap_or(AnimeType::Unknown);
    schedule.has_type = anime_type != AnimeType::Unknown;

    schedule.has_anidb_id = anime.source.as_ref().map_or(false, |s| s.anidb_id != 0);
    schedule.has_mal_id = anime.source.as_ref().map_or(false, |s| s.mal_id != 0);
    schedule.has_ann_id = anime.source.as_ref().map_or(false, |s| s.ann_id != 0);
    schedule.has_tags = !anime.tags.is_empty();
    schedule.has_episode_count = anime.episodes_count != 0;

    let unknown_eps_count = anime
        .episodes
        .iter()
        .filter(|&e| {
            let ep_type = EpisodeType::from_i32(e.r#type).unwrap_or(EpisodeType::Unknown);
            ep_type == EpisodeType::Unknown
                || e.air_date == 0.0
                || e.duration == 0.0
                || e.name.is_empty()
        })
        .count();
    schedule.has_all_episodes = unknown_eps_count == 0 && anime.episodes.len() != 0;

    schedule.has_rating = anime.rating != 0.0;
    schedule.has_description = !anime.description.is_empty();

    schedule.priority = priority_for_schedule(&schedule);

    schedule
}

fn priority_for_schedule(schedule: &UpdatedSchedule) -> SchedulePriority {
    if !schedule.has_air_date {
        return SchedulePriority::NeedAiringDetails;
    }

    if !schedule.has_type || !schedule.has_episode_count {
        return SchedulePriority::NeedAiringDetails;
    }

    if !schedule.has_tags {
        return SchedulePriority::NeedTags;
    }

    if !schedule.has_description {
        return SchedulePriority::NeedDescription;
    }

    if !schedule.has_poster {
        return SchedulePriority::NeedPoster;
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

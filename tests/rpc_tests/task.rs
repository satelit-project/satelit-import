use diesel::prelude::*;
use futures::prelude::*;
use tokio::runtime::Runtime;
use tower_grpc::Request;

use satelit_import::db::entity::*;
use satelit_import::db::{ConnectionPool, QueryError};
use satelit_import::proto::data;
use satelit_import::proto::scraping;
use satelit_import::proto::scraping::client::ScraperTasksService;
use satelit_import::rpc::ServicesBuilder;
use satelit_import::settings::Settings;

use super::all_schedules;
use crate::make_task_client;

// TODO: refactor
// TODO: compare with actual db data after rpc calls
#[test]
fn test_happy_path() -> Result<(), QueryError> {
    let mut _rt = Runtime::new().unwrap();
    let settings = make_settings();
    let pool = make_pool();
    start_rpc_server(&mut _rt, settings, pool.clone());
    super::abort_on_panic();

    let schedules = fill_schedules(&pool, ExternalSource::AniDB)?;
    assert!(all_tasks(&pool)?.is_empty());

    let run = make_task_client!(service_address())
        .and_then(|mut client| {
            let intent = scraping::TaskCreate {
                limit: 10,
                source: data::Source::Anidb as i32,
            };
            let request = Request::new(intent);
            client
                .create_task(request)
                .and_then(move |response| Ok((client, response.into_inner())))
                .map_err(|e| panic!(e))
        })
        .and_then(move |(client, task)| {
            assert_eq!(task.source, data::Source::Anidb as i32);
            for (expected, got) in schedules.iter().zip(task.jobs.iter()) {
                assert_eq!(expected.id, got.anime_id);
            }

            Ok((client, task))
        })
        .and_then(|(client, task)| {
            let scraping::Task { id, jobs, .. } = task;
            let task_id = id.clone();
            let yields = jobs.into_iter().map(move |j| {
                let mut anime = data::Anime::default();
                let mut source = data::anime::Source::default();
                source.anidb_ids.push(j.anime_id);
                anime.source = Some(source);
                anime.r#type = data::anime::Type::TvSeries as i32;

                scraping::TaskYield {
                    task_id: id.clone(),
                    job_id: j.id.clone(),
                    anime: Some(anime),
                }
            });

            futures::stream::iter_ok(yields)
                .fold(client, |client, res| {
                    client.ready().and_then(move |mut client| {
                        let request = Request::new(res);
                        client.yield_result(request).and_then(move |_| Ok(client))
                    })
                })
                .join(Ok(task_id))
        })
        .and_then(|(client, task_id)| {
            client
                .ready()
                .and_then(move |mut client| {
                    let finish = scraping::TaskFinish { task_id };
                    let req = Request::new(finish);
                    client.complete_task(req)
                })
                .and_then(|_| Ok(()))
        })
        .map_err(|e| panic!(e));

    let mut rt = Runtime::new().unwrap();
    rt.spawn(run);
    rt.shutdown_on_idle().wait().unwrap();
    _rt.shutdown_now().wait().unwrap();

    Ok(())
}

// MARK: helpers

fn start_rpc_server(rt: &mut Runtime, settings: Settings, pool: ConnectionPool) {
    use log::error;
    use satelit_import::serve_service;

    let builder = ServicesBuilder::new(settings, pool);
    let service = builder.tasks_service();
    let run = serve_service!(service, service_address());
    rt.spawn(run);
}

fn fill_schedules(
    pool: &ConnectionPool,
    source: ExternalSource,
) -> Result<Vec<Schedule>, QueryError> {
    use satelit_import::db::schema::schedules::dsl;

    let new = vec![
        NewSchedule::new(1, source),
        NewSchedule::new(2, source),
        NewSchedule::new(3, source),
        NewSchedule::new(4, source),
        NewSchedule::new(5, source),
    ];

    let conn = pool.get()?;
    diesel::insert_into(dsl::schedules)
        .values(&new)
        .execute(&conn)?;

    let inserted = all_schedules(&pool)?;
    assert_eq!(new.len(), inserted.len());

    Ok(inserted)
}

fn all_tasks(pool: &ConnectionPool) -> Result<Vec<Task>, QueryError> {
    use satelit_import::db::schema::tasks::dsl;

    let conn = pool.get()?;
    let tasks = dsl::tasks.load(&conn)?;
    Ok(tasks)
}

fn make_pool() -> ConnectionPool {
    crate::connection_pool("task-rpc-tests")
}

fn make_settings() -> Settings {
    use satelit_import::settings::Profile;
    Settings::new(Profile::Named("rpc_tests".to_string())).unwrap()
}

fn service_address() -> String {
    make_settings().rpc().task().to_string()
}

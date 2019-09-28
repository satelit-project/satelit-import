mod import;
mod task;

use diesel::prelude::*;

use satelit_import::db::entity::Schedule;
use satelit_import::db::{ConnectionPool, QueryError};

// MARK: db

fn all_schedules(pool: &ConnectionPool) -> Result<Vec<Schedule>, QueryError> {
    use satelit_import::db::schema::schedules::dsl::*;

    let conn = pool.get()?;
    let loaded = schedules.load(&conn)?;
    Ok(loaded)
}

// MARK: rpc
// TODO: pass type to macros

#[macro_export]
macro_rules! make_import_client {
    ( $ip:expr ) => {{
        use std::time::Duration;
        use tower::MakeService;
        use tower_hyper::util::{Destination, HttpConnector};
        use tower_hyper::{client, util};

        let uri: hyper::Uri = format!("http://{}/", $ip).parse().unwrap();
        let dst = Destination::try_from_uri(uri.clone()).unwrap();
        let mut conn = HttpConnector::new(2);
        conn.set_keepalive(Some(Duration::new(60, 0)));

        let settings = client::Builder::new().http2_only(true).clone();
        let mut make = client::Connect::with_builder(util::Connector::new(conn), settings);

        make.make_service(dst)
            .map_err(|e| panic!(e))
            .and_then(|conn| {
                let conn = tower_request_modifier::Builder::new()
                    .set_origin(uri)
                    .build(conn)
                    .unwrap();

                ImportService::new(conn).ready()
            })
    }};
}

#[macro_export]
macro_rules! make_task_client {
    ( $ip:expr ) => {{
        use std::time::Duration;
        use tower::MakeService;
        use tower_hyper::util::{Destination, HttpConnector};
        use tower_hyper::{client, util};

        let uri: hyper::Uri = format!("http://{}/", $ip).parse().unwrap();
        let dst = Destination::try_from_uri(uri.clone()).unwrap();
        let mut conn = HttpConnector::new(2);
        conn.set_keepalive(Some(Duration::new(60, 0)));

        let settings = client::Builder::new().http2_only(true).clone();
        let mut make = client::Connect::with_builder(util::Connector::new(conn), settings);

        make.make_service(dst)
            .map_err(|e| panic!(e))
            .and_then(|conn| {
                let conn = tower_request_modifier::Builder::new()
                    .set_origin(uri)
                    .build(conn)
                    .unwrap();

                ScraperTasksService::new(conn).ready()
            })
    }};
}

// MARK: util

fn abort_on_panic() {
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("{}", panic_info.to_string());
        std::process::abort();
    }));
}

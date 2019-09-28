use futures::prelude::*;
use tokio::runtime::Runtime;
use tower_grpc::Request;

use satelit_import::db::entity::*;
use satelit_import::db::ConnectionPool;
use satelit_import::proto::data::Source;
use satelit_import::proto::import::client::ImportService;
use satelit_import::proto::import::ImportIntent;
use satelit_import::proto::uuid::Uuid;
use satelit_import::rpc::ServicesBuilder;
use satelit_import::settings::Settings;

use crate::make_import_client;
use super::all_schedules;

#[test]
fn test_happy_path() {
    let mut _rt = Runtime::new().unwrap();
    let settings = make_settings();
    let pool = make_pool();
    start_rpc_server(&mut _rt, settings, pool.clone());
    super::abort_on_panic();

    let intent = ImportIntent {
        id: Some(Uuid { uuid: vec![] }),
        source: Source::Anidb as i32,
        new_index_url: index_url(IndexFile::V1),
        old_index_url: index_url(IndexFile::V0),
        reimport_ids: vec![],
    };

    let run = make_import_client!(service_address())
        .and_then(move |mut client| client.start_import(Request::new(intent)))
        .map_err(|e| panic!(format!("{:?}", e)))
        .and_then(move |response| {
            let result = response.into_inner();
            assert!(result.skipped_ids.is_empty());

            let schedules = all_schedules(&pool).unwrap();
            assert_eq!(schedules.len(), IndexFile::V1.anime_ids().len());
            assert!(
                schedules
                    .iter()
                    .all(|s| s.has_anidb_id && s.source == ExternalSource::AniDB)
            );

            Ok(())
        });

    let mut rt = Runtime::new().unwrap();
    rt.spawn(run);
    rt.shutdown_on_idle().wait().unwrap();
    _rt.shutdown_now().wait().unwrap();
}

// MARK: helpers

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
enum IndexFile {
    // nginx served files
    V0,
    V1,
}

fn index_url(file: IndexFile) -> String {
    // from nginx.conf
    format!("http://127.0.0.1:8081/{}", file.name())
}

fn start_rpc_server(rt: &mut Runtime, settings: Settings, pool: ConnectionPool) {
    use log::error;
    use satelit_import::serve_service;

    let builder = ServicesBuilder::new(settings, pool);
    let service = builder.import_service();
    let run = serve_service!(service, service_address());
    rt.spawn(run);
}

fn make_pool() -> ConnectionPool {
    crate::connection_pool("import-rpc-tests")
}

fn make_settings() -> Settings {
    use satelit_import::settings::Profile;
    Settings::new(Profile::Named("rpc_tests".to_string())).unwrap()
}

fn service_address() -> String {
    make_settings().rpc().import().to_string()
}

// MARK: impl IndexFile

impl IndexFile {
    fn name(&self) -> String {
        let part = toml::ser::to_string(self).unwrap();
        format!("small-1{}.xml.gz", part.trim_matches('"'))
    }

    fn anime_ids(&self) -> Vec<i32> {
        use IndexFile::*;

        match self {
            V0 => vec![],
            V1 => (1..=10).collect(),
        }
    }
}

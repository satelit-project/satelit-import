mod dump;

use futures::prelude::*;

use satelit_import::db;
use satelit_import::settings;
use satelit_import::worker;

use std::path::PathBuf;

#[test]
fn test_initial_import() {
    dump::init_env();

    tokio::run(make_import_task(DumpBundle::SmallV1));

    dump::deinit_env();
}

fn make_import_task(bundle: DumpBundle) -> impl Future<Item = (), Error = ()> + Send {
    let settings = settings::Settings::new(settings::Profile::IntegrationTests).unwrap();
    let pool = db::new_connection_pool(settings.db()).unwrap();

    let mut download_url = PathBuf::from(settings.import().dump_url());
    download_url.push(bundle.file_name());

    let download = worker::dump::downloader(
        download_url.to_str().unwrap().to_owned(),
        settings.import().download_path().to_owned(),
    );

    let copy = worker::dump::copier(
        settings.import().dump_path().to_owned(),
        settings.import().old_dump_path().to_owned(),
    );

    let extract = worker::dump::extractor(
        settings.import().download_path().to_owned(),
        settings.import().dump_path().to_owned(),
        settings.import().chunk_size(),
    );

    let import = worker::dump::importer(
        settings.import().old_dump_path().to_owned(),
        settings.import().dump_path().to_owned(),
        pool,
    );

    worker::dump::DumpImportTask::new(download, copy, extract, import)
}

#[derive(Debug, Clone, Copy)]
enum DumpBundle {
    SmallV1,
    SmallV2,
    SmallV3,
}

impl DumpBundle {
    fn file_name(&self) -> &'static str {
        use DumpBundle::*;

        match *self {
            SmallV1 => "small-1v1.xml.gz",
            SmallV2 => "small-1v2.xml.gz",
            SmallV3 => "small-1v3.xml.gz",
        }
    }
}

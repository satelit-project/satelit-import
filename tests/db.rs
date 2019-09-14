mod db_tests;

use satelit_import::db;
use satelit_import::settings;

fn connection_pool(id: &str) -> db::ConnectionPool {
    let profile = settings::Profile::Test;
    let settings = settings::Settings::new(profile).expect("failed to read settings");
    let mut settings = settings.db().clone();
    settings.url.push_str(id);

    db::new_connection_pool(&settings).expect("failed to connect to db")
}

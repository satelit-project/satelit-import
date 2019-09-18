use satelit_import::db;
use satelit_import::settings;

pub fn settings(profile: &str) -> settings::Settings {
    let profile = settings::Profile::Named(profile.to_string());
    settings::Settings::new(profile).expect("failed to read settings")
}

pub fn connection_pool(profile: &str, id: &str) -> db::ConnectionPool {
    let mut settings = settings(profile).db;
    settings.url.push_str(id);
    db::new_connection_pool(&settings).expect("failed to connect to db")
}

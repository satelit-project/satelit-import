use actix_files as af;
use actix_rt;
use actix_web::{middleware, App, HttpServer};
use env_logger;

use satelit_import::settings::{Profile, Settings};

use std::path::PathBuf;
use std::process::Command;

pub fn init_env() {
    env_logger::init();

    let config = Settings::new(Profile::IntegrationTests).unwrap();

    setup_resources(&config);
    setup_db(&config);
    serve_dumps();
}

pub fn deinit_env() {
    rm_tmp_dir();
}

fn setup_resources(config: &Settings) {
    create_tmp_dir();
    std::fs::File::create(config.import().old_dump_path()).unwrap();
}

fn setup_db(config: &Settings) {
    Command::new("diesel")
        .arg("setup")
        .arg("--database-url")
        .arg(config.db().path())
        .status()
        .unwrap();
}

fn serve_dumps() {
    std::thread::spawn(|| {
        let server = HttpServer::new(move || {
            let mut dumps_path = PathBuf::new();
            dumps_path.push(env!("CARGO_MANIFEST_DIR"));
            dumps_path.push("resources/tests/dumps");

            App::new()
                .wrap(middleware::Logger::default())
                .service(af::Files::new("/static", dumps_path).show_files_listing())
        })
        .bind("localhost:8080")
        .unwrap();

        let rt = actix_rt::System::new("actix-rt");
        let server = server.start();
        rt.run().unwrap();
    });
}

fn create_tmp_dir() {
    let path = tmp_dir_path();
    if path.exists() {
        rm_tmp_dir();
    }

    std::fs::create_dir(path).unwrap();
}

fn rm_tmp_dir() {
    let path = tmp_dir_path();
    if path.exists() {
        std::fs::remove_dir_all(path).unwrap();
    }
}

fn tmp_dir_path() -> PathBuf {
    let mut path = PathBuf::new();
    path.push(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/tests/tmp");
    path
}

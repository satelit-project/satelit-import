use actix_web::{middleware, HttpServer};
use clap::{crate_authors, crate_version};
use clap::{App, Arg};

use std::path::Path;

fn main() {
    env_logger::init_from_env(env_logger::Env::new().filter("SATELIT_LOG"));

    let app = build_app();
    let matches = app.get_matches();

    let port = matches.value_of("port").unwrap().to_string();
    let paths: Vec<String> = matches
        .values_of("path")
        .unwrap()
        .map(|s| s.to_string())
        .collect();
    let names: Vec<String> = matches
        .values_of("name")
        .unwrap()
        .map(|s| s.to_string())
        .collect();

    serve_files(port.to_string(), paths, names);
}

fn build_app<'a, 'b>() -> App<'a, 'b> {
    App::new("serve")
        .about("Serve files from a directory")
        .author(crate_authors!())
        .version(crate_version!())
        .after_help("NOTE: for every 'path' argument you must provide 'name' argument.")
        .arg(
            Arg::with_name("path")
                .long("path")
                .help("Path to a directory from where to serve files")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
                .required(true)
                .validator(validate_directory),
        )
        .arg(
            Arg::with_name("name")
                .long("name")
                .help("Name of previously specified directory. It will be used as path to files.")
                .takes_value(true)
                .multiple(true)
                .number_of_values(1)
                .required(true),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .help("Port on 'localhost' where to serve files")
                .takes_value(true)
                .number_of_values(1)
                .default_value("8082")
                .validator(validate_port),
        )
}

fn validate_directory(p: String) -> Result<(), String> {
    let path = Path::new(&p);
    if path.is_dir() {
        return Ok(());
    }

    Err(format!("Path '{}' is not a directory", p))
}

fn validate_port(p: String) -> Result<(), String> {
    match p.parse::<i32>() {
        Ok(_) => Ok(()),
        Err(_) => Err(format!("Specified port is not a number")),
    }
}

fn serve_files(port: String, paths: Vec<String>, names: Vec<String>) {
    let server = HttpServer::new(move || {
        let mut app = actix_web::App::new().wrap(middleware::Logger::default());

        for (path, name) in paths.iter().zip(names.iter()) {
            app = app.service(actix_files::Files::new(
                &format!("/{}", name),
                Path::new(path.as_str()),
            ));
        }

        app
    });

    server
        .bind(format!("127.0.0.1:{}", port))
        .expect(&format!(
            "Failed to create server at localhost at port {}",
            port
        ))
        .run()
        .expect("Failed to start server");
}

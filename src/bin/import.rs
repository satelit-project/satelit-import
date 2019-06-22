use actix_protobuf::ProtoBuf;
use actix_web::{middleware, post, HttpResponse, HttpServer};
use clap::{crate_authors, crate_version};
use clap::{App, Arg};
use prost::Message;
use reqwest::{header, Client, Url};

use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::iter::FromIterator;
use std::thread::{self, JoinHandle};

use satelit_import::proto::scheduler::intent::{ImportIntent, ImportIntentResult};

fn main() {
    env_logger::init();
    let port = option_env!("ACTIX_WEB_PORT").unwrap_or("8081").to_string();

    let app = build_app();
    let matches = app.get_matches();

    let base_url = matches.value_of("base-url").unwrap();
    let path = matches.value_of("path").unwrap_or("");
    let keys: Vec<&str> = matches.values_of("field").unwrap().collect();
    let values: Vec<&str> = matches.values_of("value").unwrap().collect();

    let data = request_data_for_path(path, &port, &keys, &values);
    let handle = start_server(port);
    send_data_request(base_url, path, data);

    handle.join().expect("Failed to join server thread");
}

fn build_app<'a, 'b>() -> App<'a, 'b> {
    App::new("import")
        .about("Interact with /import API")
        .author(crate_authors!())
        .version(crate_version!())
        .arg(
            Arg::with_name("base-url")
                .long("base-url")
                .help("API base URL")
                .takes_value(true)
                .number_of_values(1)
                .required(true),
        )
        .arg(
            Arg::with_name("path")
                .long("path")
                .help("API path for 'import' scope")
                .takes_value(true)
                .number_of_values(1)
                .validator(validate_path),
        )
        .arg(
            Arg::with_name("field")
                .long("field")
                .help("Specifies a field name of the proto associated with the API")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true),
        )
        .arg(
            Arg::with_name("value")
                .long("value")
                .help("A value for a previously specified proto's field name")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true),
        )
}

fn validate_path(path: String) -> Result<(), String> {
    const SUPPORTED_PATHS: [&str; 1] = ["intent"];

    let supported_paths = HashSet::<_, RandomState>::from_iter(SUPPORTED_PATHS.to_vec());
    if supported_paths.contains(path.as_str()) {
        return Ok(());
    }

    let msg = format!(
        "Path '{}' is not supported. Only the following paths are supported: {}",
        path,
        SUPPORTED_PATHS.to_vec().join(", ")
    );

    Err(msg)
}

fn request_data_for_path(path: &str, port: &str, keys: &[&str], values: &[&str]) -> Vec<u8> {
    assert_eq!(path, "intent");

    let mut intent = ImportIntent {
        id: "".to_string(),
        source: 0,
        reimport_ids: vec![],
        callback_url: format!("http://127.0.0.1:{}/handle_intent", port),
    };

    for (&key, &value) in keys.into_iter().zip(values) {
        match key {
            "id" => intent.id = value.to_string(),
            "source" => {
                intent.source = value
                    .parse::<i32>()
                    .expect("Failed to parse 'source' value: not an integer")
            }
            "reimport_ids" => intent.reimport_ids.push(
                value
                    .parse::<i32>()
                    .expect("Failed to parse 'reimport_ids' value: not an integer"),
            ),
            "callback_url" => intent.callback_url = value.to_string(),
            _ => panic!("Parameter not supported: {}", value),
        }
    }

    let mut body = vec![];
    intent.encode(&mut body).expect("Failed to encode proto");
    body
}

fn start_server(port: String) -> JoinHandle<()> {
    thread::spawn(move || {
        let server = HttpServer::new(|| {
            actix_web::App::new()
                .wrap(middleware::Logger::default())
                .service(handle_intent)
        })
        .bind(format!("127.0.0.1:{}", port))
        .expect(&format!(
            "Failed to create server at localhost at port {}",
            port
        ));

        server.run().expect("Failed to start server");
    })
}

#[post("/handle_intent")]
fn handle_intent(proto: ProtoBuf<ImportIntentResult>) -> HttpResponse {
    eprintln!("Received callback: {:#?}", proto);
    HttpResponse::Ok().into()
}

fn send_data_request(base_url: &str, path: &str, data: Vec<u8>) {
    let url = Url::parse(&base_url)
        .expect(&format!("Failed to parse URL: {}", base_url))
        .join("import/")
        .expect("Failed to construct URL")
        .join(path)
        .expect("Failed to construct URL");

    let response = Client::new()
        .post(url)
        .header(header::CONTENT_TYPE, "application/protobuf")
        .body(data)
        .send()
        .expect("Request failed");

    response
        .error_for_status()
        .expect("Request was unsuccessful");
}

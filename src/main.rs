use tonic::transport::Server;
use tracing::info;
use tracing_subscriber::{filter::LevelFilter, FmtSubscriber};

use satelit_import::{db, rpc, settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("loading configuration");
    let config = settings::Settings::new(settings::Profile::Default)?;

    info!("connecting to database");
    let pool = db::new_connection_pool(config.db())?;

    info!("starting services");
    let builder = rpc::ServicesBuilder::new(config.clone(), pool);
    let addr = format!(":{}", config.rpc().port()).parse()?;
    Server::builder()
        .add_service(builder.import_service())
        .add_service(builder.tasks_service())
        .serve(addr)
        .await?;

    Ok(())
}

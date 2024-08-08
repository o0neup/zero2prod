use std::net::TcpListener;

use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio_macros::main]
async fn main() -> Result<(), std::io::Error> {
    LogTracer::init().expect("Failed to set logger");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);

    let subsctiber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subsctiber).expect("Failed to set subscriber");

    let settings = get_configuration().expect("Failed to read configuration.yaml");
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", settings.app_port))
        .unwrap_or_else(|_| panic!("Failed to bind to port {}.", settings.app_port));
    let pool = PgPool::connect(&settings.database.connection_string())
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Failed to connect to psql at {}",
                &settings.database.connection_string()
            )
        });
    run(listener, pool)?.await
}

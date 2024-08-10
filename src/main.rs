use std::net::TcpListener;

use sqlx::postgres::PgPoolOptions;
use zero2prod::{configuration::get_configuration, startup::run, telemetry};

#[tokio_macros::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let settings = get_configuration().expect("Failed to read configuration.yaml");
    let listener = TcpListener::bind(&format!(
        "{}:{}",
        settings.application.host, settings.application.port
    ))
    .unwrap_or_else(|_| {
        panic!(
            "Failed to bind to address {}:{}.",
            settings.application.host, settings.application.port
        )
    });
    let pool = PgPoolOptions::new().connect_lazy_with(settings.database_url.0);
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .unwrap_or_else(|e| panic!("Failed to apply migrations: {}", e));
    run(listener, pool)?.await
}

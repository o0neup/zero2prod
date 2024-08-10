use std::net::TcpListener;

use secrecy::ExposeSecret;
use sqlx::PgPool;
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
    let pool = PgPool::connect_lazy(settings.database.connection_string().expose_secret())
        .unwrap_or_else(|_| {
            panic!(
                "Failed to connect to psql at {}",
                &settings.database.connection_string().expose_secret()
            )
        });
    run(listener, pool)?.await
}

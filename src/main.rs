use std::net::TcpListener;

use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio_macros::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = get_configuration().expect("Failed to read configuration.yaml");
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", settings.app_port))
        .unwrap_or_else(|_| panic!("Failed to bind to port {}.", settings.app_port));
    let pool = PgPool::connect(&settings.database.connection_string())
        .await
        .unwrap_or_else(|_| panic!("Failed to connect to psql at {}",
            &settings.database.connection_string()));
    run(listener, pool)?.await
}

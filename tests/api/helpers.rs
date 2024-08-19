use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use zero2prod::{
    configuration::get_configuration,
    startup::{get_connection_pool, Application},
    telemetry::{get_subscriber, init_subscriber},
};

static TRACING: Lazy<()> = Lazy::new(|| {
    let name = "IT".into();
    let env_filter = "info".into();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(name, env_filter, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(name, env_filter, std::io::sink);
        init_subscriber(subscriber);
    }
});

#[derive(Debug)]
pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("http://{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute remote request")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        let dbname = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .filter(|c| !c.is_ascii_digit())
            .take(10)
            .map(char::from)
            .collect::<String>()
            .to_lowercase()
            .replace("-", "_");
        let pg_options = c.database_url.0.clone().database(&dbname);
        c.application.port = 0;
        c.database_url.0 = pg_options;
        c
    };
    configure_database(configuration.database_url.clone().0).await;

    let server = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");

    let address = format!("127.0.0.1:{}", server.port());
    // Launch as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it
    let _ = tokio::spawn(server.run_until_stopped());
    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database_url),
    }
}

async fn configure_database(pg_options: PgConnectOptions) -> PgPool {
    let options_without_db = pg_options.clone().database("");
    let mut connection = PgConnection::connect_with(&options_without_db)
        .await
        .expect("Failed to connect to postgres");
    connection
        .execute(
            format!(
                r#"CREATE DATABASE {}"#,
                &pg_options
                    .get_database()
                    .expect("Expect db name to be present!")
            )
            .as_str(),
        )
        .await
        .unwrap_or_else(|_| {
            panic!(
                "Failed to create database {}.",
                &pg_options
                    .get_database()
                    .expect("Expect db name to be present!")
            )
        });

    let pool = PgPoolOptions::new().connect_lazy_with(pg_options);
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to apply migrations");

    pool
}

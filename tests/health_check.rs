use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use std::net::TcpListener;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
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

#[tokio_macros::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("http://{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request to health_check.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio_macros::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=kotleta&email=2hcompany%40gmail.com";
    let response = client
        .post(&format!("http://{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "2hcompany@gmail.com");
    assert_eq!(saved.name, "kotleta");
}

#[tokio_macros::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=[kotleta]&email=2hcompany%40gmail.com", "invalid name"),
        (
            "name=DROP TABLE subscriptions;&email=2hcompany%40gmail.com",
            "invalid name",
        ),
    ];
    for (body, _error) in test_cases {
        let response = client
            .post(&format!("http://{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            body
        );
    }
}

#[tokio_macros::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=kotleta", "Missing the email."),
        ("email=2hcompany%40gmail.com", "Missing the name."),
        ("", "Missing both name and email."),
    ];
    for (body, error) in test_cases {
        let response = client
            .post(&format!("http://{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            error
        );
    }
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let configuration = get_configuration().expect("Failed to fetch configuration!");
    let test_db_name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .filter(|c| !c.is_ascii_digit())
        .take(10)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
        .replace("-", "_");
    let options_for_test = configuration.database_url.0.database(&test_db_name);
    let pool = configure_database(options_for_test).await;
    let server = run(listener, pool.clone()).expect("Failed to bind address!");

    // Launch as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it
    let _ = tokio::spawn(server);
    TestApp {
        address: format!("127.0.0.1:{}", &port),
        db_pool: pool,
    }
}

pub async fn configure_database(pg_options: PgConnectOptions) -> PgPool {
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

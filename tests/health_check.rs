use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
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
    let mut configuration = get_configuration().expect("Failed to fetch configuration!");
    configuration.database.database_name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect::<String>()
        .to_lowercase();
    let pool = configure_database(&configuration.database).await;
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

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection =
        PgConnection::connect(config.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to postgres");
    connection
        .execute(format!(r#"CREATE DATABASE {}"#, &config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let pool = PgPool::connect(config.connection_string().expose_secret())
        .await
        .expect("Failed to create PgPool");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to apply migrations");

    pool
}

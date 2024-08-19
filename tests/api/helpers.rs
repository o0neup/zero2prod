use once_cell::sync::Lazy;
use rand::{distributions::Alphanumeric, Rng};
use reqwest::Url;
use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    Connection, Executor, PgConnection, PgPool,
};
use wiremock::MockServer;
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
    pub port: u16,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
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

    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
        let get_link = |s: &str| {
            let links: Vec<_> = linkify::LinkFinder::new()
                .links(s)
                .filter(|l| *l.kind() == linkify::LinkKind::Url)
                .collect();
            assert_eq!(links.len(), 1);
            let raw_link = links[0].as_str().to_owned();
            let mut confirmation_link = Url::parse(&raw_link).unwrap();
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            confirmation_link.set_port(Some(self.port)).unwrap();
            confirmation_link
        };
        let html = get_link(body["HtmlBody"].as_str().unwrap());
        let plain_text = get_link(body["TextBody"].as_str().unwrap());
        ConfirmationLinks { html, plain_text }
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;
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
        c.email_client.base_url = email_server.uri();
        c
    };
    configure_database(configuration.database_url.clone().0).await;

    let server = Application::build(configuration.clone())
        .await
        .expect("Failed to build application");
    let port = server.port();
    let address = format!("127.0.0.1:{}", &port);
    // Launch as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it
    let _ = tokio::spawn(server.run_until_stopped());
    TestApp {
        address,
        port,
        db_pool: get_connection_pool(&configuration.database_url),
        email_server,
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

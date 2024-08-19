use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{DbOptions, Settings},
    email_client::EmailClient,
    routes::{confirm, health_check, subscribe},
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(settings: Settings) -> Result<Application, std::io::Error> {
        let pool = get_connection_pool(&settings.database_url);
        let sender_email = settings
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = settings.email_client.timeout();
        let email_client = EmailClient::new(
            settings.email_client.base_url,
            sender_email,
            timeout,
            settings.email_client.auth_token,
        );
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
        let port = listener.local_addr().unwrap().port();
        let server = run(listener, pool, email_client, settings.application.base_url)?;
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configutaion: &DbOptions) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configutaion.0.clone())
}

pub struct ApplicationBaseUrl(pub String);

pub fn run(
    listener: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, std::io::Error> {
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .route("/health_check", web::get().to(health_check))
            .route("/subscriptions", web::post().to(subscribe))
            .route("/subscriptions/confirm", web::get().to(confirm))
            .app_data(connection.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

use zero2prod::{configuration::get_configuration, startup::Application, telemetry};

#[tokio_macros::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber = telemetry::get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    let settings = get_configuration().expect("Failed to read configuration.yaml");
    let server = Application::build(settings).await?;
    server.run_until_stopped().await?;
    Ok(())
}

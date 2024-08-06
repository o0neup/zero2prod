use std::net::TcpListener;

use zero2prod::{configuration::get_configuration, startup::run};

#[tokio_macros::main]
async fn main() -> Result<(), std::io::Error> {
    let settings = get_configuration().expect("Failed to read configuration.yaml");
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", settings.app_port))
        .expect(&format!("Failed to bind to port {}.", settings.app_port));
    run(listener)?.await
}

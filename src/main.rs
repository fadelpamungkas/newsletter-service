use std::net::TcpListener;

use newsletter_service::{config, startup, telemetry};
use secrecy::ExposeSecret;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let subscriber =
        telemetry::get_subscriber("newsletter_service".into(), "info".into(), std::io::stdout);
    telemetry::init_subscriber(subscriber);

    // Read configuration file
    let config = config::get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(&config.database.connection_string().expose_secret())
        .await
        .expect("Failed to connect to PostgreSQL");

    // Create a TCP listener bound to the address we configured
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind address");

    // Run the application
    startup::run(listener, connection_pool)?.await
}

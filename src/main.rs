use std::net::TcpListener;

use newsletter_service::{config::get_configuration, startup::run};
use sqlx::PgPool;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    // Read configuration file
    let config = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(&config.database.connection_string())
        .await
        .expect("Failed to connect to PostgreSQL");

    // Create a TCP listener bound to the address we configured
    let address = format!("127.0.0.1:{}", config.application_port);
    let listener = TcpListener::bind(address).expect("Failed to bind address");

    // Run the application
    run(listener, connection_pool)?.await
}

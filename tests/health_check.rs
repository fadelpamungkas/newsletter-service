use std::net::TcpListener;

use newsletter_service::{config, telemetry};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::{Connection, Executor, PgConnection, PgPool};

// Ensure that the tracing stack is only initialized once using 'once_cell'
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        telemetry::init_subscriber(subscriber);
    } else {
        let subscriber =
            telemetry::get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        telemetry::init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub connection_pool: sqlx::PgPool,
}

// Launch our application in the background ~
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut config = config::get_configuration().expect("Failed to read configuration");
    config.database.name = uuid::Uuid::new_v4().to_string();

    let connection_pool = configure_database(&config.database).await;

    let server = newsletter_service::startup::run(listener, connection_pool.clone())
        .expect("Failed to bind address.");

    // Launch the server as a background task
    tokio::spawn(server);

    TestApp {
        address,
        connection_pool,
    }
}

pub async fn configure_database(config: &config::DatabaseSettings) -> sqlx::PgPool {
    // Create database
    let mut connection =
        PgConnection::connect(&config.connection_string_no_database().expose_secret())
            .await
            .expect("Failed to connect to PostgreSQL");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Failed to connect to PostgreSQL");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    // Launch spawn_app in the background and return the address
    let app = spawn_app().await;

    // Bring reqwest to perform a request to our health_check endpoint
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    // assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Launch spawn_app in the background and return the address
    let app = spawn_app().await;

    // Bring reqwest to perform a request to our health_check endpoint
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.connection_pool)
        .await
        .expect("Failed to fetch subscriptions");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Launch spawn_app in the background and return the address
    let app = spawn_app().await;

    // Bring reqwest to perform a request to our health_check endpoint
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // assert
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

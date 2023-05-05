use std::net::TcpListener;

use newsletter_service::config::get_configuration;
use sqlx::{Connection, PgConnection};

// Launch our application in the background ~
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = newsletter_service::startup::run(listener).expect("Failed to bind address.");

    // Launch the server as a background task
    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    // Launch spawn_app in the background and return the address
    let address = spawn_app();

    // Bring reqwest to perform a request to our health_check endpoint
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", address))
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
    let address = spawn_app();

    let config = get_configuration().expect("Failed to read configuration");
    let connection_string = config.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to PostgreSQL");

    // Bring reqwest to perform a request to our health_check endpoint
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    // assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch subscriptions");

    assert_eq!(saved.email, "hello@gmail.com");
    assert_eq!(saved.name, "hello");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Launch spawn_app in the background and return the address
    let address = spawn_app();

    // Bring reqwest to perform a request to our health_check endpoint
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", address))
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

// #[tokio::test]
// async fn subscribe_returns_a_400_when_fields_are_present_but_invalid() {
//     // Launch spawn_app in the background and return the address
//     let address = spawn_app();
//
//     // Bring reqwest to perform a request to our health_check endpoint
//     let client = reqwest::Client::new();
//
//     let test_cases = vec![
//         (
//             "name=&email=ursula_le_guin%40gmail.com",
//             "empty name",
//             "missing the name",
//         ),
//         (
//             "name=ursula_le_guin&email=",
//             "empty email",
//             "missing the email",
//         ),
//         (
//             "name=ursula_le_guin&email=not-an-email",
//             "invalid email",
//             "missing the email",
//         ),
//     ];
//
//     for (invalid_body, error_message, error_description) in test_cases {
//         let response = client
//             .post(format!("{}/subscriptions", address))
//             .header("Content-Type", "application/x-www-form-urlencoded")
//             .body(invalid_body)
//             .send()
//             .await
//             .expect("Failed to execute request.");
//
//         // assert
//         assert_eq!(
//             400,
//             response.status().as_u16(),
//             "The API did not fail with 400 Bad Request when the payload was {}.",
//             error_description
//         );
//     }
// }

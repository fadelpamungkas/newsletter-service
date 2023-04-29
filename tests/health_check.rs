use std::net::TcpListener;

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

// Launch our application in the background ~
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listener.local_addr().unwrap().port();
    let server = newsletter_service::run(listener).expect("Failed to bind address.");

    // Launch the server as a background task
    tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

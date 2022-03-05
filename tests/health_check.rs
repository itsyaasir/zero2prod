// !//tests/health_check.rs

use std::{net::TcpListener, vec};

use sqlx::{Connection, Executor, PgConnection};
use zero2prod::configuration::get_configuration;

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();

    //     Use reqwest
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute the request");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

// Spawn app
// Spins up an instance of our application and returns its address

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to port a random port");

    // Retrieve the OS port
    let port = listener.local_addr().unwrap().port();

    let server = zero2prod::startup::run(listener).expect("Failed to bind address");

    // Spawn
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app();

    // Load config file
    let configuration = get_configuration().expect("Failed to load configuraion file");
    // Connection stirng
    let connection_string = configuration.database.connection_string();
    // Connection
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    // Act
    let response = client
        .post(&format!("{}/subscription", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

//
#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "Missing the email"),
        ("email=ursula_le_guin%40gmail.com", "Missing the name"),
        ("", "Missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscription", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

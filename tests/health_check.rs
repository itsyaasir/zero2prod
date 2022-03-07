//! tests/health_check.rs

use std::{net::TcpListener, vec};

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[actix_rt::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app().await;

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

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

// Spawn app
// Spins up an instance of our application and returns its address

async fn spawn_app() -> String {
    // The first time initialize is invoked the code in TRACING is executed.
    // All other invocations will instead skip execution
    Lazy::force(&TRACING);
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to port a random port");

    // Retrieve the OS port assigned by the OS
    let port = listener.local_addr().unwrap().port();
    // Address
    let address = format!("http://127.0.0.1:{}", port);
    // Load config file
    let mut configuration = get_configuration().expect("Failed to load configuraion file");
    // Generate a new db with uuid
    configuration.database.database_name = format!("test_{}", Uuid::new_v4().to_string());

    // Create a new database here
    let connection_pool = configure_db(&configuration.database).await;

    let server = run(listener, connection_pool.clone()).expect("Failed to bind address");

    // Spawn
    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    };
    format!("http://127.0.0.1:{}", port)
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app().await;

    // Load config file
    let configuration = get_configuration().expect("Failed to load configuraion file");
    // Connection string
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
    let app_address = spawn_app().await;
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

pub async fn configure_db(config: &DatabaseSettings) -> PgPool {
    // Create a database
    let connection_string = config.connection_string_without_db();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to the database");
    // Create a database
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    //
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to the database");

    // Migration
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to run migrations");

    connection_pool
}

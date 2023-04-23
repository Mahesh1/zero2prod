// use serde::Deserialize;
// use sqlx::{Connection, PgConnection};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
// use zero2prod::startup::run;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Listner Failed to bind Random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration =
        get_configuration().expect("Failed to get configuration in health_check.rs");
    configuration.database.database_name = Uuid::new_v4().to_string();
    // let connection_pool = PgPool::connect(&configuration.database.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres in health_cheeck.rs");
    let connection_pool = configure_database(&configuration.database).await;

    let server = zero2prod::startup::run(listener, connection_pool.clone())
        .expect("Run Failed to start TEST server & bind");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(dbconfig: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&dbconfig.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, dbconfig.database_name).as_str())
        .await
        .expect("Failed to create database");
    let connection_pool = PgPool::connect(&dbconfig.connection_string())
        .await
        .expect("Failed to connect to newly created db");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate newly created db");
    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    // spawn_app().await.expect("Failed to spawn our app.");
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to reqwest .... ");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_ok200_for_valid_form_data() {
    //Arrange
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    // let configuration = get_configuration().expect("Failed to db read configuration");
    // let connection_string = configuration.database.connection_string();
    // let mut connection = PgPool::connect(&connection_string)
    // .await
    // .expect("Failed to connect to Postgres");

    //Act
    let body = "name=shri%20swami&email=sarvapriya%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("test subscribe: failed to execute post request");

    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to feetch saved subscriptions");

    assert_eq!(saved.email, "sarvapriya@gmail.com");
    assert_eq!(saved.name, "shri swami");
}

#[tokio::test]
async fn subscribe_returns_bad_data_400_missing_data() {
    //subscribe test so send post req with data & check if data is bad
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=shri%20swami", "missing email"),
        ("email=sarvapriya%40gmail.com", "missing name"),
        ("", "missing both name & email"),
    ];
    for (body_testcase, error_msg) in test_cases {
        let response = client
            .post(&format!("{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body_testcase)
            .send()
            .await
            .expect("failed to execute subscriptions handler cuz of bad data test");
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API failed with 400 bad req when payload was {}.",
            error_msg
        )
    }
}

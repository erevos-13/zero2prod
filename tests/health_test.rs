use secrecy::{ExposeSecret, Secret};
use sqlx::Executor;
use sqlx::{postgres::types::PgPoint, Connection, PgConnection, PgPool};
use std::env::var;
use std::io::{sink, stdout};
use std::sync::LazyLock;
use std::{fmt::format, io::Error, net::TcpListener};
use uuid::Uuid;
use zero2prod::telemetry::{get_subscriber, init_subscriber};
use zero2prod::{
    configuration::{self, get_configuration, DatabaseSettings},
    startup::run,
};

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, sink);
        init_subscriber(subscriber);
    }
});

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let res = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Fail to execute request");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}

#[tokio::test]
async fn subscribe_return_a_200_for_valid_from_dat() {
    //Arrange
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    //Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let res = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Fail to execute request");

    assert_eq!(200, res.status().as_u16());

    let save_data = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Fail to fetch saved sub");
    dbg!(&save_data);
    assert_eq!(save_data.email, "ursula_le_guin@gmail.com");
    assert_eq!(save_data.name, "le guin");
}

#[tokio::test]
async fn subscribe_return_a_400_for_valid_from_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let res = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Fail to execute request");

        assert_eq!(
            400,
            res.status().as_u16(),
            "The api did not fail with 400 {}",
            error_message
        );
    }
}

async fn spawn_app() -> TestApp {
    LazyLock::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Fail to bind port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut config = get_configuration().expect("Fail to read config");
    config.database.database_name = Uuid::new_v4().to_string();
    let conn_pool = configure_db(&config.database).await;

    let server = run(listener, conn_pool.clone()).expect("Fail tto bind address");
    let _ = tokio::spawn(server);
    TestApp {
        address,
        db_pool: conn_pool,
    }
}

async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let maintences_settgings = DatabaseSettings {
        database_name: "postgres".to_string(),
        username: "postgres".to_string(),
        password: Secret::new("password".to_string()),
        ..config.clone()
    };
    let mut conn = PgConnection::connect(&maintences_settgings.connection_string().expose_secret())
        .await
        .expect("Fail to connect to Postgres");

    conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("fail to create DB");

    let conn_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("Fail to connect to postgress");
    sqlx::migrate!("./migrations")
        .run(&conn_pool)
        .await
        .expect("fail to migrate");

    conn_pool
}

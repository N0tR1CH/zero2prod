use sqlx::Executor;
use sqlx::{Connection, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{DatabaseSettings, get_configuration},
    startup::run,
};

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await.expect("Failed to spawn our app");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await.expect("Failed to spawn our app");
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("select email, name from subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Expected to fetch subscription");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await.expect("Failed to spawn our app");
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (body, err_msg) in test_cases {
        let response = client
            .post(format!("{}/subscriptions", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");
        let status = response.status();
        assert_eq!(
            400, status,
            "The API did not fail with 400 Bad Request when the payload was {}.",
            err_msg,
        );
    }
}

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> Result<TestApp, std::io::Error> {
    let mut configuration = get_configuration().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&configuration.database).await;
    let server = run("127.0.0.1:0".to_string(), db_pool.clone())
        .await
        .expect("Failed to bind an address");
    let local_addr = server.local_addr().expect("cannot get local address");
    let _handle = tokio::spawn(async {
        let _ = server.await;
        eprintln!("server went down or was not created at first place");
    });
    let address = format!("http://{}", local_addr);
    let test_app = TestApp { address, db_pool };
    Ok(test_app)
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"create database "{}""#, config.database_name).as_str())
        .await
        .expect("Failed to create the database");
    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to migrate the database");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    connection_pool
}

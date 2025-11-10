use sqlx::{Connection, PgConnection};
use zero2prod::{configuration::get_configuration, startup::run};

#[tokio::test]
async fn health_check_works() {
    let app_address = spawn_app().await.expect("Failed to spawn our app");
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{app_address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app_address = spawn_app().await.expect("Failed to spawn our app");
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_string = configuration.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{app_address}/subscriptions"))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("select email, name from subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscriptions");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app().await.expect("Failed to spawn our app");
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (body, err_msg) in test_cases {
        let response = client
            .post(format!("{app_address}/subscriptions"))
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

async fn spawn_app() -> Result<String, std::io::Error> {
    let server = run("127.0.0.1:0".to_string())
        .await
        .expect("Failed to bind an address");
    let local_addr = server.local_addr().expect("cannot get local address");
    tokio::spawn(async {
        let _ = server.await;
        eprintln!("server went down or was not created at first place");
    });
    Ok(format!("http://{}", local_addr))
}

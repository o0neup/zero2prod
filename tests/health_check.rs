use std::net::TcpListener;

#[tokio_macros::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("http://{}/health_check", address))
        .send()
        .await
        .expect("Failed to execute request to health_check.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio_macros::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let body = "name=kotleta&email=2hcompany%40gmail.com";
    let response = client
        .post(&format!("http://{}/subscriptions", &app_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(200, response.status().as_u16());
}

#[tokio_macros::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=kotleta", "Missing the email."),
        ("email=2hcompany%40gmail.com", "Missing the name."),
        ("", "Missing both name and email."),
    ];
    for (body, error) in test_cases {
        let response = client
            .post(&format!("http://{}/subscriptions", &app_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            error
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port.");
    let port = listener.local_addr().unwrap().port();
    let server = zero2prod::startup::run(listener).expect("Failed to bind address!");

    // Launch as a background task
    // tokio::spawn returns a handle to the spawned future,
    // but we have no use for it
    let _ = tokio::spawn(server);
    format!("127.0.0.1:{}", &port)
}

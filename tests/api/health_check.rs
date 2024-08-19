use crate::helpers::spawn_app;

#[tokio_macros::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(&format!("http://{}/health_check", &test_app.address))
        .send()
        .await
        .expect("Failed to execute request to health_check.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

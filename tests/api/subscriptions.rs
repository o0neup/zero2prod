use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[tokio_macros::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = "name=kotleta&email=2hcompany%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscriptions(body.to_string()).await;

    assert_eq!(200, response.status().as_u16());
}

#[tokio_macros::test]
async fn subscribe_persists_new_subscriber() {
    let test_app = spawn_app().await;

    let body = "name=kotleta&email=2hcompany%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.to_string()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "2hcompany@gmail.com");
    assert_eq!(saved.name, "kotleta");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio_macros::test]
async fn subscribe_returns_400_for_invalid_form_data() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=[kotleta]&email=2hcompany%40gmail.com", "invalid name"),
        (
            "name=DROP TABLE subscriptions;&email=2hcompany%40gmail.com",
            "invalid name",
        ),
    ];
    for (body, _error) in test_cases {
        let response = test_app.post_subscriptions(body.to_string()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            body
        );
    }
}

#[tokio_macros::test]
async fn subscribe_returns_400_when_data_is_missing() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=kotleta", "Missing the email."),
        ("email=2hcompany%40gmail.com", "Missing the name."),
        ("", "Missing both name and email."),
    ];
    for (body, error) in test_cases {
        let response = test_app.post_subscriptions(body.to_string()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            error
        );
    }
}

#[tokio_macros::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = "name=lol&email=kekovich@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
}

#[tokio_macros::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    let app = spawn_app().await;
    let body = "name=lol&email=kekovich@gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscriptions(body.into()).await;
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = app.get_confirmation_links(email_request);
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

// #[tokio_macros::test]
// async fn subscribe_returns_

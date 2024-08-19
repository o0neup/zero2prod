use crate::helpers::spawn_app;

#[tokio_macros::test]
async fn subscribe_returns_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    println!("{:?}", test_app);

    let body = "name=kotleta&email=2hcompany%40gmail.com";
    let response = test_app.post_subscriptions(body.to_string()).await;

    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "2hcompany@gmail.com");
    assert_eq!(saved.name, "kotleta");
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

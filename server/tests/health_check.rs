use std::net::TcpListener;

use reqwest::StatusCode;

use serde::Serialize;

use server::app;

#[tokio::test]
async fn health_check_works() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/health_check", addr))
        .send()
        .await
        .expect("Failed to execute get request");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());
}

#[tokio::test]
async fn subcribe_returns_success_for_valid_request() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let new_subscriber = NewSubscriber {
        name: Some("Test Subscrber".into()),
        email: Some("test@test.com".into()),
    };

    let res = client
        .post(format!("{}/subscriptions", addr))
        .form(&new_subscriber)
        .send()
        .await
        .expect("Failed to execute post request");

    assert!(res.status().is_success());
}

#[tokio::test]
async fn subcribe_returns_bad_request_for_missing_data() {
    let addr = spawn_app();
    let client = reqwest::Client::new();

    let test_cases: Vec<NewSubscriber> = vec![
        NewSubscriber {
            name: Some("Missing email".into()),
            email: None,
        },
        NewSubscriber {
            name: None,
            email: Some("missing_name@test.com".into()),
        },
        NewSubscriber {
            name: None,
            email: None,
        },
    ];

    for body in test_cases {
        let res = client
            .post(format!("{}/subscriptions", addr))
            .form(&body)
            .send()
            .await
            .expect("Failed to execute post request");

        assert_eq!(StatusCode::BAD_REQUEST, res.status());
    }
}

#[derive(Debug, Serialize)]
struct NewSubscriber {
    name: Option<String>,
    email: Option<String>,
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to listen on random port");
    let port = listener.local_addr().unwrap().port();

    let server = app::run(listener).expect("Failed to spawn app instance");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}

use std::net::TcpListener;

use reqwest::StatusCode;

use serde::Serialize;

use sqlx::PgPool;

use server::app;

#[sqlx::test(migrations = "../migrations")]
async fn health_check_works(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;
    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/health_check", &app.addr))
        .send()
        .await
        .expect("Failed to execute get request");

    assert!(res.status().is_success());
    assert_eq!(Some(0), res.content_length());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn subcribe_returns_success_for_valid_request(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let client = reqwest::Client::new();

    let new_subscriber = NewSubscriber {
        name: Some("Test Subscrber".into()),
        email: Some("test@test.com".into()),
    };

    let res = client
        .post(format!("{}/subscriptions", &app.addr))
        .form(&new_subscriber)
        .send()
        .await
        .expect("Failed to execute post request");

    assert!(res.status().is_success());

    let subscription = sqlx::query!("select name, email from subscriptions")
        .fetch_one(&pool)
        .await
        .expect("Failed to fetch inserted row");

    assert_eq!(new_subscriber.name.unwrap(), subscription.name);
    assert_eq!(new_subscriber.email.unwrap(), subscription.email);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn subcribe_returns_bad_request_for_missing_data(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;
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
            .post(format!("{}/subscriptions", &app.addr))
            .form(&body)
            .send()
            .await
            .expect("Failed to execute post request");

        assert_eq!(StatusCode::BAD_REQUEST, res.status());
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct NewSubscriber {
    name: Option<String>,
    email: Option<String>,
}

struct TestApp {
    pub addr: String,
}

impl TestApp {
    async fn spawn(pool: &PgPool) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to listen on random port");
        let port = listener.local_addr().unwrap().port();

        let addr = format!("http://127.0.0.1:{}", port);

        let server = app::run(pool.clone(), listener).expect("Failed to spawn app instance");
        let _ = tokio::spawn(server);

        Self { addr }
    }
}

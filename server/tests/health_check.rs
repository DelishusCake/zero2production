use std::net::TcpListener;
use std::sync::Arc;

use reqwest::{Client, Response};

use serde::Serialize;

use sqlx::PgPool;

use server::app::{self, AppState};
use server::crypto::Crypto;

#[sqlx::test(migrations = "../migrations")]
async fn health_check_works(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let res = app.health_check().await.expect("Failed to execute request");

    assert!(res.status().is_success());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn subcribe_returns_success_for_valid_request(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let new_subscriber = NewSubscriber {
        name: Some("Test Subscrber".into()),
        email: Some("test@test.com".into()),
    };

    let res = app
        .subscription_create(&new_subscriber)
        .await
        .expect("Failed to execute request");

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

    let test_cases: Vec<(String, NewSubscriber)> = vec![
        (
            "missing email".into(),
            NewSubscriber {
                name: Some("Test name".into()),
                email: None,
            },
        ),
        (
            "missing name".into(),
            NewSubscriber {
                name: None,
                email: Some("test@test.com".into()),
            },
        ),
        (
            "missing both email and name".into(),
            NewSubscriber {
                name: None,
                email: None,
            },
        ),
        (
            "malformed email".into(),
            NewSubscriber {
                name: Some("Test name".into()),
                email: Some("bad email address".into()),
            },
        ),
    ];

    for (desc, new_subscriber) in test_cases {
        let res = app
            .subscription_create(&new_subscriber)
            .await
            .expect("Failed to execute request");

        assert!(
            res.status().is_client_error(),
            "API did not fail when payload was {}",
            desc
        );
    }
    Ok(())
}

#[derive(Debug, Serialize)]
struct NewSubscriber {
    name: Option<String>,
    email: Option<String>,
}

struct TestApp {
    addr: String,
    client: reqwest::Client,
}

impl TestApp {
    pub async fn spawn(pool: &PgPool) -> Self {
        use rand::{distributions::Alphanumeric, Rng};

        let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to listen on random port");
        let port = listener.local_addr().unwrap().port();

        let addr = format!("http://127.0.0.1:{}", port);

        let rand_key: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let crypto = Crypto::new(&rand_key).expect("Failed to create crypto signing key");
        let crypto = Arc::new(crypto);

        let state = AppState {
            pool: pool.clone(),
            crypto,
        };

        let server = app::run(state, listener).expect("Failed to spawn app instance");
        let _ = tokio::spawn(server);

        let client = Client::new();

        Self { addr, client }
    }

    pub async fn health_check(&self) -> reqwest::Result<Response> {
        self.client
            .get(format!("{}/health_check", &self.addr))
            .send()
            .await
    }

    pub async fn subscription_create(
        &self,
        new_subscriber: &NewSubscriber,
    ) -> reqwest::Result<Response> {
        self.client
            .post(format!("{}/subscriptions", &self.addr))
            .form(new_subscriber)
            .send()
            .await
    }
}

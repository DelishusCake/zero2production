use reqwest::StatusCode;

use sqlx::PgPool;

use wiremock::matchers::*;
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::{NewSubscriber, TestApp};

#[sqlx::test(migrations = "../migrations")]
async fn subcribe_returns_success_for_valid_request(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

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

#[sqlx::test(migrations = "../migrations")]
async fn subcribe_sends_a_confirmation_email_for_valid_request(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // Expect a send-email request
        .expect(1)
        .mount(&app.email_server)
        .await;

    let new_subscriber = NewSubscriber {
        name: Some("Test Subscrber".into()),
        email: Some("test@test.com".into()),
    };

    let res = app
        .subscription_create(&new_subscriber)
        .await
        .expect("Failed to execute request");

    assert!(res.status().is_success());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn subcribe_sends_a_confirmation_email_with_link(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // Expect a send-email request
        .expect(1)
        .mount(&app.email_server)
        .await;

    let new_subscriber = NewSubscriber {
        name: Some("Test Subscrber".into()),
        email: Some("test@test.com".into()),
    };

    let _res = app
        .subscription_create(&new_subscriber)
        .await
        .expect("Failed to execute request");

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    let html_link = extract_email_link(&body["HtmlBody"].as_str().unwrap());
    let text_link = extract_email_link(&body["TextBody"].as_str().unwrap());

    assert_eq!(html_link, text_link);

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn subscription_can_be_confirmed(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        // Expect a send-email request
        .expect(1)
        .mount(&app.email_server)
        .await;

    let new_subscriber = NewSubscriber {
        name: Some("Test Subscrber".into()),
        email: Some("test@test.com".into()),
    };

    let _res = app
        .subscription_create(&new_subscriber)
        .await
        .expect("Failed to execute request");

    let subscription = sqlx::query!(
        "select confirmed_at from subscriptions where email=$1",
        new_subscriber.email.clone().unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch updated row");

    assert!(subscription.confirmed_at.is_none());

    let email_request = &app.email_server.received_requests().await.unwrap()[0];

    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    let link = extract_email_link(&body["HtmlBody"].as_str().unwrap());

    let res = app
        .client
        .get(&link)
        .send()
        .await
        .expect("Failed to follow confirmation link");

    assert_eq!(StatusCode::OK, res.status());

    let subscription = sqlx::query!(
        "select confirmed_at from subscriptions where email=$1",
        new_subscriber.email.clone().unwrap()
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to fetch updated row");

    assert!(subscription.confirmed_at.is_some());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn subcribe_is_consistent_if_email_send_fails(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        // Ensure that send-email fails
        .respond_with(ResponseTemplate::new(500))
        // Expect a send-email request
        .expect(1)
        .mount(&app.email_server)
        .await;

    let new_subscriber = NewSubscriber {
        name: Some("Test Subscrber".into()),
        email: Some("test@test.com".into()),
    };

    let res = app
        .subscription_create(&new_subscriber)
        .await
        .expect("Failed to execute request");

    assert!(res.status().is_server_error());

    let subscription = sqlx::query!("select name, email from subscriptions")
        .fetch_optional(&pool)
        .await
        .expect("Failed to fetch row");

    assert!(subscription.is_none());

    Ok(())
}

fn extract_email_link(body: &str) -> String {
    let links: Vec<_> = linkify::LinkFinder::new()
        .links(body)
        .filter(|l| *l.kind() == linkify::LinkKind::Url)
        .collect();
    assert_eq!(1, links.len());
    links[0].as_str().to_string()
}

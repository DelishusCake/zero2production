use sqlx::PgPool;

use crate::helpers::{NewSubscriber, TestApp};

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

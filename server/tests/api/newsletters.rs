use reqwest::StatusCode;

use sqlx::PgPool;

use uuid::Uuid;

use wiremock::matchers::*;
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::{Credentials, TestApp, TestUser};
use crate::helpers::{NewSubscriber, Newsletter, NewsletterContent};

#[sqlx::test(migrations = "../migrations")]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let creds = TestUser::register(&pool, "test@test.com", "test_password")
        .await
        .credentials();

    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter = Newsletter {
        title: Some("Newsletter Title".into()),
        content: Some(NewsletterContent {
            text: Some("Newsletter Body".into()),
            html: Some("<p>Newsletter Body</p>".into()),
        }),
    };
    let res = app
        .newsletter_publish(Some(&creds), &newsletter)
        .await
        .expect("Failed to send request to create newsletter");

    assert_eq!(StatusCode::OK, res.status());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn newsletters_without_credentials_are_unauthorized(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let newsletter = Newsletter {
        title: Some("Newsletter Title".into()),
        content: Some(NewsletterContent {
            text: Some("Newsletter Body".into()),
            html: Some("<p>Newsletter Body</p>".into()),
        }),
    };
    let res = app
        .newsletter_publish(None, &newsletter)
        .await
        .expect("Failed to send request to create newsletter");

    assert_eq!(StatusCode::UNAUTHORIZED, res.status());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn newsletters_with_bad_credentials_are_rejected(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let creds = Credentials {
        username: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
    };

    let newsletter = Newsletter {
        title: Some("Newsletter Title".into()),
        content: Some(NewsletterContent {
            text: Some("Newsletter Body".into()),
            html: Some("<p>Newsletter Body</p>".into()),
        }),
    };
    let res = app
        .newsletter_publish(Some(&creds), &newsletter)
        .await
        .expect("Failed to send request to create newsletter");

    assert!(res.status().is_client_error());

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn malformed_newsletters_are_rejected(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let creds = TestUser::register(&pool, "test@test.com", "test_password")
        .await
        .credentials();

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let test_cases = vec![
        (
            "Missing Title",
            Newsletter {
                title: None,
                content: Some(NewsletterContent {
                    text: Some("Newsletter Body".into()),
                    html: Some("<p>Newsletter Body</p>".into()),
                }),
            },
        ),
        (
            "Missing Body",
            Newsletter {
                title: Some("Newsletter Title".into()),
                content: None,
            },
        ),
        (
            "Missing Text Body",
            Newsletter {
                title: Some("Newsletter Title".into()),
                content: Some(NewsletterContent {
                    text: None,
                    html: Some("<p>Newsletter Body</p>".into()),
                }),
            },
        ),
        (
            "Missing HTML Body",
            Newsletter {
                title: Some("Newsletter Title".into()),
                content: Some(NewsletterContent {
                    text: Some("Newsletter Body".into()),
                    html: None,
                }),
            },
        ),
    ];
    for (test_name, newsletter) in test_cases {
        let res = app
            .newsletter_publish(Some(&creds), &newsletter)
            .await
            .expect("Failed to send request to create newsletter");

        assert_eq!(StatusCode::BAD_REQUEST, res.status(), "{}", test_name);
    }

    Ok(())
}

#[sqlx::test(migrations = "../migrations")]
async fn newsletters_are_not_delivered_to_subscribers_with_bad_emails(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let creds = TestUser::register(&pool, "test@test.com", "test_password")
        .await
        .credentials();

    create_confirmed_subscriber(&pool, "Test Name A", "good_email_a@test.com").await;
    create_confirmed_subscriber(&pool, "Test Name B", "good_email_b@test.com").await;
    create_confirmed_subscriber(&pool, "Test Name C", "bad_email_address").await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(2) // Match with number of good email addresses
        .mount(&app.email_server)
        .await;

    let newsletter = Newsletter {
        title: Some("Newsletter Title".into()),
        content: Some(NewsletterContent {
            text: Some("Newsletter Body".into()),
            html: Some("<p>Newsletter Body</p>".into()),
        }),
    };
    let res = app
        .newsletter_publish(Some(&creds), &newsletter)
        .await
        .expect("Failed to send request to create newsletter");

    assert_eq!(StatusCode::OK, res.status());

    Ok(())
}

async fn create_unconfirmed_subscriber(app: &TestApp) {
    // Scoped email mock for the subscription creation
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    let unconfirmed_subscriber = NewSubscriber {
        email: Some("test@test.com".to_string()),
        name: Some("Test User".to_string()),
    };
    app.subscription_create(&unconfirmed_subscriber)
        .await
        .expect("Failed to create unconfirmed subscription");
}

async fn create_confirmed_subscriber(pool: &PgPool, name: &str, email: &str) {
    use chrono::Utc;

    // NOTE: Manually insert confirmed subscriptions to skip confirmation process
    sqlx::query!(
        "insert into subscriptions(name, email, confirmed_at) values ($1, $2, $3);",
        name,
        email,
        Utc::now()
    )
    .execute(pool)
    .await
    .expect("Failed to insert confirmed subscriber");
}

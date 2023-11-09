use reqwest::StatusCode;

use sqlx::PgPool;

use wiremock::matchers::*;
use wiremock::{Mock, ResponseTemplate};

use crate::helpers::{Credentials, NewSubscriber, Newsletter, NewsletterContent, TestApp};

#[sqlx::test(migrations = "../migrations")]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers(
    pool: PgPool,
) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let creds = generate_creds();

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
async fn malformed_newsletters_are_rejected(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let creds = generate_creds();

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

fn generate_creds() -> Credentials {
    use uuid::Uuid;

    Credentials {
        username: Uuid::new_v4().to_string(),
        password: Uuid::new_v4().to_string(),
    }
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

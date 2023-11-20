use actix_web::dev::HttpServiceFactory;
use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse, Responder, ResponseError};

use serde::Deserialize;

use sqlx::PgPool;

use thiserror::Error;

use zero2prod::client::{Email, EmailClient};
use zero2prod::domain::EmailAddress;
use zero2prod::repo::{ConfirmedSubscription, SubscriptionRepo};

use crate::auth::Administrator;

#[derive(Debug, Deserialize)]
pub struct PublishBody {
    title: String,
    content: PublishBodyContent,
}
#[derive(Debug, Deserialize)]
pub struct PublishBodyContent {
    text: String,
    html: String,
}

impl TryFrom<PublishBody> for Email {
    type Error = NewsletterError;

    fn try_from(body: PublishBody) -> Result<Self, NewsletterError> {
        Ok(Self {
            subject: body.title,
            text_body: body.content.text,
            html_body: body.content.html,
        })
    }
}

#[tracing::instrument(name = "Publish a newsletter", skip(pool, email_client))]
#[post("")]
async fn publish(
    admin: Administrator, // Administrator guard
    body: web::Json<PublishBody>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<impl Responder, NewsletterError> {
    let pool = pool.get_ref();
    // Get the email to send out from the body
    let email: Email = body.0.try_into()?;
    // Get the list of confirmed email subscriptions, filtering out bad data
    let recipients: Vec<EmailAddress> = SubscriptionRepo::fetch_all_confirmed(pool)
        .await?
        .into_iter()
        .filter_map(parse_confirmed_email)
        .collect();
    // Send the emails out
    // TODO: Find a better method to send email blast
    for recipient in recipients {
        email_client
            .send(&recipient, &email)
            .await
            .map_err(NewsletterError::SendEmailError)?;
    }
    Ok(HttpResponse::Ok())
}

fn parse_confirmed_email(subscription: ConfirmedSubscription) -> Option<EmailAddress> {
    subscription
        .email
        .parse()
        .map_err(|error| {
            tracing::warn!(
                error.cause_chain = ?error,
                "Skipping a confirmed subscription (id: {}, email: {})", 
                subscription.id, 
                subscription.email);
        })
        .ok()
}

#[derive(Debug, Error)]
pub enum NewsletterError {
    #[error("Internal Server Error")]
    SendEmailError(#[from] reqwest::Error),

    #[error("Internal Server Error")]
    DatabaseError(#[from] sqlx::Error),
}

impl ResponseError for NewsletterError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::SendEmailError(_) | Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Subscriptions API endpoints
pub fn scope() -> impl HttpServiceFactory {
    web::scope("/newsletters").service(publish)
}

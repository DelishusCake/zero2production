use actix_web::dev::HttpServiceFactory;
use actix_web::{post, web, HttpResponse, Responder};

use serde::Deserialize;

use sqlx::PgPool;

use zero2prod::client::{Email, EmailClient};
use zero2prod::domain::EmailAddress;
use zero2prod::repo::{ConfirmedSubscription, SubscriptionRepo};

use crate::auth::Administrator;
use crate::error::{RestError, RestResult};

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
    type Error = RestError;

    fn try_from(body: PublishBody) -> RestResult<Self> {
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
) -> RestResult<impl Responder> {
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
            .map_err(RestError::FailedToSendEmail)?;
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

/// Subscriptions API endpoints
pub fn scope() -> impl HttpServiceFactory {
    web::scope("/newsletters").service(publish)
}

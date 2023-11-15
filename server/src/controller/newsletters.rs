use actix_web::dev::HttpServiceFactory;
use actix_web::{post, web, HttpResponse, Responder};

use serde::Deserialize;

use sqlx::PgPool;

use zero2prod::client::{Email, EmailClient};
use zero2prod::repo::SubscriptionRepo;

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

    let email: Email = body.0.try_into()?;

    for subscription in SubscriptionRepo::fetch_all_confirmed(pool).await? {
        match subscription.email.parse() {
            Ok(recipient) => {
                email_client
                    .send(&recipient, &email)
                    .await
                    .map_err(RestError::FailedToSendEmail)?;
            }
            Err(error) => {
                tracing::warn!(
                    error.cause_chain = ?error,
                    "Skipping a confirmed subscription (id: {}, email: {})", 
                    subscription.id, 
                    subscription.email);
            }
        };
    }

    Ok(HttpResponse::Ok())
}

/// Subscriptions API endpoints
pub fn scope() -> impl HttpServiceFactory {
    web::scope("/newsletters").service(publish)
}

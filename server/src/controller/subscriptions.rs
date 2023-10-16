use actix_web::dev::HttpServiceFactory;
use actix_web::{post, get, web, HttpResponse, Responder};

use jwt::{SignWithKey, VerifyWithKey};

use serde::{Deserialize, Serialize};

use sqlx::PgPool;

use uuid::Uuid;

use zero2prod::model::{NewSubscription, Subscription};

use crate::client::EmailClient;
use crate::crypto::SigningKey;
use crate::error::{RestError, RestResult};

#[derive(Debug, Deserialize)]
pub struct NewSubscriptionForm {
    name: String,
    email: String,
}

impl TryInto<NewSubscription> for NewSubscriptionForm {
    type Error = String;

    fn try_into(self) -> Result<NewSubscription, Self::Error> {
        let name = self.name.parse()?;
        let email = self.email.parse()?;

        Ok(NewSubscription { name, email })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Confirmation(Uuid);

impl From<Confirmation> for Uuid {
    fn from(value: Confirmation) -> Uuid {
        value.0
    }
}

#[tracing::instrument(
    name = "Create a new subscriber",
    skip(pool, signing_key, email_client)
)]
#[post("")]
async fn create(
    pool: web::Data<PgPool>,
    signing_key: web::Data<SigningKey>,
    email_client: web::Data<EmailClient>,
    form: web::Form<NewSubscriptionForm>,
) -> RestResult<impl Responder> {
    let new_subscription: NewSubscription = form.0.try_into().map_err(RestError::ParseError)?;

    // Transaction context
    {
        let mut tx = pool.begin().await?;

        let id = Subscription::insert(&mut *tx, &new_subscription).await?;

        let confirmation = Confirmation(id);

        send_confirmation_email(&email_client, &signing_key, &new_subscription, confirmation)
            .await?;

        tx.commit().await?;
    }

    Ok(HttpResponse::Created())
}


#[tracing::instrument(name = "Confirm a subscription by token", skip(pool, signing_key))]
#[get("/confirm/{token_str}")]
async fn confirm(
    pool: web::Data<PgPool>,
    signing_key: web::Data<SigningKey>,
    path: web::Path<(String, )>,
) -> RestResult<impl Responder> {
    let (token_str, ) = path.into_inner();

    let confirmation: Confirmation = token_str
        .verify_with_key(signing_key.as_ref())
        .map_err(|_| RestError::InvalidConfirmationToken)?;

    Subscription::confirm_by_id(&**pool, confirmation.into()).await?;

    Ok(HttpResponse::Ok())
}

async fn send_confirmation_email(
    email_client: &EmailClient,
    signing_key: &SigningKey,
    subscription: &NewSubscription,
    confirmation: Confirmation,
) -> RestResult<()> {
    let confirmation_token = confirmation
        .sign_with_key(signing_key)
        .map_err(|_| RestError::FailedToSignToken)?;

    tracing::debug!("Confirmation Token: {:?}", confirmation_token);

    let confirmation_url = format!(
        "http://localhost/subscriptions/confirm/{}",
        confirmation_token
    );

    let subject = format!("Welcome {}!", subscription.name.as_ref());
    let html_body = format!("<h1>Welcome to our newsletter!</h1><p>Click <a href=\"{}\">here</a> to confirm your subscription.</p>", confirmation_url);
    let text_body = format!(
        "Welcome to our newsletter!\n\nTo confirm your subscription, visit this web page: {}",
        confirmation_url
    );

    email_client
        .send(subscription.email.clone(), &subject, &html_body, &text_body)
        .await
        .map_err(|_| RestError::FailedToSendEmail)?;

    Ok(())
}

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/subscriptions")
        .service(create)
        .service(confirm)
}

use actix_web::dev::HttpServiceFactory;
use actix_web::http::StatusCode;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder, ResponseError};

use serde::Deserialize;

use sqlx::PgPool;

use thiserror::Error;

use url::Url;

use crate::client::{Email, EmailClient};
use crate::crypto::{SigningKey, Token, TokenError};
use crate::repo::{NewSubscription, SubscriptionRepo};

/// Form deserialization wrapper for parsing new subscriptions
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

/// Create endpoint for new subscriptions
#[tracing::instrument(
    name = "Create a new subscriber",
    skip(req, pool, signing_key, email_client)
)]
#[post("")]
async fn create(
    req: HttpRequest,
    pool: web::Data<PgPool>,
    signing_key: web::Data<SigningKey>,
    email_client: web::Data<EmailClient>,
    form: web::Form<NewSubscriptionForm>,
) -> Result<impl Responder, SubscribeError> {
    // Unwrap the signing key
    let signing_key = signing_key.get_ref();
    // Parse the new subscriber form
    let new_subscription: NewSubscription =
        form.0.try_into().map_err(SubscribeError::ParseError)?;

    // Transaction context
    // The subscription request should fail if something goes wrong
    {
        let mut tx = pool.begin().await?;
        // Insert the new subscription
        let id = SubscriptionRepo::insert(&mut *tx, &new_subscription).await?;
        // Sign a confirmation token for the user to use when confirming their email
        let token = Token::builder(id)
            .sign(signing_key.as_ref())
            .map_err(SubscribeError::SignTokenError)?;
        // Get the confirmation URL to send to the user
        let confirmation_url = req.url_for("confirm_subscription", [&token])?;
        // Build the confirmation email
        let recipient = new_subscription.email.clone();
        let email = build_confirmation_email(&new_subscription, confirmation_url);
        // Send the confirmation email
        email_client
            .send(&recipient, &email)
            .await
            .map_err(SubscribeError::SendEmailError)?;
        // Commit the new subscriber to the database if everything worked
        tx.commit().await?;
    }

    Ok(HttpResponse::Created())
}

/// Subscription confirmation endpoint
#[tracing::instrument(name = "Confirm a subscription by token", skip(pool, signing_key))]
#[get("/confirm/{token_str}", name = "confirm_subscription")]
async fn confirm(
    pool: web::Data<PgPool>,
    signing_key: web::Data<SigningKey>,
    path: web::Path<(String,)>,
) -> Result<impl Responder, SubscribeError> {
    // Get the string for the confirmation token from the URL path
    let (token_str,) = path.into_inner();
    // Unwrap the signing key
    let signing_key = signing_key.get_ref();
    // Parse, verify, and extract the subscription ID from the confirmation token
    let subscription_id = token_str
        .parse::<Token>()
        .and_then(|token| token.verify(signing_key.as_ref()))
        .map_err(SubscribeError::VerifyTokenError)?;
    // Confirm the subscription
    SubscriptionRepo::confirm_by_id(pool.get_ref(), subscription_id).await?;

    Ok(HttpResponse::Ok())
}

/// Build a confirmation email object for a new subscriber
/// TODO: Move this somewhere else
fn build_confirmation_email(subscription: &NewSubscription, confirmation_url: Url) -> Email {
    let subject = format!("Welcome {}!", subscription.name.as_ref());
    let html_body = format!("<h1>Welcome to our newsletter!</h1><p>Click <a href=\"{}\">here</a> to confirm your subscription.</p>", confirmation_url);
    let text_body = format!(
        "Welcome to our newsletter!\n\nTo confirm your subscription, visit this web page: {}",
        confirmation_url
    );

    Email {
        subject,
        html_body,
        text_body,
    }
}

#[derive(Debug, Error)]
pub enum SubscribeError {
    #[error("Failed to parse {0}")]
    ParseError(String),

    #[error("Failed to sign token")]
    SignTokenError(TokenError),

    #[error("Failed to verify token")]
    VerifyTokenError(TokenError),

    #[error("Internal Server Error")]
    GenerateUrlError(#[from] actix_web::error::UrlGenerationError),

    #[error("Internal Server Error")]
    SendEmailError(#[from] reqwest::Error),

    #[error("Internal Server Error")]
    DatabaseError(#[from] sqlx::Error),
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::VerifyTokenError(_) => StatusCode::UNAUTHORIZED,
            Self::DatabaseError(_)
            | Self::SignTokenError(_)
            | Self::SendEmailError(_)
            | Self::GenerateUrlError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

/// Subscriptions API endpoints
pub fn scope() -> impl HttpServiceFactory {
    web::scope("/subscriptions")
        .service(create)
        .service(confirm)
}

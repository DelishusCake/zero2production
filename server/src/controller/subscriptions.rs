use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Form;

use serde::{Deserialize, Serialize};

use uuid::Uuid;

use zero2prod::model::{NewSubscription, Subscription};

use crate::app::AppState;
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

#[tracing::instrument(name = "Create a new subscriber", skip(pool, signing_key))]
pub async fn create(
    State(AppState { pool, signing_key }): State<AppState>,
    Form(form): Form<NewSubscriptionForm>,
) -> RestResult<StatusCode> {
    let new_subscription: NewSubscription = form.try_into().map_err(RestError::ParseError)?;

    let id = Subscription::insert(&pool, new_subscription).await?;
    let confirmation_token = Confirmation(id).sign_token(&signing_key)?;

    tracing::debug!("Confirmation Token: {:?}", confirmation_token);
    // TODO: Send email with confirmation token

    Ok(StatusCode::CREATED)
}

#[tracing::instrument(name = "Confirm a subscription by token", skip(pool, signing_key))]
pub async fn confirm(
    State(AppState { pool, signing_key }): State<AppState>,
    Path(token_str): Path<String>,
) -> RestResult<StatusCode> {
    let confirmation = Confirmation::verify_token(&signing_key, &token_str)?;

    Subscription::confirm_by_id(&pool, confirmation.into()).await?;

    Ok(StatusCode::OK)
}

#[derive(Debug, Serialize, Deserialize)]
struct Confirmation(Uuid);

impl Confirmation {
    pub fn sign_token(&self, key: &SigningKey) -> anyhow::Result<String> {
        use jwt::SignWithKey;

        let token = self.0.sign_with_key(key)?;
        Ok(token)
    }

    pub fn verify_token(key: &SigningKey, token: &str) -> anyhow::Result<Self> {
        use jwt::VerifyWithKey;

        let claims = token.verify_with_key(key)?;
        Ok(claims)
    }
}

impl Into<Uuid> for Confirmation {
    fn into(self) -> Uuid {
        self.0
    }
}

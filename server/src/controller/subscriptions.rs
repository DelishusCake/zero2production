use axum::extract::State;
use axum::http::StatusCode;
use axum::Form;

use serde::Deserialize;

use sqlx::PgPool;

use zero2prod::model::{NewSubscription, Subscription};

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

#[tracing::instrument(name = "Create a new subscriber", skip(pool))]
pub async fn create(
    State(pool): State<PgPool>,
    Form(form): Form<NewSubscriptionForm>,
) -> RestResult<StatusCode> {
    let new_subscription: NewSubscription = form.try_into().map_err(RestError::ParseError)?;

    let _id = Subscription::insert(&pool, new_subscription).await?;

    Ok(StatusCode::CREATED)
}

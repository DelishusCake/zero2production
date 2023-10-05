use actix_web::dev::HttpServiceFactory;
use actix_web::{post, web, HttpResponse, Responder};

use serde::Deserialize;

use sqlx::PgPool;

use zero2prod::model::{NewSubscription, Subscription};

use crate::error::{RestError, RestResult};

#[derive(Debug, Deserialize)]
struct NewSubscriptionForm {
    name: String,
    email: String,
}

impl TryInto<NewSubscription> for NewSubscriptionForm {
    type Error = RestError;

    fn try_into(self) -> RestResult<NewSubscription> {
        let name = self.name;
        let email = self.email.parse()?;

        Ok(NewSubscription {
            name,
            email,
        })
    }
}

#[tracing::instrument(name = "Create a new subscriber", skip(pool))]
#[post("")]
async fn create(
    pool: web::Data<PgPool>,
    form: web::Form<NewSubscriptionForm>,
) -> RestResult<impl Responder> {
    let new_subscription: NewSubscription = form.0.try_into()?;

    let _id = Subscription::insert(&pool, new_subscription)
        .await
        .map_err(|e| {
            tracing::error!("{:?}", e);
            RestError::InternalServerError
        })?;

    Ok(HttpResponse::Created())
}

pub fn scope() -> impl HttpServiceFactory {
    web::scope("/subscriptions").service(create)
}

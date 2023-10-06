use uuid::Uuid;

use chrono::{DateTime, Utc};

use serde::Serialize;

use sqlx::PgPool;

use crate::domain::{EmailAddress, PersonName};

#[derive(Debug)]
pub struct NewSubscription {
    pub name: PersonName,
    pub email: EmailAddress,
}

#[derive(Debug, Serialize)]
pub struct Subscription {
    pub id: Uuid,

    pub name: String,
    pub email: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Subscription {
    #[tracing::instrument(name = "Insert subscriber", skip(pool))]
    pub async fn insert(pool: &PgPool, new_subscriber: NewSubscription) -> sqlx::Result<Uuid> {
        let row = sqlx::query!(
            "insert into subscriptions(name, email) values ($1, $2) returning id",
            new_subscriber.name.as_ref(),
            new_subscriber.email.as_ref(),
        )
        .fetch_one(pool)
        .await?;

        Ok(row.id)
    }
}

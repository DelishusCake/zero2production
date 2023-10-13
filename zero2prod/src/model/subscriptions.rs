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

    pub confirmed_at: Option<DateTime<Utc>>,

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

    #[tracing::instrument(name = "Confirm a subscriber by id", skip(pool))]
    pub async fn confirm_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<()> {
        let confirmed_at = Utc::now();
        sqlx::query!(
            "update subscriptions set confirmed_at=$2 where id=$1",
            id,
            confirmed_at,
        )
        .execute(pool)
        .await?;
        Ok(())
    }
}

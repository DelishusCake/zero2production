use uuid::Uuid;

use chrono::{DateTime, Utc};

use serde::Serialize;

use sqlx::PgExecutor;

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
    #[tracing::instrument(name = "Insert subscriber", skip(executor))]
    pub async fn insert(
        executor: impl PgExecutor<'_>,
        new_subscriber: &NewSubscription,
    ) -> sqlx::Result<Uuid> {
        let row = sqlx::query!(
            "insert into subscriptions(name, email) values ($1, $2) returning id",
            new_subscriber.name.as_ref(),
            new_subscriber.email.as_ref(),
        )
        .fetch_one(executor)
        .await?;

        Ok(row.id)
    }

    #[tracing::instrument(name = "Confirm a subscriber by id", skip(executor))]
    pub async fn confirm_by_id(executor: impl PgExecutor<'_>, id: Uuid) -> sqlx::Result<()> {
        let confirmed_at = Utc::now();
        sqlx::query!(
            "update subscriptions set confirmed_at=$2 where id=$1",
            id,
            confirmed_at,
        )
        .execute(executor)
        .await?;
        Ok(())
    }

    #[tracing::instrument(name = "Fetch all confirmed subscriptions", skip(executor))]
    pub async fn fetch_all_confirmed(executor: impl PgExecutor<'_>) -> sqlx::Result<Vec<Self>> {
        let subscriptions = sqlx::query_as!(
            Self,
            "select * from subscriptions where confirmed_at is not null"
        )
        .fetch_all(executor)
        .await?;
        Ok(subscriptions)
    }
}

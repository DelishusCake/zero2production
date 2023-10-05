use uuid::Uuid;

use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use sqlx::PgPool;

#[derive(Debug, Deserialize)]
pub struct NewSubscription {
    pub name: String,
    pub email: String,
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
    pub async fn insert(pool: &PgPool, new_subscriber: NewSubscription) -> sqlx::Result<Uuid> {
        let row = sqlx::query!(
            "insert into subscriptions(name, email) values ($1, $2) returning id",
            new_subscriber.name,
            new_subscriber.email
        )
        .fetch_one(pool)
        .await?;

        Ok(row.id)
    }
}

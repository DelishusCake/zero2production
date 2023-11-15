use uuid::Uuid;

use chrono::{DateTime, Utc};

use serde::Serialize;

use sqlx::PgExecutor;

use crate::domain::{EmailAddress, PersonName};

/// New Subscription request
#[derive(Debug)]
pub struct NewSubscription {
    pub name: PersonName,
    pub email: EmailAddress,
}

/// Stored Subscription record
#[derive(Debug, Serialize)]
pub struct Subscription {
    /// ID of the subscription
    pub id: Uuid,
    /// User supplied data
    /// TODO: Should these be parsed back into domain objects?
    pub name: String,
    pub email: String,
    /// Confirmation timestamp.
    /// `None` if the subscription is not confirmed, and therefore cannot receive newsletter emails
    pub confirmed_at: Option<DateTime<Utc>>,
    /// Creation and update timestamps
    /// NOTE: Auto-set and updated by database triggers
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Stored subscription record that has been confirmed via email
#[derive(Debug)]
pub struct ConfirmedSubscription {
    /// ID of the subscription
    pub id: Uuid,
    /// User supplied email
    /// TODO: Should this be parsed back into domain objects?
    pub email: String,
}

/// Repository for interfacing with subscription-related tables
pub struct SubscriptionRepo;

impl SubscriptionRepo {
    #[tracing::instrument(name = "Insert subscriber", skip(executor))]
    pub async fn insert<'con>(
        executor: impl PgExecutor<'con>,
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
    pub async fn confirm_by_id<'con>(executor: impl PgExecutor<'con>, id: Uuid) -> sqlx::Result<()> {
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
    pub async fn fetch_all_confirmed<'con>(
        executor: impl PgExecutor<'con>,
    ) -> sqlx::Result<Vec<ConfirmedSubscription>> {
        let subscriptions = sqlx::query_as!(
            ConfirmedSubscription,
            "select id, email from subscriptions where confirmed_at is not null"
        )
        .fetch_all(executor)
        .await?;

        Ok(subscriptions)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    fn insert_creates_new_subscriber_record(pool: PgPool) {
        let new_subscriber = NewSubscription {
            email: "test@test.com".parse().unwrap(),
            name: "Test Name".parse().unwrap(),
        };

        let id = SubscriptionRepo::insert(&pool, &new_subscriber)
            .await
            .expect("Failed to insert new record");

        let subscription =
            sqlx::query_as!(Subscription, "select * from subscriptions where id=$1", id)
                .fetch_one(&pool)
                .await
                .expect("Failed to query for record");

        assert_eq!(id, subscription.id);
        assert_eq!(new_subscriber.name, subscription.name.parse().unwrap());
        assert_eq!(new_subscriber.email, subscription.email.parse().unwrap());
    }

    #[sqlx::test(migrations = "../migrations")]
    fn confirm_sets_confirmed_at(pool: PgPool) {
        let new_subscriber = NewSubscription {
            email: "test@test.com".parse().unwrap(),
            name: "Test Name".parse().unwrap(),
        };

        let id = SubscriptionRepo::insert(&pool, &new_subscriber)
            .await
            .expect("Failed to insert new record");

        SubscriptionRepo::confirm_by_id(&pool, id)
            .await
            .expect("Failed to confirm record");

        let subscription =
            sqlx::query_as!(Subscription, "select * from subscriptions where id=$1", id)
                .fetch_one(&pool)
                .await
                .expect("Failed to query for record");

        assert!(subscription.confirmed_at.is_some());
    }

    #[sqlx::test(migrations = "../migrations")]
    fn fetch_all_confirmed_does_not_return_unconfirmed_records(pool: PgPool) {
        let new_subscriber = NewSubscription {
            email: "test@test.com".parse().unwrap(),
            name: "Test Name".parse().unwrap(),
        };

        SubscriptionRepo::insert(&pool, &new_subscriber)
            .await
            .expect("Failed to insert new record");

        let confirmed = SubscriptionRepo::fetch_all_confirmed(&pool)
            .await
            .expect("Failed to fetch confirmed");

        assert!(confirmed.is_empty());
    }
}


use uuid::Uuid;

use chrono::Utc;

use sqlx::PgExecutor;

use crate::domain::{EmailAddress, PersonName};

/// New Subscription request
#[derive(Debug)]
pub struct NewSubscription {
    pub name: PersonName,
    pub email: EmailAddress,
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
    pub async fn confirm_by_id<'con>(
        executor: impl PgExecutor<'con>,
        id: Uuid,
    ) -> sqlx::Result<()> {
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
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test]
    fn insert_creates_new_subscriber_record(pool: PgPool) {
        let new_subscriber = NewSubscription {
            email: "test@test.com".parse().unwrap(),
            name: "Test Name".parse().unwrap(),
        };

        let id = SubscriptionRepo::insert(&pool, &new_subscriber)
            .await
            .expect("Failed to insert new record");

        let row = sqlx::query!("select * from subscriptions where id=$1", id)
            .fetch_one(&pool)
            .await
            .expect("Failed to query for record");

        assert_eq!(id, row.id);
        assert_eq!(new_subscriber.name, row.name.parse().unwrap());
        assert_eq!(new_subscriber.email, row.email.parse().unwrap());
    }

    #[sqlx::test]
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

        let row = sqlx::query!("select * from subscriptions where id=$1", id)
            .fetch_one(&pool)
            .await
            .expect("Failed to query for record");

        assert!(row.confirmed_at.is_some());
    }

    #[sqlx::test]
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

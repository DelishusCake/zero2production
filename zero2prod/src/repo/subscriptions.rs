use uuid::Uuid;

use chrono::Utc;

use sqlx::{Executor, PgExecutor};

use crate::model::{ConfirmedSubscription, NewSubscription};

/// Subscription repository trait, must be implemented for each database used.
/// NOTE: Intended to facilitate easier testing/mocking
/// TODO: Swap async-trait for std async traits when those become stable
/// https://github.com/orgs/rust-lang/projects/28/views/2?pane=issue&itemId=21990165
#[async_trait::async_trait]
pub trait SubscriptionRepo {
    type DB: sqlx::Database;

    /// Insert a new subscriber into the database
    async fn insert<'con>(
        executor: impl Executor<'con, Database = Self::DB>,
        new_subscriber: &NewSubscription,
    ) -> sqlx::Result<Uuid>;

    /// Confirm an existing subscriber by database ID
    async fn confirm_by_id<'con>(
        executor: impl Executor<'con, Database = Self::DB>,
        id: Uuid,
    ) -> sqlx::Result<()>;

    /// Fetch all subscribers that have been confirmed
    async fn fetch_all_confirmed<'con>(
        executor: impl Executor<'con, Database = Self::DB>,
    ) -> sqlx::Result<Vec<ConfirmedSubscription>>;
}

/// Postgres Subscription Repositiory
#[derive(Debug)]
pub struct PgSubscriptionRepo;

#[async_trait::async_trait]
impl SubscriptionRepo for PgSubscriptionRepo {
    type DB = sqlx::Postgres;

    #[tracing::instrument(name = "Insert subscriber", skip(executor))]
    async fn insert<'con>(
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
    async fn confirm_by_id<'con>(executor: impl PgExecutor<'con>, id: Uuid) -> sqlx::Result<()> {
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
    async fn fetch_all_confirmed<'con>(
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

    use crate::model::{NewSubscription, Subscription};

    use super::*;

    #[sqlx::test(migrations = "../migrations")]
    fn insert_creates_new_subscriber_record(pool: PgPool) {
        let new_subscriber = NewSubscription {
            email: "test@test.com".parse().unwrap(),
            name: "Test Name".parse().unwrap(),
        };

        let id = PgSubscriptionRepo::insert(&pool, &new_subscriber)
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

        let id = PgSubscriptionRepo::insert(&pool, &new_subscriber)
            .await
            .expect("Failed to insert new record");

        PgSubscriptionRepo::confirm_by_id(&pool, id)
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

        PgSubscriptionRepo::insert(&pool, &new_subscriber)
            .await
            .expect("Failed to insert new record");

        let confirmed = PgSubscriptionRepo::fetch_all_confirmed(&pool)
            .await
            .expect("Failed to fetch confirmed");

        assert!(confirmed.is_empty());
    }
}

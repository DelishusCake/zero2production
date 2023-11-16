use chrono::{DateTime, Utc};

use sqlx::PgExecutor;

use uuid::Uuid;

use crate::domain::OAuth2Provider;

#[derive(Debug)]
pub struct NewUser {
    pub oauth2_provider: OAuth2Provider,
    pub oauth2_provider_id: String,
}

#[derive(Debug)]
pub struct User {
    pub id: Uuid,

    pub oauth2_provider: OAuth2Provider,
    pub oauth2_provider_id: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UsersRepo;

impl UsersRepo {
    #[tracing::instrument("Insert a new user record", skip(executor))]
    pub async fn insert<'conn>(
        executor: impl PgExecutor<'conn>,
        new_user: &NewUser,
    ) -> sqlx::Result<Uuid> {
        let row = sqlx::query!(
            "insert into users(oauth2_provider, oauth2_provider_id) values ($1, $2) returning id;",
            new_user.oauth2_provider.as_ref(),
            &new_user.oauth2_provider_id
        )
        .fetch_one(executor)
        .await?;
        Ok(row.id)
    }

    #[tracing::instrument("Fetch a user record by OAuth2 provider and ID", skip(executor))]
    pub async fn fetch_by_oauth2<'conn>(
        executor: impl PgExecutor<'conn>,
        oauth2_provider: OAuth2Provider,
        oauth2_provider_id: &str,
    ) -> sqlx::Result<Option<User>> {
        let maybe_user = sqlx::query_as!(
            User,
            r#"select * from users where oauth2_provider=$1 and oauth2_provider_id=$2"#,
            oauth2_provider.as_ref(),
            oauth2_provider_id,
        )
        .fetch_optional(executor)
        .await?;
        Ok(maybe_user)
    }
}

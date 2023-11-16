use secrecy::Secret;

use sqlx::PgExecutor;

use uuid::Uuid;

use crate::domain::EmailAddress;

#[derive(Debug)]
pub struct NewUser {
    pub email: EmailAddress,
    pub password_hash: String,
}

#[derive(Debug)]
pub struct UserCredentials {
    pub id: Uuid,
    pub password_hash: Secret<String>,
}

pub struct UsersRepo;

impl UsersRepo {
    #[tracing::instrument("Insert a new user record", skip(executor))]
    pub async fn insert<'conn>(
        executor: impl PgExecutor<'conn>,
        new_user: &NewUser,
    ) -> sqlx::Result<Uuid> {
        let email = new_user.email.as_ref();
        let password_hash = &new_user.password_hash;
        let row = sqlx::query!(
            "insert into users(email, password_hash) values ($1, $2) returning id;",
            email,
            password_hash
        )
        .fetch_one(executor)
        .await?;
        Ok(row.id)
    }

    pub async fn fetch_credentials_by_email<'conn>(
        executor: impl PgExecutor<'conn>,
        email: &EmailAddress,
    ) -> sqlx::Result<Option<UserCredentials>> {
        let maybe_credentials = sqlx::query_as!(
            UserCredentials,
            "select id, password_hash from users where email=$1",
            email.as_ref()
        )
        .fetch_optional(executor)
        .await?;
        Ok(maybe_credentials)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secrecy::ExposeSecret;
    use sqlx::PgPool;

    #[sqlx::test(migrations = "../migrations")]
    fn can_insert_new_users(pool: PgPool) {
        let new_user = NewUser {
            email: "test@test.com".parse().unwrap(),
            password_hash: "test_password_hash".into(),
        };

        let id = UsersRepo::insert(&pool, &new_user)
            .await
            .expect("Failed to insert new user");

        let row = sqlx::query!("select * from users where id=$1", id)
            .fetch_one(&pool)
            .await
            .expect("Failed to fetch inserted row");
        assert_eq!(id, row.id);
        assert_eq!(new_user.email.as_ref(), &row.email);
        assert_eq!(new_user.password_hash, row.password_hash);
    }

    #[sqlx::test(migrations = "../migrations")]
    fn can_fetch_user_credentials_by_email(pool: PgPool) {
        let new_user = NewUser {
            email: "test@test.com".parse().unwrap(),
            password_hash: "test_password_hash".into(),
        };

        let user_id = UsersRepo::insert(&pool, &new_user)
            .await
            .expect("Failed to insert new user");

        let creds = UsersRepo::fetch_credentials_by_email(&pool, &new_user.email)
            .await
            .expect("Failed to fetch user credentials by email")
            .expect("Fetched credentials are empty");

        assert_eq!(user_id, creds.id);
        assert_eq!(&new_user.password_hash, creds.password_hash.expose_secret());
    }
}

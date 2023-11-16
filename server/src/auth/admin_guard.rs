use std::future::Future;
use std::pin::Pin;

use actix_web::{dev, web, FromRequest, HttpRequest};

use argon2::{Argon2, PasswordHash, PasswordVerifier};

use anyhow::Context;

use secrecy::Secret;

use sqlx::PgPool;

use uuid::Uuid;

use zero2prod::domain::EmailAddress;
use zero2prod::repo::UsersRepo;

use crate::auth::Credentials;
use crate::error::{RestError, RestResult};
use crate::telemetry::spawn_blocking_with_tracing;

#[derive(Debug)]
pub struct Administrator(Uuid);

impl FromRequest for Administrator {
    type Error = RestError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            // NOTE: Must be registered with the application at startup
            let pool: &PgPool = req
                .app_data::<web::Data<PgPool>>()
                .expect("PgPool not registered for application");
            // Pull the credentials from the headers
            let creds = Credentials::from_headers(req.headers())
                .map_err(RestError::FailedToAuthenticate)?;
            // Get the user and verify the credentials
            let user_id = validate_credentials(pool, &creds).await?;
            // TODO: Actually check user authorization
            Ok(Administrator(user_id))
        })
    }
}

impl AsRef<Uuid> for Administrator {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

#[tracing::instrument("Validate credentials", skip(credentials, pool))]
async fn validate_credentials(pool: &PgPool, credentials: &Credentials) -> RestResult<Uuid> {
    let email: EmailAddress = credentials
        .username
        .parse()
        .map_err(RestError::ParseError)?;
    let password = credentials.password.clone();

    let user = UsersRepo::fetch_credentials_by_email(pool, &email)
        .await?
        .context("No user stored for email")
        .map_err(RestError::FailedToAuthenticate)?;

    spawn_blocking_with_tracing(move || verify_password_hash(password, user.password_hash))
        .await
        .context("Failed to spawn blocking task")??;

    Ok(user.id)
}

#[tracing::instrument("Verify password hash", skip(password, password_hash))]
fn verify_password_hash(password: Secret<String>, password_hash: Secret<String>) -> RestResult<()> {
    use secrecy::ExposeSecret;

    let password_hash = PasswordHash::new(password_hash.expose_secret())
        .context("Failed to parse stored password hash")?;

    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &password_hash)
        .context("Failed to verify password hash")
        .map_err(RestError::FailedToAuthenticate)?;

    Ok(())
}

use std::future::Future;
use std::pin::Pin;

use actix_web::http::StatusCode;
use actix_web::{dev, web, FromRequest, HttpRequest, ResponseError};

use argon2::{Argon2, PasswordHash, PasswordVerifier};

use anyhow::Context;

use secrecy::Secret;

use sqlx::PgPool;

use thiserror::Error;

use uuid::Uuid;

use zero2prod::domain::EmailAddress;
use zero2prod::repo::UsersRepo;

use crate::auth::Credentials;
use crate::telemetry::spawn_blocking_with_tracing;

#[derive(Debug)]
pub struct Administrator(Uuid);

impl FromRequest for Administrator {
    type Error = AuthError;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let req = req.clone();
        Box::pin(async move {
            // NOTE: Must be registered with the application at startup
            let pool: &PgPool = req
                .app_data::<web::Data<PgPool>>()
                .expect("PgPool not registered for application");
            // Pull the credentials from the headers
            let creds =
                Credentials::from_headers(req.headers()).map_err(AuthError::AuthenticationError)?;
            // Get the user and verify the credentials
            let user_id = validate_credentials(pool, &creds).await?;
            // TODO: Actually check user authorization
            Ok(Self(user_id))
        })
    }
}

impl AsRef<Uuid> for Administrator {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

#[tracing::instrument("Validate credentials", skip(credentials, pool))]
async fn validate_credentials(pool: &PgPool, credentials: &Credentials) -> Result<Uuid, AuthError> {
    let email: EmailAddress = credentials
        .username
        .parse()
        .map_err(AuthError::ParseError)?;
    let password = credentials.password.clone();

    let user = UsersRepo::fetch_credentials_by_email(pool, &email)
        .await?
        .context("No user stored for email")
        .map_err(AuthError::AuthenticationError)?;

    spawn_blocking_with_tracing(move || verify_password_hash(password, user.password_hash))
        .await
        .context("Failed to spawn blocking task")?
        .map_err(AuthError::AuthenticationError)?;

    Ok(user.id)
}

#[tracing::instrument("Verify password hash", skip(password, password_hash))]
fn verify_password_hash(
    password: Secret<String>,
    password_hash: Secret<String>,
) -> anyhow::Result<()> {
    use secrecy::ExposeSecret;

    let password_hash = PasswordHash::new(password_hash.expose_secret())
        .context("Failed to parse stored password hash")?;

    Argon2::default()
        .verify_password(password.expose_secret().as_bytes(), &password_hash)
        .context("Failed to verify password hash")?;

    Ok(())
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Failed to parse: {0}")]
    ParseError(String),

    #[error("Failed to authenticate")]
    AuthenticationError(#[from] anyhow::Error),

    #[error("Internal Server Error")]
    DatabaseError(#[from] sqlx::Error),
}

impl ResponseError for AuthError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::AuthenticationError(_) | Self::DatabaseError(_) => StatusCode::UNAUTHORIZED,
        }
    }
}

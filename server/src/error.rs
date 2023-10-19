use actix_web::error::UrlGenerationError;
use actix_web::http::StatusCode;
use actix_web::ResponseError;

use thiserror::Error;

pub type RestResult<T> = Result<T, RestError>;

// TODO: I18n for errors
#[derive(Debug, Error)]
pub enum RestError {
    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Unauthorized Access: {0}")]
    Unauthorized(String),

    #[error("Internal Server Error: {0}")]
    InternalError(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<UrlGenerationError> for RestError {
    fn from(e: UrlGenerationError) -> Self {
        tracing::error!("Failed to generate URL for controller: {}", e);
        Self::InternalError("URL generation".into())
    }
}

impl From<sqlx::Error> for RestError {
    fn from(_e: sqlx::Error) -> Self {
        Self::InternalError("Database error".into())
    }
}

impl From<zero2prod::error::Error> for RestError {
    fn from(e: zero2prod::error::Error) -> Self {
        use zero2prod::error::Error as E;
        match e {
            E::ParsingError(msg) => Self::ParseError(msg),
            E::TokenVerification(_) => Self::Unauthorized("Failed to verify token".into()),
            E::TokenSigning(_) => Self::InternalError("Failed to sign token".into()),
            E::SendEmailError(_) => Self::InternalError("Failed to send email".into()),
            E::DatabaseError(_) => Self::InternalError("Database error".into()),
        }
    }
}

impl ResponseError for RestError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            Self::InternalError(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

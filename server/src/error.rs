use actix_web::http::StatusCode;
use actix_web::ResponseError;

pub type RestResult<T> = Result<T, RestError>;

/// Errors that can be returned by the Rest API
/// TODO: I18n for errors
#[derive(Debug, thiserror::Error)]
pub enum RestError {
    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Failed to Sign Token")]
    FailedToSignToken(zero2prod::crypto::TokenError),

    #[error("Failed to Verify Token")]
    FailedToVerifyToken(zero2prod::crypto::TokenError),

    #[error("Failed to Authenticate User")]
    FailedToAuthenticate(anyhow::Error),

    #[error("Failed to Send Email")]
    FailedToSendEmail(reqwest::Error),

    #[error("Internal Server Error")]
    FailedToGenerateUrl(#[from] actix_web::error::UrlGenerationError),

    #[error("Internal Server Error")]
    DatabaseError(#[from] sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ResponseError for RestError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::FailedToAuthenticate(_) | Self::FailedToVerifyToken(_) => {
                StatusCode::UNAUTHORIZED
            }
            Self::Other(_)
            | Self::DatabaseError(_)
            | Self::FailedToSendEmail(_)
            | Self::FailedToSignToken(_)
            | Self::FailedToGenerateUrl(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type RestResult<T> = Result<T, RestError>;

// TODO: I18n for errors
#[derive(Debug, Error)]
pub enum RestError {
    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Invalid Confirmation Token")]
    InvalidConfirmationToken,

    #[error("Failed to Sign Token")]
    FailedToSignToken,

    #[error("Failed to Send Email")]
    FailedToSendEmail,

    #[error("Internal Server Error")]
    DatabaseError(#[from] sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl RestError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::InvalidConfirmationToken => StatusCode::UNAUTHORIZED,
            Self::DatabaseError(_)
            | Self::FailedToSignToken
            | Self::FailedToSendEmail
            | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for RestError {
    fn into_response(self) -> Response {
        let body = format!("{}", self);
        let status_code = self.status_code();

        (status_code, body).into_response()
    }
}

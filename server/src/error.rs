use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type RestResult<T> = Result<T, RestError>;

#[derive(Debug, Error)]
pub enum RestError {
    #[error("Parse Error: {0}")]
    ParseError(String),

    #[error("Internal Server Error")]
    DatabaseError(#[from] sqlx::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl RestError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::DatabaseError(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

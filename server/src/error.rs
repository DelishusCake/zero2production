use actix_web::http::StatusCode;
use actix_web::ResponseError;

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

impl ResponseError for RestError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ParseError(_) => StatusCode::BAD_REQUEST,
            Self::DatabaseError(_) | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

use actix_web::http::StatusCode;
use actix_web::ResponseError;

use thiserror::Error;

pub type RestResult<T> = Result<T, RestError>;

#[derive(Debug, Error)]
pub enum RestError {
    #[error("Bad Request")]
    BadRequest,

    #[error("Internal Server Error")]
    InternalServerError,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl ResponseError for RestError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest => StatusCode::BAD_REQUEST,
            Self::InternalServerError | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

use actix_web::http::StatusCode;
use actix_web::ResponseError;

use thiserror::Error;

pub type RestResult<T> = Result<T, RestError>;

#[derive(Debug, Error)]
pub enum RestError {
    #[error("Bad Request: {0}")]
    BadRequest(String),

    #[error("Internal Server Error")]
    InternalServerError,

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl From<zero2prod::Error> for RestError {
    fn from(e: zero2prod::Error) -> Self {
        use zero2prod::Error as E;
        match e {
            E::ParsingError(msg) => Self::BadRequest(msg),
        }
    }
}

impl ResponseError for RestError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::InternalServerError | Self::Other(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

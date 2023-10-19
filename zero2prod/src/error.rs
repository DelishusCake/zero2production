pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    // Parsing errors
    #[error("{0}")]
    ParsingError(String),
    // JWT errors
    #[error("Failed to sign token")]
    TokenSigning(jwt::Error),
    #[error("Failed to verify token")]
    TokenVerification(jwt::Error),
    // Email client errors
    #[error("Failed to send email: {0}")]
    SendEmailError(reqwest::Error),
    // Database errors
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
}

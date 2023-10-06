#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to parse: {0}")]
    ParsingError(String),
}

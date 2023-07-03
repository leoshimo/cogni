//! Errors for cogni library crate

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no API key provided")]
    NoAPIKey,

    #[error("failed to fetch - {0}")]
    FailedToFetch(#[from] reqwest::Error),

    #[error("no messages provided")]
    NoMessagesProvided,

    #[error("unexpected response - {0}")]
    UnexpectedResponse(String),

    #[error("IO Error - {0}")]
    IO(#[from] std::io::Error),

    #[error("Serialization Error - {0}")]
    JSON(#[from] serde_json::Error),
}

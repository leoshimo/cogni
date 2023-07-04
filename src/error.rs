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

    #[error("io error - {0}")]
    IO(#[from] std::io::Error),

    #[error("json serialization error - {0}")]
    JSON(#[from] serde_json::Error),

    #[error("openai api returned error - {}", .error.message)]
    OpenAIError { error: crate::openai::APIError },
}

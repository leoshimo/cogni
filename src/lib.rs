pub mod cli;
pub mod exec;
pub mod openai;
pub mod parse;

pub use exec::exec;
pub use parse::parse_messages;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API Key is not defined")]
    NoAPIKey,

    #[error("Error from HTTP client - {0}")]
    FailedToFetch(reqwest::Error),

    #[error("Unexpected response: {0}")]
    UnexpectedResponse(String),

    #[error("IO Error")]
    IO(std::io::Error),

    #[error("JSON Serialization Error")]
    JSON(serde_json::Error),
}

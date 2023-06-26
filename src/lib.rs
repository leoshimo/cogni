pub mod cli;
pub mod exec;
pub mod openai;

pub use exec::exec;

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
}

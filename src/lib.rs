pub mod cli;
pub mod openai;

// TODO: Move `exec` + etc to own module
// TODO: Write unit tests for `show_chat_response`

use anyhow::{Context, Result};
use openai::{ChatCompletionResponse, FinishReason};
use std::io::{self, BufWriter, Write};

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

/// Execute the invocation
pub async fn exec(invocation: cli::Invocation) -> Result<()> {
    use cli::Invocation::*;

    match invocation {
        ChatCompletion(args) => {
            let client = openai::Client::new(args.api_key)
                .with_context(|| "Failed to initialize HTTP client")?;
            let request = openai::ChatCompletionRequest::builder()
                .model(args.model)
                .messages(args.messages)
                .temperature(args.temperature)
                .build()
                .with_context(|| "Failed to create request")?;

            let res = client.chat_complete(&request).await?;
            show_chat_response(io::stdout(), &res)?;
        }
    }

    Ok(())
}

/// Show formatted output for `ChatCompletionRequest`
fn show_chat_response(dest: impl Write, resp: &ChatCompletionResponse) -> Result<(), Error> {
    let mut writer = BufWriter::new(dest);
    let choice = match resp.choices.len() {
        1 => &resp.choices[0],
        _ => {
            return Err(Error::UnexpectedResponse(format!(
                "More then 1 choice in response: {:?}",
                resp
            )))
        }
    };

    match choice.finish_reason {
        FinishReason::Stop => write!(writer, "{}\n", choice.message.content).map_err(Error::IO)?,
        _ => {
            return Err(Error::UnexpectedResponse(format!(
                "Received unrecognized stop reason for choice: {:?}",
                choice
            )))
        }
    }

    Ok(())
}


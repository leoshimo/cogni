//! Executor for cogni

use crate::cli::ChatCompletionArgs;
use crate::cli::Invocation;
use crate::openai;
use crate::Error;

use anyhow::{Context, Result};
use openai::{ChatCompletionResponse, FinishReason};
use std::io::{self, BufWriter, Write};

/// Execute the invocation
pub async fn exec(inv: Invocation) -> Result<()> {
    use Invocation::*;

    match inv {
        ChatCompletion(args) => {
            let client = openai::Client::new(args.api_key.clone())
                .with_context(|| "Failed to initialize HTTP client")?;

            // TODO: Lifetimes for `ChatCompletionRequest` fields
            let request = openai::ChatCompletionRequest::builder()
                .model(args.model.clone())
                .messages(args.messages.clone())
                .temperature(args.temperature)
                .build()
                .with_context(|| "Failed to create request")?;

            let res = client.chat_complete(&request).await?;
            show_response(io::stdout(), &args, &res)?;
        }
    }

    Ok(())
}

/// Show formatted output for `ChatCompletionRequest`
fn show_response(
    dest: impl Write,
    args: &ChatCompletionArgs,
    resp: &ChatCompletionResponse,
) -> Result<(), Error> {
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
        FinishReason::Stop => writeln!(writer, "{}", choice.message.content).map_err(Error::IO)?,
        _ => {
            return Err(Error::UnexpectedResponse(format!(
                "Received unrecognized stop reason for choice: {:?}",
                choice
            )))
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {

    use std::fmt::Write;
    use crate::{
        cli::ChatCompletionArgs,
        openai::{ChatCompletionResponse, Choice, FinishReason, Message},
    };

    use super::show_response;

    use anyhow::Result;

    #[test]
    fn show_chat_response_plaintext() -> Result<()> {
        let mut output = vec![];
        let args = ChatCompletionArgs::default();
        let resp = ChatCompletionResponse::builder()
            .choices(vec![Choice {
                message: Message::assistant("Hello world"),
                finish_reason: FinishReason::Stop,
            }])
            .build()?;

        let res = show_response(&mut output, &args, &resp);

        assert!(res.is_ok(), "Showing response should succeed");
        assert_eq!(output, b"Hello world\n");

        Ok(())
    }
}

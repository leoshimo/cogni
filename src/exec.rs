//! Executor for cogni

use crate::cli::ChatCompletionArgs;
use crate::cli::Invocation;
use crate::cli::OutputFormat;
use crate::openai;
use crate::openai::Message;
use crate::Error;

use anyhow::{Context, Result};
use openai::{ChatCompletionResponse, FinishReason};
use std::fs::File;
use std::io::IsTerminal;
use std::io::Read;
use std::io::{self, BufWriter, Write};

/// Execute the invocation
pub async fn exec(inv: Invocation) -> Result<()> {
    use Invocation::*;

    match inv {
        // TODO: Move into chat module?
        ChatCompletion(args) => {
            let client = openai::Client::new(args.api_key.clone())
                .with_context(|| "Failed to initialize HTTP client")?;

            let msgs = [args.messages.clone(), read_messages_from_file(&args.file)?].concat();
            if msgs.is_empty() {
                return Err(Error::NoInput.into());
            }

            // TODO: Lifetimes for `ChatCompletionRequest` fields
            let request = openai::ChatCompletionRequest::builder()
                .model(args.model.clone())
                .messages(msgs)
                .temperature(args.temperature)
                .timeout(args.timeout)
                .build()
                .with_context(|| "Failed to create request")?;

            let res = client.chat_complete(&request).await?;
            show_response(io::stdout(), &args, &res)?;
        }
    }

    Ok(())
}

/// Read messages from non-tty stdin or file specified by `args.file`
fn read_messages_from_file(file: &str) -> Result<Vec<Message>> {
    let reader: Option<Box<dyn Read>> = match file {
        "-" => {
            let stdin = io::stdin();
            if stdin.is_terminal() {
                None
            } else {
                Some(Box::new(stdin))
            }
        }
        file => Some(Box::new(File::open(file)?)),
    };

    match reader {
        None => Ok(vec![]),
        Some(mut reader) => {
            let mut content = String::new();
            reader.read_to_string(&mut content)?;
            Ok(vec![Message::user(&content)])
        }
    }
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
        FinishReason::Stop => {
            let output = match args.output_format {
                OutputFormat::Plaintext => choice.message.content.to_string(),
                OutputFormat::JSON => serde_json::to_string(resp).map_err(Error::JSON)?,
                OutputFormat::JSONPretty => {
                    serde_json::to_string_pretty(resp).map_err(Error::JSON)?
                }
            };
            writeln!(writer, "{}", output).map_err(Error::IO)?
        }
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
    use std::time::Duration;

    use chrono::DateTime;
    use predicates::prelude::*;
    use predicates::str;

    use crate::{
        cli::{ChatCompletionArgs, ChatCompletionArgsBuilder, OutputFormat},
        openai::{
            ChatCompletionResponse, ChatCompletionResponseBuilder, Choice, FinishReason, Message,
            Usage,
        },
    };

    use super::*;

    use anyhow::Result;

    #[test]
    fn show_chat_response_plaintext() -> Result<()> {
        let mut output = vec![];
        let args = default_args()
            .output_format(OutputFormat::Plaintext)
            .build()?;
        let resp = default_resp()
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

    #[test]
    fn show_chat_response_json() -> Result<()> {
        let mut output = vec![];
        let args = default_args().output_format(OutputFormat::JSON).build()?;
        let resp = default_resp()
            .choices(vec![Choice {
                message: Message::assistant("Hello world"),
                finish_reason: FinishReason::Stop,
            }])
            .build()?;

        let _ = show_response(&mut output, &args, &resp);
        let output = String::from_utf8(output).expect("Should be valid string");

        let is_json = str::starts_with("{")
            .and(str::contains("\"content\":\"Hello world\""))
            .and(str::ends_with("}\n"));
        assert!(is_json.eval(&dbg!(output)));

        Ok(())
    }

    #[test]
    fn show_chat_response_json_pretty() -> Result<()> {
        let mut output = vec![];
        let args = default_args()
            .output_format(OutputFormat::JSONPretty)
            .build()?;
        let resp = default_resp()
            .choices(vec![Choice {
                message: Message::assistant("Hello world"),
                finish_reason: FinishReason::Stop,
            }])
            .build()?;

        let _ = show_response(&mut output, &args, &resp);
        let output = String::from_utf8(output).expect("Should be valid string");

        let is_json = str::starts_with("{\n")
            .and(str::contains("\"content\": \"Hello world\""))
            .and(str::ends_with("}\n"));
        assert!(is_json.eval(&dbg!(output)));

        Ok(())
    }

    fn default_args() -> ChatCompletionArgsBuilder {
        ChatCompletionArgs::builder()
            .api_key(Some(String::default()))
            .messages(vec![])
            .model(String::default())
            .temperature(1.0)
            .output_format(OutputFormat::Plaintext)
            .timeout(Duration::from_secs(10))
            .file("-".to_string())
            .to_owned()
    }

    fn default_resp() -> ChatCompletionResponseBuilder {
        ChatCompletionResponse::builder()
            .id(String::default())
            .object(String::default())
            .created(DateTime::default())
            .choices(vec![])
            .model(String::default())
            .usage(Usage::default())
            .to_owned()
    }
}

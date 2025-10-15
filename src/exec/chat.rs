//! Implements chat subcommand

use crate::cli::{Invocation, OutputFormat};
use crate::openai::{self, FinishReason, Message, Reasoning, Response};
use crate::parse;
use crate::Error;

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{self, BufWriter, IsTerminal, Read, Write};

/// Executes `Invocation` via given args
pub async fn exec(args: Invocation) -> Result<()> {
    let base_url =
        std::env::var("OPENAI_API_ENDPOINT").unwrap_or("https://api.openai.com".to_string());

    let client = openai::Client::new(args.api_key.clone(), base_url)
        .with_context(|| "failed to create http client")?;

    let file_msgs = read_messages_from_file(&args.file)
        .with_context(|| format!("failed to open {}", &args.file))?;

    let msgs = [args.messages.clone(), file_msgs].concat();

    if msgs.is_empty() {
        return Err(Error::NoMessagesProvided.into());
    }

    // TODO: Lifetimes for `ResponseRequest` fields
    let mut builder = openai::ResponseRequest::builder();

    builder
        .model(args.model.clone())
        .messages(msgs)
        .temperature(args.temperature)
        .timeout(args.timeout);

    if let Some(effort) = args.reasoning_effort {
        builder.reasoning(Some(Reasoning::from_effort(effort)));
    }

    let request = builder
        .build()
        .with_context(|| "failed to create request")?;

    let res = client
        .create_response(&request)
        .await
        .with_context(|| "failed to fetch request")?;

    show_response(io::stdout(), &args, &res)?;
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
        Some(mut r) => Ok(parse::parse_messages(&mut r)?),
    }
}

/// Show formatted output for a Responses API result
fn show_response(dest: impl Write, args: &Invocation, resp: &Response) -> Result<(), Error> {
    let mut writer = BufWriter::new(dest);
    let choice = match resp.choices.len() {
        1 => &resp.choices[0],
        _ => {
            return Err(Error::UnexpectedResponse(format!(
                "Unexpected number of choices in response: {:?}",
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
    use predicate::str;
    use predicates::prelude::*;

    use crate::{
        cli::{Invocation, InvocationBuilder, OutputFormat},
        openai::{Choice, FinishReason, Message, Response, ResponseBuilder, Usage},
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

    fn default_args() -> InvocationBuilder {
        Invocation::builder()
            .api_key(Some(String::default()))
            .messages(vec![])
            .model(String::default())
            .temperature(1.0)
            .output_format(OutputFormat::Plaintext)
            .timeout(Duration::from_secs(10))
            .file("-".to_string())
            .to_owned()
    }

    fn default_resp() -> ResponseBuilder {
        Response::builder()
            .created(DateTime::default())
            .choices(vec![])
            .model(String::default())
            .usage(Usage::default())
            .to_owned()
    }
}

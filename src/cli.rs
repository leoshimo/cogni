//! Command line interface for cogni

use std::time::Duration;

use crate::openai::Message;
use clap::{
    arg, builder::PossibleValue, command, value_parser, ArgGroup, ArgMatches, Command, ValueEnum,
};
use derive_builder::Builder;

/// CLI invocations that can be launched
#[derive(Debug, Default, Builder)]
pub struct Invocation {
    pub api_key: Option<String>,
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
    pub output_format: OutputFormat,
    pub file: String,
    pub timeout: Duration,
}

/// The format that invocation's results are in
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum OutputFormat {
    #[default]
    Plaintext,
    JSON,
    JSONPretty,
}

/// Parse commandline arguments into `Invocation`. May exit with help or error message
#[must_use]
pub fn parse() -> Invocation {
    cli().get_matches().into()
}

/// Top-level command
fn cli() -> Command {
    command!()
        .arg(arg!(model: -m --model <MODEL> "Sets model. See https://platform.openai.com/docs/models for model identifiers.").default_value("gpt-4-1106-preview"))
        .arg(
            arg!(temperature: -t --temperature <TEMP> "Sets temperature")
                .value_parser(value_parser!(f32))
                .default_value("0.0"),
        )
        .arg(
            arg!(timeout: -T --timeout <DURATION> "Sets timeout duration in seconds")
                .value_parser(value_parser!(u64))
                .default_value("60")
        )
        .arg(arg!(system_message: -s --system <MSG> "Sets system prompt").required(false))
        .arg(
            arg!(assistant_messages: -a --assistant <MSG> ... "Appends assistant message")
                .required(false),
        )
        .arg(arg!(user_messages: -u --user <MSG> ... "Appends user message").required(false))
        .arg(
            arg!(api_key: --apikey <API_KEY> "Sets API Key to use")
                .env("OPENAI_API_KEY")
                .hide_env_values(true),
        )
        .arg(
            arg!(output_format: --output_format <FORMAT> "Sets output format")
                .value_parser(value_parser!(OutputFormat))
                .conflicts_with("output_format_short")
                .default_value_ifs([
                    ("json", "true", Some("json")),
                    ("jsonp", "true", Some("jsonpretty")),
                ])
                .default_value("plaintext"),
        )
        .arg(arg!(--json "Shorthand for --output_format json"))
        .arg(arg!(--jsonp "Shorthand for --output_format jsonpretty"))
        .group(ArgGroup::new("output_format_short").args(["json", "jsonp"]))
        .arg(arg!(file: [FILE] "File providing messages to append to chat log. If \"-\", reads from non-tty stdin").default_value("-"))
}

impl From<ArgMatches> for Invocation {
    fn from(matches: ArgMatches) -> Self {
        let api_key = matches.get_one::<String>("api_key").cloned();
        let messages = Invocation::messages_from_matches(&matches);
        let model = matches
            .get_one::<String>("model")
            .expect("Models is required")
            .to_string();

        let temperature = *matches
            .get_one::<f32>("temperature")
            .expect("Temperature is required");

        let timeout = matches
            .get_one::<u64>("timeout")
            .map(|t| Duration::from_secs(*t))
            .expect("Timeout is required");

        let output_format = *matches
            .get_one::<OutputFormat>("output_format")
            .expect("Output format is required");

        let file = matches
            .get_one::<String>("file")
            .expect("File is required")
            .to_string();

        Self {
            api_key,
            messages,
            model,
            temperature,
            timeout,
            output_format,
            file,
        }
    }
}

impl Invocation {
    /// Builder
    pub fn builder() -> InvocationBuilder {
        InvocationBuilder::default()
    }

    /// Given `clap::ArgMatches`, creates a vector of `Message` with assigned roles and ordering
    fn messages_from_matches(matches: &ArgMatches) -> Vec<Message> {
        let mut messages = vec![];

        if let Some(user_msgs) = matches.get_many::<String>("user_messages") {
            messages.extend(
                user_msgs
                    .map(|c| Message::user(c))
                    .zip(matches.indices_of("user_messages").unwrap()),
            );
        }
        if let Some(asst_msgs) = matches.get_many::<String>("assistant_messages") {
            messages.extend(
                asst_msgs
                    .map(|c| Message::assistant(c))
                    .zip(matches.indices_of("assistant_messages").unwrap()),
            );
        }
        messages.sort_by(|(_a, a_idx), (_b, b_idx)| a_idx.cmp(b_idx));
        let mut messages = messages.into_iter().map(|(a, _)| a).collect::<Vec<_>>();

        // System message is always first
        if let Some(system_msg) = matches.get_one::<String>("system_message") {
            messages.insert(0, Message::system(system_msg));
        }

        messages
    }
}

impl ValueEnum for OutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Plaintext, Self::JSON, Self::JSONPretty]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Plaintext => PossibleValue::new("plaintext"),
            Self::JSON => PossibleValue::new("json"),
            Self::JSONPretty => PossibleValue::new("jsonpretty"),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn chat_one_msgs() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec!["cogni", "-u", "USER"])
            .map(Invocation::from)?;

        assert_eq!(args.messages, vec![Message::user("USER")]);
        Ok(())
    }

    #[test]
    fn chat_many_msgs() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec!["cogni", "-u", "USER1", "-a", "ROBOT", "-u", "USER2"])
            .map(Invocation::from)?;

        assert_eq!(
            args.messages,
            vec![
                Message::user("USER1"),
                Message::assistant("ROBOT"),
                Message::user("USER2"),
            ],
            "messages should contain all messages in order"
        );
        Ok(())
    }

    #[test]
    fn chat_many_msgs_with_system_prompt() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec![
                "cogni", "-s", "SYSTEM", "-u", "USER1", "-a", "ROBOT", "-u", "USER2",
            ])
            .map(Invocation::from)?;

        assert_eq!(
            args.messages,
            vec![
                Message::system("SYSTEM"),
                Message::user("USER1"),
                Message::assistant("ROBOT"),
                Message::user("USER2"),
            ],
            "messages should contain all messages in order"
        );
        Ok(())
    }

    #[test]
    fn chat_many_msgs_with_system_prompt_last() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec![
                "cogni", "-s", "SYSTEM", "-u", "USER1", "-a", "ROBOT", "-u", "USER2",
            ])
            .map(Invocation::from)?;

        assert_eq!(
            args.messages,
            vec![
                Message::system("SYSTEM"),
                Message::user("USER1"),
                Message::assistant("ROBOT"),
                Message::user("USER2"),
            ],
            "system message is always brought to front, followed by assistant and user messages in order");

        Ok(())
    }

    #[test]
    fn chat_output_format_default() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec!["cogni", "-u", "ABC"])
            .map(Invocation::from)?;

        assert_eq!(
            args.output_format,
            OutputFormat::Plaintext,
            "Default output format should be plaintext"
        );
        Ok(())
    }

    #[test]
    fn chat_output_format_explicit_json() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec!["cogni", "-u", "ABC", "--output_format", "json"])
            .map(Invocation::from)?;

        assert_eq!(args.output_format, OutputFormat::JSON);
        Ok(())
    }

    #[test]
    fn chat_output_format_shorthand_json() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec!["cogni", "-u", "ABC", "--json"])
            .map(Invocation::from)?;

        assert_eq!(args.output_format, OutputFormat::JSON);
        Ok(())
    }

    #[test]
    fn chat_output_format_shorthand_jsonp() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec!["cogni", "-u", "ABC", "--jsonp"])
            .map(Invocation::from)?;

        assert_eq!(args.output_format, OutputFormat::JSONPretty);
        Ok(())
    }

    #[test]
    fn chat_file_default() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec!["cogni"])
            .map(Invocation::from)?;

        assert_eq!(args.file, "-");
        Ok(())
    }

    #[test]
    fn chat_file_positional() -> Result<()> {
        let args = cli()
            .try_get_matches_from(vec!["cogni", "dialog_log"])
            .map(Invocation::from)?;

        assert_eq!(args.file, "dialog_log");
        Ok(())
    }
}

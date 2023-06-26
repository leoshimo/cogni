//! Command line interface for cogni

// TODO: Support stdin

use crate::openai::Message;
use clap::{
    arg, builder::PossibleValue, command, value_parser, ArgGroup, ArgMatches, Command, ValueEnum,
};
use derive_builder::Builder;

/// CLI invocations that can be launched
#[derive(Debug)]
pub enum Invocation {
    /// Invoke chat completion,
    ChatCompletion(ChatCompletionArgs),
}

/// Arguments parsed for ChatCompletion
#[derive(Debug, Default, Builder)]
#[builder(default)]
pub struct ChatCompletionArgs {
    pub api_key: Option<String>,
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
    pub output_format: OutputFormat,
}

/// The format that invocation's results are in
#[derive(Debug, Default, PartialEq, Clone, Copy)]
pub enum OutputFormat {
    #[default]
    Plaintext,
    JSON,
    JSONPretty,
}

/// Parse commandline arguments into `ChatCompletionArgs`. May exit with help or error message
#[must_use]
pub fn parse() -> Invocation {
    cli().get_matches().into()
}

/// Top-level command
fn cli() -> Command {
    command!()
        .subcommand(chat_completion_cmd())
        .subcommand_required(true)
}

/// Subcommand for chat completion interface
fn chat_completion_cmd() -> Command {
    Command::new("chat")
        .about("Chat Completion")
        .arg(arg!(model: -m --model <MODEL> "Sets model").default_value("gpt-3.5-turbo"))
        .arg(
            arg!(temperature: -t --temperature <TEMP> "Sets temperature")
                .value_parser(value_parser!(f32))
                .default_value("1.0"),
        )
        .arg(arg!(system_message: -s --system <MSG> "Sets system prompt").required(false))
        .arg(
            arg!(assistant_messages: -a --assistant <MSG> ... "Appends assistant message")
                .required(false),
        )
        .arg(arg!(user_messages: -u --user <MSG> ... "Appends user message").required(true))
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
}

impl From<ArgMatches> for Invocation {
    fn from(matches: ArgMatches) -> Self {
        use Invocation::*;

        let (name, submatch) = matches.subcommand().expect("Subcommands are required");

        match name {
            "chat" => ChatCompletion(ChatCompletionArgs::from(submatch.to_owned())),
            _ => {
                panic!("Unrecognized subcommand");
            }
        }
    }
}

impl From<ArgMatches> for ChatCompletionArgs {
    fn from(matches: ArgMatches) -> Self {
        let api_key = matches.get_one::<String>("api_key").cloned();
        let messages = ChatCompletionArgs::messages_from_matches(&matches);
        let model = matches
            .get_one::<String>("model")
            .expect("Models is required")
            .to_string();

        let temperature = *matches
            .get_one::<f32>("temperature")
            .expect("Temperature is required");

        let output_format = *matches
            .get_one::<OutputFormat>("output_format")
            .expect("Output format is required");

        Self {
            api_key,
            messages,
            model,
            temperature,
            output_format,
        }
    }
}

impl ChatCompletionArgs {
    /// Builder
    pub fn builder() -> ChatCompletionArgsBuilder {
        ChatCompletionArgsBuilder::default()
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
    use super::Invocation::*;
    use super::*;
    use anyhow::Result;

    #[test]
    fn chat_no_args_is_err() {
        let res = cli()
            .try_get_matches_from(vec!["cogni"])
            .map(Invocation::from);
        assert!(res.is_err());
    }

    #[test]
    fn chat_one_msgs() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "chat", "-u", "USER"])
            .map(Invocation::from)?;
        let ChatCompletion(args) = res;

        assert_eq!(args.messages, vec![Message::user("USER")]);
        Ok(())
    }

    #[test]
    fn chat_many_msgs() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec![
                "cogni", "chat", "-u", "USER1", "-a", "ROBOT", "-u", "USER2",
            ])
            .map(Invocation::from)?;
        let ChatCompletion(args) = res;

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
        let res = cli()
            .try_get_matches_from(vec![
                "cogni", "chat", "-s", "SYSTEM", "-u", "USER1", "-a", "ROBOT", "-u", "USER2",
            ])
            .map(Invocation::from)?;
        let ChatCompletion(args) = res;

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
        let res = cli()
            .try_get_matches_from(vec![
                "cogni", "chat", "-s", "SYSTEM", "-u", "USER1", "-a", "ROBOT", "-u", "USER2",
            ])
            .map(Invocation::from)?;
        let ChatCompletion(args) = res;

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
        let res = cli()
            .try_get_matches_from(vec!["cogni", "chat", "-u", "ABC"])
            .map(Invocation::from)?;
        let ChatCompletion(args) = res;

        assert_eq!(
            args.output_format,
            OutputFormat::Plaintext,
            "Default output format should be plaintext"
        );
        Ok(())
    }

    #[test]
    fn chat_output_format_explicit_json() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec![
                "cogni",
                "chat",
                "-u",
                "ABC",
                "--output_format",
                "json",
            ])
            .map(Invocation::from)?;
        let ChatCompletion(args) = res;

        assert_eq!(args.output_format, OutputFormat::JSON);
        Ok(())
    }

    #[test]
    fn chat_output_format_shorthand_json() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "chat", "-u", "ABC", "--json"])
            .map(Invocation::from)?;
        let ChatCompletion(args) = res;

        assert_eq!(args.output_format, OutputFormat::JSON);
        Ok(())
    }

    #[test]
    fn chat_output_format_shorthand_jsonp() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "chat", "-u", "ABC", "--jsonp"])
            .map(Invocation::from)?;
        let ChatCompletion(args) = res;

        assert_eq!(args.output_format, OutputFormat::JSONPretty);
        Ok(())
    }
}

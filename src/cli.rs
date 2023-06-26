//! Command line interface for cogni

use crate::openai::Message;
use clap::{arg, command, value_parser, ArgMatches, Command};

/// CLI invocations that can be launched
#[derive(Debug)]
pub enum Invocation {
    /// Invoke chat completion,
    ChatCompletion(ChatCompletionArgs),
}

/// Arguments parsed for ChatCompletion
#[derive(Debug, Default)]
pub struct ChatCompletionArgs {
    pub api_key: Option<String>,
    pub messages: Vec<Message>,
    pub model: String,
    pub temperature: f32,
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

        Self {
            api_key,
            messages,
            model,
            temperature,
        }
    }
}

impl ChatCompletionArgs {
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
    fn many_msgs_with_system_prompt_last() -> Result<()> {
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
}

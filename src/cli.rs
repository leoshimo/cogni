//! Command line interface for cogni

use std::time::Duration;

use crate::openai::Message;
use clap::{
    arg, builder::PossibleValue, command, value_parser, Arg, ArgGroup, ArgMatches, Command,
    ValueEnum,
};
use derive_builder::Builder;

/// CLI invocations that can be launched
#[derive(Debug)]
pub enum Invocation {
    /// Invoke chat completion,
    ChatCompletion(ChatCompletionArgs),

    /// Invoke template run
    RunTemplate(RunTemplateArgs),
}

/// Arguments parsed for ChatCompletion
#[derive(Debug, Default, Builder)]
pub struct ChatCompletionArgs {
    /// Messages composing chat completion request (prepended)
    pub messages: Vec<Message>,
    /// File to source additional messages to append to `messages`
    pub file: String,
    /// The format that response is output in
    pub output_format: OutputFormat,
    /// API Key to use
    pub api_key: Option<String>,

    pub model: String,
    pub temperature: f32,
    pub timeout: Duration,
}

/// Arguments parsed for RunTemplate
#[derive(Debug, Default, Builder)]
pub struct RunTemplateArgs {
    /// Spec
    pub template_spec: String,
    /// The format that response is output in
    pub output_format: OutputFormat,
    /// API Key to use
    pub api_key: Option<String>,
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
        .subcommand(run_template_cmd())
        .subcommand_required(true)
}

/// Subcommand for chat completion CLI interface
fn chat_completion_cmd() -> Command {
    let cmd = Command::new("chat")
        .about("Run Ad-Hoc Chat Completions")
        .arg(
            arg_openai_api_key()
        )
        .arg(arg!(model: -m --model <MODEL> "Sets model").default_value("gpt-3.5-turbo"))
        .arg(
            arg!(temperature: -t --temperature <TEMP> "Sets temperature")
                .value_parser(value_parser!(f32))
                .default_value("0.0"),
        )
        .arg(
            arg!(timeout: -T --timeout <DURATION> "Sets timeout duration in seconds")
                .value_parser(value_parser!(u64))
                .default_value("30")
        )
        .arg(arg!(system_message: -s --system <MSG> "Sets system prompt").required(false))
        .arg(
            arg!(assistant_messages: -a --assistant <MSG> ... "Appends assistant message")
                .required(false),
        )
        .arg(arg!(user_messages: -u --user <MSG> ... "Appends user message").required(false))
        .arg(arg!(file: [FILE] "File providing messages to append to chat log. If \"-\", reads from non-tty stdin").default_value("-"));

    add_args_output_format(cmd)
}

/// Subcommand for template exec CLI interface
fn run_template_cmd() -> Command {
    let cmd = Command::new("run")
        .about("Run Chat Completion Templates")
        .arg(
            arg_openai_api_key()
        )
        .arg(
            arg!(template_spec: [TEMPLATE_SPEC]
                 "Specifier for template to execute. \
                  Supports template commands in configuration directory and absolute paths to templates.\
                  If \"-\" and stdin is non-tty, uses stdin").default_value("-")
        );

    add_args_output_format(cmd)
}

/// Arg for OpenAI API Key CLI argument
fn arg_openai_api_key() -> Arg {
    arg!(api_key: --apikey <API_KEY> "Sets API Key to use. Overrides OPENAI_API_KEY")
        .env("OPENAI_API_KEY")
        .hide_env_values(true)
}

/// Adds common output_format flags
fn add_args_output_format(cmd: Command) -> Command {
    cmd.arg(
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
            "run" => RunTemplate(RunTemplateArgs::from(submatch.to_owned())),
            _ => {
                panic!("Unrecognized subcommand");
            }
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

impl From<ArgMatches> for RunTemplateArgs {
    fn from(matches: ArgMatches) -> Self {
        let api_key = matches.get_one::<String>("api_key").cloned();
        let output_format = *matches
            .get_one::<OutputFormat>("output_format")
            .expect("Output format is required");
        let template_spec = matches
            .get_one::<String>("template_spec")
            .expect("template_spec is required")
            .to_string();
        Self {
            template_spec,
            api_key,
            output_format,
        }
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

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    #[test]
    fn no_subcmd_is_err() {
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
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

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
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

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
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

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
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

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
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

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
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

        assert_eq!(args.output_format, OutputFormat::JSON);
        Ok(())
    }

    #[test]
    fn chat_output_format_shorthand_json() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "chat", "-u", "ABC", "--json"])
            .map(Invocation::from)?;
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

        assert_eq!(args.output_format, OutputFormat::JSON);
        Ok(())
    }

    #[test]
    fn chat_output_format_shorthand_jsonp() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "chat", "-u", "ABC", "--jsonp"])
            .map(Invocation::from)?;
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

        assert_eq!(args.output_format, OutputFormat::JSONPretty);
        Ok(())
    }

    #[test]
    fn chat_file_default() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "chat"])
            .map(Invocation::from)?;
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

        assert_eq!(args.file, "-");
        Ok(())
    }

    #[test]
    fn chat_file_positional() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "chat", "dialog_log"])
            .map(Invocation::from)?;
        let args = match res {
            ChatCompletion(args) => args,
            _ => return Err(format!("Unexpected invocation parsed - {:?}", res).into()),
        };

        assert_eq!(args.file, "dialog_log");
        Ok(())
    }

    #[test]
    fn run_no_template_spec() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "run"])
            .map(Invocation::from)?;

        let args = match res {
            Invocation::RunTemplate(args) => args,
            _ => return Err(format!("Unexpected invocation: {:?}", res).into()),
        };

        assert_eq!(args.template_spec, "-");
        Ok(())
    }

    #[test]
    fn run_with_template_spec() -> Result<()> {
        let res = cli()
            .try_get_matches_from(vec!["cogni", "run", "my_cmd"])
            .map(Invocation::from)?;

        let args = match res {
            Invocation::RunTemplate(args) => args,
            _ => return Err(format!("Unexpected invocation: {:?}", res).into()),
        };

        assert_eq!(args.template_spec, "my_cmd");
        Ok(())
    }
}

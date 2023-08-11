//! Parse from input streams

use std::io::Read;

use crate::openai::Message;
use crate::Error;
use serde::{Deserialize, Serialize};

// TODO: feat: Handle single line user-messages, multi-line multi-role messages
/// Read from `std::io::Read` into a vector of messages
pub fn parse_messages(input: &mut impl Read) -> Result<Vec<Message>, Error> {
    let mut content = String::new();
    input.read_to_string(&mut content).map_err(Error::IO)?;

    if content.trim().is_empty() {
        Ok(vec![])
    } else {
        Ok(vec![Message::user(&content)])
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Template {
    messages: Vec<Message>,
}

/// Read from `std::io::Read` into a template that describes a chat completion request
pub fn parse_template(input: &mut impl Read) -> Result<Template, Error> {
    let mut content = String::new();
    input.read_to_string(&mut content)?;
    Ok(toml::from_str::<Template>(&content)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use assert_fs::prelude::*;
    use std::fs::File;

    #[test]
    fn parse_short_message() {
        let mut data = "Hello world".as_bytes();
        let messages = parse_messages(&mut data).expect("parse_messages should succeed");
        assert_eq!(
            messages,
            vec![Message::user("Hello world")],
            "Should have single message for user"
        );
    }

    #[test]
    fn parse_empty_input() {
        let mut data = "".as_bytes();
        let messages = parse_messages(&mut data).expect("parse_messages should succeed");
        assert_eq!(messages, vec![], "Should have no messages");
    }

    #[test]
    fn parse_long_message_from_file() {
        let file = assert_fs::NamedTempFile::new("input.txt").unwrap();
        file.write_str("Hello world").unwrap();

        let mut file = File::open(file.path()).unwrap();
        let messages = parse_messages(&mut file).expect("should succeed");

        assert_eq!(messages, vec![Message::user("Hello world")]);
    }

    #[test]
    fn parse_empty_template() {
        let mut data = "".as_bytes();
        let t = parse_template(&mut data);
        assert!(t.is_err());
    }

    #[test]
    fn parse_template_with_messages() {
        let mut data = r#"
[[messages]]
role = "system"
content = "This is a system message"

[[messages]]
role = "user"
content = '''This is a multiline content
that uses multiple lines'''

[[messages]]
role = "assistant"
content = '''This one has "quoted strings"'''
        "#
        .as_bytes();
        let t = parse_template(&mut data).expect("parse_template should succeed");

        assert_eq!(
            t.messages,
            vec![
                Message::system("This is a system message"),
                Message::user("This is a multiline content\nthat uses multiple lines"),
                Message::assistant("This one has \"quoted strings\""),
            ]
        );
    }
}

//! Parse from input streams

use std::io::Read;

use crate::openai::Message;
use crate::Error;

/// Read from `std::io::Read` into a vector of messages
pub fn parse_messages(input: &mut impl Read) -> Result<Vec<Message>, Error> {
    let mut content = String::new();
    input.read_to_string(&mut content).map_err(Error::IO)?;

    Ok(vec![Message::user(&content)])
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
    fn parse_long_message_from_file() {
        let file = assert_fs::NamedTempFile::new("input.txt").unwrap();
        file.write_str("Hello world").unwrap();

        let mut file = File::open(file.path()).unwrap();
        let messages = parse_messages(&mut file).expect("should succeed");

        assert_eq!(messages, vec![Message::user("Hello world")]);
    }
}

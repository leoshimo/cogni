pub mod cli;
pub mod error;
pub mod exec;
pub mod openai;
pub mod parse;

pub use error::Error;
pub use exec::exec;
pub use parse::parse_messages;

pub type Result<T> = std::result::Result<T, Error>;

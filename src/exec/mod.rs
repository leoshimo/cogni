//! Executor for cogni
pub mod chat;
pub mod template;

use crate::cli::Invocation;
use anyhow::Result;

/// Execute the invocation
pub async fn exec(inv: Invocation) -> Result<()> {
    use Invocation::*;
    match inv {
        ChatCompletion(args) => chat::exec(args).await,
        RunTemplate(args) => template::exec(args).await,
    }
}

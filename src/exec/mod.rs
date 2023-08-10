//! Executor for cogni
pub mod chat;

use crate::cli::Invocation;
use anyhow::Result;

/// Execute the invocation
pub async fn exec(inv: Invocation) -> Result<()> {
    use Invocation::*;
    match inv {
        ChatCompletion(args) => chat::exec(args).await,
    }
}


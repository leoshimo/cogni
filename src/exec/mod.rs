//! Executor for cogni
pub mod chat;

use crate::cli::Invocation;
use anyhow::Result;

/// Execute the invocation
pub async fn exec(inv: Invocation) -> Result<()> {
    chat::exec(inv).await
}

use anyhow::Result;
use cogni::cli;

#[tokio::main]
async fn main() -> Result<()> {
    let invocation = cli::parse();
    cogni::exec(invocation).await?;
    Ok(())
}

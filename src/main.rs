use anyhow::{Context, Result};
use cogni::{cli, openai};

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::parse();

    let client = openai::Client::new(args.api_key);
    let request = openai::ChatCompletionRequest::builder()
        .model(args.model)
        .messages(args.messages)
        .temperature(args.temperature)
        .build()
        .with_context(|| "Failed to create request")?;

    let res = client.chat_complete(&request).await?;
    println!("{:?}", res);

    Ok(())
}

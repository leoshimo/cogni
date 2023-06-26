pub mod cli;
pub mod openai;

use anyhow::{Result, Context};

/// Execute the invocation
pub async fn exec(invocation: cli::Invocation) -> Result<()> {
    use cli::Invocation::*;

    match invocation {
        ChatCompletion(args) => {
            let client = openai::Client::new(args.api_key)
                .with_context(|| "Failed to initialize HTTP client")?;
            let request = openai::ChatCompletionRequest::builder()
                .model(args.model)
                .messages(args.messages)
                .temperature(args.temperature)
                .build()
                .with_context(|| "Failed to create request")?;

            let res = client.chat_complete(&request).await?;
            println!("{:?}", res);
        }
    }


    Ok(())
}

use anyhow::{Context, Result};
use cogni::openai;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let api_key = env::var("OPENAI_API_KEY").ok();
    let model = "gpt-4";
    let messages = vec![openai::Message {
        role: openai::Role::User,
        content: "Hello world".to_string(),
    }];
    let temperature = 0.7;

    let client = openai::Client::new(api_key);
    let request = openai::ChatCompletionRequest::builder()
        .model(model.to_string())
        .messages(messages)
        .temperature(Some(temperature))
        .build()
        .with_context(|| "Failed to create request")?;

    let res = client.chat_complete(&request).await?;
    println!("{:?}", res);

    Ok(())
}

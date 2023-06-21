use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Convienience Client for OpenAI Chat Completions API
pub struct Client {
    /// Inner client
    client: reqwest::Client,
    /// Default API Key
    api_key: Option<String>,
}

/// Requests for chat_completion
/// Reference: https://platform.openai.com/docs/api-reference/chat
#[derive(Builder, Default)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: Option<f32>,
}

/// Responses from chat_completion
/// Reference: https://platform.openai.com/docs/api-reference/chat
#[derive(Debug, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    #[serde(with = "ts_seconds")]
    pub created: DateTime<Utc>,
    pub choices: Vec<Choice>,
    pub model: String,
    pub usage: Usage,
}

/// Errors from module
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API Key is not defined")]
    NoAPIKey,

    #[error("Failed to fetch from API")]
    FailedToFetch(reqwest::Error),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Assistant,
    User,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: FinishReason,
}

impl Client {
    pub fn new(api_key: Option<String>) -> Client {
        Client {
            client: reqwest::Client::new(),
            api_key,
        }
    }

    pub async fn chat_complete(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, Error> {
        let api_key = &self.api_key.as_ref().ok_or(Error::NoAPIKey)?;
        let model = &request.model;

        let resp = self
            .client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(api_key)
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": model,
                "messages": request.messages,
                "temperature": request.temperature,
            }))
            .send()
            .await
            .map_err(Error::FailedToFetch)?;

        let res: ChatCompletionResponse = resp.json().await.map_err(Error::FailedToFetch)?;

        Ok(res)
    }
}

impl ChatCompletionRequest {
    pub fn builder() -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder::default()
    }
}

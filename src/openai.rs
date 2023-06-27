//! Interacting with OpenAI API

use std::time::Duration;

use crate::Error;
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
/// Reference: <https://platform.openai.com/docs/api-reference/chat>
#[derive(Builder, Default)]
#[builder(default)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

/// Responses from chat_completion
/// Reference: <https://platform.openai.com/docs/api-reference/chat>
#[derive(Builder, Default, Debug, Serialize, Deserialize)]
#[builder(default)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    #[serde(with = "ts_seconds")]
    pub created: DateTime<Utc>,
    pub choices: Vec<Choice>,
    pub model: String,
    pub usage: Usage,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

impl Message {
    pub fn system(content: &str) -> Message {
        Message {
            role: Role::System,
            content: content.to_string(),
        }
    }
    pub fn user(content: &str) -> Message {
        Message {
            role: Role::User,
            content: content.to_string(),
        }
    }
    pub fn assistant(content: &str) -> Message {
        Message {
            role: Role::Assistant,
            content: content.to_string(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Assistant,
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: FinishReason,
}

impl Client {
    pub fn new(api_key: Option<String>) -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .map_err(Error::FailedToFetch)?;
        Ok(Self { client, api_key })
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

impl ChatCompletionResponse {
    pub fn builder() -> ChatCompletionResponseBuilder {
        ChatCompletionResponseBuilder::default()
    }
}

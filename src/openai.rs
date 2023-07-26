//! Interactions with OpenAI APIs

use std::time::Duration;

use crate::Error;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Convienience Client for OpenAI Chat Completions API
pub struct Client {
    /// Inner client
    client: reqwest::Client,
    /// Default API Key
    api_key: Option<String>,
    /// Base URL for API Endpoint
    base_url: String,
}

/// Requests for chat_completion
/// Reference: <https://platform.openai.com/docs/api-reference/chat>
#[derive(Builder, Default)]
pub struct ChatCompletionRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    timeout: Duration,
}

/// Responses from chat_completion
/// Reference: <https://platform.openai.com/docs/api-reference/chat>
#[derive(Builder, Default, Debug, Serialize, Deserialize)]
pub struct ChatCompletion {
    #[serde(with = "ts_seconds")]
    pub created: DateTime<Utc>,
    pub choices: Vec<Choice>,
    pub model: String,
    pub usage: Usage,
}

/// API Errors from OpenAI
#[derive(Debug, Deserialize)]
pub struct APIError {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub param: Option<String>,
    pub code: Option<String>,
}

/// Wraps `APIError` for deserializing OpenAI Response
#[derive(Debug, Deserialize)]
struct APIErrorContainer {
    error: APIError,
}

/// Messages in chat completion request and response
#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    Assistant,
    User,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    FunctionCall,
    ContentFilter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: FinishReason,
}

impl Client {
    pub fn new(api_key: Option<String>, base_url: String) -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .build()
            .map_err(Error::FailedToFetch)?;
        Ok(Self {
            client,
            api_key,
            base_url,
        })
    }

    pub async fn chat_complete(
        &self,
        request: &ChatCompletionRequest,
    ) -> Result<ChatCompletion, Error> {
        let api_key = &self.api_key.as_ref().ok_or(Error::NoAPIKey)?;
        let model = &request.model;

        let resp = self
            .client
            .post(self.chat_endpoint())
            .bearer_auth(api_key)
            .timeout(request.timeout)
            .header("Content-Type", "application/json")
            .json(&json!({
                "model": model,
                "messages": request.messages,
                "temperature": request.temperature,
            }))
            .send()
            .await
            .map_err(Error::FailedToFetch)?;

        match resp.status() {
            StatusCode::OK => {
                let res: ChatCompletion = resp.json().await.map_err(Error::FailedToFetch)?;
                Ok(res)
            }
            _ => {
                let error = resp
                    .json::<APIErrorContainer>()
                    .await
                    .map_err(Error::FailedToFetch)?
                    .error;
                Err(Error::OpenAIError { error })
            }
        }
    }

    fn chat_endpoint(&self) -> String {
        format!("{}{}", self.base_url, "/v1/chat/completions")
    }
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

impl ChatCompletionRequest {
    pub fn builder() -> ChatCompletionRequestBuilder {
        ChatCompletionRequestBuilder::default()
    }
}

impl ChatCompletion {
    pub fn builder() -> ChatCompletionBuilder {
        ChatCompletionBuilder::default()
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use anyhow::Result;
    use chrono::TimeZone;

    #[test]
    fn parse_chat_completion_response() -> Result<()> {
        let data = r#"{
             "created": 1688413145,
             "model": "gpt-3.5-turbo-0613",
             "choices": [{
                 "index": 0,
                 "message": {
                     "role": "assistant",
                     "content": "Hello! How can I assist you today?"
                 },
                 "finish_reason": "stop"
             }],
             "usage": {
                 "prompt_tokens": 8,
                 "completion_tokens": 9,
                 "total_tokens": 17
             }
        }
        "#;

        let resp = serde_json::from_str::<ChatCompletion>(data)?;

        assert_eq!(resp.created, Utc.timestamp_opt(1688413145, 0).unwrap());
        assert_eq!(
            resp.choices,
            vec![Choice {
                message: Message {
                    role: Role::Assistant,
                    content: "Hello! How can I assist you today?".to_string()
                },
                finish_reason: FinishReason::Stop
            }]
        );
        assert_eq!(resp.model, "gpt-3.5-turbo-0613");
        assert_eq!(
            resp.usage,
            Usage {
                prompt_tokens: 8,
                completion_tokens: 9,
                total_tokens: 17,
            }
        );

        Ok(())
    }

    #[test]
    fn parse_chat_completion_error() -> Result<()> {
        let data = r#"{
            "error": {
                "message": "An error message",
                "type": "invalid_request_error",
                "param": null,
                "code": null
            }
        }
        "#;

        let resp = serde_json::from_str::<APIErrorContainer>(data)?.error;

        assert_eq!(resp.message, "An error message");
        assert_eq!(resp.error_type, "invalid_request_error");
        assert_eq!(resp.param, None);
        assert_eq!(resp.code, None);

        Ok(())
    }
}

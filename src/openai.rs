//! Interactions with OpenAI APIs

use std::convert::TryFrom;
use std::time::Duration;

use crate::Error;
use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Convienience Client for OpenAI Responses API
pub struct Client {
    /// Inner client
    client: reqwest::Client,
    /// Default API Key
    api_key: Option<String>,
    /// Base URL for API Endpoint
    base_url: String,
}

/// Requests for the Responses API
/// Reference: <https://platform.openai.com/docs/api-reference/responses>
#[derive(Builder, Default)]
pub struct ResponseRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    timeout: Duration,
    #[builder(default)]
    reasoning: Option<Reasoning>,
}

/// Responses from the Responses API
/// Reference: <https://platform.openai.com/docs/api-reference/responses>
#[derive(Builder, Default, Debug, Serialize, Deserialize)]
pub struct Response {
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

/// Messages in Responses API request and response
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
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Reasoning {
    pub effort: ReasoningEffort,
}

impl Reasoning {
    pub fn from_effort(effort: ReasoningEffort) -> Self {
        Self { effort }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
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

    pub async fn create_response(&self, request: &ResponseRequest) -> Result<Response, Error> {
        let api_key = &self.api_key.as_ref().ok_or(Error::NoAPIKey)?;

        let resp = self
            .client
            .post(self.responses_endpoint())
            .bearer_auth(api_key)
            .timeout(request.timeout)
            .header("Content-Type", "application/json")
            .json(&request.to_payload())
            .send()
            .await
            .map_err(Error::FailedToFetch)?;

        match resp.status() {
            StatusCode::OK => {
                let responses: ResponsesAPIResponse =
                    resp.json().await.map_err(Error::FailedToFetch)?;
                let response = Response::try_from(responses).map_err(Error::UnexpectedResponse)?;
                Ok(response)
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

    fn responses_endpoint(&self) -> String {
        format!("{}{}", self.base_url, "/v1/responses")
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

impl Message {
    fn to_responses_input(&self) -> serde_json::Value {
        json!({
            "role": self.role.as_str(),
            "content": [{
                "type": "text",
                "text": self.content.clone(),
            }]
        })
    }
}

impl Role {
    fn as_str(&self) -> &'static str {
        match self {
            Role::System => "system",
            Role::Assistant => "assistant",
            Role::User => "user",
        }
    }
}

impl ResponseRequest {
    pub fn builder() -> ResponseRequestBuilder {
        ResponseRequestBuilder::default()
    }

    fn to_payload(&self) -> Value {
        let input = self
            .messages
            .iter()
            .map(Message::to_responses_input)
            .collect::<Vec<_>>();

        let mut payload = json!({
            "model": self.model,
            "input": input,
            "temperature": self.temperature,
        });

        if let Some(reasoning) = &self.reasoning
            && let Some(obj) = payload.as_object_mut() {
                obj.insert(
                    "reasoning".to_string(),
                    serde_json::to_value(reasoning).expect("failed to serialize reasoning"),
                );
            }

        payload
    }
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
    }
}

#[derive(Debug, Deserialize)]
struct ResponsesAPIResponse {
    #[serde(with = "ts_seconds")]
    created: DateTime<Utc>,
    model: String,
    #[serde(default)]
    output: Vec<ResponseOutput>,
    usage: ResponsesUsage,
}

#[derive(Debug, Deserialize)]
struct ResponsesUsage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ResponseOutput {
    #[serde(rename = "type")]
    item_type: String,
    role: Option<Role>,
    #[serde(default)]
    content: Vec<ResponseContent>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ResponseContent {
    OutputText {
        text: String,
    },
    Text {
        text: String,
    },
    #[serde(other)]
    Other,
}

impl ResponseContent {
    fn as_text(&self) -> Option<&str> {
        match self {
            ResponseContent::OutputText { text } => Some(text),
            ResponseContent::Text { text } => Some(text),
            ResponseContent::Other => None,
        }
    }
}

impl ResponseOutput {
    fn aggregated_text(&self) -> Option<String> {
        let mut aggregated = String::new();

        for part in &self.content {
            if let Some(text) = part.as_text() {
                aggregated.push_str(text);
            }
        }

        if aggregated.is_empty() {
            None
        } else {
            Some(aggregated)
        }
    }
}

impl TryFrom<ResponsesAPIResponse> for Response {
    type Error = String;

    fn try_from(value: ResponsesAPIResponse) -> Result<Self, Self::Error> {
        let mut choices = Vec::new();

        for output in value.output.into_iter() {
            if output.item_type != "message" {
                continue;
            }

            let text = output
                .aggregated_text()
                .ok_or_else(|| "response message missing text content".to_string())?;

            let message = Message {
                role: output.role.unwrap_or(Role::Assistant),
                content: text,
            };

            choices.push(Choice {
                message,
                finish_reason: FinishReason::Stop,
            });
        }

        if choices.is_empty() {
            return Err("response did not contain any assistant messages".to_string());
        }

        Ok(Response {
            created: value.created,
            choices,
            model: value.model,
            usage: Usage {
                input_tokens: value.usage.input_tokens,
                output_tokens: value.usage.output_tokens,
                total_tokens: value.usage.total_tokens,
            },
        })
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use anyhow::Result;
    use chrono::TimeZone;
    use std::time::Duration;

    #[test]
    fn parse_response_payload() -> Result<()> {
        let data = r#"{
             "created": 1688413145,
             "model": "gpt-5",
             "output": [{
                 "id": "msg_123",
                 "type": "message",
                 "role": "assistant",
                 "content": [{
                     "type": "output_text",
                     "text": "Hello! How can I assist you today?"
                 }]
             }],
             "usage": {
                 "input_tokens": 8,
                 "output_tokens": 9,
                 "total_tokens": 17
             }
        }
        "#;

        let resp = serde_json::from_str::<ResponsesAPIResponse>(data)?;
        let resp = Response::try_from(resp).map_err(|e| anyhow::anyhow!(e))?;

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
        assert_eq!(resp.model, "gpt-5");
        assert_eq!(
            resp.usage,
            Usage {
                input_tokens: 8,
                output_tokens: 9,
                total_tokens: 17,
            }
        );

        Ok(())
    }

    #[test]
    fn response_payload_includes_reasoning() -> Result<()> {
        let request = ResponseRequest::builder()
            .model("gpt-5".to_string())
            .messages(vec![Message::user("Hello")])
            .temperature(0.0)
            .timeout(Duration::from_secs(30))
            .reasoning(Some(Reasoning::from_effort(ReasoningEffort::High)))
            .build()
            .expect("request builds");

        let payload = request.to_payload();

        assert_eq!(payload["reasoning"]["effort"], "high");
        assert_eq!(payload["model"], "gpt-5");

        Ok(())
    }

    #[test]
    fn response_payload_omits_reasoning_when_not_set() -> Result<()> {
        let request = ResponseRequest::builder()
            .model("gpt-5".to_string())
            .messages(vec![Message::user("Hello")])
            .temperature(0.0)
            .timeout(Duration::from_secs(30))
            .build()
            .expect("request builds");

        let payload = request.to_payload();

        assert!(payload.get("reasoning").is_none());
        Ok(())
    }

    #[test]
    fn response_try_from_concatenates_segments() -> Result<()> {
        let data = r#"{
             "created": 1688413145,
             "model": "gpt-5",
             "output": [{
                 "id": "msg_123",
                 "type": "message",
                 "role": "assistant",
                 "content": [{
                     "type": "text",
                     "text": "Hello"
                 }, {
                     "type": "output_text",
                     "text": " world"
                 }]
             }],
             "usage": {
                 "input_tokens": 3,
                 "output_tokens": 4,
                 "total_tokens": 7
             }
        }
        "#;

        let resp = serde_json::from_str::<ResponsesAPIResponse>(data)?;
        let resp = Response::try_from(resp).map_err(|e| anyhow::anyhow!(e))?;

        assert_eq!(resp.choices.len(), 1);
        assert_eq!(resp.choices[0].message.content, "Hello world");
        Ok(())
    }

    #[test]
    fn response_try_from_errors_without_assistant_message() -> Result<()> {
        let data = r#"{
             "created": 1688413145,
             "model": "gpt-5",
             "output": [{
                 "id": "reasoning_1",
                 "type": "reasoning",
                 "content": [{
                     "type": "text",
                     "text": "Thinking..."
                 }]
             }],
             "usage": {
                 "input_tokens": 1,
                 "output_tokens": 1,
                 "total_tokens": 2
             }
        }
        "#;

        let resp = serde_json::from_str::<ResponsesAPIResponse>(data)?;
        let err = Response::try_from(resp).expect_err("should error");

        assert!(
            err.contains("did not contain any assistant messages"),
            "unexpected error text: {err}"
        );

        Ok(())
    }

    #[test]
    fn parse_response_error() -> Result<()> {
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

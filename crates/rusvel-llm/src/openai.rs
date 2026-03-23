//! `OpenAI` HTTP adapter implementing [`LlmPort`].

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use rusvel_core::domain::*;
use rusvel_core::error::RusvelError;
use rusvel_core::ports::LlmPort;

// ════════════════════════════════════════════════════════════════════
//  OpenAiProvider
// ════════════════════════════════════════════════════════════════════

/// `OpenAI` API adapter.
///
/// Talks to `https://api.openai.com/v1` (or a custom base URL for
/// Azure `OpenAI` / compatible proxies).
pub struct OpenAiProvider {
    base_url: String,
    api_key: String,
    client: Client,
}

impl OpenAiProvider {
    /// Create a provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, "https://api.openai.com/v1")
    }

    /// Create a provider with a custom base URL.
    pub fn with_base_url(api_key: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            base_url: url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
            client: Client::new(),
        }
    }
}

// ════════════════════════════════════════════════════════════════════
//  LlmPort implementation
// ════════════════════════════════════════════════════════════════════

#[async_trait]
impl LlmPort for OpenAiProvider {
    async fn generate(&self, request: LlmRequest) -> rusvel_core::error::Result<LlmResponse> {
        let oai_req = to_openai_request(&request);
        let url = format!("{}/chat/completions", self.base_url);

        debug!(model = %request.model.model, "openai generate");

        let http_resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&oai_req)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = http_resp.status();
        if !status.is_success() {
            let body = http_resp.text().await.unwrap_or_default();
            return Err(map_openai_http_error(status.as_u16(), &body));
        }

        let oai_resp: OpenAiChatResponse = http_resp.json().await.map_err(map_reqwest_error)?;

        Ok(from_openai_response(oai_resp))
    }

    async fn embed(&self, model: &ModelRef, text: &str) -> rusvel_core::error::Result<Vec<f32>> {
        let url = format!("{}/embeddings", self.base_url);
        let body = serde_json::json!({
            "model": model.model,
            "input": text,
        });

        debug!(model = %model.model, "openai embed");

        let http_resp = self
            .client
            .post(&url)
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = http_resp.status();
        if !status.is_success() {
            let body = http_resp.text().await.unwrap_or_default();
            return Err(map_openai_http_error(status.as_u16(), &body));
        }

        let resp: OpenAiEmbedResponse = http_resp.json().await.map_err(map_reqwest_error)?;

        resp.data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| RusvelError::Llm("OpenAI returned empty embeddings".into()))
    }

    async fn list_models(&self) -> rusvel_core::error::Result<Vec<ModelRef>> {
        let url = format!("{}/models", self.base_url);

        debug!("openai list_models");

        let http_resp = self
            .client
            .get(&url)
            .bearer_auth(&self.api_key)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = http_resp.status();
        if !status.is_success() {
            let body = http_resp.text().await.unwrap_or_default();
            return Err(map_openai_http_error(status.as_u16(), &body));
        }

        let resp: OpenAiModelsResponse = http_resp.json().await.map_err(map_reqwest_error)?;

        Ok(resp
            .data
            .into_iter()
            .map(|m| ModelRef {
                provider: ModelProvider::OpenAI,
                model: m.id,
            })
            .collect())
    }
}

// ════════════════════════════════════════════════════════════════════
//  OpenAI wire types
// ════════════════════════════════════════════════════════════════════

#[derive(Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<serde_json::Value>,
}

#[derive(Default, Serialize, Deserialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAiChatResponse {
    #[serde(default)]
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    #[serde(default)]
    message: OpenAiMessage,
    #[serde(default)]
    finish_reason: Option<String>,
}

#[derive(Default, Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
}

#[derive(Deserialize)]
struct OpenAiEmbedResponse {
    #[serde(default)]
    data: Vec<OpenAiEmbedding>,
}

#[derive(Deserialize)]
struct OpenAiEmbedding {
    #[serde(default)]
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct OpenAiModelsResponse {
    #[serde(default)]
    data: Vec<OpenAiModelInfo>,
}

#[derive(Deserialize)]
struct OpenAiModelInfo {
    id: String,
}

// ════════════════════════════════════════════════════════════════════
//  Mapping helpers
// ════════════════════════════════════════════════════════════════════

fn to_openai_request(req: &LlmRequest) -> OpenAiChatRequest {
    let messages = req
        .messages
        .iter()
        .map(|m| OpenAiMessage {
            role: match m.role {
                LlmRole::System => "system".into(),
                LlmRole::User => "user".into(),
                LlmRole::Assistant => "assistant".into(),
                LlmRole::Tool => "tool".into(),
            },
            content: extract_text(&m.content),
        })
        .collect();

    // Map tool definitions to OpenAI function-calling format.
    let tools: Vec<serde_json::Value> = req
        .tools
        .iter()
        .map(|t| {
            serde_json::json!({
                "type": "function",
                "function": t,
            })
        })
        .collect();

    OpenAiChatRequest {
        model: req.model.model.clone(),
        messages,
        temperature: req.temperature,
        max_tokens: req.max_tokens,
        tools,
    }
}

fn from_openai_response(resp: OpenAiChatResponse) -> LlmResponse {
    let choice = resp.choices.into_iter().next();

    let (text, finish_reason) = match choice {
        Some(c) => {
            let reason = match c.finish_reason.as_deref() {
                Some("stop") => FinishReason::Stop,
                Some("length") => FinishReason::Length,
                Some("tool_calls" | "function_call") => FinishReason::ToolUse,
                Some("content_filter") => FinishReason::ContentFilter,
                Some(other) => FinishReason::Other(other.into()),
                None => FinishReason::Other("unknown".into()),
            };
            (c.message.content, reason)
        }
        None => (String::new(), FinishReason::Other("no_choices".into())),
    };

    let usage = resp.usage.unwrap_or_default();

    LlmResponse {
        content: Content::text(text),
        finish_reason,
        usage: LlmUsage {
            input_tokens: usage.prompt_tokens,
            output_tokens: usage.completion_tokens,
        },
        metadata: serde_json::json!({}),
    }
}

/// Extract concatenated text from all `Part::Text` parts.
fn extract_text(content: &Content) -> String {
    content
        .parts
        .iter()
        .filter_map(|p| match p {
            Part::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

// ════════════════════════════════════════════════════════════════════
//  Error mapping
// ════════════════════════════════════════════════════════════════════

fn map_reqwest_error(e: reqwest::Error) -> RusvelError {
    if e.is_connect() {
        RusvelError::Llm(format!("cannot connect to OpenAI API: {e}"))
    } else if e.is_timeout() {
        RusvelError::Llm(format!("OpenAI request timed out: {e}"))
    } else {
        RusvelError::Llm(format!("OpenAI HTTP error: {e}"))
    }
}

fn map_openai_http_error(status: u16, body: &str) -> RusvelError {
    match status {
        401 => RusvelError::Unauthorized("invalid or missing OpenAI API key".into()),
        404 => RusvelError::NotFound {
            kind: "model".into(),
            id: body.to_string(),
        },
        429 => RusvelError::Llm("OpenAI rate limit exceeded — retry later".into()),
        _ => RusvelError::Llm(format!("OpenAI returned HTTP {status}: {body}")),
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> LlmRequest {
        LlmRequest {
            model: ModelRef {
                provider: ModelProvider::OpenAI,
                model: "gpt-4o".into(),
            },
            messages: vec![
                LlmMessage {
                    role: LlmRole::System,
                    content: Content::text("You are helpful."),
                },
                LlmMessage {
                    role: LlmRole::User,
                    content: Content::text("Hello!"),
                },
            ],
            tools: vec![],
            temperature: Some(0.7),
            max_tokens: Some(512),
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn to_openai_request_maps_roles() {
        let req = sample_request();
        let wire = to_openai_request(&req);
        assert_eq!(wire.model, "gpt-4o");
        assert_eq!(wire.messages.len(), 2);
        assert_eq!(wire.messages[0].role, "system");
        assert_eq!(wire.messages[1].role, "user");
    }

    #[test]
    fn to_openai_request_wraps_tools() {
        let mut req = sample_request();
        req.tools = vec![serde_json::json!({
            "name": "get_weather",
            "parameters": {"type": "object"}
        })];
        let wire = to_openai_request(&req);
        assert_eq!(wire.tools.len(), 1);
        assert_eq!(wire.tools[0]["type"], "function");
    }

    #[test]
    fn from_openai_response_maps_stop() {
        let resp = OpenAiChatResponse {
            choices: vec![OpenAiChoice {
                message: OpenAiMessage {
                    role: "assistant".into(),
                    content: "Hi!".into(),
                },
                finish_reason: Some("stop".into()),
            }],
            usage: Some(OpenAiUsage {
                prompt_tokens: 10,
                completion_tokens: 3,
            }),
        };
        let llm_resp = from_openai_response(resp);
        assert_eq!(llm_resp.finish_reason, FinishReason::Stop);
        assert_eq!(llm_resp.usage.input_tokens, 10);
        assert_eq!(llm_resp.usage.output_tokens, 3);
    }

    #[test]
    fn from_openai_response_empty_choices() {
        let resp = OpenAiChatResponse {
            choices: vec![],
            usage: None,
        };
        let llm_resp = from_openai_response(resp);
        assert_eq!(
            llm_resp.finish_reason,
            FinishReason::Other("no_choices".into())
        );
    }

    #[test]
    fn map_openai_http_error_401() {
        let err = map_openai_http_error(401, "{}");
        assert!(matches!(err, RusvelError::Unauthorized(_)));
    }

    #[test]
    fn map_openai_http_error_429() {
        let err = map_openai_http_error(429, "{}");
        match err {
            RusvelError::Llm(msg) => assert!(msg.contains("rate limit")),
            other => panic!("expected Llm, got: {other}"),
        }
    }

    #[test]
    fn embed_response_deserialize() {
        let json =
            r#"{"data":[{"embedding":[0.1,0.2,0.3],"index":0}],"model":"text-embedding-3-small"}"#;
        let resp: OpenAiEmbedResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 1);
        assert_eq!(resp.data[0].embedding, vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn models_response_deserialize() {
        let json = r#"{"data":[{"id":"gpt-4o"},{"id":"gpt-4o-mini"}]}"#;
        let resp: OpenAiModelsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.data.len(), 2);
        assert_eq!(resp.data[0].id, "gpt-4o");
    }
}

//! Claude (Anthropic) HTTP adapter implementing [`LlmPort`].

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use rusvel_core::domain::*;
use rusvel_core::error::RusvelError;
use rusvel_core::ports::LlmPort;

// ════════════════════════════════════════════════════════════════════
//  ClaudeProvider
// ════════════════════════════════════════════════════════════════════

/// Anthropic Claude API adapter.
///
/// Talks to `https://api.anthropic.com/v1` (or a custom base URL).
pub struct ClaudeProvider {
    base_url: String,
    api_key: String,
    client: Client,
}

impl ClaudeProvider {
    /// Create a provider with the given API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self::with_base_url(api_key, "https://api.anthropic.com/v1")
    }

    /// Create a provider with a custom base URL (e.g. for proxies).
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
impl LlmPort for ClaudeProvider {
    async fn generate(&self, request: LlmRequest) -> rusvel_core::error::Result<LlmResponse> {
        let claude_req = to_claude_request(&request);
        let url = format!("{}/messages", self.base_url);

        debug!(model = %request.model.model, "claude generate");

        let http_resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&claude_req)
            .send()
            .await
            .map_err(map_reqwest_error)?;

        let status = http_resp.status();
        if !status.is_success() {
            let body = http_resp.text().await.unwrap_or_default();
            return Err(map_claude_http_error(status.as_u16(), &body));
        }

        let claude_resp: ClaudeResponse =
            http_resp.json().await.map_err(map_reqwest_error)?;

        Ok(from_claude_response(claude_resp))
    }

    async fn embed(
        &self,
        _model: &ModelRef,
        _text: &str,
    ) -> rusvel_core::error::Result<Vec<f32>> {
        Err(RusvelError::Llm(
            "Claude does not support embeddings — use an embedding-capable provider".into(),
        ))
    }

    async fn list_models(&self) -> rusvel_core::error::Result<Vec<ModelRef>> {
        Ok(vec![
            model_ref("claude-sonnet-4-20250514"),
            model_ref("claude-haiku-4-20250414"),
            model_ref("claude-opus-4-20250514"),
            model_ref("claude-3-5-sonnet-20241022"),
            model_ref("claude-3-5-haiku-20241022"),
        ])
    }
}

fn model_ref(name: &str) -> ModelRef {
    ModelRef {
        provider: ModelProvider::Claude,
        model: name.into(),
    }
}

// ════════════════════════════════════════════════════════════════════
//  Claude wire types
// ════════════════════════════════════════════════════════════════════

#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct ClaudeMessage {
    role: String,
    content: serde_json::Value,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    #[serde(default)]
    content: Vec<ClaudeContentBlock>,
    #[serde(default)]
    stop_reason: Option<String>,
    #[serde(default)]
    usage: ClaudeUsage,
}

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClaudeContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },
}

#[derive(Default, Deserialize)]
struct ClaudeUsage {
    #[serde(default)]
    input_tokens: u32,
    #[serde(default)]
    output_tokens: u32,
}

// ════════════════════════════════════════════════════════════════════
//  Mapping helpers
// ════════════════════════════════════════════════════════════════════

fn to_claude_request(req: &LlmRequest) -> ClaudeRequest {
    let mut system: Option<String> = None;
    let mut messages = Vec::new();

    for m in &req.messages {
        match m.role {
            LlmRole::System => {
                system = Some(extract_text(&m.content));
            }
            LlmRole::User => messages.push(ClaudeMessage {
                role: "user".into(),
                content: serde_json::Value::String(extract_text(&m.content)),
            }),
            LlmRole::Assistant => messages.push(ClaudeMessage {
                role: "assistant".into(),
                content: serde_json::Value::String(extract_text(&m.content)),
            }),
            LlmRole::Tool => messages.push(ClaudeMessage {
                role: "user".into(),
                content: serde_json::json!([{
                    "type": "tool_result",
                    "tool_use_id": "unknown",
                    "content": extract_text(&m.content),
                }]),
            }),
        }
    }

    ClaudeRequest {
        model: req.model.model.clone(),
        messages,
        max_tokens: req.max_tokens.unwrap_or(4096),
        system,
        temperature: req.temperature,
        tools: req.tools.clone(),
    }
}

fn from_claude_response(resp: ClaudeResponse) -> LlmResponse {
    let mut parts = Vec::new();

    for block in &resp.content {
        match block {
            ClaudeContentBlock::Text { text } => {
                parts.push(Part::Text(text.clone()));
            }
            ClaudeContentBlock::ToolUse { id, name, input } => {
                let call = serde_json::json!({
                    "type": "tool_use",
                    "id": id,
                    "name": name,
                    "input": input,
                });
                parts.push(Part::Text(call.to_string()));
            }
        }
    }

    let finish_reason = match resp.stop_reason.as_deref() {
        Some("end_turn") | Some("stop") => FinishReason::Stop,
        Some("max_tokens") => FinishReason::Length,
        Some("tool_use") => FinishReason::ToolUse,
        Some(other) => FinishReason::Other(other.into()),
        None => FinishReason::Other("unknown".into()),
    };

    LlmResponse {
        content: Content { parts },
        finish_reason,
        usage: LlmUsage {
            input_tokens: resp.usage.input_tokens,
            output_tokens: resp.usage.output_tokens,
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
        RusvelError::Llm(format!("cannot connect to Claude API: {e}"))
    } else if e.is_timeout() {
        RusvelError::Llm(format!("Claude request timed out: {e}"))
    } else {
        RusvelError::Llm(format!("Claude HTTP error: {e}"))
    }
}

fn map_claude_http_error(status: u16, body: &str) -> RusvelError {
    match status {
        401 => RusvelError::Unauthorized("invalid or missing Claude API key".into()),
        404 => RusvelError::NotFound {
            kind: "model".into(),
            id: body.to_string(),
        },
        429 => RusvelError::Llm("Claude rate limit exceeded — retry later".into()),
        529 => RusvelError::Llm("Claude API overloaded — retry later".into()),
        _ => RusvelError::Llm(format!("Claude returned HTTP {status}: {body}")),
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
                provider: ModelProvider::Claude,
                model: "claude-sonnet-4-20250514".into(),
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
            max_tokens: Some(1024),
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn to_claude_request_extracts_system() {
        let req = sample_request();
        let wire = to_claude_request(&req);
        assert_eq!(wire.system.as_deref(), Some("You are helpful."));
        // System message should NOT appear in the messages array
        assert_eq!(wire.messages.len(), 1);
        assert_eq!(wire.messages[0].role, "user");
    }

    #[test]
    fn to_claude_request_sets_max_tokens() {
        let req = sample_request();
        let wire = to_claude_request(&req);
        assert_eq!(wire.max_tokens, 1024);
    }

    #[test]
    fn to_claude_request_default_max_tokens() {
        let mut req = sample_request();
        req.max_tokens = None;
        let wire = to_claude_request(&req);
        assert_eq!(wire.max_tokens, 4096);
    }

    #[test]
    fn from_claude_response_text() {
        let resp = ClaudeResponse {
            content: vec![ClaudeContentBlock::Text {
                text: "Hi there!".into(),
            }],
            stop_reason: Some("end_turn".into()),
            usage: ClaudeUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
        };
        let llm_resp = from_claude_response(resp);
        assert_eq!(llm_resp.finish_reason, FinishReason::Stop);
        assert_eq!(llm_resp.usage.input_tokens, 10);
        match &llm_resp.content.parts[0] {
            Part::Text(t) => assert_eq!(t, "Hi there!"),
            _ => panic!("expected text"),
        }
    }

    #[test]
    fn from_claude_response_tool_use() {
        let resp = ClaudeResponse {
            content: vec![ClaudeContentBlock::ToolUse {
                id: "call_1".into(),
                name: "get_weather".into(),
                input: serde_json::json!({"city": "London"}),
            }],
            stop_reason: Some("tool_use".into()),
            usage: ClaudeUsage::default(),
        };
        let llm_resp = from_claude_response(resp);
        assert_eq!(llm_resp.finish_reason, FinishReason::ToolUse);
    }

    #[test]
    fn map_claude_http_error_401() {
        let err = map_claude_http_error(401, "{}");
        assert!(matches!(err, RusvelError::Unauthorized(_)));
    }

    #[test]
    fn map_claude_http_error_429() {
        let err = map_claude_http_error(429, "{}");
        match err {
            RusvelError::Llm(msg) => assert!(msg.contains("rate limit")),
            other => panic!("expected Llm, got: {other}"),
        }
    }

    #[test]
    fn list_models_returns_known_models() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let provider = ClaudeProvider::new("test-key");
        let models = rt.block_on(provider.list_models()).unwrap();
        assert!(models.len() >= 3);
        assert!(models.iter().all(|m| m.provider == ModelProvider::Claude));
    }
}

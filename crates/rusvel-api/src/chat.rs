//! Chat API handler — streaming conversation with the god agent.
//!
//! `POST /api/chat` accepts a message and streams the response via SSE.
//! Conversation history is persisted in `ObjectStore`.

use std::convert::Infallible;
use std::pin::Pin;
use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, KeepAliveStream, Sse};
use chrono::Utc;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use rusvel_agent::AgentEvent;
use rusvel_core::domain::{AgentConfig, Content, ModelProvider, ModelRef};
use rusvel_core::id::SessionId;
use rusvel_core::ports::{AgentPort, StoragePort};

use crate::AppState;
use crate::config::ChatConfig;

// ── Request / Response types ─────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub conversation_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub conversation_id: String,
    pub role: String, // "user" | "assistant" | "system"
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ConversationSummary {
    pub id: String,
    pub title: String,
    pub updated_at: String,
    pub message_count: usize,
}

// ── Handlers ─────────────────────────────────────────────────

/// `POST /api/chat` — stream a response via SSE.
pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ChatRequest>,
) -> Result<
    Sse<KeepAliveStream<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>>,
    (StatusCode, String),
> {
    let profile = state.profile.as_ref().ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        "no profile loaded".into(),
    ))?;

    let conversation_id = body
        .conversation_id
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());

    // Load conversation history (last 50 messages)
    let history = load_history(&state.storage, &conversation_id, 50)
        .await
        .unwrap_or_default();

    // Store the user message
    let user_msg = ChatMessage {
        id: uuid::Uuid::now_v7().to_string(),
        conversation_id: conversation_id.clone(),
        role: "user".into(),
        content: body.message.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    let _ = store_message(&state.storage, &user_msg).await;

    // Load chat config (model, effort, tools, etc.)
    let chat_config: ChatConfig = state
        .storage
        .objects()
        .get("chat_config", "current")
        .await
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    // Build AgentConfig from chat config + profile
    let system_prompt = profile.to_system_prompt();
    let model_ref = parse_model_ref(&chat_config.model);
    let agent_config = AgentConfig {
        profile_id: None,
        session_id: SessionId::new(),
        model: Some(model_ref),
        tools: chat_config.allowed_tools.clone(),
        instructions: Some(system_prompt),
        budget_limit: chat_config.max_budget_usd,
        metadata: serde_json::json!({}),
    };

    // Build user input with conversation history context
    let mut user_input = String::new();
    for msg in &history {
        match msg.role.as_str() {
            "user" => user_input.push_str(&format!("User: {}\n\n", msg.content)),
            "assistant" => user_input.push_str(&format!("Assistant: {}\n\n", msg.content)),
            _ => {}
        }
    }
    user_input.push_str(&body.message);

    // Create agent run
    let run_id = state
        .agent_runtime
        .create(agent_config)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Stream via AgentRuntime
    let (tx, rx) = tokio::sync::mpsc::channel::<AgentEvent>(32);
    let runtime = state.agent_runtime.clone();
    let run_id_owned = run_id;
    let input = Content::text(&user_input);
    tokio::spawn(async move {
        let _ = runtime.run_streaming(&run_id_owned, input, tx).await;
    });

    let storage = state.storage.clone();
    let conv_id = conversation_id.clone();

    let stream: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> =
        Box::pin(ReceiverStream::new(rx).map(move |event| {
            let sse_event = match event {
                AgentEvent::TextDelta { text } => Event::default()
                    .event("delta")
                    .data(
                        serde_json::json!({"text": text, "conversation_id": conv_id}).to_string(),
                    ),
                AgentEvent::ToolCallStart { id, name, args } => {
                    Event::default().event("tool_call").data(
                        serde_json::json!({
                            "id": id,
                            "name": name,
                            "args": args,
                            "conversation_id": conv_id,
                        })
                        .to_string(),
                    )
                }
                AgentEvent::ToolCallEnd {
                    id,
                    name,
                    result,
                    is_error,
                } => Event::default().event("tool_result").data(
                    serde_json::json!({
                        "id": id,
                        "name": name,
                        "result": result,
                        "is_error": is_error,
                        "conversation_id": conv_id,
                    })
                    .to_string(),
                ),
                AgentEvent::Done { output } => {
                    let full_text: String = output
                        .content
                        .parts
                        .iter()
                        .filter_map(|p| match p {
                            rusvel_core::domain::Part::Text(t) => Some(t.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    let cost = output.cost_estimate;

                    // Store assistant message (fire and forget)
                    let storage = storage.clone();
                    let conv_id_inner = conv_id.clone();
                    let text = full_text.clone();
                    tokio::spawn(async move {
                        let msg = ChatMessage {
                            id: uuid::Uuid::now_v7().to_string(),
                            conversation_id: conv_id_inner,
                            role: "assistant".into(),
                            content: text,
                            created_at: Utc::now().to_rfc3339(),
                        };
                        let _ = store_message(&storage, &msg).await;
                    });

                    Event::default().event("done").data(
                        serde_json::json!({
                            "text": full_text,
                            "cost_usd": cost,
                            "conversation_id": conv_id
                        })
                        .to_string(),
                    )
                }
                AgentEvent::Error { message } => Event::default()
                    .event("error")
                    .data(serde_json::json!({"message": message}).to_string()),
            };
            Ok(sse_event)
        }));

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

/// Parse a model string into a ModelRef.
fn parse_model_ref(model: &str) -> ModelRef {
    if let Some((provider, name)) = model.split_once('/') {
        let provider = match provider {
            "ollama" => ModelProvider::Ollama,
            "openai" => ModelProvider::OpenAI,
            "claude" => ModelProvider::Claude,
            "gemini" => ModelProvider::Gemini,
            other => ModelProvider::Other(other.into()),
        };
        ModelRef {
            provider,
            model: name.into(),
        }
    } else {
        ModelRef {
            provider: ModelProvider::Claude,
            model: model.into(),
        }
    }
}

/// `GET /api/chat/conversations` — list all conversations.
pub async fn list_conversations(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ConversationSummary>>, (StatusCode, String)> {
    let all = state
        .storage
        .objects()
        .list("chat_message", rusvel_core::domain::ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Group by conversation_id
    let mut convos: std::collections::HashMap<String, Vec<ChatMessage>> =
        std::collections::HashMap::new();
    for val in all {
        if let Ok(msg) = serde_json::from_value::<ChatMessage>(val) {
            convos
                .entry(msg.conversation_id.clone())
                .or_default()
                .push(msg);
        }
    }

    let mut summaries: Vec<ConversationSummary> = convos
        .into_iter()
        .map(|(id, mut msgs)| {
            msgs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            let title = msgs
                .iter()
                .find(|m| m.role == "user")
                .map(|m| {
                    if m.content.len() > 60 {
                        format!("{}...", &m.content[..57])
                    } else {
                        m.content.clone()
                    }
                })
                .unwrap_or_else(|| "New conversation".into());
            let updated_at = msgs
                .last()
                .map(|m| m.created_at.clone())
                .unwrap_or_default();
            ConversationSummary {
                id,
                title,
                updated_at,
                message_count: msgs.len(),
            }
        })
        .collect();

    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(Json(summaries))
}

/// `GET /api/chat/conversations/{id}` — get message history.
pub async fn get_history(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Vec<ChatMessage>>, (StatusCode, String)> {
    let msgs = load_history(&state.storage, &id, 200)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(msgs))
}

// ── Internal helpers ─────────────────────────────────────────

async fn load_history(
    storage: &Arc<dyn StoragePort>,
    conversation_id: &str,
    limit: usize,
) -> rusvel_core::error::Result<Vec<ChatMessage>> {
    let all = storage
        .objects()
        .list("chat_message", rusvel_core::domain::ObjectFilter::default())
        .await?;

    let mut msgs: Vec<ChatMessage> = all
        .into_iter()
        .filter_map(|v| serde_json::from_value::<ChatMessage>(v).ok())
        .filter(|m| m.conversation_id == conversation_id)
        .collect();

    msgs.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    if msgs.len() > limit {
        msgs = msgs[msgs.len() - limit..].to_vec();
    }

    Ok(msgs)
}

async fn store_message(
    storage: &Arc<dyn StoragePort>,
    msg: &ChatMessage,
) -> rusvel_core::error::Result<()> {
    storage
        .objects()
        .put("chat_message", &msg.id, serde_json::to_value(msg)?)
        .await
}

//! Department infrastructure — registry-driven config, chat, and events.
//!
//! All departments share the same 6 parameterized handlers.
//! Department definitions come from the `DepartmentRegistry`.
//! Config uses three-layer cascade: global → department → session.

use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, KeepAliveStream, Sse};
use axum::Json;
use chrono::Utc;
use std::pin::Pin;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use rusvel_core::config::{LayeredConfig, ResolvedConfig};
use rusvel_core::domain::{EventFilter, UserProfile};
use rusvel_core::id::EventId;
use rusvel_core::ports::{EmbeddingPort, StoragePort, VectorStorePort};
use rusvel_core::registry::DepartmentDef;
use rusvel_llm::stream::{ClaudeCliStreamer, StreamEvent};

use crate::chat::{ChatMessage, ChatRequest, ConversationSummary};
use crate::AppState;

// ── Department Config (stored per-dept as LayeredConfig) ─────

const CONFIG_STORE_KEY: &str = "dept_config";

fn msg_namespace(engine: &str) -> String {
    format!("dept_msg_{engine}")
}

async fn load_dept_config(
    engine: &str,
    state: &Arc<AppState>,
) -> LayeredConfig {
    state.storage.objects()
        .get(CONFIG_STORE_KEY, engine)
        .await
        .ok()
        .flatten()
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// Resolve a department's effective config: registry defaults + stored overrides + user context.
fn resolve_dept_config(
    dept_def: &DepartmentDef,
    stored: &LayeredConfig,
    profile: Option<&UserProfile>,
) -> ResolvedConfig {
    // Start with registry defaults
    let mut base = dept_def.default_config.clone();

    // Set system prompt from registry + user context
    let user_context = profile.map(rusvel_core::UserProfile::to_system_prompt).unwrap_or_default();
    if base.system_prompt.is_none() {
        base.system_prompt = Some(format!("{}\n\n{user_context}", dept_def.system_prompt));
    }

    // Stored overrides on top of registry defaults
    let merged = stored.overlay(&base);
    merged.resolve()
}

// ── Legacy DepartmentConfig (for backward-compatible JSON responses) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentConfig {
    pub engine: String,
    pub model: String,
    pub effort: String,
    pub max_budget_usd: Option<f64>,
    pub permission_mode: String,
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub system_prompt: String,
    pub add_dirs: Vec<String>,
    pub max_turns: Option<u32>,
}

impl From<(&str, ResolvedConfig)> for DepartmentConfig {
    fn from((engine, r): (&str, ResolvedConfig)) -> Self {
        Self {
            engine: engine.into(),
            model: r.model,
            effort: r.effort,
            max_budget_usd: r.max_budget_usd,
            permission_mode: r.permission_mode,
            allowed_tools: r.allowed_tools,
            disallowed_tools: r.disallowed_tools,
            system_prompt: r.system_prompt,
            add_dirs: r.add_dirs,
            max_turns: r.max_turns,
        }
    }
}

impl DepartmentConfig {
    /// Convert incoming `DepartmentConfig` update into a `LayeredConfig` for storage.
    fn to_layered(&self) -> LayeredConfig {
        LayeredConfig {
            model: Some(self.model.clone()),
            effort: Some(self.effort.clone()),
            max_budget_usd: self.max_budget_usd,
            permission_mode: Some(self.permission_mode.clone()),
            allowed_tools: Some(self.allowed_tools.clone()),
            disallowed_tools: Some(self.disallowed_tools.clone()),
            system_prompt: Some(self.system_prompt.clone()),
            add_dirs: Some(self.add_dirs.clone()),
            max_turns: self.max_turns,
        }
    }
}

// ── Validate dept param against registry ─────────────────────

fn validate_dept<'a>(
    state: &'a Arc<AppState>,
    dept: &str,
) -> Result<&'a DepartmentDef, (StatusCode, String)> {
    state.registry.get(dept).ok_or_else(|| {
        (StatusCode::NOT_FOUND, format!("Unknown department: {dept}"))
    })
}

// ── Registry endpoint ────────────────────────────────────────

pub async fn list_departments(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<serde_json::Value>> {
    let depts: Vec<serde_json::Value> = state.registry.list().iter()
        .map(|d| serde_json::to_value(d).unwrap_or_default())
        .collect();
    Json(depts)
}

// ── Profile endpoints ────────────────────────────────────────

pub async fn get_profile(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    let profile = state.profile.as_ref()
        .map(|p| serde_json::to_value(p).unwrap_or_default())
        .unwrap_or(serde_json::json!(null));
    Json(profile)
}

pub async fn update_profile(
    State(state): State<Arc<AppState>>,
    Json(profile): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Store profile in ObjectStore for now
    state.storage.objects()
        .put("user_profile", "current", profile.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(profile))
}

// ── Parameterized department handlers ────────────────────────

pub async fn dept_config_get(
    State(state): State<Arc<AppState>>,
    Path(dept): Path<String>,
) -> Result<Json<DepartmentConfig>, (StatusCode, String)> {
    let dept_def = validate_dept(&state, &dept)?;
    let stored = load_dept_config(&dept, &state).await;
    let resolved = resolve_dept_config(dept_def, &stored, state.profile.as_ref());
    Ok(Json(DepartmentConfig::from((dept.as_str(), resolved))))
}

pub async fn dept_config_update(
    State(state): State<Arc<AppState>>,
    Path(dept): Path<String>,
    Json(config): Json<DepartmentConfig>,
) -> Result<Json<DepartmentConfig>, (StatusCode, String)> {
    validate_dept(&state, &dept)?;
    let layered = config.to_layered();
    let value = serde_json::to_value(&layered)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.storage.objects()
        .put(CONFIG_STORE_KEY, &dept, value)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(config))
}

pub async fn dept_chat(
    State(state): State<Arc<AppState>>,
    Path(dept): Path<String>,
    Json(body): Json<ChatRequest>,
) -> Result<
    Sse<KeepAliveStream<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>>,
    (StatusCode, String),
> {
    let dept_def = validate_dept(&state, &dept)?;
    let engine_kind = dept_def.engine_kind;
    let stored = load_dept_config(&dept, &state).await;
    let mut resolved = resolve_dept_config(dept_def, &stored, state.profile.as_ref());
    let namespace = msg_namespace(&dept);

    let conversation_id = body.conversation_id
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());

    // Load history
    let history = load_namespaced_history(&state.storage, &namespace, &conversation_id, 50)
        .await
        .unwrap_or_default();

    // Store user message
    let user_msg = ChatMessage {
        id: uuid::Uuid::now_v7().to_string(),
        conversation_id: conversation_id.clone(),
        role: "user".into(),
        content: body.message.clone(),
        created_at: Utc::now().to_rfc3339(),
    };
    let _ = store_namespaced_message(&state.storage, &namespace, &user_msg).await;

    // !build interceptor
    if let Some(build_cmd) = crate::build_cmd::parse_build_command(&body.message) {
        let storage = state.storage.clone();
        let engine_owned = dept.clone();
        let conv_id = conversation_id.clone();
        let ns = namespace.clone();

        let (tx, rx) = tokio::sync::mpsc::channel::<Event>(8);
        tokio::spawn(async move {
            let _ = tx.send(Event::default()
                .event("delta")
                .data(serde_json::json!({
                    "text": format!("Building {}...\n\n", build_cmd.entity_type.label()),
                    "conversation_id": conv_id,
                }).to_string())
            ).await;

            let result = crate::build_cmd::execute_build(&build_cmd, &engine_owned, &storage).await;
            let response_text = match result {
                Ok(confirmation) => confirmation,
                Err(e) => format!("**Build failed:** {e}"),
            };

            let assistant_msg = ChatMessage {
                id: uuid::Uuid::now_v7().to_string(),
                conversation_id: conv_id.clone(),
                role: "assistant".into(),
                content: response_text.clone(),
                created_at: Utc::now().to_rfc3339(),
            };
            let _ = store_namespaced_message(&storage, &ns, &assistant_msg).await;
            let _ = tx.send(Event::default()
                .event("done")
                .data(serde_json::json!({
                    "text": response_text,
                    "cost_usd": 0.0,
                    "conversation_id": conv_id,
                }).to_string())
            ).await;
        });

        let stream: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> =
            Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)
                .map(Ok::<_, Infallible>));
        return Ok(Sse::new(stream).keep_alive(KeepAlive::default()));
    }

    // /skill-name interceptor
    let effective_message = if let Some(expanded) = crate::skills::resolve_skill(&state, &dept, &body.message).await {
        expanded
    } else {
        body.message.clone()
    };

    // @agent-name mention override
    if let Some(agent_name) = extract_agent_mention(&body.message)
        && let Ok(agents) = state.storage.objects()
            .list("agents", rusvel_core::domain::ObjectFilter::default()).await
        {
            let found = agents.into_iter()
                .filter_map(|v| serde_json::from_value::<rusvel_core::domain::AgentProfile>(v).ok())
                .find(|a| a.name.eq_ignore_ascii_case(&agent_name));
            if let Some(agent) = found {
                resolved.system_prompt = agent.instructions.clone();
                resolved.model = agent.default_model.model.clone();
                if !agent.allowed_tools.is_empty() {
                    resolved.allowed_tools = agent.allowed_tools.clone();
                }
            }
        }

    // Load enabled rules and append to system prompt
    let rules = crate::rules::load_rules_for_engine(&state, &dept).await;
    if !rules.is_empty() {
        resolved.system_prompt.push_str("\n\n--- Rules ---\n");
        for rule in &rules {
            resolved.system_prompt.push_str(&format!("[{}]: {}\n", rule.name, rule.content));
        }
    }

    // RAG: retrieve relevant knowledge
    if let (Some(embed_port), Some(vector_store)) = (&state.embedding, &state.vector_store)
        && let Ok(query_emb) = embed_port.embed_one(&body.message).await {
            let results = vector_store.search(&query_emb, 5).await.unwrap_or_default();
            if !results.is_empty() {
                resolved.system_prompt.push_str("\n\n--- Relevant Knowledge ---\n");
                for r in &results {
                    resolved.system_prompt.push_str(&format!("[score: {:.2}] {}\n", r.score, r.entry.content));
                }
            }
        }

    // Build prompt
    let prompt = build_dept_prompt(&resolved.system_prompt, &history, &effective_message);

    // Load MCP server config
    let mcp_config = crate::mcp_servers::build_mcp_config_for_engine(&state, &dept).await;
    let mut cli_args = resolved.to_claude_args();
    if let Some(ref mcp_json) = mcp_config {
        cli_args.push("--mcp-config".into());
        cli_args.push(mcp_json.clone());
    }

    // Stream
    let streamer = ClaudeCliStreamer::new();
    let rx = streamer.stream_with_args(&prompt, &cli_args);

    let storage = state.storage.clone();
    let events_port = state.events.clone();
    let conv_id = conversation_id.clone();
    let ns = namespace.clone();

    let stream: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> =
        Box::pin(ReceiverStream::new(rx).map(move |event| {
            let sse_event = match &event {
                StreamEvent::Delta { text } => Event::default()
                    .event("delta")
                    .data(serde_json::json!({"text": text, "conversation_id": conv_id}).to_string()),
                StreamEvent::Done { full_text, cost_usd } => {
                    let storage = storage.clone();
                    let events_port = events_port.clone();
                    let conv_id_inner = conv_id.clone();
                    let ns_inner = ns.clone();
                    let text = full_text.clone();
                    let eng = engine_kind;
                    let cost = *cost_usd;
                    tokio::spawn(async move {
                        let msg = ChatMessage {
                            id: uuid::Uuid::now_v7().to_string(),
                            conversation_id: conv_id_inner.clone(),
                            role: "assistant".into(),
                            content: text,
                            created_at: Utc::now().to_rfc3339(),
                        };
                        let _ = store_namespaced_message(&storage, &ns_inner, &msg).await;
                        let _ = events_port.emit(rusvel_core::domain::Event {
                            id: EventId::new(),
                            session_id: None,
                            run_id: None,
                            source: eng,
                            kind: format!("{eng}.chat.completed"),
                            payload: serde_json::json!({
                                "conversation_id": conv_id_inner,
                                "cost_usd": cost,
                                "response_length": msg.content.len(),
                            }),
                            created_at: Utc::now(),
                            metadata: serde_json::json!({}),
                        }).await;
                        crate::hook_dispatch::dispatch_hooks(
                            &format!("{eng}.chat.completed"),
                            serde_json::json!({
                                "conversation_id": conv_id_inner,
                                "cost_usd": cost,
                                "response_length": msg.content.len(),
                            }),
                            &eng.to_string(),
                            storage.clone(),
                        );
                    });
                    Event::default().event("done").data(
                        serde_json::json!({
                            "text": full_text,
                            "cost_usd": cost_usd,
                            "conversation_id": conv_id
                        }).to_string(),
                    )
                }
                StreamEvent::Error { message } => Event::default()
                    .event("error")
                    .data(serde_json::json!({"message": message}).to_string()),
            };
            Ok(sse_event)
        }));

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub async fn dept_conversations(
    State(state): State<Arc<AppState>>,
    Path(dept): Path<String>,
) -> Result<Json<Vec<ConversationSummary>>, (StatusCode, String)> {
    validate_dept(&state, &dept)?;
    let namespace = msg_namespace(&dept);
    let all = state.storage.objects()
        .list(&namespace, rusvel_core::domain::ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut convos: std::collections::HashMap<String, Vec<ChatMessage>> =
        std::collections::HashMap::new();
    for val in all {
        if let Ok(msg) = serde_json::from_value::<ChatMessage>(val) {
            convos.entry(msg.conversation_id.clone()).or_default().push(msg);
        }
    }

    let mut summaries: Vec<ConversationSummary> = convos
        .into_iter()
        .map(|(id, mut msgs)| {
            msgs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            let title = msgs.iter()
                .find(|m| m.role == "user").map_or_else(|| "New conversation".into(), |m| if m.content.len() > 60 { format!("{}...", &m.content[..57]) } else { m.content.clone() });
            let updated_at = msgs.last().map(|m| m.created_at.clone()).unwrap_or_default();
            ConversationSummary { id, title, updated_at, message_count: msgs.len() }
        })
        .collect();
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(Json(summaries))
}

pub async fn dept_history(
    State(state): State<Arc<AppState>>,
    Path((dept, id)): Path<(String, String)>,
) -> Result<Json<Vec<ChatMessage>>, (StatusCode, String)> {
    validate_dept(&state, &dept)?;
    let namespace = msg_namespace(&dept);
    load_namespaced_history(&state.storage, &namespace, &id, 200)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn dept_events(
    State(state): State<Arc<AppState>>,
    Path(dept): Path<String>,
) -> Result<Json<Vec<rusvel_core::domain::Event>>, (StatusCode, String)> {
    let dept_def = validate_dept(&state, &dept)?;
    state.events.query(EventFilter {
        source: Some(dept_def.engine_kind),
        limit: Some(50),
        ..Default::default()
    }).await.map(Json).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

// ── Helpers ──────────────────────────────────────────────────

fn build_dept_prompt(system_prompt: &str, history: &[ChatMessage], user_message: &str) -> String {
    let mut parts = Vec::new();
    parts.push(format!("<system>\n{system_prompt}\n</system>"));
    for msg in history {
        match msg.role.as_str() {
            "user" => parts.push(msg.content.clone()),
            "assistant" => parts.push(format!("<assistant>\n{}\n</assistant>", msg.content)),
            _ => {}
        }
    }
    parts.push(user_message.to_string());
    parts.join("\n\n")
}

async fn load_namespaced_history(
    storage: &Arc<dyn StoragePort>,
    namespace: &str,
    conversation_id: &str,
    limit: usize,
) -> rusvel_core::error::Result<Vec<ChatMessage>> {
    let all = storage.objects()
        .list(namespace, rusvel_core::domain::ObjectFilter::default())
        .await?;
    let mut msgs: Vec<ChatMessage> = all.into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .filter(|m: &ChatMessage| m.conversation_id == conversation_id)
        .collect();
    msgs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    if msgs.len() > limit {
        msgs = msgs[msgs.len() - limit..].to_vec();
    }
    Ok(msgs)
}

async fn store_namespaced_message(
    storage: &Arc<dyn StoragePort>,
    namespace: &str,
    msg: &ChatMessage,
) -> rusvel_core::error::Result<()> {
    storage.objects()
        .put(namespace, &msg.id, serde_json::to_value(msg)?)
        .await
}

/// Extract @agent-name from a message.
fn extract_agent_mention(message: &str) -> Option<String> {
    for word in message.split_whitespace() {
        if word.starts_with('@') && word.len() > 1 {
            return Some(word[1..].to_string());
        }
    }
    None
}

//! Department infrastructure — shared config, chat, and event patterns.
//!
//! Each department (Code, Content, Harvest, GTM, Forge) gets:
//! - Its own `DepartmentConfig` (model, effort, tools, system prompt)
//! - Its own chat endpoint (streaming SSE via claude -p)
//! - Its own message namespace (isolated conversation history)
//! - Its own event stream (filtered by EngineKind)
//!
//! Thin wrappers per department call these generic functions with the engine kind.

use std::convert::Infallible;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use chrono::Utc;
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

use rusvel_core::domain::{EventFilter, EngineKind, UserProfile};
use rusvel_core::id::EventId;
use rusvel_core::ports::{EventPort, StoragePort};
use rusvel_llm::stream::{ClaudeCliStreamer, StreamEvent};

use crate::chat::{ChatMessage, ChatRequest, ConversationSummary};
use crate::AppState;

// ── Department Config ────────────────────────────────────────

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

impl DepartmentConfig {
    pub fn default_for(engine: &str, profile: Option<&UserProfile>) -> Self {
        let user_context = profile
            .map(|p| p.to_system_prompt())
            .unwrap_or_default();

        match engine {
            "code" => Self {
                engine: "code".into(),
                model: "sonnet".into(),
                effort: "high".into(),
                max_budget_usd: None,
                permission_mode: "default".into(),
                allowed_tools: vec![],
                disallowed_tools: vec![],
                system_prompt: format!(
                    "You are the Code department of RUSVEL.\n\n\
                     You have full access to Claude Code tools:\n\
                     - Read, Write, Edit files across all project directories\n\
                     - Run shell commands (build, test, git, npm, cargo, etc.)\n\
                     - Search codebases with grep and glob\n\
                     - Fetch web content and search the web\n\
                     - Spawn sub-agents for parallel work\n\
                     - Manage background tasks\n\n\
                     Focus: code intelligence, implementation, debugging, testing, refactoring.\n\
                     When writing code, follow existing patterns. Be thorough.\n\n\
                     {user_context}"
                ),
                add_dirs: vec![".".into()],
                max_turns: None,
            },
            "content" => Self {
                engine: "content".into(),
                model: "sonnet".into(),
                effort: "medium".into(),
                max_budget_usd: None,
                permission_mode: "plan".into(),
                allowed_tools: vec![],
                disallowed_tools: vec![],
                system_prompt: format!(
                    "You are the Content department of RUSVEL.\n\n\
                     Focus: content creation, platform adaptation, publishing strategy.\n\
                     Draft in Markdown. Adapt for LinkedIn, Twitter/X, DEV.to, Substack.\n\
                     {user_context}"
                ),
                add_dirs: vec![],
                max_turns: None,
            },
            "harvest" => Self {
                engine: "harvest".into(),
                model: "sonnet".into(),
                effort: "medium".into(),
                max_budget_usd: None,
                permission_mode: "plan".into(),
                allowed_tools: vec![],
                disallowed_tools: vec![],
                system_prompt: format!(
                    "You are the Harvest department of RUSVEL.\n\n\
                     Focus: finding opportunities, scoring gigs, drafting proposals.\n\
                     Sources: Upwork, LinkedIn, GitHub.\n\
                     {user_context}"
                ),
                add_dirs: vec![],
                max_turns: None,
            },
            "gtm" => Self {
                engine: "gtm".into(),
                model: "sonnet".into(),
                effort: "medium".into(),
                max_budget_usd: None,
                permission_mode: "plan".into(),
                allowed_tools: vec![],
                disallowed_tools: vec![],
                system_prompt: format!(
                    "You are the GoToMarket department of RUSVEL.\n\n\
                     Focus: CRM, outreach sequences, deal management, invoicing.\n\
                     {user_context}"
                ),
                add_dirs: vec![],
                max_turns: None,
            },
            _ => Self {
                engine: engine.into(),
                model: "sonnet".into(),
                effort: "medium".into(),
                max_budget_usd: None,
                permission_mode: "default".into(),
                allowed_tools: vec![],
                disallowed_tools: vec![],
                system_prompt: format!("You are the {engine} department of RUSVEL.\n\n{user_context}"),
                add_dirs: vec![],
                max_turns: None,
            },
        }
    }

    pub fn to_claude_args(&self) -> Vec<String> {
        let mut args = vec![
            "--model".into(), self.model.clone(),
            "--effort".into(), self.effort.clone(),
            "--permission-mode".into(), self.permission_mode.clone(),
        ];
        if let Some(budget) = self.max_budget_usd {
            args.extend(["--max-budget-usd".into(), budget.to_string()]);
        }
        if !self.allowed_tools.is_empty() {
            args.extend(["--allowedTools".into(), self.allowed_tools.join(" ")]);
        }
        if !self.disallowed_tools.is_empty() {
            args.extend(["--disallowedTools".into(), self.disallowed_tools.join(" ")]);
        }
        for dir in &self.add_dirs {
            args.extend(["--add-dir".into(), dir.clone()]);
        }
        if let Some(turns) = self.max_turns {
            args.extend(["--max-turns".into(), turns.to_string()]);
        }
        args
    }

    fn store_key() -> &'static str { "dept_config" }

    fn msg_namespace(engine: &str) -> String {
        format!("dept_msg_{engine}")
    }
}

// ── Generic Handlers ─────────────────────────────────────────

pub async fn get_dept_config(
    engine: &str,
    state: &Arc<AppState>,
) -> Result<DepartmentConfig, (StatusCode, String)> {
    let stored = state.storage.objects()
        .get(DepartmentConfig::store_key(), engine)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(stored
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_else(|| DepartmentConfig::default_for(engine, state.profile.as_ref())))
}

pub async fn update_dept_config(
    engine: &str,
    state: &Arc<AppState>,
    config: DepartmentConfig,
) -> Result<DepartmentConfig, (StatusCode, String)> {
    let value = serde_json::to_value(&config)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;
    state.storage.objects()
        .put(DepartmentConfig::store_key(), engine, value)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(config)
}

pub async fn department_chat_handler(
    engine: &str,
    state: Arc<AppState>,
    body: ChatRequest,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let config = get_dept_config(engine, &state).await?;
    let namespace = DepartmentConfig::msg_namespace(engine);

    let conversation_id = body.conversation_id
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());

    // Load history from department namespace
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

    // Load enabled rules and append to system prompt
    let rules = crate::rules::load_rules_for_engine(&state, engine).await;
    let mut system_prompt = config.system_prompt.clone();
    if !rules.is_empty() {
        system_prompt.push_str("\n\n--- Rules ---\n");
        for rule in &rules {
            system_prompt.push_str(&format!("[{}]: {}\n", rule.name, rule.content));
        }
    }

    // Build prompt with enriched system prompt
    let prompt = build_dept_prompt(&system_prompt, &history, &body.message);

    // Resolve engine kind for event emission
    let engine_kind = match engine {
        "code" => EngineKind::Code,
        "content" => EngineKind::Content,
        "harvest" => EngineKind::Harvest,
        "gtm" => EngineKind::GoToMarket,
        _ => EngineKind::Forge,
    };

    // Stream with department-specific flags
    let streamer = ClaudeCliStreamer::new();
    let cli_args = config.to_claude_args();
    let rx = streamer.stream_with_args(&prompt, &cli_args);

    let storage = state.storage.clone();
    let events_port = state.events.clone();
    let conv_id = conversation_id.clone();
    let ns = namespace.clone();

    let stream = ReceiverStream::new(rx).map(move |event| {
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
                    // Emit event so department Events tab shows activity
                    let _ = events_port.emit(rusvel_core::domain::Event {
                        id: EventId::new(),
                        session_id: None,
                        run_id: None,
                        source: eng,
                        kind: format!("{}.chat.completed", eng.to_string()),
                        payload: serde_json::json!({
                            "conversation_id": conv_id_inner,
                            "cost_usd": cost,
                            "response_length": msg.content.len(),
                        }),
                        created_at: Utc::now(),
                        metadata: serde_json::json!({}),
                    }).await;
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
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

pub async fn list_dept_conversations(
    engine: &str,
    state: &Arc<AppState>,
) -> Result<Vec<ConversationSummary>, (StatusCode, String)> {
    let namespace = DepartmentConfig::msg_namespace(engine);
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
                .find(|m| m.role == "user")
                .map(|m| if m.content.len() > 60 { format!("{}...", &m.content[..57]) } else { m.content.clone() })
                .unwrap_or_else(|| "New conversation".into());
            let updated_at = msgs.last().map(|m| m.created_at.clone()).unwrap_or_default();
            ConversationSummary { id, title, updated_at, message_count: msgs.len() }
        })
        .collect();
    summaries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(summaries)
}

pub async fn get_dept_history(
    engine: &str,
    state: &Arc<AppState>,
    conversation_id: &str,
) -> Result<Vec<ChatMessage>, (StatusCode, String)> {
    let namespace = DepartmentConfig::msg_namespace(engine);
    load_namespaced_history(&state.storage, &namespace, conversation_id, 200)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

pub async fn get_dept_events(
    engine_kind: EngineKind,
    state: &Arc<AppState>,
) -> Result<Vec<rusvel_core::domain::Event>, (StatusCode, String)> {
    state.events.query(EventFilter {
        source: Some(engine_kind),
        limit: Some(50),
        ..Default::default()
    }).await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
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

// ── Department Wrapper Macro ─────────────────────────────────
//
// Each department gets 6 thin wrappers that call the generic handlers.
// This macro eliminates the boilerplate.

macro_rules! dept_wrappers {
    ($name:ident, $engine_str:expr, $engine_kind:expr) => {
        paste::paste! {
            pub async fn [<$name _chat>](
                State(state): State<Arc<AppState>>,
                Json(body): Json<ChatRequest>,
            ) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
                department_chat_handler($engine_str, state, body).await
            }
            pub async fn [<$name _config_get>](
                State(state): State<Arc<AppState>>,
            ) -> Result<Json<DepartmentConfig>, (StatusCode, String)> {
                get_dept_config($engine_str, &state).await.map(Json)
            }
            pub async fn [<$name _config_update>](
                State(state): State<Arc<AppState>>,
                Json(config): Json<DepartmentConfig>,
            ) -> Result<Json<DepartmentConfig>, (StatusCode, String)> {
                update_dept_config($engine_str, &state, config).await.map(Json)
            }
            pub async fn [<$name _conversations>](
                State(state): State<Arc<AppState>>,
            ) -> Result<Json<Vec<ConversationSummary>>, (StatusCode, String)> {
                list_dept_conversations($engine_str, &state).await.map(Json)
            }
            pub async fn [<$name _history>](
                State(state): State<Arc<AppState>>,
                axum::extract::Path(id): axum::extract::Path<String>,
            ) -> Result<Json<Vec<ChatMessage>>, (StatusCode, String)> {
                get_dept_history($engine_str, &state, &id).await.map(Json)
            }
            pub async fn [<$name _events>](
                State(state): State<Arc<AppState>>,
            ) -> Result<Json<Vec<rusvel_core::domain::Event>>, (StatusCode, String)> {
                get_dept_events($engine_kind, &state).await.map(Json)
            }
        }
    };
}

dept_wrappers!(code, "code", EngineKind::Code);
dept_wrappers!(content, "content", EngineKind::Content);
dept_wrappers!(harvest, "harvest", EngineKind::Harvest);
dept_wrappers!(gtm, "gtm", EngineKind::GoToMarket);
dept_wrappers!(forge, "forge", EngineKind::Forge);

// All department wrappers generated by macro above

//! Capability Engine — discover online resources, generate entity bundles, auto-install.
//!
//! `POST /api/capability/build` takes a natural language description and:
//! 1. Uses Claude with WebSearch/WebFetch to discover MCP servers, skills, agents online
//! 2. Generates a bundle of entities (agents, skills, rules, MCP servers, hooks, workflows)
//! 3. Persists all entities to `ObjectStore`
//! 4. Returns what was installed
//!
//! Also wired into department chat via `!capability <description>` prefix.

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

use rusvel_core::ports::StoragePort;
use rusvel_llm::stream::{ClaudeCliStreamer, StreamEvent};

use crate::AppState;
use crate::build_cmd::extract_json;

// ── Types ────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CapabilityRequest {
    pub description: String,
    pub engine: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CapabilityBundle {
    #[serde(default)]
    pub agents: Vec<serde_json::Value>,
    #[serde(default)]
    pub skills: Vec<serde_json::Value>,
    #[serde(default)]
    pub rules: Vec<serde_json::Value>,
    #[serde(default)]
    pub mcp_servers: Vec<serde_json::Value>,
    #[serde(default)]
    pub hooks: Vec<serde_json::Value>,
    #[serde(default)]
    pub workflows: Vec<serde_json::Value>,
    #[serde(default)]
    pub explanation: String,
}

#[derive(Debug, Serialize)]
pub struct InstallResult {
    pub installed: Vec<String>,
    pub explanation: String,
}

// ── System Prompt ────────────────────────────────────────────

const CAPABILITY_PROMPT: &str = r#"You are RUSVEL's Capability Engine. When the user describes what they need, you discover, design, and output the exact configurations to make it work.

You have access to tools:
- WebSearch: find MCP servers, skills, agents online
- WebFetch: fetch documentation, APIs, package info
- Bash: verify packages exist (pnpm view, npx --help, etc.)

## Your Process
1. UNDERSTAND — Parse what the user needs
2. DISCOVER — Search online for existing tools:
   - mcp.so for MCP servers (3000+ available)
   - npm registry for @modelcontextprotocol/* packages
   - GitHub for agent/skill templates
3. GENERATE — Create the exact configuration entities
4. OUTPUT — Return ONLY a JSON object (no markdown, no explanation outside JSON)

## Output Format
Return a single JSON object with these arrays (include only what's needed):

{
  "agents": [{"name":"...","role":"...","instructions":"...","default_model":{"provider":"anthropic","model":"sonnet"},"allowed_tools":[],"capabilities":[],"budget_limit":null,"metadata":{}}],
  "skills": [{"name":"...","description":"...","prompt_template":"...","metadata":{}}],
  "rules": [{"name":"...","content":"...","enabled":true,"metadata":{}}],
  "mcp_servers": [{"name":"...","description":"...","server_type":"stdio","command":"npx","args":["-y","@package/name"],"url":null,"env":{},"enabled":true,"metadata":{}}],
  "hooks": [{"name":"...","event":"chat.completed","matcher":"*","hook_type":"command","action":"...","enabled":true,"metadata":{}}],
  "workflows": [{"name":"...","description":"...","steps":[{"agent_name":"...","prompt_template":"...","step_type":"sequential"}],"metadata":{}}],
  "explanation": "What was built and why, in 2-3 sentences"
}

## Rules
- ALWAYS verify MCP server packages exist before recommending (pnpm view or search mcp.so)
- Write practical, specific system prompts for agents — not generic ones
- Use {{input}} as placeholder in skill prompt_templates
- Skill names should be kebab-case (e.g., "review-code", "scan-jobs")
- Include rules for safety/quality when relevant
- Prefer stdio MCP servers (npx) over http when both exist
"#;

// ── Endpoint ─────────────────────────────────────────────────

/// `POST /api/capability/build` — discover and install capabilities from natural language.
pub async fn build_capability(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CapabilityRequest>,
) -> Result<
    Sse<KeepAliveStream<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>>,
    (StatusCode, String),
> {
    let engine = body.engine.clone().unwrap_or_else(|| "global".into());
    let prompt = format!(
        "{CAPABILITY_PROMPT}\n\n\
         User request: {}\n\
         Department: {engine}\n\n\
         Search online, then respond with the JSON configuration.",
        body.description,
    );

    let streamer = ClaudeCliStreamer::new();
    let args = vec![
        "--model".into(),
        "sonnet".to_string(),
        "--effort".into(),
        "high".to_string(),
        "--max-turns".into(),
        "5".to_string(),
        "--permission-mode".into(),
        "default".to_string(),
    ];
    let rx = streamer.stream_with_args(&prompt, &args);

    let storage = state.storage.clone();
    let engine_owned = engine.clone();

    let stream: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> = Box::pin(
        tokio_stream::wrappers::ReceiverStream::new(rx).map(move |event| {
            let sse = match &event {
                StreamEvent::Delta { text } => Event::default()
                    .event("delta")
                    .data(serde_json::json!({"text": text}).to_string()),
                StreamEvent::Done {
                    full_text,
                    cost_usd,
                } => {
                    let storage = storage.clone();
                    let engine = engine_owned.clone();
                    let text = full_text.clone();
                    let _cost = *cost_usd;
                    tokio::spawn(async move {
                        match parse_and_install(&text, &engine, &storage).await {
                            Ok(result) => {
                                tracing::info!(
                                    "Capability Engine installed {} entities: {}",
                                    result.installed.len(),
                                    result.installed.join(", ")
                                );
                            }
                            Err(e) => {
                                tracing::warn!("Capability Engine install failed: {e}");
                            }
                        }
                    });
                    Event::default().event("done").data(
                        serde_json::json!({
                            "text": full_text,
                            "cost_usd": cost_usd,
                        })
                        .to_string(),
                    )
                }
                StreamEvent::Error { message } => Event::default()
                    .event("error")
                    .data(serde_json::json!({"message": message}).to_string()),
            };
            Ok(sse)
        }),
    );

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// ── Inline variant for department chat (!capability prefix) ──

/// Called from `department_chat_handler` when message starts with `!capability`.
pub async fn build_capability_inline(
    engine: &str,
    description: &str,
    storage: Arc<dyn StoragePort>,
    namespace: &str,
    conversation_id: &str,
) -> Result<
    Sse<KeepAliveStream<Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>>>>,
    (StatusCode, String),
> {
    let prompt = format!(
        "{CAPABILITY_PROMPT}\n\n\
         User request: {description}\n\
         Department: {engine}\n\n\
         Search online, then respond with the JSON configuration.",
    );

    let (tx, rx) = tokio::sync::mpsc::channel::<Event>(32);
    let storage_clone = storage.clone();
    let engine_owned = engine.to_string();
    let ns = namespace.to_string();
    let conv_id = conversation_id.to_string();

    tokio::spawn(async move {
        let _ = tx
            .send(
                Event::default().event("delta").data(
                    serde_json::json!({
                        "text": "Searching online and building capabilities...\n\n",
                        "conversation_id": conv_id,
                    })
                    .to_string(),
                ),
            )
            .await;

        let streamer = ClaudeCliStreamer::new();
        let args = vec![
            "--model".into(),
            "sonnet".to_string(),
            "--effort".into(),
            "high".to_string(),
            "--max-turns".into(),
            "5".to_string(),
            "--permission-mode".into(),
            "default".to_string(),
        ];
        let stream_rx = streamer.stream_with_args(&prompt, &args);

        use tokio_stream::wrappers::ReceiverStream;

        let mut stream = ReceiverStream::new(stream_rx);
        let mut _full_response = String::new();

        while let Some(event) = stream.next().await {
            match event {
                StreamEvent::Delta { text } => {
                    let _ = tx
                        .send(
                            Event::default().event("delta").data(
                                serde_json::json!({
                                    "text": text,
                                    "conversation_id": conv_id,
                                })
                                .to_string(),
                            ),
                        )
                        .await;
                }
                StreamEvent::Done {
                    full_text,
                    cost_usd,
                } => {
                    _full_response = full_text.clone();

                    // Parse and install the bundle
                    let install_summary =
                        match parse_and_install(&full_text, &engine_owned, &storage_clone).await {
                            Ok(result) => format!(
                                "\n\n---\n**Installed:** {}\n\n{}",
                                result.installed.join(", "),
                                result.explanation,
                            ),
                            Err(e) => format!("\n\n---\n**Install note:** {e}"),
                        };

                    let final_text = format!("{full_text}{install_summary}");

                    // Store assistant message
                    let msg = crate::chat::ChatMessage {
                        id: uuid::Uuid::now_v7().to_string(),
                        conversation_id: conv_id.clone(),
                        role: "assistant".into(),
                        content: final_text.clone(),
                        created_at: Utc::now().to_rfc3339(),
                    };
                    let _ = storage_clone
                        .objects()
                        .put(&ns, &msg.id, serde_json::to_value(&msg).unwrap_or_default())
                        .await;

                    let _ = tx
                        .send(
                            Event::default().event("done").data(
                                serde_json::json!({
                                    "text": final_text,
                                    "cost_usd": cost_usd,
                                    "conversation_id": conv_id,
                                })
                                .to_string(),
                            ),
                        )
                        .await;
                }
                StreamEvent::Error { message } => {
                    let _ = tx
                        .send(
                            Event::default().event("error").data(
                                serde_json::json!({
                                    "message": message,
                                })
                                .to_string(),
                            ),
                        )
                        .await;
                }
            }
        }
    });

    let stream: Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send>> =
        Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx).map(Ok::<_, Infallible>));

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

// ── Parse + Install ──────────────────────────────────────────

/// Parse a capability bundle from LLM output and install all entities into `ObjectStore`.
async fn parse_and_install(
    text: &str,
    engine: &str,
    storage: &Arc<dyn StoragePort>,
) -> Result<InstallResult, String> {
    let json_str = extract_json(text).ok_or("No JSON found in response")?;
    let bundle: CapabilityBundle = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse capability bundle: {e}"))?;

    let mut installed = vec![];

    // Install agents
    for agent in &bundle.agents {
        let id = uuid::Uuid::now_v7().to_string();
        let mut val = agent.clone();
        inject_metadata(&mut val, engine);
        val.as_object_mut().map(|m| {
            m.insert("id".into(), serde_json::Value::String(id.clone()));
            m.entry("created_by".to_string())
                .or_insert("capability-engine".into());
        });
        if storage.objects().put("agents", &id, val).await.is_ok() {
            let name = agent
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unnamed");
            installed.push(format!("agent:{name}"));
        }
    }

    // Install skills
    for skill in &bundle.skills {
        let id = uuid::Uuid::now_v7().to_string();
        let mut val = skill.clone();
        inject_metadata(&mut val, engine);
        if let Some(m) = val.as_object_mut() {
            m.insert("id".into(), serde_json::Value::String(id.clone()));
        }
        if storage.objects().put("skills", &id, val).await.is_ok() {
            let name = skill
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unnamed");
            installed.push(format!("skill:{name}"));
        }
    }

    // Install rules
    for rule in &bundle.rules {
        let id = uuid::Uuid::now_v7().to_string();
        let mut val = rule.clone();
        inject_metadata(&mut val, engine);
        if let Some(m) = val.as_object_mut() {
            m.insert("id".into(), serde_json::Value::String(id.clone()));
        }
        if storage.objects().put("rules", &id, val).await.is_ok() {
            let name = rule
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unnamed");
            installed.push(format!("rule:{name}"));
        }
    }

    // Install MCP servers
    for mcp in &bundle.mcp_servers {
        let id = uuid::Uuid::now_v7().to_string();
        let mut val = mcp.clone();
        inject_metadata(&mut val, engine);
        if let Some(m) = val.as_object_mut() {
            m.insert("id".into(), serde_json::Value::String(id.clone()));
        }
        if storage.objects().put("mcp_servers", &id, val).await.is_ok() {
            let name = mcp
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unnamed");
            installed.push(format!("mcp:{name}"));
        }
    }

    // Install hooks
    for hook in &bundle.hooks {
        let id = uuid::Uuid::now_v7().to_string();
        let mut val = hook.clone();
        inject_metadata(&mut val, engine);
        if let Some(m) = val.as_object_mut() {
            m.insert("id".into(), serde_json::Value::String(id.clone()));
        }
        if storage.objects().put("hooks", &id, val).await.is_ok() {
            let name = hook
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unnamed");
            installed.push(format!("hook:{name}"));
        }
    }

    // Install workflows
    for wf in &bundle.workflows {
        let id = uuid::Uuid::now_v7().to_string();
        let mut val = wf.clone();
        inject_metadata(&mut val, engine);
        if let Some(m) = val.as_object_mut() {
            m.insert("id".into(), serde_json::Value::String(id.clone()));
        }
        if storage.objects().put("workflows", &id, val).await.is_ok() {
            let name = wf.get("name").and_then(|n| n.as_str()).unwrap_or("unnamed");
            installed.push(format!("workflow:{name}"));
        }
    }

    Ok(InstallResult {
        installed,
        explanation: bundle.explanation,
    })
}

/// Inject engine metadata into a JSON value's metadata field.
fn inject_metadata(val: &mut serde_json::Value, engine: &str) {
    if let Some(obj) = val.as_object_mut() {
        let meta = obj
            .entry("metadata")
            .or_insert_with(|| serde_json::json!({}));
        if let Some(meta_obj) = meta.as_object_mut() {
            meta_obj.insert("engine".into(), engine.into());
            meta_obj.insert("created_by".into(), "capability-engine".into());
        }
    }
}

/// Check if a message is a `!capability` command. Returns the description if so.
pub fn parse_capability_command(message: &str) -> Option<String> {
    let trimmed = message.trim();
    if trimmed.starts_with("!capability") {
        let rest = trimmed
            .trim_start_matches("!capability")
            .trim_start_matches(':')
            .trim();
        if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        }
    } else {
        None
    }
}

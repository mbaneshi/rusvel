# Pattern Extraction — Implementation Design & Code Samples

> **Date:** 2026-03-30 | **Status:** Ready for implementation
> **Sprint:** [Sprint 6](sprint-6-pattern-extraction.md)
> **ADRs:** [ADR-015, ADR-016, ADR-017](../design/decisions.md)
> **Prior plan:** [pattern-extraction-from-repos.md](pattern-extraction-from-repos.md)

---

## Phase A: Flow Resilience — Retry Policies, Timeouts, Export/Import

### A1. Domain Types

**File:** `crates/rusvel-core/src/domain.rs` — add after `FlowErrorBehavior`

```rust
/// Per-node retry policy with exponential backoff (ADR-015).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (0 = no retries).
    pub max_retries: u32,
    /// Initial delay between retries in milliseconds.
    pub initial_delay_ms: u64,
    /// Multiplier applied to delay after each attempt (e.g. 2.0 for doubling).
    pub backoff_multiplier: f64,
    /// Maximum delay cap in milliseconds.
    pub max_delay_ms: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            backoff_multiplier: 2.0,
            max_delay_ms: 30_000,
        }
    }
}
```

**File:** `crates/rusvel-core/src/domain.rs` — add fields to `FlowNodeDef`

```rust
pub struct FlowNodeDef {
    pub id: crate::id::FlowNodeId,
    pub node_type: String,
    pub name: String,
    #[serde(default)]
    pub parameters: serde_json::Value,
    #[serde(default)]
    pub position: (f64, f64),
    #[serde(default)]
    pub on_error: FlowErrorBehavior,
    /// Automatic retry on failure (ADR-015). None = fail immediately.
    #[serde(default)]
    pub retry_policy: Option<RetryPolicy>,
    /// Per-node timeout in seconds (ADR-015). None = no timeout.
    #[serde(default)]
    pub timeout_secs: Option<u64>,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}
```

### A2. Timeout Error Variant

**File:** `crates/rusvel-core/src/error.rs` — add variant

```rust
pub enum RusvelError {
    // ... existing variants ...

    // ── Timeout ────────────────────────────────────────────────
    #[error("timeout: {0}")]
    Timeout(String),

    // ... rest unchanged ...
}
```

### A3. Retry + Timeout Executor

**File:** `crates/flow-engine/src/executor.rs` — new helper function

```rust
use std::sync::Arc;
use std::time::Duration;

use rusvel_core::domain::RetryPolicy;
use rusvel_core::error::{Result, RusvelError};

use crate::nodes::{NodeContext, NodeHandler, NodeOutput};

/// Execute a node handler with optional timeout and retry policy.
///
/// On timeout, returns `RusvelError::Timeout`.
/// On failure with retries remaining, sleeps with exponential backoff.
/// After all retries exhausted, returns the last error.
/// Records attempt count in returned metadata.
async fn execute_with_retry(
    handler: &Arc<dyn NodeHandler>,
    ctx: &NodeContext,
    retry: Option<&RetryPolicy>,
    timeout: Option<u64>,
) -> Result<NodeOutput> {
    let max_attempts = retry.map_or(1, |r| 1 + r.max_retries);
    let mut last_err = None;

    for attempt in 0..max_attempts {
        // Apply backoff delay (skip on first attempt)
        if attempt > 0 {
            if let Some(policy) = retry {
                let delay_ms = (policy.initial_delay_ms as f64
                    * policy.backoff_multiplier.powi(attempt as i32 - 1))
                    as u64;
                let capped = delay_ms.min(policy.max_delay_ms);
                tokio::time::sleep(Duration::from_millis(capped)).await;
            }
        }

        // Execute with optional timeout
        let result = if let Some(secs) = timeout {
            match tokio::time::timeout(
                Duration::from_secs(secs),
                handler.execute(ctx),
            )
            .await
            {
                Ok(inner) => inner,
                Err(_elapsed) => Err(RusvelError::Timeout(format!(
                    "node {} timed out after {secs}s (attempt {}/{})",
                    ctx.node.name,
                    attempt + 1,
                    max_attempts,
                ))),
            }
        } else {
            handler.execute(ctx).await
        };

        match result {
            Ok(output) => return Ok(output),
            Err(e) => {
                tracing::warn!(
                    node = %ctx.node.name,
                    attempt = attempt + 1,
                    max = max_attempts,
                    error = %e,
                    "node execution failed"
                );
                last_err = Some(e);
            }
        }
    }

    Err(last_err.unwrap_or_else(|| {
        RusvelError::Internal("execute_with_retry: no attempts made".into())
    }))
}
```

**Callsite change** — replace executor.rs:291 bare `handler.execute(&ctx).await`:

```rust
// Before:
match handler.execute(&ctx).await {

// After:
match execute_with_retry(
    handler,
    &ctx,
    node_def.retry_policy.as_ref(),
    node_def.timeout_secs,
).await {
```

### A4. Export / Import

**File:** `crates/flow-engine/src/lib.rs` — add methods to `FlowEngine`

```rust
impl FlowEngine {
    /// Export a flow as portable JSON (strips execution state, adds version).
    pub async fn export_flow(&self, id: &FlowId) -> Result<serde_json::Value> {
        let flow = self.get_flow(id).await?;
        let mut val = serde_json::to_value(&flow)?;
        // Add export metadata
        if let Some(obj) = val.as_object_mut() {
            obj.insert("_export_version".into(), serde_json::json!("1.0"));
            obj.insert(
                "_exported_at".into(),
                serde_json::json!(chrono::Utc::now().to_rfc3339()),
            );
        }
        Ok(val)
    }

    /// Import a flow from portable JSON. Assigns a new FlowId to avoid collisions.
    pub async fn import_flow(&self, data: serde_json::Value) -> Result<FlowId> {
        let mut flow: FlowDef = serde_json::from_value(data)?;
        // Assign fresh ID
        flow.id = FlowId::new();
        // Strip export metadata
        if let Some(obj) = flow.metadata.as_object_mut() {
            obj.remove("_export_version");
            obj.remove("_exported_at");
        }
        self.save_flow(&flow).await?;
        Ok(flow.id)
    }
}
```

**File:** `crates/rusvel-api/src/flow_routes.rs` — add handlers

```rust
pub async fn export_flow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let flow_id: FlowId = id.parse().map_err(|_| {
        (StatusCode::BAD_REQUEST, "invalid flow ID".into())
    })?;
    let json = state
        .flow_engine
        .export_flow(&flow_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    Ok((
        [(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"flow-{id}.json\""),
        )],
        axum::Json(json),
    ))
}

pub async fn import_flow(
    State(state): State<Arc<AppState>>,
    Json(data): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let new_id = state
        .flow_engine
        .import_flow(data)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({ "id": new_id.to_string() })),
    ))
}

// Register in router:
// .route("/api/flows/:id/export", get(flow_routes::export_flow))
// .route("/api/flows/import", post(flow_routes::import_flow))
```

### A5. Test: Retry Counter

**File:** `crates/flow-engine/tests/retry_timeout.rs`

```rust
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use flow_engine::nodes::{NodeContext, NodeHandler, NodeOutput};
use rusvel_core::domain::{FlowErrorBehavior, FlowNodeDef, RetryPolicy};
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::FlowNodeId;

/// A node handler that fails the first N calls, then succeeds.
struct FailNTimes {
    counter: AtomicU32,
    fail_count: u32,
}

#[async_trait]
impl NodeHandler for FailNTimes {
    fn node_type(&self) -> &str { "test_fail_n" }

    async fn execute(&self, _ctx: &NodeContext) -> Result<NodeOutput> {
        let attempt = self.counter.fetch_add(1, Ordering::SeqCst);
        if attempt < self.fail_count {
            Err(RusvelError::Internal(format!("deliberate failure #{attempt}")))
        } else {
            Ok(NodeOutput {
                data: serde_json::json!({"attempt": attempt}),
                output_name: "main".into(),
            })
        }
    }
}

#[tokio::test]
async fn retry_policy_succeeds_after_failures() {
    let handler: Arc<dyn NodeHandler> = Arc::new(FailNTimes {
        counter: AtomicU32::new(0),
        fail_count: 2,
    });

    let node_def = FlowNodeDef {
        id: FlowNodeId::new(),
        node_type: "test_fail_n".into(),
        name: "test-node".into(),
        parameters: serde_json::json!({}),
        position: (0.0, 0.0),
        on_error: FlowErrorBehavior::StopFlow,
        retry_policy: Some(RetryPolicy {
            max_retries: 3,
            initial_delay_ms: 10, // fast for tests
            backoff_multiplier: 1.0,
            max_delay_ms: 100,
        }),
        timeout_secs: None,
        metadata: serde_json::json!({}),
    };

    let ctx = NodeContext {
        node: node_def.clone(),
        inputs: Default::default(),
        variables: Default::default(),
    };

    // Should succeed on 3rd attempt (0-indexed: fails at 0, 1; succeeds at 2)
    let result = flow_engine::executor::execute_with_retry(
        &handler,
        &ctx,
        node_def.retry_policy.as_ref(),
        node_def.timeout_secs,
    )
    .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.data["attempt"], 2);
}

#[tokio::test]
async fn timeout_aborts_slow_node() {
    struct SlowNode;

    #[async_trait]
    impl NodeHandler for SlowNode {
        fn node_type(&self) -> &str { "test_slow" }
        async fn execute(&self, _ctx: &NodeContext) -> Result<NodeOutput> {
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            Ok(NodeOutput {
                data: serde_json::json!({}),
                output_name: "main".into(),
            })
        }
    }

    let handler: Arc<dyn NodeHandler> = Arc::new(SlowNode);
    let node_def = FlowNodeDef {
        id: FlowNodeId::new(),
        node_type: "test_slow".into(),
        name: "slow-node".into(),
        parameters: serde_json::json!({}),
        position: (0.0, 0.0),
        on_error: FlowErrorBehavior::StopFlow,
        retry_policy: None,
        timeout_secs: Some(1), // 1 second timeout
        metadata: serde_json::json!({}),
    };

    let ctx = NodeContext {
        node: node_def.clone(),
        inputs: Default::default(),
        variables: Default::default(),
    };

    let result = flow_engine::executor::execute_with_retry(
        &handler,
        &ctx,
        None,
        node_def.timeout_secs,
    )
    .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("timed out"), "expected timeout error, got: {err}");
}

#[test]
fn backward_compat_deser_no_retry_fields() {
    let json = r#"{
        "id": "node-1",
        "node_type": "code",
        "name": "test",
        "parameters": {},
        "on_error": "StopFlow",
        "metadata": {}
    }"#;
    let node: FlowNodeDef = serde_json::from_str(json).unwrap();
    assert!(node.retry_policy.is_none());
    assert!(node.timeout_secs.is_none());
}
```

---

## Phase B: Multi-Channel Messaging

### B1. Domain Types

**File:** `crates/rusvel-core/src/domain.rs` — add in messaging section

```rust
// ════════════════════════════════════════════════════════════════════
//  Channel Messages — unified inbox for multi-channel messaging
// ════════════════════════════════════════════════════════════════════

/// Direction of a channel message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MessageDirection {
    Inbound,
    Outbound,
}

/// A message sent or received through any channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelMessage {
    pub id: String,
    pub channel_kind: String,
    pub direction: MessageDirection,
    pub sender: String,
    pub content: String,
    pub raw_payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}
```

### B2. Slack Adapter

**File:** `crates/rusvel-channel/src/slack.rs` (new)

```rust
//! Slack Incoming Webhook adapter for [`ChannelPort`].
//!
//! Environment:
//! - `RUSVEL_SLACK_WEBHOOK_URL` — required for [`Self::from_env`] to return `Some`.

use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::SessionId;
use rusvel_core::ports::ChannelPort;
use serde_json::Value;

pub struct SlackChannel {
    webhook_url: String,
    client: reqwest::Client,
}

impl SlackChannel {
    #[must_use]
    pub fn from_env() -> Option<Arc<Self>> {
        let webhook_url = std::env::var("RUSVEL_SLACK_WEBHOOK_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;
        Some(Arc::new(Self {
            webhook_url,
            client: reqwest::Client::new(),
        }))
    }
}

fn extract_text(payload: &Value) -> Result<String> {
    payload
        .get("text")
        .or_else(|| payload.get("message"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| {
            RusvelError::Validation("slack payload needs `text` or `message`".into())
        })
}

#[async_trait]
impl ChannelPort for SlackChannel {
    fn channel_kind(&self) -> &'static str {
        "slack"
    }

    async fn send_message(&self, session_id: &SessionId, payload: Value) -> Result<()> {
        let text = extract_text(&payload)?;
        let body = serde_json::json!({
            "text": format!("[session {session_id}] {text}"),
        });
        let resp = self
            .client
            .post(&self.webhook_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RusvelError::Storage(e.to_string()))?;
        if !resp.status().is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(RusvelError::Validation(format!("slack webhook: {txt}")));
        }
        Ok(())
    }
}
```

### B3. Discord Adapter

**File:** `crates/rusvel-channel/src/discord.rs` (new)

```rust
//! Discord Webhook adapter for [`ChannelPort`].
//!
//! Environment:
//! - `RUSVEL_DISCORD_WEBHOOK_URL` — required for [`Self::from_env`] to return `Some`.

use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::SessionId;
use rusvel_core::ports::ChannelPort;
use serde_json::Value;

pub struct DiscordChannel {
    webhook_url: String,
    client: reqwest::Client,
}

impl DiscordChannel {
    #[must_use]
    pub fn from_env() -> Option<Arc<Self>> {
        let webhook_url = std::env::var("RUSVEL_DISCORD_WEBHOOK_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;
        Some(Arc::new(Self {
            webhook_url,
            client: reqwest::Client::new(),
        }))
    }
}

fn extract_content(payload: &Value) -> Result<String> {
    payload
        .get("text")
        .or_else(|| payload.get("message"))
        .or_else(|| payload.get("content"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| {
            RusvelError::Validation(
                "discord payload needs `text`, `message`, or `content`".into(),
            )
        })
}

#[async_trait]
impl ChannelPort for DiscordChannel {
    fn channel_kind(&self) -> &'static str {
        "discord"
    }

    async fn send_message(&self, session_id: &SessionId, payload: Value) -> Result<()> {
        let content = extract_content(&payload)?;
        let body = serde_json::json!({
            "content": format!("[session {session_id}] {content}"),
        });
        let resp = self
            .client
            .post(&self.webhook_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| RusvelError::Storage(e.to_string()))?;
        if !resp.status().is_success() {
            let txt = resp.text().await.unwrap_or_default();
            return Err(RusvelError::Validation(format!("discord webhook: {txt}")));
        }
        Ok(())
    }
}
```

### B4. Channel Registry

**File:** `crates/rusvel-channel/src/registry.rs` (new)

```rust
//! Multi-channel registry — routes messages by channel kind.
//!
//! Implements [`ChannelPort`] itself so it can be a drop-in replacement
//! for a single channel in `AppState`.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::SessionId;
use rusvel_core::ports::ChannelPort;
use serde_json::Value;

pub struct ChannelRegistry {
    channels: HashMap<String, Arc<dyn ChannelPort>>,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
        }
    }

    pub fn register(&mut self, channel: Arc<dyn ChannelPort>) {
        self.channels
            .insert(channel.channel_kind().to_string(), channel);
    }

    pub fn get(&self, kind: &str) -> Option<&Arc<dyn ChannelPort>> {
        self.channels.get(kind)
    }

    pub fn is_empty(&self) -> bool {
        self.channels.is_empty()
    }

    pub fn list_kinds(&self) -> Vec<&str> {
        self.channels
            .values()
            .map(|c| c.channel_kind())
            .collect()
    }

    /// Send to a specific channel by kind.
    pub async fn send_to(
        &self,
        kind: &str,
        session_id: &SessionId,
        payload: Value,
    ) -> Result<()> {
        let channel = self.channels.get(kind).ok_or_else(|| {
            RusvelError::NotFound {
                kind: "channel".into(),
                id: kind.into(),
            }
        })?;
        channel.send_message(session_id, payload).await
    }

    /// Broadcast to all registered channels. Returns per-channel results.
    pub async fn broadcast(
        &self,
        session_id: &SessionId,
        payload: &Value,
    ) -> Vec<(&str, Result<()>)> {
        let mut results = Vec::new();
        for (kind, channel) in &self.channels {
            let res = channel
                .send_message(session_id, payload.clone())
                .await;
            results.push((kind.as_str(), res));
        }
        results
    }
}

#[async_trait]
impl ChannelPort for ChannelRegistry {
    fn channel_kind(&self) -> &'static str {
        "registry"
    }

    /// Routes by `payload["channel"]` if present, otherwise sends to the first
    /// registered channel.
    async fn send_message(
        &self,
        session_id: &SessionId,
        payload: Value,
    ) -> Result<()> {
        if let Some(kind) = payload.get("channel").and_then(|v| v.as_str()) {
            return self.send_to(kind, session_id, payload).await;
        }
        // Default: send to first channel
        if let Some(channel) = self.channels.values().next() {
            return channel.send_message(session_id, payload).await;
        }
        Err(RusvelError::Validation("no channels registered".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry() {
        let reg = ChannelRegistry::new();
        assert!(reg.is_empty());
        assert!(reg.list_kinds().is_empty());
    }
}
```

### B5. Updated lib.rs

**File:** `crates/rusvel-channel/src/lib.rs` (replace)

```rust
//! Outbound messaging [`ChannelPort`] adapters and multi-channel registry.

mod discord;
mod registry;
mod slack;
mod telegram;

pub use discord::DiscordChannel;
pub use registry::ChannelRegistry;
pub use rusvel_core::ports::ChannelPort;
pub use slack::SlackChannel;
pub use telegram::TelegramChannel;
```

### B6. Channel API Routes

**File:** `crates/rusvel-api/src/channel_routes.rs` (new)

```rust
//! Multi-channel messaging API — list, send, broadcast, inbox, inbound webhooks.

use std::sync::Arc;

use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use rusvel_core::domain::{ChannelMessage, MessageDirection};
use rusvel_core::ports::ObjectFilter;
use serde::Deserialize;

use crate::AppState;

// ── List registered channels ────────────────────────────────────

pub async fn list_channels(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let kinds: Vec<&str> = if let Some(ref ch) = state.channel {
        // Downcast to ChannelRegistry if possible
        // Fallback: just return the single channel kind
        vec![ch.channel_kind()]
    } else {
        vec![]
    };
    Ok(Json(serde_json::json!({ "channels": kinds })))
}

// ── Send to specific channel ────────────────────────────────────

#[derive(Deserialize)]
pub struct SendBody {
    pub channel: String,
    pub session_id: String,
    pub text: String,
}

pub async fn send_message(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SendBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    let Some(ref ch) = state.channel else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "no channels configured".into()));
    };
    let payload = serde_json::json!({
        "channel": body.channel,
        "text": body.text,
    });
    let sid = body.session_id.into();
    ch.send_message(&sid, payload)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;

    // Persist outbound message
    let msg = ChannelMessage {
        id: uuid::Uuid::now_v7().to_string(),
        channel_kind: body.channel,
        direction: MessageDirection::Outbound,
        sender: "rusvel".into(),
        content: body.text,
        raw_payload: serde_json::json!({}),
        created_at: Utc::now(),
        metadata: serde_json::json!({}),
    };
    let _ = state
        .storage
        .objects()
        .put(
            "channel_messages",
            &msg.id,
            serde_json::to_value(&msg).unwrap_or_default(),
        )
        .await;

    Ok(StatusCode::NO_CONTENT)
}

// ── Broadcast to all channels ───────────────────────────────────

#[derive(Deserialize)]
pub struct BroadcastBody {
    pub session_id: String,
    pub text: String,
}

pub async fn broadcast(
    State(state): State<Arc<AppState>>,
    Json(body): Json<BroadcastBody>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(ref ch) = state.channel else {
        return Err((StatusCode::SERVICE_UNAVAILABLE, "no channels configured".into()));
    };
    // ChannelRegistry broadcast sends to all; single channel just sends once
    let payload = serde_json::json!({ "text": body.text });
    let sid = body.session_id.into();
    ch.send_message(&sid, payload)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(Json(serde_json::json!({ "status": "sent" })))
}

// ── Unified inbox ───────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct InboxQuery {
    pub channel_kind: Option<String>,
    pub direction: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_messages(
    State(state): State<Arc<AppState>>,
    Query(q): Query<InboxQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let filter = ObjectFilter::default();
    let messages = state
        .storage
        .objects()
        .list("channel_messages", filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut filtered: Vec<serde_json::Value> = messages
        .into_iter()
        .filter(|m| {
            if let Some(ref kind) = q.channel_kind {
                if m.get("channel_kind").and_then(|v| v.as_str()) != Some(kind) {
                    return false;
                }
            }
            if let Some(ref dir) = q.direction {
                if m.get("direction").and_then(|v| v.as_str()) != Some(dir) {
                    return false;
                }
            }
            true
        })
        .collect();

    let limit = q.limit.unwrap_or(50);
    filtered.truncate(limit);

    Ok(Json(serde_json::json!({ "messages": filtered })))
}

// ── Inbound webhook ─────────────────────────────────────────────

pub async fn inbound_webhook(
    State(state): State<Arc<AppState>>,
    Path(kind): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<StatusCode, (StatusCode, String)> {
    let content = payload
        .get("text")
        .or_else(|| payload.get("message"))
        .or_else(|| payload.get("content"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let sender = payload
        .get("sender")
        .or_else(|| payload.get("from"))
        .or_else(|| payload.get("user"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let msg = ChannelMessage {
        id: uuid::Uuid::now_v7().to_string(),
        channel_kind: kind,
        direction: MessageDirection::Inbound,
        sender,
        content,
        raw_payload: payload,
        created_at: Utc::now(),
        metadata: serde_json::json!({}),
    };

    state
        .storage
        .objects()
        .put(
            "channel_messages",
            &msg.id,
            serde_json::to_value(&msg).unwrap_or_default(),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Emit event for hook/trigger system
    if let Some(ref events) = state.events {
        let _ = events
            .emit(rusvel_core::domain::Event {
                id: rusvel_core::id::EventId::new(),
                kind: "channel.message.received".into(),
                payload: serde_json::json!({
                    "message_id": msg.id,
                    "channel_kind": msg.channel_kind,
                }),
                session_id: None,
                created_at: Utc::now(),
                metadata: serde_json::json!({}),
            })
            .await;
    }

    Ok(StatusCode::CREATED)
}

// Route registration (add to lib.rs build_router_with_frontend):
//
// .route("/api/channels", get(channel_routes::list_channels))
// .route("/api/channels/send", post(channel_routes::send_message))
// .route("/api/channels/broadcast", post(channel_routes::broadcast))
// .route("/api/channels/messages", get(channel_routes::list_messages))
// .route("/api/channels/:kind/inbound", post(channel_routes::inbound_webhook))
```

### B7. Composition Root Wiring

**File:** `crates/rusvel-app/src/main.rs` — replace channel wiring

```rust
// Before:
let outbound_channel: Option<Arc<dyn rusvel_channel::ChannelPort>> =
    rusvel_channel::TelegramChannel::from_env()
        .map(|c| c as Arc<dyn rusvel_channel::ChannelPort>);

// After:
let mut channel_registry = rusvel_channel::ChannelRegistry::new();
if let Some(tg) = rusvel_channel::TelegramChannel::from_env() {
    tracing::info!("channel registered: telegram");
    channel_registry.register(tg);
}
if let Some(sl) = rusvel_channel::SlackChannel::from_env() {
    tracing::info!("channel registered: slack");
    channel_registry.register(sl);
}
if let Some(dc) = rusvel_channel::DiscordChannel::from_env() {
    tracing::info!("channel registered: discord");
    channel_registry.register(dc);
}
let outbound_channel: Option<Arc<dyn rusvel_channel::ChannelPort>> =
    if channel_registry.is_empty() {
        None
    } else {
        Some(Arc::new(channel_registry))
    };
```

---

## Phase C: Self-Improvement Loop

### C1. Domain Types

**File:** `crates/rusvel-core/src/domain.rs`

```rust
// ════════════════════════════════════════════════════════════════════
//  Session Context & Build Intelligence (ADR-017)
// ════════════════════════════════════════════════════════════════════

/// Reference to an entity created during a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRef {
    pub kind: String,
    pub id: String,
    pub name: String,
}

/// Persisted session context — what happened, what was decided, what was built.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: String,
    pub department: String,
    pub key_decisions: Vec<String>,
    pub entities_created: Vec<EntityRef>,
    pub errors_encountered: Vec<String>,
    pub conversation_summary: String,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Tracks a single `!build` invocation and its usage over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildRecord {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub entity_name: String,
    pub department: String,
    pub description: String,
    pub usage_count: u32,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// AI-generated suggestion to create a new entity based on session patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildSuggestion {
    pub id: String,
    pub entity_type: String,
    pub suggested_name: String,
    pub description: String,
    pub reasoning: String,
    pub department: String,
    pub status: SuggestionStatus,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

/// Status of a build suggestion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuggestionStatus {
    Pending,
    Accepted,
    Dismissed,
}
```

### C2. Session Context Persistence

**File:** `crates/rusvel-api/src/session_context.rs` (new)

```rust
//! Session context extraction and persistence (ADR-017).
//!
//! ObjectStore kinds: `"session_contexts"`, `"build_records"`, `"build_suggestions"`.

use std::sync::Arc;

use chrono::Utc;
use rusvel_core::domain::{
    BuildSuggestion, EntityRef, SessionContext, SuggestionStatus,
};
use rusvel_core::ports::{ObjectFilter, StoragePort};
use serde_json::json;
use tracing::warn;

const SESSION_CTX_STORE: &str = "session_contexts";
const BUILD_SUGGESTIONS_STORE: &str = "build_suggestions";

/// Save session context by asking an LLM to extract key information.
///
/// Uses `ClaudeCliStreamer` with haiku tier for cost efficiency.
pub async fn save_session_context(
    storage: &Arc<dyn StoragePort>,
    department: &str,
    session_id: &str,
    conversation_messages: &[serde_json::Value],
) -> Result<SessionContext, String> {
    // Build the extraction prompt
    let messages_text: String = conversation_messages
        .iter()
        .filter_map(|m| {
            let role = m.get("role")?.as_str()?;
            let content = m.get("content")?.as_str()?;
            Some(format!("[{role}] {content}"))
        })
        .take(30) // Cap for token budget
        .collect::<Vec<_>>()
        .join("\n---\n");

    let prompt = format!(
        "Analyze this conversation from the '{department}' department and extract:\n\
         1. key_decisions: list of important decisions made\n\
         2. entities_created: list of {{kind, id, name}} for any skills/rules/agents/hooks created\n\
         3. errors_encountered: list of errors or issues hit\n\
         4. conversation_summary: 2-3 sentence summary\n\n\
         Respond as JSON: {{\"key_decisions\": [...], \"entities_created\": [...], \
         \"errors_encountered\": [...], \"conversation_summary\": \"...\"}}\n\n\
         Conversation:\n{messages_text}"
    );

    // Call Claude via CLI streamer (haiku for cost)
    let streamer = rusvel_llm::stream::ClaudeCliStreamer::new();
    let args = vec![
        "--model".to_string(),
        "haiku".to_string(),
        "--max-turns".to_string(),
        "1".to_string(),
        "--permission-mode".to_string(),
        "plan".to_string(),
    ];
    let mut rx = streamer.stream_with_args(&prompt, &args);
    let mut full_text = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            rusvel_llm::stream::StreamEvent::Delta { text } => full_text.push_str(&text),
            rusvel_llm::stream::StreamEvent::Done { full_text: t, .. } => {
                full_text = t;
                break;
            }
            rusvel_llm::stream::StreamEvent::Error { message } => {
                return Err(format!("LLM error: {message}"));
            }
        }
    }

    // Parse LLM response
    let json_str = crate::build_cmd::extract_json(&full_text)
        .ok_or_else(|| "could not extract JSON from LLM response".to_string())?;

    #[derive(serde::Deserialize)]
    struct Extracted {
        #[serde(default)]
        key_decisions: Vec<String>,
        #[serde(default)]
        entities_created: Vec<EntityRef>,
        #[serde(default)]
        errors_encountered: Vec<String>,
        #[serde(default)]
        conversation_summary: String,
    }

    let extracted: Extracted = serde_json::from_str(json_str)
        .map_err(|e| format!("parse error: {e}"))?;

    let ctx = SessionContext {
        session_id: session_id.into(),
        department: department.into(),
        key_decisions: extracted.key_decisions,
        entities_created: extracted.entities_created,
        errors_encountered: extracted.errors_encountered,
        conversation_summary: extracted.conversation_summary,
        created_at: Utc::now(),
        metadata: json!({}),
    };

    // Persist
    storage
        .objects()
        .put(
            SESSION_CTX_STORE,
            &ctx.session_id,
            serde_json::to_value(&ctx).unwrap_or_default(),
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(ctx)
}

/// Load the most recent session context for a department.
pub async fn load_session_context(
    storage: &Arc<dyn StoragePort>,
    department: &str,
) -> Option<SessionContext> {
    let all = storage
        .objects()
        .list(SESSION_CTX_STORE, ObjectFilter::default())
        .await
        .ok()?;

    all.into_iter()
        .filter_map(|v| serde_json::from_value::<SessionContext>(v).ok())
        .filter(|c| c.department == department)
        .max_by_key(|c| c.created_at)
}

/// Analyze session patterns and generate build suggestions.
pub async fn extract_patterns(
    storage: &Arc<dyn StoragePort>,
    department: &str,
    session_context: &SessionContext,
) -> Result<Vec<BuildSuggestion>, String> {
    // Load existing skills and rules for comparison
    let existing_skills = storage
        .objects()
        .list("skills", ObjectFilter::default())
        .await
        .unwrap_or_default();
    let existing_rules = storage
        .objects()
        .list("rules", ObjectFilter::default())
        .await
        .unwrap_or_default();

    let prompt = format!(
        "You are analyzing a completed session in the '{department}' department.\n\n\
         Session summary: {summary}\n\
         Decisions: {decisions}\n\
         Errors: {errors}\n\n\
         Existing skills ({sk_count}): {skills}\n\
         Existing rules ({rl_count}): {rules}\n\n\
         Based on patterns in this session, suggest NEW skills or rules that don't \
         already exist. For each suggestion, provide:\n\
         {{\"entity_type\": \"skill\"|\"rule\"|\"hook\", \"suggested_name\": \"...\", \
         \"description\": \"...\", \"reasoning\": \"why this would help\"}}\n\n\
         Respond as JSON array: [{{...}}, ...]\n\
         If no suggestions, respond: []",
        summary = session_context.conversation_summary,
        decisions = session_context.key_decisions.join("; "),
        errors = session_context.errors_encountered.join("; "),
        sk_count = existing_skills.len(),
        skills = existing_skills
            .iter()
            .filter_map(|s| s.get("name").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join(", "),
        rl_count = existing_rules.len(),
        rules = existing_rules
            .iter()
            .filter_map(|r| r.get("name").and_then(|v| v.as_str()))
            .collect::<Vec<_>>()
            .join(", "),
    );

    let streamer = rusvel_llm::stream::ClaudeCliStreamer::new();
    let args = vec![
        "--model".to_string(),
        "haiku".to_string(),
        "--max-turns".to_string(),
        "1".to_string(),
        "--permission-mode".to_string(),
        "plan".to_string(),
    ];
    let mut rx = streamer.stream_with_args(&prompt, &args);
    let mut full_text = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            rusvel_llm::stream::StreamEvent::Delta { text } => full_text.push_str(&text),
            rusvel_llm::stream::StreamEvent::Done { full_text: t, .. } => {
                full_text = t;
                break;
            }
            rusvel_llm::stream::StreamEvent::Error { message } => {
                warn!("pattern extraction LLM error: {message}");
                return Ok(vec![]);
            }
        }
    }

    // Parse suggestions
    let json_str = crate::build_cmd::extract_json(&full_text)
        .unwrap_or("[]");

    #[derive(serde::Deserialize)]
    struct RawSuggestion {
        entity_type: String,
        suggested_name: String,
        description: String,
        reasoning: String,
    }

    let raw: Vec<RawSuggestion> = serde_json::from_str(json_str).unwrap_or_default();

    let mut suggestions = Vec::new();
    for r in raw {
        let suggestion = BuildSuggestion {
            id: uuid::Uuid::now_v7().to_string(),
            entity_type: r.entity_type,
            suggested_name: r.suggested_name,
            description: r.description,
            reasoning: r.reasoning,
            department: department.into(),
            status: SuggestionStatus::Pending,
            created_at: Utc::now(),
            metadata: json!({}),
        };
        let _ = storage
            .objects()
            .put(
                BUILD_SUGGESTIONS_STORE,
                &suggestion.id,
                serde_json::to_value(&suggestion).unwrap_or_default(),
            )
            .await;
        suggestions.push(suggestion);
    }

    Ok(suggestions)
}
```

### C3. Build Suggestions API

**File:** `crates/rusvel-api/src/build_suggestions.rs` (new)

```rust
//! Build suggestions and history API (ADR-017).

use std::sync::Arc;

use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use rusvel_core::domain::{BuildRecord, BuildSuggestion, SuggestionStatus};
use rusvel_core::ports::ObjectFilter;
use serde::Deserialize;

use crate::AppState;

const SUGGESTIONS_STORE: &str = "build_suggestions";
const RECORDS_STORE: &str = "build_records";

// ── List suggestions ────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct SuggestionQuery {
    pub department: Option<String>,
    pub status: Option<String>,
}

pub async fn list_suggestions(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SuggestionQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let all = state
        .storage
        .objects()
        .list(SUGGESTIONS_STORE, ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let filtered: Vec<serde_json::Value> = all
        .into_iter()
        .filter(|v| {
            if let Some(ref dept) = q.department {
                if v.get("department").and_then(|d| d.as_str()) != Some(dept) {
                    return false;
                }
            }
            if let Some(ref status) = q.status {
                if v.get("status").and_then(|s| s.as_str()) != Some(status) {
                    return false;
                }
            }
            true
        })
        .collect();

    Ok(Json(serde_json::json!({ "suggestions": filtered })))
}

// ── Accept suggestion ───────────────────────────────────────────

pub async fn accept_suggestion(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let val = state
        .storage
        .objects()
        .get(SUGGESTIONS_STORE, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "suggestion not found".into()))?;

    let mut suggestion: BuildSuggestion = serde_json::from_value(val)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    suggestion.status = SuggestionStatus::Accepted;

    // Persist updated status
    state
        .storage
        .objects()
        .put(
            SUGGESTIONS_STORE,
            &id,
            serde_json::to_value(&suggestion).unwrap_or_default(),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Auto-create the entity via !build
    let cmd = crate::build_cmd::BuildCommand {
        entity_type: match suggestion.entity_type.as_str() {
            "skill" => crate::build_cmd::BuildEntityType::Skill,
            "rule" => crate::build_cmd::BuildEntityType::Rule,
            "hook" => crate::build_cmd::BuildEntityType::Hook,
            "agent" => crate::build_cmd::BuildEntityType::Agent,
            _ => crate::build_cmd::BuildEntityType::Skill,
        },
        description: suggestion.description.clone(),
    };
    let result = crate::build_cmd::execute_build(&cmd, &suggestion.department, &state.storage)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(serde_json::json!({
        "status": "accepted",
        "build_result": result,
    })))
}

// ── Dismiss suggestion ──────────────────────────────────────────

pub async fn dismiss_suggestion(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let val = state
        .storage
        .objects()
        .get(SUGGESTIONS_STORE, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "suggestion not found".into()))?;

    let mut suggestion: BuildSuggestion = serde_json::from_value(val)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    suggestion.status = SuggestionStatus::Dismissed;

    state
        .storage
        .objects()
        .put(
            SUGGESTIONS_STORE,
            &id,
            serde_json::to_value(&suggestion).unwrap_or_default(),
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Build history ───────────────────────────────────────────────

#[derive(Deserialize, Default)]
pub struct HistoryQuery {
    pub department: Option<String>,
}

pub async fn list_build_history(
    State(state): State<Arc<AppState>>,
    Query(q): Query<HistoryQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let all = state
        .storage
        .objects()
        .list(RECORDS_STORE, ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let filtered: Vec<serde_json::Value> = all
        .into_iter()
        .filter(|v| {
            if let Some(ref dept) = q.department {
                if v.get("department").and_then(|d| d.as_str()) != Some(dept) {
                    return false;
                }
            }
            true
        })
        .collect();

    Ok(Json(serde_json::json!({ "history": filtered })))
}

// Route registration (add to lib.rs build_router_with_frontend):
//
// .route("/api/build/suggestions", get(build_suggestions::list_suggestions))
// .route("/api/build/suggestions/:id/accept", post(build_suggestions::accept_suggestion))
// .route("/api/build/suggestions/:id/dismiss", post(build_suggestions::dismiss_suggestion))
// .route("/api/build/history", get(build_suggestions::list_build_history))
```

### C4. Build Record Creation in !build

**File:** `crates/rusvel-api/src/build_cmd.rs` — add after each `persist_*` call in `execute_build`

```rust
// After line 153 (Ok(confirmation)), before the return:

// Track build in history (ADR-017)
let record = rusvel_core::domain::BuildRecord {
    id: uuid::Uuid::now_v7().to_string(),
    entity_type: cmd.entity_type.label().to_string(),
    entity_id: /* extract from confirmation or persist_* return */,
    entity_name: /* extract from confirmation */,
    department: engine.to_string(),
    description: cmd.description.clone(),
    usage_count: 0,
    created_at: chrono::Utc::now(),
    metadata: serde_json::json!({ "created_by": "!build" }),
};
let _ = storage
    .objects()
    .put(
        "build_records",
        &record.id,
        serde_json::to_value(&record).unwrap_or_default(),
    )
    .await;
```

To make this clean, refactor each `persist_*` to return `(String, String, String)` — `(confirmation_text, entity_id, entity_name)` — then create the BuildRecord in `execute_build` after the match.

### C5. Session Restore in Chat

**File:** `crates/rusvel-api/src/department.rs` — in `dept_chat`, after rules are loaded

```rust
// After loading rules and before building the final prompt:

// Restore previous session context (ADR-017)
if let Some(prev) = crate::session_context::load_session_context(
    &state.storage,
    &dept,
).await {
    let ctx_block = format!(
        "\n\n--- Previous Session Context ---\n\
         Summary: {}\n\
         Key decisions: {}\n\
         Entities created: {}\n\
         ---\n",
        prev.conversation_summary,
        prev.key_decisions.join("; "),
        prev.entities_created
            .iter()
            .map(|e| format!("{} '{}'", e.kind, e.name))
            .collect::<Vec<_>>()
            .join(", "),
    );
    system_prompt.push_str(&ctx_block);
}
```

### C6. Session End Endpoint

**File:** `crates/rusvel-api/src/department.rs` — new handler

```rust
/// End a session: extract context, generate suggestions, emit event.
pub async fn end_session(
    State(state): State<Arc<AppState>>,
    Path((dept, session_id)): Path<(String, String)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Load conversation messages from session store
    let messages = state
        .storage
        .sessions()
        .get_messages(&session_id.clone().into())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let msg_values: Vec<serde_json::Value> = messages
        .iter()
        .map(|m| serde_json::to_value(m).unwrap_or_default())
        .collect();

    // Save session context
    let ctx = crate::session_context::save_session_context(
        &state.storage,
        &dept,
        &session_id,
        &msg_values,
    )
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    // Extract patterns and generate suggestions
    let suggestions = crate::session_context::extract_patterns(
        &state.storage,
        &dept,
        &ctx,
    )
    .await
    .unwrap_or_default();

    // Emit session.ended event
    if let Some(ref events) = state.events {
        let _ = events
            .emit(rusvel_core::domain::Event {
                id: rusvel_core::id::EventId::new(),
                kind: "session.ended".into(),
                payload: serde_json::json!({
                    "department": dept,
                    "session_id": session_id,
                    "suggestions_count": suggestions.len(),
                }),
                session_id: Some(session_id.into()),
                created_at: chrono::Utc::now(),
                metadata: serde_json::json!({}),
            })
            .await;
    }

    Ok(Json(serde_json::json!({
        "context": ctx,
        "suggestions": suggestions,
    })))
}

// Route registration:
// .route("/api/dept/:dept/sessions/:id/end", post(department::end_session))
```

---

## Route Registration Summary

Add to `crates/rusvel-api/src/lib.rs` in `build_router_with_frontend`:

```rust
// Phase A: Flow export/import
.route("/api/flows/:id/export", get(flow_routes::export_flow))
.route("/api/flows/import", post(flow_routes::import_flow))

// Phase B: Multi-channel messaging
.route("/api/channels", get(channel_routes::list_channels))
.route("/api/channels/send", post(channel_routes::send_message))
.route("/api/channels/broadcast", post(channel_routes::broadcast))
.route("/api/channels/messages", get(channel_routes::list_messages))
.route("/api/channels/:kind/inbound", post(channel_routes::inbound_webhook))

// Phase C: Self-improvement
.route("/api/dept/:dept/sessions/:id/end", post(department::end_session))
.route("/api/build/suggestions", get(build_suggestions::list_suggestions))
.route("/api/build/suggestions/:id/accept", post(build_suggestions::accept_suggestion))
.route("/api/build/suggestions/:id/dismiss", post(build_suggestions::dismiss_suggestion))
.route("/api/build/history", get(build_suggestions::list_build_history))
```

---

## JSON Examples

### Flow with retry policy

```json
{
  "id": "flow-001",
  "name": "LLM Pipeline",
  "description": "Analyze then summarize",
  "nodes": [
    {
      "id": "node-1",
      "node_type": "agent",
      "name": "analyzer",
      "parameters": { "prompt": "Analyze: {{trigger}}", "model": "sonnet" },
      "on_error": "StopFlow",
      "retry_policy": {
        "max_retries": 3,
        "initial_delay_ms": 1000,
        "backoff_multiplier": 2.0,
        "max_delay_ms": 30000
      },
      "timeout_secs": 60,
      "metadata": {}
    }
  ],
  "connections": [],
  "variables": {},
  "metadata": {}
}
```

### Channel send request

```json
{
  "channel": "slack",
  "session_id": "sess-abc",
  "text": "Pipeline completed: 3 opportunities scored"
}
```

### Build suggestion response

```json
{
  "suggestions": [
    {
      "id": "sug-001",
      "entity_type": "skill",
      "suggested_name": "quick-pipeline-report",
      "description": "Generate a concise pipeline status report from harvest data",
      "reasoning": "You ran 'harvest pipeline' + manual formatting 3 times this session",
      "department": "harvest",
      "status": "Pending",
      "created_at": "2026-03-30T14:30:00Z",
      "metadata": {}
    }
  ]
}
```

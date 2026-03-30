# Pattern Extraction from Reference Repos

> **Status:** Plan — not yet implemented
> **Created:** 2026-03-30
> **Source:** repos/ audit of 6 reference codebases
> **Rule:** Extract the pattern, not the code. Rebuild inside RUSVEL's hexagonal architecture.

---

## Summary

Three high-value pattern extractions from reference repos that plug real gaps in RUSVEL. Two medium-value extractions for later. Implementation order follows urgency.

---

## Feature 1: Flow Engine — Retry Policies, Node Timeouts, Workflow Export

**Source:** n8n (workflow automation platform)
**Priority:** HIGH — flows that can't recover from failure aren't production-ready
**Gap:** Nodes fail once and that's it. No automatic retries, no backoff, no per-node timeouts. Agent nodes calling LLMs are especially flaky.

### What already exists (no work needed)

RUSVEL's flow-engine is more mature than it appears:

- DAG execution via petgraph with topological sort
- 3 error behaviors: `StopFlow`, `ContinueOnFail`, `UseErrorOutput`
- Checkpoint persistence on failure (`flow_checkpoints` ObjectStore)
- Resume from checkpoint (`POST /api/executions/:id/resume`)
- Single-node retry (`POST /api/executions/:id/nodes/:node_id/retry`)
- Flows stored as JSON in ObjectStore (already portable)
- 6 node types: code, condition, agent, browser_trigger, browser_action, parallel_evaluate

### What to build

#### 1a. RetryPolicy struct (rusvel-core/src/domain.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,           // e.g. 3
    pub initial_delay_ms: u64,      // e.g. 1000
    pub backoff_multiplier: f64,    // e.g. 2.0
    pub max_delay_ms: u64,          // e.g. 30000
}
```

Add two optional fields to `FlowNodeDef` (both `#[serde(default)]` for backward compat):

```rust
pub retry_policy: Option<RetryPolicy>,
pub timeout_secs: Option<u64>,
```

#### 1b. Timeout error variant (rusvel-core/src/error.rs)

```rust
#[error("timeout: {0}")]
Timeout(String),
```

#### 1c. Retry + timeout wrapper (flow-engine/src/executor.rs)

Extract a helper that wraps `handler.execute(&ctx)`:

```rust
async fn execute_with_retry(
    handler: &Arc<dyn NodeHandler>,
    ctx: &NodeContext,
    retry: Option<&RetryPolicy>,
    timeout: Option<u64>,
) -> Result<NodeOutput>
```

Logic:
1. Wrap each attempt in `tokio::time::timeout` if `timeout_secs` set
2. On failure, check `retry_policy` — if retries remain, sleep with exponential backoff
3. After all retries exhausted, return the last error (falls through to existing `on_error` handling)

Replace the bare `handler.execute(&ctx).await` at executor.rs:291 with this helper.

#### 1d. Workflow export/import (flow-engine/src/lib.rs + rusvel-api/src/flow_routes.rs)

FlowEngine methods:
- `export_flow(id) -> Result<Value>` — loads flow, returns portable JSON with version metadata
- `import_flow(json) -> Result<FlowId>` — validates, assigns new FlowId, saves

API endpoints:
- `GET /api/flows/:id/export` — Content-Disposition download
- `POST /api/flows/import` — accepts JSON body, returns new ID

### Files to modify

| File | Change | Est. lines |
|------|--------|-----------|
| `crates/rusvel-core/src/domain.rs` | Add `RetryPolicy`, 2 fields on `FlowNodeDef` | +15 |
| `crates/rusvel-core/src/error.rs` | Add `Timeout` variant | +3 |
| `crates/flow-engine/src/executor.rs` | Add `execute_with_retry`, wrap node execution | +60 |
| `crates/flow-engine/src/lib.rs` | Add `export_flow`, `import_flow` | +40 |
| `crates/rusvel-api/src/flow_routes.rs` | Add export/import handlers + routes | +30 |

### Tests

- Retry: custom `NodeHandler` with `AtomicU32` counter — fails first N calls, succeeds after. Verify retry count matches policy.
- Timeout: handler sleeps 10s, `timeout_secs: 1`, verify `Failed` with timeout error.
- Export/import round-trip: different ID, same structure.
- Backward compat: existing flow JSON without retry/timeout deserializes with `None`.

---

## Feature 2: Multi-Channel Messaging

**Source:** OpenClaw (multi-channel messaging platform)
**Priority:** HIGH — every B2B client will ask "does it work in Slack?"
**Gap:** Only Telegram. Adding each new channel is a one-off effort instead of a plugin.

### What already exists

- `ChannelPort` trait in `rusvel-core/src/ports.rs`: `channel_kind()` + `send_message(session_id, payload)`
- `TelegramChannel` adapter in `rusvel-channel/src/telegram.rs`
- `dept-messaging` placeholder department
- Single `POST /api/system/notify` endpoint
- `AppState.channel: Option<Arc<dyn ChannelPort>>`

### What to build

#### 2a. Domain types (rusvel-core/src/domain.rs)

```rust
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageDirection { Inbound, Outbound }
```

#### 2b. Channel Registry (rusvel-channel/src/registry.rs — new)

```rust
pub struct ChannelRegistry {
    channels: HashMap<String, Arc<dyn ChannelPort>>,
}
```

Methods: `register()`, `get()`, `send(kind, session_id, payload)`, `broadcast()`, `list_kinds()`.

Implements `ChannelPort` itself — routes by `payload["channel"]` key or broadcasts to all. Drop-in replacement for single channel in AppState.

#### 2c. Slack adapter (rusvel-channel/src/slack.rs — new)

```rust
pub struct SlackChannel {
    webhook_url: String,  // RUSVEL_SLACK_WEBHOOK_URL
    client: reqwest::Client,
}
```

Implements `ChannelPort`. `send_message` POSTs `{"text": ...}` to webhook URL. `from_env()` pattern matches `TelegramChannel`.

#### 2d. Discord adapter (rusvel-channel/src/discord.rs — new)

```rust
pub struct DiscordChannel {
    webhook_url: String,  // RUSVEL_DISCORD_WEBHOOK_URL
    client: reqwest::Client,
}
```

Implements `ChannelPort`. Discord webhooks accept `{"content": ...}`.

#### 2e. Channel API routes (rusvel-api/src/channel_routes.rs — new)

- `GET /api/channels` — list registered channel kinds + status
- `POST /api/channels/send` — send to specific channel: `{channel, session_id, text}`
- `POST /api/channels/broadcast` — send to all channels
- `GET /api/channels/messages` — unified inbox (ObjectStore kind `"channel_messages"`)
- `POST /api/channels/:kind/inbound` — generic inbound webhook, stores `ChannelMessage`, emits `channel.message.received` event

#### 2f. Composition root (rusvel-app/src/main.rs)

Replace single `TelegramChannel::from_env()` with:

```rust
let mut registry = ChannelRegistry::new();
if let Some(tg) = TelegramChannel::from_env() { registry.register(tg); }
if let Some(sl) = SlackChannel::from_env()     { registry.register(sl); }
if let Some(dc) = DiscordChannel::from_env()   { registry.register(dc); }
let channel: Option<Arc<dyn ChannelPort>> = if registry.is_empty() {
    None
} else {
    Some(Arc::new(registry))
};
```

### Files to create/modify

| File | Action | Est. lines |
|------|--------|-----------|
| `crates/rusvel-core/src/domain.rs` | Add `ChannelMessage`, `MessageDirection` | +20 |
| `crates/rusvel-channel/src/slack.rs` | Create — Slack webhook adapter | ~80 |
| `crates/rusvel-channel/src/discord.rs` | Create — Discord webhook adapter | ~80 |
| `crates/rusvel-channel/src/registry.rs` | Create — multi-channel router | ~100 |
| `crates/rusvel-channel/src/lib.rs` | Update re-exports | +6 |
| `crates/rusvel-api/src/channel_routes.rs` | Create — inbox + webhook endpoints | ~200 |
| `crates/rusvel-api/src/lib.rs` | Register module + routes | +10 |
| `crates/rusvel-app/src/main.rs` | Wire ChannelRegistry | ~15 (replace existing) |

### Tests

- Each adapter: verify payload format matches platform API
- Registry: register 2 channels, verify routing by kind and broadcast
- Inbound: POST mock webhook, verify ChannelMessage stored in ObjectStore
- Integration: send via `/api/channels/send`, verify in `/api/channels/messages`

---

## Feature 3: Session Hooks & Self-Improvement Loop

**Source:** everything-claude-code (session persistence patterns)
**Priority:** HIGH — `!build` is the most unique thing about RUSVEL, but it's manual and stateless
**Gap:** RUSVEL forgets between sessions and doesn't learn from its own patterns automatically.

### What already exists

- `!build` command creates Skills/Rules/Hooks/Agents/MCP via LLM (build_cmd.rs)
- Skills stored in ObjectStore, scoped by `metadata.engine`
- Rules injected into system prompts via `load_rules_for_engine()`
- Hooks dispatched fire-and-forget on events
- Full CRUD for all entity types
- 16 hook event types defined

### What to build

#### 3a. Domain types (rusvel-core/src/domain.rs)

```rust
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

pub struct EntityRef {
    pub kind: String,   // "skill", "rule", "agent", "hook"
    pub id: String,
    pub name: String,
}

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

pub struct BuildSuggestion {
    pub id: String,
    pub entity_type: String,
    pub suggested_name: String,
    pub description: String,
    pub reasoning: String,
    pub department: String,
    pub status: SuggestionStatus,  // Pending | Accepted | Dismissed
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}
```

#### 3b. Session context persistence (rusvel-api/src/session_context.rs — new)

ObjectStore kinds: `"session_contexts"`, `"build_records"`, `"build_suggestions"`.

`save_session_context(storage, agent, department, conversation_id, messages)`:
1. Collects conversation messages
2. Sends to LLM (haiku tier for cost) with extraction prompt
3. Stores `SessionContext` in ObjectStore

`load_session_context(storage, department)`:
- Returns most recent context for the department

`extract_patterns(storage, agent, department, session_context)`:
1. Takes session context + existing skills/rules
2. LLM identifies: repeated patterns → skills, common instructions → rules, error patterns → hooks
3. Returns `Vec<BuildSuggestion>`, stored in ObjectStore

#### 3c. Session restore in chat (rusvel-api/src/department.rs)

In `dept_chat`, after loading rules, before sending to agent:

```rust
if let Ok(Some(prev)) = load_session_context(&state.storage, &dept).await {
    system_prompt.push_str(&format!(
        "\n\n--- Previous Session ---\nSummary: {}\nDecisions: {}",
        prev.conversation_summary,
        prev.key_decisions.join("; "),
    ));
}
```

#### 3d. Build history tracking (rusvel-api/src/build_cmd.rs)

After each successful `execute_build`, create a `BuildRecord` in ObjectStore.

In `resolve_skill()` (skills.rs), increment `usage_count` on the matching BuildRecord.

#### 3e. Session end endpoint (rusvel-api/src/department.rs)

`POST /api/dept/:dept/sessions/:id/end`:
1. Loads conversation history
2. Calls `save_session_context()`
3. Calls `extract_patterns()`
4. Emits `"session.ended"` event
5. Returns context + suggestions

#### 3f. Build suggestions API (rusvel-api/src/build_suggestions.rs — new)

- `GET /api/build/suggestions?department=X` — list pending
- `POST /api/build/suggestions/:id/accept` — auto-creates the entity via existing build logic
- `POST /api/build/suggestions/:id/dismiss`
- `GET /api/build/history?department=X` — build records with usage stats

### Files to create/modify

| File | Action | Est. lines |
|------|--------|-----------|
| `crates/rusvel-core/src/domain.rs` | Add `SessionContext`, `EntityRef`, `BuildRecord`, `BuildSuggestion`, `SuggestionStatus` | +60 |
| `crates/rusvel-api/src/session_context.rs` | Create — save/load context, extract patterns | ~200 |
| `crates/rusvel-api/src/build_suggestions.rs` | Create — suggestions CRUD + accept/dismiss | ~150 |
| `crates/rusvel-api/src/build_cmd.rs` | Add BuildRecord creation after successful builds | +20 |
| `crates/rusvel-api/src/department.rs` | Session restore in chat + session end endpoint | +30 |
| `crates/rusvel-api/src/skills.rs` | Usage tracking in `resolve_skill` | +5 |
| `crates/rusvel-api/src/lib.rs` | Register new modules + routes | +10 |

### Tests

- Session context save/load round-trip with mock AgentPort
- Pattern extraction produces valid BuildSuggestions
- Build history tracks usage_count increments
- Session restore injects context into system prompt

---

## Medium-Value (Later)

### Supabase — Frontend Design System Reference

**When:** When RUSVEL's frontend design system work begins.
**What:** Use Supabase's component organization as structural reference — not their visual style, just how shared components are packaged and exported.
**Not code:** Read their `packages/ui/` structure, extract the naming/export pattern, apply to `frontend/src/lib/components/`.

### Tauri — Desktop Distribution

**When:** Only if a client or product decision requires a native desktop installer (.dmg/.exe).
**What:** Study Rust↔JS bridge pattern for clean webview communication. RUSVEL's current single-binary + rust-embed approach is likely sufficient.

---

## Implementation Order

```
1. Flow retry/timeout/export    — smallest, most self-contained, immediate safety benefit
2. Multi-channel messaging      — medium scope, all new files, high business value
3. Session hooks/self-improve   — largest scope, most existing code touched, highest differentiation
```

All three are independent — no cross-feature dependencies. Can be parallelized across branches.

## Line Budget Verification

| Crate | Current (est.) | Added | Risk |
|-------|---------------|-------|------|
| flow-engine | ~1819 | +100 | ~1919 — safe |
| rusvel-channel | ~112 | +260 | ~372 — safe |
| rusvel-core (domain.rs) | multi-file | +95 | safe |
| rusvel-api | multi-file, 31 modules | +400 across 3 new files | safe per-file |

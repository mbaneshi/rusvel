# Task #22: Event Triggers

> Read this file, then do the task. Only modify files listed below.

## Goal

Event triggers subscribe to event patterns and auto-start agents or flows when matching events fire.

## Files to Read First

- `crates/rusvel-core/src/domain.rs` — Event type (kind: String, payload: Value)
- `crates/rusvel-core/src/ports.rs` — EventPort trait (subscribe, publish methods)
- `crates/rusvel-event/src/lib.rs` — EventBus implementation
- `crates/rusvel-agent/src/lib.rs` — AgentRuntime, AgentConfig
- `crates/rusvel-builtin-tools/src/delegate.rs` — delegate_agent pattern
- `crates/rusvel-builtin-tools/src/flow.rs` — invoke_flow pattern

## What to Build

### 1. New types in `crates/rusvel-core/src/domain.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTrigger {
    pub id: String,
    pub name: String,
    /// Glob pattern for event.kind (e.g. "browser.data.*", "content.published")
    pub event_pattern: String,
    /// What to do when triggered
    pub action: TriggerAction,
    /// Optional: only fire for events in this department
    pub department_id: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerAction {
    /// Run an agent with this config
    RunAgent { persona: Option<String>, prompt_template: String, tools: Vec<String> },
    /// Execute a flow by ID
    RunFlow { flow_id: String },
}
```

### 2. New module `crates/rusvel-event/src/triggers.rs`

- `TriggerManager` struct: holds `Vec<EventTrigger>` + `Arc<dyn AgentPort>` + `Arc<dyn StoragePort>`
- `register_trigger(trigger)` — add to list
- `start(event_rx: broadcast::Receiver<Event>)` — spawn a tokio task that listens for events, matches against patterns, and spawns agent/flow actions
- Pattern matching: exact, prefix*, or * wildcard (reuse same logic as tool hooks)

### 3. Wire in `crates/rusvel-event/src/lib.rs`

- Add `pub mod triggers;`
- Export `TriggerManager`

## Files to Modify

- `crates/rusvel-core/src/domain.rs` — add EventTrigger, TriggerAction
- `crates/rusvel-event/src/triggers.rs` (new)
- `crates/rusvel-event/src/lib.rs` — add module

## Verify

```bash
cargo check -p rusvel-event && cargo check --workspace
```

## Depends On

- #18 delegate_agent (done)
- #19 invoke_flow (done)

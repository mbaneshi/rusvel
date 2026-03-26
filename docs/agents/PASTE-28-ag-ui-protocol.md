# Task #28: AG-UI Protocol

> Read this file, then do the task. Only modify files listed below.

## Goal

Map SSE events to the AG-UI (Agent-User Interaction) schema. Add RUN_STARTED, STATE_DELTA, TOOL_CALL_START, TOOL_CALL_END event types so any AG-UI compatible frontend can render agent activity.

## Files to Read First

- `crates/rusvel-agent/src/lib.rs` — AgentEvent enum (TextDelta, ToolCall, ToolResult, Done, Error)
- `crates/rusvel-api/src/chat.rs` — SSE streaming handler, how AgentEvents become SSE
- Search web for "AG-UI protocol specification" or "agent-user interaction protocol"

## What to Build

### 1. AG-UI event types in `crates/rusvel-agent/src/lib.rs`

Extend or replace `AgentEvent` with AG-UI compatible events:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgUiEvent {
    RunStarted { run_id: String, timestamp: String },
    TextDelta { text: String },
    ToolCallStart { tool_call_id: String, tool_name: String, args: serde_json::Value },
    ToolCallEnd { tool_call_id: String, tool_name: String, output: String, is_error: bool },
    StateDelta { delta: serde_json::Value },   // JSON Patch (RFC 6902)
    StepStarted { step_id: String, step_name: String },
    StepCompleted { step_id: String },
    RunCompleted { run_id: String, output: String },
    RunFailed { run_id: String, error: String },
}
```

### 2. Conversion layer

Add `impl From<AgentEvent> for AgUiEvent` or a mapping function. Keep `AgentEvent` as the internal type, add `AgUiEvent` as the wire format.

### 3. Update SSE in `crates/rusvel-api/src/chat.rs`

Convert `AgentEvent` to `AgUiEvent` before sending via SSE. Each SSE event should have:
- `event:` field set to the AG-UI event type name (e.g. `TEXT_DELTA`, `TOOL_CALL_START`)
- `data:` field as JSON of the event

### 4. Add STATE_DELTA emission

In engine tool handlers (e.g. code_analyze, content_draft), emit `StateDelta` events with partial results so the frontend can show progress.

## Files to Modify

- `crates/rusvel-agent/src/lib.rs` — add AgUiEvent enum + conversion
- `crates/rusvel-api/src/chat.rs` — map to AgUiEvent in SSE stream

## Verify

```bash
cargo check -p rusvel-agent && cargo check -p rusvel-api && cargo check --workspace
```

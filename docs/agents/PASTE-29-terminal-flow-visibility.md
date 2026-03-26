# Task #29: Terminal Phase 5 — Flow/Playbook Visibility

> Read this file, then do the task. Only modify files listed below.

## Goal

Each flow node and playbook step executes in its own visible terminal pane. Users see a dashboard of panes during DAG execution.

## Files to Read First

- `crates/rusvel-core/src/terminal.rs` — PaneSource enum
- `crates/flow-engine/src/lib.rs` — FlowEngine, node execution loop
- `crates/rusvel-api/src/terminal.rs` — WebSocket handler
- `crates/rusvel-api/src/flow_routes.rs` — flow API routes

## What to Build

### 1. New PaneSource variants

In `crates/rusvel-core/src/terminal.rs`, add:
```rust
FlowNode { flow_id: String, node_id: String, execution_id: String },
PlaybookStep { playbook_id: String, step_index: usize, run_id: String },
```

### 2. Flow execution visibility

In `crates/flow-engine/src/lib.rs`:
- Accept an optional `Arc<dyn TerminalPort>` in FlowEngine
- Before each node executes, create a pane with `PaneSource::FlowNode`
- Stream node execution output to the pane
- On completion, write a summary line

### 3. API endpoint

In `crates/rusvel-api/src/flow_routes.rs`:
- `GET /api/flows/:id/executions/:exec_id/panes` — list all panes for a flow execution

### 4. Frontend

In `frontend/src/routes/flows/[id]/+page.svelte` (or similar):
- Show a grid of terminal panes for an active flow execution
- Each pane labeled with the node name

## Files to Modify

- `crates/rusvel-core/src/terminal.rs` — add PaneSource variants
- `crates/flow-engine/src/lib.rs` — optional TerminalPort, pane creation per node
- `crates/rusvel-api/src/flow_routes.rs` — add panes endpoint

## Verify

```bash
cargo check -p flow-engine && cargo check -p rusvel-api && cargo check --workspace
```

## Depends On

- #16 Terminal Web Bridge (done)
- #26 Durable Execution
- #27 Playbooks

# Task #24: Terminal Phase 4 — Agent Visibility

> Read this file, then do the task. Only modify files listed below.

## Goal

When an agent delegates to a sub-agent via `delegate_agent`, the sub-agent's execution is visible in a terminal pane. Users can watch agents work in real time.

## Files to Read First

- `crates/rusvel-core/src/terminal.rs` — PaneSource enum, Window/Pane types
- `crates/rusvel-agent/src/lib.rs` — AgentRuntime, AgentEvent, run_streaming_loop
- `crates/rusvel-builtin-tools/src/delegate.rs` — delegate_agent tool
- `crates/rusvel-api/src/terminal.rs` — WebSocket handler
- `frontend/src/routes/terminal/+page.svelte` — terminal page

## What to Build

### Backend

1. **`crates/rusvel-builtin-tools/src/delegate.rs`** — When delegate_agent runs, create a terminal pane (`PaneSource::Delegation { run_id }`) and stream AgentEvents to it. Modify the handler to:
   - Create a pane via TerminalPort before running the agent
   - Use `run_streaming` instead of `run`
   - Forward AgentEvent::TextDelta to the pane via `write_pane`
   - Forward ToolCall/ToolResult as formatted text

2. **Add `terminal_open` and `terminal_watch` tools** in `crates/rusvel-builtin-tools/src/terminal_tools.rs`:
   - `terminal_open` — create a new terminal pane, return pane_id
   - `terminal_watch` — subscribe to an existing pane's output (for agent observation)

3. **`crates/rusvel-api/src/terminal.rs`** — Add `GET /api/terminal/runs/:run_id/panes` to list panes associated with a delegation run.

### Frontend

4. **`frontend/src/lib/components/DelegationTerminal.svelte`** — Component that shows a delegation's terminal output. Takes `runId` prop, fetches panes, renders xterm.

## Files to Modify

- `crates/rusvel-builtin-tools/src/delegate.rs`
- `crates/rusvel-builtin-tools/src/terminal_tools.rs` (new)
- `crates/rusvel-builtin-tools/src/lib.rs` — add mod + register
- `crates/rusvel-api/src/terminal.rs`
- `frontend/src/lib/components/DelegationTerminal.svelte` (new)

## Verify

```bash
cargo check -p rusvel-builtin-tools && cargo check -p rusvel-api
cd frontend && pnpm check
```

## Depends On

- #16 Terminal Web Bridge (done)
- #18 delegate_agent (done)

# Task #35: Terminal Phase 6-7 — CDP Browser Panes + TUI Surface

> Read this file, then do the task. Only modify files listed below.

## Goal

Browser capture logs visible as terminal panes. Ratatui TUI gets a terminal panel.

## Files to Read First

- `crates/rusvel-core/src/terminal.rs` — PaneSource enum
- `crates/rusvel-tui/src/lib.rs` — existing TUI layout
- `crates/rusvel-api/src/terminal.rs` — WebSocket handler
- `crates/rusvel-core/src/domain.rs` — BrowserEvent (added by #25)

## What to Build

### 1. Browser PaneSource

In `crates/rusvel-core/src/terminal.rs`, add:
```rust
Browser { tab_id: String, platform: String },
```

### 2. CDP capture pane

When BrowserPort captures data (`BrowserEvent::DataCaptured`), write a formatted log line to a browser pane:
```
[14:32:01] upwork | job_listing | "Senior Rust Developer - Remote" | score: 8.5
```

This should be wired in the event trigger system or in the browser observation loop.

### 3. TUI terminal panel

In `crates/rusvel-tui/src/lib.rs`:
- Add a 5th panel "Terminal" to the ratatui layout
- Show a list of active panes (name, source, status)
- Allow selecting a pane to see its output in the panel
- Key: `t` to switch to terminal panel, arrow keys to select pane

## Files to Modify

- `crates/rusvel-core/src/terminal.rs` — add Browser PaneSource
- `crates/rusvel-tui/src/lib.rs` — add terminal panel

## Verify

```bash
cargo check -p rusvel-tui && cargo check --workspace
```

## Depends On

- #16 Terminal Web Bridge (done)
- #25 BrowserPort + CDP

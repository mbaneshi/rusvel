# Task #17: Terminal Phase 3 — Department Integration

> Read this file, then do the task. Only modify files listed below.

## Goal

Add a "Terminal" tab to the department panel and lazy pane creation per department.

## Files to Read First

- `crates/rusvel-core/src/terminal.rs` — PaneSource, WindowSource types
- `crates/rusvel-core/src/department/mod.rs` — DepartmentManifest, ContributionKind
- `crates/rusvel-api/src/terminal.rs` — existing WebSocket handler
- `frontend/src/routes/dept/[id]/+page.svelte` — department page layout with tabs

## What to Build

### Backend
1. **`crates/rusvel-api/src/terminal.rs`** — Add a REST endpoint `GET /api/terminal/dept/:dept_id` that creates or returns an existing terminal pane for that department. Use `PaneSource::Department` with the dept ID. Returns `{ pane_id: "uuid" }`.

### Frontend
2. **`frontend/src/routes/dept/[id]/+page.svelte`** — Add a "Terminal" tab alongside existing tabs (Chat, Engine, etc.). When selected, render an xterm.js terminal connecting to the dept-specific pane via `/api/terminal/ws?pane_id={id}`.
3. Create **`frontend/src/lib/components/DeptTerminal.svelte`** — Reusable component that takes a `paneId` prop, initializes xterm.js + FitAddon, connects WebSocket, handles resize.

## Files to Modify

- `crates/rusvel-api/src/terminal.rs`
- `frontend/src/routes/dept/[id]/+page.svelte`
- `frontend/src/lib/components/DeptTerminal.svelte` (new)

## Verify

```bash
cargo check -p rusvel-api
cd frontend && pnpm check
```

## Depends On

- #16 Terminal Web Bridge (must be done first)

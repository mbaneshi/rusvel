You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 2)

**Task #16 — Terminal Phase 2: Web Bridge** — WebSocket (or SSE) route from `rusvel-api` streaming pane output; minimal **`/terminal`** Svelte page with **xterm.js** (or existing dep) + **paneforge** layout; connect to `Arc<dyn TerminalPort>` / `TerminalManager` from **`rusvel-terminal`** (Sprint 1 #10 must be merged). Follow `docs/plans/native-terminal-multiplexer.md` for UX hints.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 2, task #16)
3. `docs/plans/native-terminal-multiplexer.md`
4. `crates/rusvel-terminal/` — `TerminalManager`, `TerminalPort`
5. `crates/rusvel-api/` — Axum patterns, existing WS if any
6. `frontend/package.json` — xterm / paneforge

## Allowed paths

You may edit:

- `crates/rusvel-api/**`
- `frontend/**`
- `crates/rusvel-app/src/main.rs` — **only** if wiring `TerminalPort` into `AppState` is required (minimal diff)

Do **not** run parallel with **PASTE-15** on the same branch if both heavily edit **`+layout.svelte`** — use a worktree or merge order: **#15 first**, then **#16**.

## Rules

- Session/auth: match existing API patterns (session id from query or header).
- Do not block the runtime if PTY is unavailable — UI should degrade gracefully.

## Validation (run before ending)

```bash
cargo test -p rusvel-api
cd frontend && pnpm check && pnpm build
cargo check --workspace
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 16 |
| Task title | Terminal Web Bridge |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

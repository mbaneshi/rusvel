You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 3)

**Task #19 — `invoke_flow` tool** — built-in tool that starts a `FlowEngine` run by flow id (or name), passes trigger payload, returns execution id / status for the calling agent. See `docs/plans/agent-orchestration.md` and `flow-engine` crate.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 3, task #19)
3. `docs/plans/agent-orchestration.md`
4. `crates/flow-engine/` — `run_flow`, execution types
5. `crates/rusvel-builtin-tools/`, `crates/rusvel-agent/`

## Allowed paths

You may edit:

- `crates/rusvel-builtin-tools/**`
- `crates/rusvel-agent/**` (tool registration / `AppState` injection if needed)
- `crates/flow-engine/**` — only if a small public API is missing

Composition root **`rusvel-app`** may need a thin wire to pass `FlowEngine` into tool factory — if so, minimal edit to **`rusvel-app`** is allowed with note in report.

## Rules

- Tool must be session-scoped and safe (no arbitrary flow ids from other sessions if your design forbids it).
- Return structured JSON in `ToolResult` for the LLM.

## Validation (run before ending)

```bash
cargo test -p flow-engine
cargo test -p rusvel-builtin-tools
cargo test -p rusvel-agent
cargo check --workspace
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 19 |
| Task title | invoke_flow tool |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

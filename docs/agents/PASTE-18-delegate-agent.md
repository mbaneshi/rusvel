You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 3)

**Task #18 — `delegate_agent` tool** — built-in tool that spawns a nested agent run with a chosen persona, tool subset, budget cap, and max depth (≤3). Follow `docs/plans/agent-orchestration.md` and ADR-008/009.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 3, task #18)
3. `docs/plans/agent-orchestration.md`
4. `crates/rusvel-agent/src/lib.rs` — `AgentRuntime`, tool loop
5. `crates/rusvel-builtin-tools/` — tool registration patterns

## Allowed paths

You may edit:

- `crates/rusvel-builtin-tools/**`
- `crates/rusvel-agent/**`
- `crates/rusvel-core/src/domain.rs` — only for tool definition / metadata types if needed

Do **not** run parallel with **PASTE-19** if both edit the same registration sites in **`rusvel-builtin-tools/src/lib.rs`** — serialize or split by file with human merge.

## Rules

- Depth cap and budget enforcement must be real (no infinite recursion).
- Prefer reusing `AgentRuntime` internally over duplicating the loop.

## Validation (run before ending)

```bash
cargo test -p rusvel-agent
cargo test -p rusvel-builtin-tools
cargo check --workspace
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 18 |
| Task title | delegate_agent tool |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

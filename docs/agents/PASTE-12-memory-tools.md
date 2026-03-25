You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 2)

**Task #12 — Memory Tools** — expose `memory_read` / `memory_write` / `memory_search` / `memory_delete` (or the names already planned) as built-in agent tools; optionally auto-inject top relevant memories per session rules in `sprints.md`.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 2, task #12)
3. `docs/plans/agent-sdk-features.md` — memory tool section
4. `crates/rusvel-builtin-tools/`, `crates/rusvel-memory/`, `crates/rusvel-agent/`

## Allowed paths

You may edit:

- `crates/rusvel-builtin-tools/**`
- `crates/rusvel-agent/**` (tool registration / injection only if needed)

Do **not** run parallel with **PASTE-11** on the same branch if both heavily edit `rusvel-agent` — serialize.

## Rules

- Tools go through `ToolRegistry` / existing patterns.
- Respect session namespacing from `rusvel-memory`.

## Validation (run before ending)

```bash
cargo test -p rusvel-builtin-tools
cargo test -p rusvel-agent
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 12 |
| Task title | Memory Tools |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

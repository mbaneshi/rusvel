You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 2)

**Task #11 — Context Compaction** — auto-summarize conversation when message count exceeds a threshold (e.g. 30), keep the most recent N messages (e.g. 10) verbatim, inject summary into context for the agent loop.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 2, task #11)
3. `docs/plans/agent-sdk-features.md` — compaction section
4. `crates/rusvel-agent/src/lib.rs` — `AgentRuntime`, message handling

## Allowed paths

You may edit only:

- `crates/rusvel-agent/**`

Do **not** run parallel with **PASTE-12** or **PASTE-14** if they also touch `rusvel-agent` — **one Sprint 2 agent touching `rusvel-agent` at a time**.

## Rules

- Use `AgentPort` / existing LLM calls — engines do not call `LlmPort` directly from outside agent; follow ADR-009 patterns inside `rusvel-agent`.
- Add tests in `rusvel-agent` where feasible.

## Validation (run before ending)

```bash
cargo test -p rusvel-agent
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 11 |
| Task title | Context Compaction |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

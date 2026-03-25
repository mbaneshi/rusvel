You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 2)

**Task #13 — Batch API** — add `submit_batch()` / `poll_batch()` (or equivalent) on `LlmPort` for async batch jobs with discounted pricing; wire default implementations and at least one provider path per `sprints.md` and `next-level-proposals.md` (P3). Depends on **#8** (cost routing) — assume `ModelTier` / `MetricStore` from Sprint 1 #8 exist.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 2, task #13)
3. `docs/plans/next-level-proposals.md` — Batch API / P3
4. `crates/rusvel-core/src/ports.rs` — `LlmPort`
5. `crates/rusvel-llm/` — providers, existing request types

## Allowed paths

You may edit only:

- `crates/rusvel-core/src/ports.rs` (trait methods + types as needed)
- `crates/rusvel-core/src/domain.rs` (batch request/response types if needed)
- `crates/rusvel-llm/**`

Do **not** edit `rusvel-agent`, `frontend/**`, `dept-*` in this task unless unavoidable — note in report.

## Rules

- ADR-009: engines still use `AgentPort`; batch is for **LlmPort** / jobs layer.
- Preserve backward compatibility for synchronous `LlmPort::complete` / `stream` callers.

## Validation (run before ending)

```bash
cargo test -p rusvel-llm
cargo test -p rusvel-core
cargo check --workspace
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 13 |
| Task title | Batch API on LlmPort |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

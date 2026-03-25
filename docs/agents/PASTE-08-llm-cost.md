You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 1)

**Task #8 — LLM Cost Intelligence** — ModelTier routing (Haiku / Sonnet / Opus style tiers) plus cost tracking integrated with `MetricStore` / session usage so the system can route cheaper models for simple work and record spend.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 1, Track B, task #8)
3. `docs/plans/next-level-proposals.md` — find the **LLM Cost Intelligence / P12** section for implementation hints
4. `crates/rusvel-core/src/ports.rs` — `MetricStore`, `LlmPort`
5. `crates/rusvel-llm/` — providers, routing

## Allowed paths

You may edit only:

- `crates/rusvel-llm/**`
- `crates/rusvel-core/src/ports.rs` (minimal additions for types or trait methods)
- `crates/rusvel-core/src/domain.rs` (only if cost/tier types belong there per existing patterns)

Do **not** edit `rusvel-agent`, `dept-*`, `rusvel-app`, or `frontend/**`.

## Rules

- Keep routing and accounting behind `LlmPort` / `MetricStore` patterns — no direct adapter imports from engines.
- Prefer extending existing provider types over rewriting all providers.

## Deliverables

- Model tier selection and cost tracking behavior described in Sprint 1 #8 and the proposals doc, wired so tests can validate core paths.
- `cargo test -p rusvel-llm` and `cargo test -p rusvel-core` pass.

## Validation (run before ending)

```bash
cargo test -p rusvel-llm
cargo test -p rusvel-core
cargo check --workspace
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 8 |
| Task title | LLM Cost Intelligence |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 2)

**Task #14 — Hybrid RAG** — fuse FTS5 (`rusvel-memory`) and LanceDB (`rusvel-vector`) via RRF (reciprocal rank fusion), optional rerank with a small model; expose to API or agent tools as described in `sprints.md` and `next-level-proposals.md` (P2).

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 2, task #14)
3. `docs/plans/next-level-proposals.md` — Hybrid RAG / P2
4. `crates/rusvel-memory/`, `crates/rusvel-vector/`, `crates/rusvel-api/src/knowledge.rs` (if wiring search)

## Allowed paths

You may edit:

- `crates/rusvel-memory/**`
- `crates/rusvel-vector/**`
- `crates/rusvel-api/**` (only for search / knowledge routes needed to use hybrid results)
- `crates/rusvel-core/**` (only if new port types are required)

Avoid editing `rusvel-agent` unless you must add a single API hook — prefer keeping fusion in memory/vector + API layer first.

## Rules

- No new dependency on adapter crates from engines.
- Keep batch sizes and limits explicit.

## Validation (run before ending)

```bash
cargo test -p rusvel-memory
cargo test -p rusvel-vector
cargo test -p rusvel-api
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 14 |
| Task title | Hybrid RAG |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

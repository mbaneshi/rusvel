You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 1)

**Task #6 — Remove `EngineKind` enum** — use string department IDs everywhere (aligned with ADR-005 / ADR-014). This is a **workspace-wide** refactor.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 1, task #6)
3. `docs/design/department-as-app.md`
4. `docs/design/decisions.md`

## Important — no parallel agents

**Do not run this prompt while other agents edit the same repo.** Land all other agent work first, then run **only this** agent (or use a dedicated branch and merge carefully).

## Allowed paths

**Entire workspace** — any crate that references `EngineKind` or equivalent.

## Rules

- Replace enum dispatch with string IDs consistent with `DepartmentManifest.id` / event kinds.
- `cargo test --workspace` must pass before you finish.
- Prefer mechanical renames + compiler-driven fixes over behavior changes.

## Validation (run before ending)

```bash
cargo test --workspace
cargo check --workspace
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 6 |
| Task title | Remove EngineKind, string IDs |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

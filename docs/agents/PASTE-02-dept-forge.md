You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 1)

**Task #2 — Complete dept-forge impl** — mission tools, 10 personas, forge-specific routes, safety hooks, and manifest entries per `docs/design/department-as-app.md`.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 1, Track A, task #2)
3. `docs/design/department-as-app.md`
4. `crates/dept-forge/` — all files
5. `crates/forge-engine/` — personas, mission, routes

## Allowed paths

You may edit only:

- `crates/dept-forge/**`
- `crates/forge-engine/**`

Do **not** edit `rusvel-app`, `dept-content`, `rusvel-agent`, or other `dept-*` crates unless blocked — then report the blocker.

## Rules

- Hexagonal architecture: engines do not import adapter crates.
- Follow existing Forge patterns (mission, personas) already in `forge-engine`.

## Deliverables

- Forge department registers mission-related tools, personas, routes, and safety behavior as required by the sprint row and ADR-014.
- Tests pass for touched crates.

## Validation (run before ending)

```bash
cargo test -p dept-forge
cargo test -p forge-engine
cargo check -p dept-forge
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 2 |
| Task title | Complete dept-forge impl |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

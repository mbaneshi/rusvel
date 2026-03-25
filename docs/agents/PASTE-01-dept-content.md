You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 1)

**Task #1 — Complete dept-content impl** — routes, tools, personas, skills, handlers, events, and jobs as declared in `DepartmentManifest` and `DepartmentApp::register`, aligned with `docs/design/department-as-app.md`.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 1, Track A, task #1)
3. `docs/design/department-as-app.md`
4. `crates/dept-content/` — all files
5. `crates/content-engine/` — as needed for wiring

## Allowed paths

You may edit only:

- `crates/dept-content/**`
- `crates/content-engine/**`

Do **not** edit `rusvel-app`, `dept-forge`, `rusvel-agent`, or other `dept-*` crates unless the task absolutely requires a shared type change — if so, stop and note it in the report instead of guessing.

## Rules

- Hexagonal architecture: no new imports from adapter crates into engines.
- `cargo` for Rust; `pnpm` only under `frontend/` (not in scope here unless you must not touch frontend).

## Deliverables

- Content department exposes full manifest contributions (tools, personas, skills, routes, events, jobs) per ADR-014 where the design doc and existing patterns require.
- `cargo test -p dept-content` and `cargo test -p content-engine` pass.

## Validation (run before ending)

```bash
cargo test -p dept-content
cargo test -p content-engine
cargo check -p dept-content
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 1 |
| Task title | Complete dept-content impl |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

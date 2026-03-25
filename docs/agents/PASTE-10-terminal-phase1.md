You are a RUSVEL implementation agent. Work only inside this repository.

## Task (from docs/plans/sprints.md Sprint 1)

**Task #10 — Terminal Multiplexer Phase 1** — introduce a `TerminalPort` trait (or equivalent name) in `rusvel-core`, a `rusvel-terminal` crate with `TerminalManager` using `portable-pty` (or the crate already chosen in `Cargo.toml` if present), wired to `EventPort` / `StoragePort` as needed for session state. Follow `docs/plans/native-terminal-multiplexer.md` for intent if details conflict, prefer **minimal Phase 1** that compiles and tests.

## Read first

1. `CLAUDE.md` (repo root)
2. `docs/plans/sprints.md` (Sprint 1, task #10)
3. `docs/plans/native-terminal-multiplexer.md` (skim — implement Phase 1 scope only)
4. `crates/rusvel-core/src/ports.rs`
5. `crates/rusvel-event/`, `crates/rusvel-db/` — only if you need patterns for events/storage

## Allowed paths

You may edit:

- `crates/rusvel-core/**` (ports + lib exports for the new trait)
- `crates/rusvel-terminal/**` (new crate — add to workspace `Cargo.toml` root members)
- `Cargo.toml` (workspace members for the new crate)

Do **not** edit `frontend/**` in this task (that is Terminal Phase 2 in Sprint 2). Do **not** edit `rusvel-agent` unless required to register nothing yet — Phase 1 is core + crate.

## Rules

- Add the workspace member in root `Cargo.toml`.
- Keep the public surface small: trait + manager + tests.

## Deliverables

- New terminal abstraction compiles, unit tests where reasonable, `cargo check --workspace` passes.

## Validation (run before ending)

```bash
cargo test -p rusvel-terminal
cargo check --workspace
```

## Report (required — end your reply with this)

```markdown
## Agent report

| Task # | 10 |
| Task title | Terminal Multiplexer Phase 1 |
| Summary | |

### Files touched
### Commands run
### Blockers
### Ready to merge?
```

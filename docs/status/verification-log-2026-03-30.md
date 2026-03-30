# Verification log — 2026-03-30

Evidence for claims in [`current-state.md`](current-state.md). Re-run the commands in **How to re-verify** there when updating metrics.

| Claim | Evidence |
|-------|----------|
| Workspace members | `cargo metadata --format-version 1 --no-deps` → `workspace_members` length **54**. |
| Rust LOC / files | `wc -l $(find crates -name '*.rs')` → **68,443** lines total; `find crates -name '*.rs' \| wc -l` → **293** files (counts only `crates/`, not `frontend/`). |
| Tests | `cargo test --workspace` → sum of `running N tests` lines **635**; **0 failures** (repo root, full workspace; PTY-dependent tests may fail in restricted sandboxes). |
| Test executables | `cargo test --workspace --no-run 2>&1 \| rg '^  Executable' \| wc -l` → **100** (approximate compiled test target count). |
| HTTP `.route(` registrations | `rg '\.route\(' crates/rusvel-api/src/lib.rs \| wc -l` → **141** (main Axum API router in `build_router_with_frontend`). |
| API source files | `ls crates/rusvel-api/src/*.rs \| wc -l` → **37** including `lib.rs` → **36** handler/support modules beside `lib.rs`. |
| Port traits | `rg '^pub trait ' crates/rusvel-core/src/ports.rs \| wc -l` → **21**. |
| `pub struct` / `pub enum` in `domain.rs` | `rg '^pub (struct|enum) ' crates/rusvel-core/src/domain.rs \| wc -l` → **112**. |
| `cargo build` | **0** errors (workspace, 2026-03-30). |

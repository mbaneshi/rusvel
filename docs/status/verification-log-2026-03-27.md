# Verification log — 2026-03-27

Evidence for claims in [`current-state.md`](current-state.md). Re-run the commands in **How to re-verify** there when updating metrics.

| Claim | Evidence |
|-------|----------|
| Workspace members | `cargo metadata --format-version 1 --no-deps` → `workspace_members` length **50** (see root `Cargo.toml` `[workspace].members`). |
| Rust LOC / files | `wc -l $(find crates -name '*.rs')` → **52,560** lines; `find crates -name '*.rs' \| wc -l` → **215** files (counts only `crates/`, not `frontend/`). |
| Tests | `cargo test` (full workspace) → **399** tests reported in `running N tests` sum; **0 failures** when run with normal dev environment (PTY-dependent tests may fail in restricted sandboxes). |
| Test binaries | `cargo test --no-run 2>&1 \| rg -c "Executable"` → **61** lines (approximate test target count). |
| HTTP `.route(` registrations | `rg '\.route\(' crates/rusvel-api/src/lib.rs \| wc -l` → **105** (single Axum `api` router in `build_router_with_frontend`). |
| API modules | `ls crates/rusvel-api/src/*.rs` → **27** files including `lib.rs` → **26** handler modules. |
| `hook_dispatch` wired | `department.rs` calls `crate::hook_dispatch::dispatch_hooks` after chat completion (see `rusvel-api`). |
| OutreachSend placeholder | `rusvel-app` job worker: `JobKind::OutreachSend` logs `engine_not_wired` ([`main.rs`](../../crates/rusvel-app/src/main.rs)). |
| Content platform HTTP adapters | [`content-engine/src/adapters/`](../../crates/content-engine/src/adapters/): `linkedin.rs`, `twitter.rs`, `devto.rs` implement `PlatformAdapter` with real HTTP (tokens via `ConfigPort`). |
| Port traits | `rg '^pub trait ' crates/rusvel-core/src/ports.rs` → **20** traits in `ports.rs` (includes `BrowserPort`, five `*Store` subtraits, etc.). |

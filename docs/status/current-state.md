# RUSVEL — Current State

> **Last verified:** 2026-03-27 (metrics + spot-checks; see [verification-log-2026-03-27.md](verification-log-2026-03-27.md))

---

## How to re-verify

Run from the repository root:

```bash
cargo build
cargo test
cargo metadata --format-version 1 --no-deps | python3 -c "import json,sys; print(len(json.load(sys.stdin)['workspace_members']))"
find crates -name '*.rs' | wc -l
wc -l $(find crates -name '*.rs') | tail -1
rg '\.route\(' crates/rusvel-api/src/lib.rs | wc -l
```

Use the same host environment as normal development. Some integration tests (e.g. terminal PTY) may fail in sandboxed or headless CI unless configured.

---

## Metric definitions

| Term | Meaning |
|------|---------|
| **Workspace members** | Packages listed in `[workspace].members` in root `Cargo.toml` — `cargo metadata --no-deps` count. |
| **Rust LOC** | Total lines of `*.rs` under `crates/` only (excludes `frontend/`). |
| **Rust source files** | Count of `*.rs` files under `crates/`. |
| **Tests (count)** | Sum of `running N tests` lines from `cargo test` output (~399). |
| **Test targets** | Approximate count of compiled test executables from `cargo test --no-run` (e.g. ~61); differs from **test binaries** phrasing used in older docs (~30 referred to `[[test]]` / crate-level counts). |
| **HTTP route chains** | Lines with `.route(` in `crates/rusvel-api/src/lib.rs` main API router (**105**). One line can register multiple methods (`get().post()`). |
| **API modules** | `*.rs` files in `crates/rusvel-api/src/` excluding `lib.rs` (**26**). |
| **Port traits** | `pub trait` entries in `crates/rusvel-core/src/ports.rs` (**20**, including five `*Store` subtraits and `BrowserPort`). `DepartmentApp` lives under `department/`. |

---

## 1. Numbers at a Glance

| Metric | Count |
|--------|------:|
| Workspace members | 50 |
| Rust lines of code (crates/*.rs) | ~52,560 |
| Rust source files (crates/) | 215 |
| Tests (approx., `cargo test`) | ~399 (0 failures, local full run) |
| Test targets (approx., `cargo test --no-run`) | ~61 |
| HTTP route chains (`lib.rs` `.route(`) | 105 |
| API handler modules (`rusvel-api/src/*.rs` excl. lib) | 26 |
| Port traits (`rusvel-core` `ports.rs`) | 20 |
| `pub struct` / `pub enum` in `domain.rs` | 100 |
| Departments | 12 |
| Department crates (dept-*) | 13 |
| Engines | 13 (all via `DepartmentApp`) |
| Registered agent tools | 22+ (built-in + `tool_search` + engine tools; optional memory, delegate, terminal, flow, browser) |
| MCP stdio tools (`rusvel-mcp`) | 6 |

---

## 2. Build Health

| Check | Status |
|-------|--------|
| `cargo build` | Clean — 0 errors |
| `cargo test` | Full suite passes (see re-verify note above) |

---

## 3. What Works End-to-End

These features are wired from the binary entry [`crates/rusvel-app/src/main.rs`](../../crates/rusvel-app/src/main.rs) through adapters to the HTTP API and, where noted, the embedded frontend.

**API server startup** — Boots SQLite WAL, LLM with ModelTier routing + `MetricStore` cost tracking, `EventBus`, `MemoryPort`, `ScopedToolRegistry` (built-in + engine tools + optional tools), `JobQueue`, `AgentRuntime` with streaming, optional `EmbeddingPort`, `VectorStore`, `TerminalPort`, optional `rusvel_cdp::CdpClient` / `BrowserPort`; collects `DepartmentApp` instances from 13 `dept-*` crates; `DepartmentManifest` registration order; seeds default data; spawns job worker; Axum on `:3000` with graceful shutdown.

**First-run wizard** — Interactive `cliclack` onboarding: detects Ollama, collects name/role, writes `profile.toml`, creates first session.

**Embedded frontend** — `rust-embed` compiles `frontend/build/` into the binary; filesystem fallbacks and temp extraction as implemented in app.

**Department chat (SSE)** — `POST /api/dept/{dept}/chat` streams via `AgentRuntime` (`AgentEvent` SSE). Includes config cascade, `@agent` mentions, `/skill` resolution, `!build`, rules from `ObjectStore`, per-dept MCP config, **hook dispatch** after `{engine}.chat.completed` ([`hook_dispatch.rs`](../../crates/rusvel-api/src/hook_dispatch.rs) from [`department.rs`](../../crates/rusvel-api/src/department.rs)), conversation persistence.

**God agent chat** — `POST /api/chat` with SSE, history, profile context.

**Sessions, mission, goals** — Session CRUD; `GET .../mission/today`; goals CRUD.

**Events** — `GET /api/sessions/{id}/events`, `GET /api/dept/{dept}/events`.

**Entity CRUD** — Agents, skills, rules, MCP servers, hooks, workflows (REST as routed in [`lib.rs`](../../crates/rusvel-api/src/lib.rs)).

**Approvals (ADR-008)** — `GET /api/approvals`, approve/reject; job worker respects approval-gated jobs.

**Capability engine** — `POST /api/capability/build`; `!build` in chat.

**Workflows** — CRUD + `POST /api/workflows/{id}/run`.

**Hooks** — CRUD + `GET /api/hooks/events`; dispatch on events (command / http / prompt).

**Engines (core depth)** — `CodeEngine`, `ContentEngine`, `HarvestEngine`, `FlowEngine` instantiated with real logic; engine routes under `/api/dept/code/*`, `/api/dept/content/*`, `/api/dept/harvest/*`, `/api/flows/*`, playbooks, kits, brief, etc.
**Content publishing** — `content-engine` includes **real HTTP adapters** for DEV.to, Twitter/X, LinkedIn ([`adapters/`](../../crates/content-engine/src/adapters/)); credentials via `ConfigPort` keys (e.g. `twitter_token`, `linkedin_token`).

**Job queue worker** — Polls jobs; handles `CodeAnalyze`, `ContentPublish`, `HarvestScan` with `session_id` scoping.

**RusvelBase** — `/api/db/*` routes; UI under `/database/*`.

**Knowledge/RAG** — Multiple `/api/knowledge/*` routes (list, ingest, search, hybrid-search, stats, related, delete).

**Code-to-content** — `POST /api/dept/content/from-code`.

**MCP server (stdio)** — `--mcp` → `rusvel_mcp` JSON-RPC; **6** tools in `tool_definitions()`.

**MCP client** — `rusvel-mcp-client` for external servers.

**CLI** — One-shot, `shell`, `--tui`.

**Terminal** — API routes for dept pane, run panes, WebSocket (`/api/terminal/*`).

**Browser (CDP)** — `/api/browser/*` when CDP client wired.

**System / visual** — `/api/system/*`, visual report routes for regression testing.

**Seed data** — Default agents, skills, rules on first run.

---

## 4. Built but Needs More Work

**Extended GTM / CRM depth** — **OutreachSend** jobs: worker explicitly **not wired** to GTM engine ([`main.rs`](../../crates/rusvel-app/src/main.rs) `JobKind::OutreachSend` → `engine_not_wired`).

**Authentication/authorization** — `rusvel-auth` is not full API middleware; env/in-memory style credentials for many paths.

**Eight “business” engines** — Finance, Product, Growth, Distro, Legal, Support, Infra (and GTM beyond chat) are thinner than Forge/Code/Content/Harvest; chat works via `DepartmentApp` + registry.

**Gap vs old monoliths** — See [`gap-analysis.md`](gap-analysis.md): historical comparison to `old/` repos; **not** a substitute for §3–4 here.

---

## 5. Test Breakdown (high level)

Tests are spread across crates; highest concentration in `rusvel-llm`, `forge-engine`, `rusvel-api`, `rusvel-db`, `harvest-engine`, `rusvel-core`, `rusvel-agent`, `content-engine`, etc.

Some surface crates (`rusvel-app`, `rusvel-cli`, `rusvel-mcp`, `rusvel-tui`, …) rely on integration or have minimal unit tests. **Not** all `dept-*` crates have dedicated test binaries; **stub** domain engines (finance, product, …) often have fewer tests than core engines.

---

## 6. Next steps (from gaps + sprint intent)

1. Wire **GTM** / **OutreachSend** through the job pipeline with real engine logic and approvals.
2. Add **auth middleware** and a clear model for API keys/sessions if exposing beyond localhost.
3. Continue **Sprint** themes in [`../plans/sprints.md`](../plans/sprints.md) (reference only).
4. Re-run **§ How to re-verify** monthly or after large merges; append rows to [`verification-log-2026-03-27.md`](verification-log-2026-03-27.md) or a new dated log.

---

## 7. Historical (archived metrics)

Older snapshots (e.g. 48 crates, 124 handlers, 222 tests, 30 test binaries) referred to earlier tree sizes and counting methods. **Do not use** for current planning; use **§1** and **Metric definitions** above.

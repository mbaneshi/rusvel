# RUSVEL — Current State

> Last verified: 2026-03-23 (generated from live codebase inspection)

---

## 1. Numbers at a Glance

| Metric | Count |
|---|---|
| Workspace crates | 27 |
| Rust lines of code | 22,015 |
| Passing tests | 192 (0 failures, 1 ignored) |
| API route registrations | 44 |
| API modules | 16 (`agents`, `analytics`, `approvals`, `build_cmd`, `capability`, `chat`, `config`, `department`, `help`, `hook_dispatch`, `hooks`, `mcp_servers`, `routes`, `rules`, `skills`, `workflows`) |
| Frontend pages | 4 (`/`, `/chat`, `/settings`, `/dept/[id]`) |
| Frontend Svelte components | 37 (32 lib components + 4 pages + layout) |
| Frontend files (`.svelte` + `.ts`) | 50 |
| Frontend lines | 5,039 |

---

## 2. Build Health

| Check | Status |
|---|---|
| `cargo build` | Clean — 0 errors, 0 warnings |
| `cargo test` | All 192 pass, 0 failures |
| `svelte-check` | 0 errors, 0 warnings (561 files checked) |

---

## 3. What Works End-to-End

These features are wired from binary entry point through adapters to the frontend.

**API server startup** — `rusvel-app/main.rs` boots all adapters (SQLite WAL, Claude CLI LLM, EventBus, MemoryStore, ToolRegistry, JobQueue, AgentRuntime), builds ForgeEngine, seeds default data, spawns job worker, loads department registry, starts Axum on `:3000` with graceful shutdown.

**First-run wizard** — Interactive `cliclack` onboarding: detects Ollama, collects name/role, writes `profile.toml`, creates first session.

**Embedded frontend** — `rust-embed` compiles `frontend/build/` into the binary. Falls back through filesystem locations, then extracts embedded assets to temp dir.

**Department chat (SSE streaming)** — Parameterized `POST /api/dept/{dept}/chat` streams Claude CLI output via SSE. Includes:
- Three-layer config cascade: registry defaults + stored overrides + user context
- `@agent-name` mentions override model/instructions/tools from ObjectStore
- `/skill-name` resolution expands skill prompt templates inline
- `!build` command interceptor creates entities (agents, skills, rules, etc.) from natural language
- Rules loaded from ObjectStore and appended to system prompt
- MCP server config loaded per-department and passed to `claude -p`
- Hook dispatch fires asynchronously after `{engine}.chat.completed` events
- Conversation persistence via namespaced ObjectStore

**God agent chat** — `POST /api/chat` with SSE streaming, conversation history, profile context.

**Session management** — Full CRUD: create, list, get sessions via API + CLI.

**Mission planning** — `GET /api/sessions/{id}/mission/today` generates daily plans via ForgeEngine. CLI: `rusvel forge mission today`.

**Goal management** — CRUD for goals scoped to sessions.

**Event system** — EventBus emits + persists events; `GET /api/sessions/{id}/events` and `GET /api/dept/{dept}/events` query them.

**Entity CRUD endpoints** — Full REST for:
- Agents (`/api/agents`) — list, create, get, update, delete
- Skills (`/api/skills`) — list, create, get, update, delete
- Rules (`/api/rules`) — list, create, get, update, delete
- MCP servers (`/api/mcp-servers`) — list, create, update, delete
- Hooks (`/api/hooks`) — list, create, update, delete + event listing
- Workflows (`/api/workflows`) — list, create, get, update, delete + execute

**Approval flow (ADR-008)** — `GET /api/approvals` lists jobs in `AwaitingApproval` status; `POST .../approve` and `.../reject` change state. Job worker skips approval-gated jobs.

**Capability Engine** — `POST /api/capability/build` uses Claude with web tools to discover MCP servers/skills online, generates a bundle, and auto-installs entities. Also available inline via `!capability` prefix in department chat.

**Workflow execution** — `POST /api/workflows/{id}/run` runs multi-step agent pipelines sequentially, feeding each step's output into the next. Supports variable substitution via `{{key}}` templates.

**Hook dispatch** — Three hook types: `command` (shell), `http` (POST), `prompt` (claude -p). Fires asynchronously on event match with exact, wildcard, and suffix matching.

**Seed data** — On first run, seeds 5 default agents (rust-engine, svelte-ui, test-writer, content-writer, proposal-writer), 5 skills (Code Review, Blog Draft, Proposal Draft, Test Generator, Daily Standup), and 3 rules (Hexagonal Architecture, Human Approval Gate, Crate Size Limit).

**MCP server (stdio)** — `--mcp` flag dispatches to `rusvel_mcp::run_stdio()` for JSON-RPC over stdin/stdout.

**CLI surface** — `rusvel session create`, `rusvel forge mission today`, `--mcp` flag.

**Background job worker** — Polls job queue every 5 seconds, handles ContentPublish, HarvestScan, OutreachSend, CodeAnalyze (all placeholder implementations), marks complete/failed.

---

## 4. Built but Needs More Work

**Job worker handlers** — All job kinds (ContentPublish, HarvestScan, OutreachSend, CodeAnalyze) return placeholder JSON. No real engine logic is invoked.

**Engine crates (7 new engines)** — These 7 newer engines exist with domain types, port traits, and tests, but are not wired into the API or job worker:
- `distro-engine` (518 lines, 2 tests) — distribution/publishing
- `finance-engine` (391 lines, 3 tests) — invoicing/accounting
- `growth-engine` (349 lines, 3 tests) — growth analytics
- `infra-engine` (570 lines, 2 tests) — infrastructure/deployment
- `legal-engine` (546 lines, 2 tests) — contracts/compliance
- `product-engine` (370 lines, 3 tests) — product management
- `support-engine` (549 lines, 2 tests) — customer support

**Original engines** — `content-engine`, `harvest-engine`, `gtm-engine`, and `code-engine` have domain logic and tests but limited integration with the API beyond their department chat endpoints.

**TUI surface** — `rusvel-tui` (267 lines) has layout + widgets defined but is not launched from main.

**Frontend settings page** — Route exists (`/settings`) but scope of configuration UI is unclear.

**Frontend workflow builder** — `WorkflowBuilder.svelte` and `AgentNode.svelte` components exist but are not yet connected to a route.

**Analytics** — `GET /api/analytics` endpoint exists (82 lines) but unclear what data it aggregates.

**Help endpoint** — `POST /api/help` (109 lines) exists, likely AI-powered, but is a thin wrapper.

---

## 5. Not Built Yet

- **Real engine execution in job worker** — Job handler match arms are placeholders; no crate-level engine logic is invoked for async jobs.
- **Content publishing pipeline** — content-engine has domain types but no end-to-end publish flow through the API.
- **Harvest scan automation** — harvest-engine has scoring/proposal types but no automated source scanning.
- **Outreach sending** — gtm-engine has deal stages/sequences but no actual email/message sending.
- **Code analysis pipeline** — code-engine has parser/BM25 search but no triggered analysis flow.
- **Authentication/authorization** — `rusvel-auth` (139 lines) is in-memory from env vars; no middleware on API routes.
- **Frontend for approvals** — API exists but no UI for reviewing/approving jobs.
- **Frontend for workflows** — CRUD API + execution exist but no page wired to the builder components.
- **Frontend for agents/skills/rules management** — CRUD APIs exist but no dedicated management UI (entities are managed via `!build` in chat or direct API calls).
- **Billing/metering** — Cost tracking exists in chat events (`cost_usd`) but no aggregation or budget enforcement.
- **Multi-user** — Single-user only; no auth middleware, no user scoping.
- **Frontend embedding in release binary** — `rust-embed` is configured and works, but requires `frontend/build/` to exist at compile time.

---

## 6. Test Breakdown by Crate

| Crate | Tests |
|---|---|
| rusvel-llm | 44 |
| rusvel-core | 19 |
| forge-engine | 15 |
| rusvel-agent | 12 (+ 4 integration) |
| rusvel-db | 11 |
| rusvel-api | 11 |
| rusvel-memory | 8 |
| content-engine | 7 |
| harvest-engine | 7 |
| rusvel-jobs | 7 |
| code-engine | 6 |
| rusvel-config | 6 |
| gtm-engine | 5 |
| rusvel-auth | 5 |
| rusvel-tool | 5 |
| finance-engine | 3 |
| growth-engine | 3 |
| product-engine | 3 |
| rusvel-event | 3 |
| distro-engine | 2 |
| infra-engine | 2 |
| legal-engine | 2 |
| support-engine | 2 |
| **Total** | **192** |

Crates with 0 tests: `rusvel-app`, `rusvel-cli`, `rusvel-mcp`, `rusvel-tui`.

---

## 7. Crate Size Ranking (Rust lines)

| Crate | Lines |
|---|---|
| rusvel-api | 3,724 |
| rusvel-llm | 2,203 |
| rusvel-core | 2,063 |
| rusvel-db | 1,518 |
| content-engine | 1,158 |
| harvest-engine | 1,154 |
| rusvel-cli | 1,063 |
| rusvel-agent | 1,017 |
| code-engine | 903 |
| forge-engine | 901 |
| gtm-engine | 870 |
| rusvel-app | 670 |
| infra-engine | 570 |
| support-engine | 549 |
| legal-engine | 546 |
| distro-engine | 518 |
| rusvel-memory | 481 |
| rusvel-jobs | 405 |
| finance-engine | 391 |
| product-engine | 370 |
| growth-engine | 349 |
| rusvel-tool | 288 |
| rusvel-config | 285 |
| rusvel-mcp | 280 |
| rusvel-tui | 267 |
| rusvel-event | 178 |
| rusvel-auth | 139 |

Note: `rusvel-api` (3,724 lines) and `rusvel-llm` (2,203 lines) exceed the 2,000-line crate size limit from ADR. `rusvel-core` (2,063 lines) is borderline.

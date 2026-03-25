# RUSVEL — Current State

> Last verified: 2026-03-26 (generated from live codebase inspection)

---

## 1. Numbers at a Glance

| Metric | Count |
|---|---|
| Workspace crates | 48 |
| Rust lines of code | ~43,670 |
| Rust source files | 185 |
| Tests | 222 in 30 test binaries (0 failures) |
| API handler functions | 124 across 23 modules |
| Port traits (rusvel-core) | 19 (14 Port + 5 Store sub-traits) |
| Domain types (rusvel-core) | 82 |
| Departments | 12 |
| Department crates (dept-*) | 13 |
| Engines | 13 (all via DepartmentApp) |
| Registered tools | 22+ (10 built-in incl. tool_search + 12 engine) |
| MCP tools | 6 |
| Frontend pages | 12+ (`/`, `/chat`, `/database/schema`, `/database/tables`, `/database/sql`, `/dept/[id]`, `/flows`, `/knowledge`, `/settings`, `+layout`, `+error`) |
| Frontend Svelte components | ~40+ |

---

## 2. Build Health

| Check | Status |
|---|---|
| `cargo build` | Clean — 0 errors |
| `cargo test` | 222 tests in 30 binaries, 0 failures |

---

## 3. What Works End-to-End

These features are wired from binary entry point through adapters to the frontend.

**API server startup** — `rusvel-app/main.rs` boots all adapters (SQLite WAL, LLM with ModelTier routing + CostTracker, EventBus, MemoryStore, ScopedToolRegistry with 22+ tools, JobQueue, AgentRuntime with streaming, EmbeddingPort, VectorStore, TerminalPort), collects DepartmentApp instances from 13 dept-* crates, resolves dependencies via DepartmentManifest, calls register() in order, seeds default data, spawns job worker, starts Axum on `:3000` with graceful shutdown.

**First-run wizard** — Interactive `cliclack` onboarding: detects Ollama, collects name/role, writes `profile.toml`, creates first session.

**Embedded frontend** — `rust-embed` compiles `frontend/build/` into the binary. Falls back through filesystem locations, then extracts embedded assets to temp dir.

**Department chat (SSE streaming)** — Parameterized `POST /api/dept/{dept}/chat` streams via AgentRuntime (AgentEvent-based SSE). Includes:
- Three-layer config cascade: registry defaults + stored overrides + user context
- `@agent-name` mentions override model/instructions/tools from ObjectStore
- `/skill-name` resolution expands skill prompt templates inline
- `!build` command interceptor creates entities from natural language
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

**Capability Engine** — `POST /api/capability/build` uses Claude with web tools to discover MCP servers/skills online, generates a bundle, and auto-installs entities. Also available inline via `!build` prefix in department chat.

**Workflow execution** — `POST /api/workflows/{id}/run` runs multi-step agent pipelines sequentially, feeding each step's output into the next. Supports variable substitution via `{{key}}` templates.

**Hook dispatch** — Three hook types: `command` (shell), `http` (POST), `prompt` (claude -p). Fires asynchronously on event match with exact, wildcard, and suffix matching.

**Domain engines wired** — CodeEngine (analyze, search, metrics), ContentEngine (draft, from-code, publish with approval), HarvestEngine (scan, score, propose, pipeline), FlowEngine (DAG execution) all instantiated in main.rs with real domain logic.

**Job queue worker** — Background worker polls every 5 seconds, handles CodeAnalyze, ContentPublish, HarvestScan jobs via real engines with session_id scoping.

**RusvelBase (database browser)** — Schema introspection via rusvel-schema crate. API routes: `/api/db/tables`, `/api/db/tables/{table}/schema`, `/api/db/tables/{table}/rows`, `/api/db/sql`. Frontend UI at `/database/*`.

**Knowledge/RAG** — Vector-backed knowledge base with 5 API routes at `/api/knowledge`. Uses rusvel-embed + rusvel-vector (LanceDB).

**Code-to-Content pipeline** — `POST /api/dept/content/from-code` generates content items from code analysis results.

**MCP server (stdio)** — `--mcp` flag dispatches to `rusvel_mcp::run_stdio()` for JSON-RPC over stdin/stdout. 6 user-callable tools.

**MCP client** — rusvel-mcp-client crate for connecting to external MCP servers and discovering tools.

**Built-in tools** — 9 tools in rusvel-builtin-tools for the agent execution pipeline.

**CLI surface** — 3-tier: one-shot commands (11 departments + session/forge), REPL shell (reedline), TUI dashboard (ratatui).

**Seed data** — On first run, seeds 5 default agents, 5 skills, and 3 rules.

**Flow Engine** — DAG workflow executor using petgraph. 3 node types (code, condition, agent). 7 API routes at `/api/flows`.

---

## 4. Built but Needs More Work

**8 domain engines (growing)** — GTM, Finance, Product, Growth, Distro, Legal, Support, Infra exist with domain types and DepartmentApp wrappers. Chat works for all 12 departments via the DepartmentApp pattern.

**Authentication/authorization** — `rusvel-auth` is in-memory from env vars; no middleware on API routes.

**OutreachSend jobs** — GTM engine not wired, job handler is placeholder.

---

## 5. Test Breakdown by Crate

222 tests in 30 test binaries across the workspace, 0 failures. Distribution varies by crate;
highest concentration in rusvel-llm, forge-engine, rusvel-api, rusvel-db, harvest-engine,
rusvel-core (including DepartmentManifest + RegistrationContext tests), and rusvel-agent.

Crates with 0 tests: `rusvel-app`, `rusvel-cli`, `rusvel-mcp`, `rusvel-tui`, `rusvel-event`,
`rusvel-memory`, `rusvel-builtin-tools`, `rusvel-jobs`, `rusvel-auth`, `rusvel-deploy`,
`rusvel-vector`, `rusvel-terminal`, `rusvel-engine-tools`, all 13 dept-* crates,
and the 8 stub engines.

---

## 6. New Since Last Update (2026-03-24)

- **ADR-014: DepartmentApp pattern** — `EngineKind` enum removed, replaced by `DepartmentApp` trait + `DepartmentManifest`. Each department is a self-contained `dept-*` crate.
- **13 dept-* crates** — `dept-forge`, `dept-code`, `dept-content`, `dept-harvest`, `dept-flow`, `dept-gtm`, `dept-finance`, `dept-product`, `dept-growth`, `dept-distro`, `dept-legal`, `dept-support`, `dept-infra`
- **rusvel-engine-tools** — 12 engine-specific tools for agent execution
- **rusvel-terminal** — TerminalPort adapter (Terminal Phase 1)
- **AgentRuntime streaming** — `run_streaming()` returns `mpsc::Receiver<AgentEvent>` with TextDelta, ToolCall, ToolResult, Done, Error events. Replaces ClaudeCliStreamer.
- **ScopedToolRegistry** — Per-department tool scoping with deferred loading and `tool_search` meta-tool
- **ModelTier routing** — Quick/Standard/Premium model selection based on task complexity
- **CostTracker** — Per-session token usage and cost tracking
- **LlmStreamEvent** — `stream()` method on `LlmPort` for incremental LLM output
- **22+ registered tools** — 10 built-in (incl. tool_search) + 12 engine-specific
- **TerminalPort** — New port trait in rusvel-core
- **Approval UI shipped** — Frontend ApprovalCard, ApprovalQueue components
- **ToolCallCard** — Frontend component for displaying tool calls in chat
- **Manifest-aligned navigation** — Frontend nav driven by DepartmentManifest data
- **Crate count**: 34 → 48
- **Rust lines**: ~34k → ~43,670
- **Source files**: → 185
- **Tests**: 118 → 222 in 30 binaries (0 failures)
- **API handlers**: 79 → 124 across 23 modules
- **Port traits**: 14 → 19 (14 Port + 5 Store)

## 7. History: Changes from 2026-03-23 Audit to 2026-03-24

- **rusvel-schema** — New crate for database schema introspection
- **rusvel-builtin-tools** — 9 built-in tools for agent execution
- **rusvel-mcp-client** — Connect to external MCP servers
- **rusvel-deploy** — Deployment port adapter
- **rusvel-embed** — Text embedding adapter
- **rusvel-vector** — LanceDB vector store for semantic search
- **flow-engine** — DAG workflow engine with petgraph
- **Database browser UI** — `/database/schema`, `/database/tables`, `/database/sql` routes
- **Knowledge/RAG routes** — 5 API routes at `/api/knowledge`
- **Code-to-content pipeline** — `/api/dept/content/from-code`
- **Job queue session_id scoping** — Jobs now use `job.session_id` for proper context
- **Engine tab in department UI** — Department-specific engine actions in frontend
- **Crate count**: 27 → 34
- **Rust lines**: ~22k → ~34k
- **API routes**: 44 → 79
- **API modules**: 16 → 22
- **Port traits**: 10 → 19

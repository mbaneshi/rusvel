# RUSVEL — Current State

> Last verified: 2026-03-24 (generated from live codebase inspection)

---

## 1. Numbers at a Glance

| Metric | Count |
|---|---|
| Workspace crates | 34 |
| Rust lines of code | ~34,286 |
| Total tests (#[test]) | 118 |
| Tests passing | 103 (3 failing in rusvel-api code_to_content) |
| API route registrations | 79 |
| API modules | 22 (`agents`, `analytics`, `approvals`, `build_cmd`, `capability`, `chat`, `config`, `db_routes`, `department`, `engine_routes`, `flow_routes`, `help`, `hook_dispatch`, `hooks`, `knowledge`, `mcp_servers`, `routes`, `rules`, `skills`, `system`, `visual_report`, `workflows`) |
| Port traits (rusvel-core) | 19 (18 in ports.rs + Engine trait) |
| Domain types (rusvel-core) | 82 |
| Departments | 12 |
| Engines (wired / stub) | 5 / 8 |
| MCP tools | 6 |
| Frontend pages | 12 (`/`, `/chat`, `/database/schema`, `/database/tables`, `/database/sql`, `/dept/[id]`, `/flows`, `/knowledge`, `/settings`, `+layout`, `+error`) |
| Frontend Svelte components | ~40+ |

---

## 2. Build Health

| Check | Status |
|---|---|
| `cargo build` | Clean — 0 errors |
| `cargo test` | 103 pass, 3 fail (code_to_content integration tests) |

---

## 3. What Works End-to-End

These features are wired from binary entry point through adapters to the frontend.

**API server startup** — `rusvel-app/main.rs` boots all adapters (SQLite WAL, Claude CLI LLM, EventBus, MemoryStore, ToolRegistry, JobQueue, AgentRuntime, EmbeddingPort, VectorStore), builds ForgeEngine + CodeEngine + ContentEngine + HarvestEngine + FlowEngine, seeds default data, spawns job worker, loads department registry, starts Axum on `:3000` with graceful shutdown.

**First-run wizard** — Interactive `cliclack` onboarding: detects Ollama, collects name/role, writes `profile.toml`, creates first session.

**Embedded frontend** — `rust-embed` compiles `frontend/build/` into the binary. Falls back through filesystem locations, then extracts embedded assets to temp dir.

**Department chat (SSE streaming)** — Parameterized `POST /api/dept/{dept}/chat` streams Claude CLI output via SSE. Includes:
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

**8 domain stub engines** — GTM, Finance, Product, Growth, Distro, Legal, Support, Infra exist with domain types and tests, but engine-specific logic isn't invoked via API or jobs. Chat works for all departments via generic agent.

**Approval workflow UI** — API endpoints exist but no frontend UI for reviewing/approving jobs.

**Authentication/authorization** — `rusvel-auth` is in-memory from env vars; no middleware on API routes.

**OutreachSend jobs** — GTM engine not wired, job handler is placeholder.

---

## 5. Test Breakdown by Crate

| Crate | #[test] |
|---|---|
| rusvel-llm | ~40 |
| forge-engine | 15 |
| rusvel-api (build_cmd + knowledge + integration) | 15+ |
| rusvel-db | 14 |
| harvest-engine | 12 (source) |
| rusvel-agent (persona) | 6 |
| rusvel-core (domain + config + id + registry) | 19 |
| rusvel-config | 6 |
| code-engine | 4 |
| content-engine | 3 |
| rusvel-mcp-client | 3 |
| rusvel-embed | 1 |
| flow-engine | 1 |
| rusvel-schema | 4 |
| **Total** | **118** |

Crates with 0 tests: `rusvel-app`, `rusvel-cli`, `rusvel-mcp`, `rusvel-tui`, `rusvel-event`, `rusvel-memory`, `rusvel-tool`, `rusvel-builtin-tools`, `rusvel-jobs`, `rusvel-auth`, `rusvel-deploy`, `rusvel-vector`, and all 8 stub engines.

---

## 6. New Since Last Audit (2026-03-23)

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

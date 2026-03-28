# How RUSVEL Works

A narrative for a senior developer joining the codebase: how data and control move from process start through HTTP (or CLI/MCP), into agents and engines, and back out again. This is not API reference; it names real types and files so you can jump into the tree. Read **`crates/rusvel-app/src/main.rs`** top-down once: every adapter choice and spawn is there, and every other module becomes “how this port is used.”

---

## 1. What RUSVEL Is

RUSVEL is a single Rust binary that hosts an AI-native “virtual agency”: many **departments** (forge, code, content, harvest, GTM, finance, growth, and the rest of the `installed_departments()` list) share one SQLite-backed store, one job queue, one agent runtime, and one web UI. A solo operator (or small team) works in **sessions** (workspaces), chats with department-scoped or global assistants, runs background jobs (publish content, scan opportunities, cron briefings, outreach sequences), and keeps goals, events, and arbitrary JSON documents in one place. The same binary can act as an HTTP API, an MCP server (`--mcp` or HTTP MCP alongside the API), a one-shot or REPL CLI, or a ratatui dashboard (`--tui`), because **`main.rs` chooses the surface after shared infrastructure is up**.

The architecture is hexagonal: **engines and departments depend only on port traits in `rusvel-core`**, while adapters (`rusvel-db`, `rusvel-llm`, `rusvel-agent`, `rusvel-channel`, …) implement those ports and are wired only in the composition root. Shared vocabulary—**`Session`**, **`Event`**, **`Job`**, **`Goal`**, **`Content`**, **`AgentConfig`**, **`LlmMessage`**, tool definitions—lives in **`crates/rusvel-core/src/domain.rs`**. That file is the conceptual dictionary for the whole system; when you read a handler and see a domain type, it almost certainly originates there.

---

## 2. Boot Sequence

When you run `cargo run` (or the installed `rusvel` binary without a subcommand), `main` in `crates/rusvel-app/src/main.rs` performs a fixed orchestration.

The order is deliberate: anything that **`DepartmentApp::register`** might touch—**`JobPort`**, **`StoragePort`**, **`AgentPort`**, **`EventPort`**—must exist before **`boot_departments`**. Conversely, **embedding and LanceDB** are often initialized only when building **`AppState`** for the web server, because departments boot with **`None`** for those ports in **`main`** and RAG is an enhancement layered onto chat later. **`harvest_engine.configure_rag(embedding, vector_store)`** runs after those adapters succeed so opportunity scoring can attach semantic hints without blocking core boot.

Tracing initializes first, then the data directory (typically `~/.rusvel`) is ensured. The primary database opens via `Database::open`, which enables SQLite WAL, foreign keys, and runs embedded migrations from `crates/rusvel-db/src/migrations.rs`—creating tables for events, objects, sessions, runs, jobs, metrics, and related indexes.

LLM access is built as a `MultiProvider` with providers such as `ClaudeCliProvider` and `CursorAgentProvider`, then wrapped in `CostTrackingLlm` with `MetricStore` (the same `Database` implements `MetricStore`) so usage can be attributed and tracked. The `EventBus` is constructed with an `EventStore` adapter on that database. Session-scoped semantic memory uses `MemoryStore` (a separate SQLite file for FTS). A `ToolRegistry` is created, `rusvel_builtin_tools::register_all` and `tool_search::register` populate it, and it is exposed as `Arc<dyn ToolPort>`. The job queue is **the same database** implementing `JobPort`—ADR-003’s single queue.

`AgentRuntime::new(llm, tools, memory)` constructs the object that implements `AgentPort` (ADR-009: engines talk to the agent abstraction, not `LlmPort` directly). Sessions are bridged with `SessionAdapter`, which implements `SessionPort` on top of `StoragePort`.

**Department boot (ADR-014)** comes next: `boot::installed_departments()` returns a vector of `Box<dyn DepartmentApp>`, and `boot::boot_departments` runs three phases—collect manifests, `validate_unique_ids` and `resolve_dependency_order` (topological sort on `DepartmentManifest::depends_on`), then for each department in order calls `DepartmentApp::register(&mut ctx)`. The `RegistrationContext` in `crates/rusvel-core/src/department/context.rs` holds shared ports plus **registrars** for tools, event handlers, and job handlers. `finalize` produces `DepartmentsBootArtifacts`: a `DepartmentRegistry` (for API/UI), `Vec<ToolRegistration>`, `Vec<EventSubscription>`, a `HashMap` of job handlers, and any failed department IDs.

`main` keeps the registry, event subscriptions, and failure list; it calls `boot::spawn_department_event_dispatch(events, event_subscriptions)`, which subscribes to the broadcast `EventBus` and, for each published `Event` whose `kind` matches a subscription, spawns the department’s async handler.

After boot, **domain engines** are still constructed explicitly for API and worker paths: `ForgeEngine::new`, `CodeEngine`, `ContentEngine` (with platform adapters), `HarvestEngine` (with optional browser/CDP), `FlowEngine`, `GtmEngine`. Built-in tools gain browser, delegate, terminal, and artifact helpers; `rusvel_engine_tools` registers harvest, content, and code tools on the same `ToolRegistry`. Defaults are seeded; `TriggerManager` loads `EventTrigger` rows from the object store and subscribes to the bus.

A **background task** is spawned: `tokio::spawn` around a loop that calls `JobPort::dequeue(&[])`, matches on `JobKind`, and either runs engine methods or emits events—skipping rows in `JobStatus::AwaitingApproval` (ADR-008) with a short sleep. A `watch` channel supports graceful shutdown alongside the HTTP server.

**`WebhookReceiver`** and **`CronScheduler`** are constructed after **`StoragePort`** and **`JobPort`** are available so inbound HTTP signatures or timer ticks enqueue **`Job`** rows or emit **`Event`** records through the same ports the UI uses—there is no second queue. For knowledge, **`spawn_knowledge_indexer`** connects **`EventBus`** activity to optional re-indexing when embedding and vector ports exist; document ingestion APIs populate **`ObjectStore`** and Lance paths under the data directory.

Finally the CLI is parsed. With no subcommand and without `--mcp` / `--tui`, the binary builds `AppState` (`crates/rusvel-api/src/lib.rs`), optionally initializes `FastEmbedAdapter` and `LanceVectorStore` for RAG, attaches the department registry, and calls `rusvel_api::build_router_with_frontend` then `start_server` on `127.0.0.1:3000`. Static assets come from `frontend/build`, `~/.rusvel/frontend`, a path next to the binary, or `extract_embedded_frontend()` from rust-embed. `start_server` binds TCP and uses Axum’s graceful shutdown; a second Ctrl+C or a timer after shutdown can force exit so stuck SSE streams do not block forever.

**CLI and MCP paths** reuse the same database, agent runtime, and forge engine instances where applicable: `rusvel_cli::run` receives engine handles for tier-one department commands; `RusvelMcp::new` wraps Forge plus `SessionPort` for JSON-RPC tools over stdio or, with `--mcp-http`, routes nested under `rusvel_mcp::http::nest_mcp_http`. The first-run wizard runs only when there is no profile file, stdin is a TTY, and no subcommand flags demand another mode—so automation and servers skip it.

---

## 3. The Port System

`crates/rusvel-core/src/ports.rs` defines the boundaries. **`StoragePort`** fans out to five store traits: **`EventStore`**, **`ObjectStore`**, **`SessionStore`**, **`JobStore`**, **`MetricStore`**—all implemented by `rusvel_db::Database` in `store.rs` using `spawn_blocking` around rusqlite. **`LlmPort`** is generate/embed/list_models; **`AgentPort`** is the higher-level run API used by engines and HTTP handlers. **`ToolPort`** lists and invokes tools. **`EventPort`** publishes and queries **`Event`** records. **`JobPort`** enqueues, dequeues, completes, fails, and approval-holds jobs. **`MemoryPort`** backs retrieval-augmented patterns in the agent. **`SessionPort`**, **`ConfigPort`**, **`AuthPort`**, **`EmbeddingPort`**, **`VectorStorePort`**, **`DeployPort`**, **`TerminalPort`**, **`ChannelPort`**, and **`BrowserPort`** cover sessions, TOML config, credentials, embeddings, LanceDB search, deploy hooks, PTY multiplexing, outbound notify, and CDP.

Engines in `*-engine` crates take `Arc<dyn …Port>` in constructors and never reference `rusvel-db` or `rusvel-llm` by type. That rule keeps domain logic testable and swappable. The **composition root** (`main.rs`) is the only place that chooses concrete adapters and passes them into engines, departments, and `AppState`.

**`ToolRegistry`** in `rusvel-tool` implements **`ToolPort`** with interior mutability: definitions and handlers live in a `RwLock<HashMap>`. Tools are **`ToolDefinition`** values plus async closures returning **`ToolResult`**. **`check_permission`** resolves **`ToolPermission`** rules per department id and glob patterns so the same registry can enforce different modes (auto, confirm, deny) per surface. **`ScopedToolRegistry`** (same crate) can restrict which tool names a given department sees, aligning with “defer loading” patterns and the **`tool_search`** meta-tool registered from builtins.

---

## 4. Department Lifecycle

`DepartmentApp` in `crates/rusvel-core/src/department/app.rs` requires **`manifest()`** (pure metadata: id, UI, prompts, declared tools/routes/commands in `DepartmentManifest`) and **`register(&mut RegistrationContext)`** (async side effects: push tools, `event_handlers.on(...)`, `job_handlers.handle(...)`, and optionally stash an internal `Arc` to an engine). **`shutdown`** defaults to a no-op.

Boot order is deterministic: manifests are validated for unique ids; **`resolve_dependency_order`** uses Kahn’s algorithm and errors on cycles. **`ForgeDepartment`** in `dept-forge` illustrates the pattern: in `register` it builds `ForgeEngine::new` with ports from `ctx`, stores it in a `OnceLock`, and registers tools like `forge.mission.today` via `ctx.tools.add` with JSON Schema and a `ToolHandler` closure that calls `eng.mission_today(&sid)`. The static **`forge_manifest()`** in `dept-forge/src/manifest.rs` fills UI tabs, quick actions, persona contributions, and route/command metadata.

`RegistrationContext::finalize` collapses registrars into **`DepartmentsBootArtifacts`**. The **`DepartmentRegistry`** drives `GET /api/departments` and the SvelteKit department routes. **`EventSubscription`** entries bridge **`EventBus`** broadcasts to department code: **`spawn_department_event_dispatch`** loops on **`event_bus.subscribe()`**, compares **`event.kind`** to each subscription’s string, and **`tokio::spawn`** the handler so slow department work does not block the bus.

Tool and job handler maps are part of the same artifact. `context.rs` documents that the host should transfer **`Vec<ToolRegistration>`** onto the live **`ToolPort`**. In the current binary, **`main.rs`** destructures **`DepartmentsBootArtifacts`** with `..` for tools and job handlers, then registers **built-in**, **engine**, and **artifact** tools on the shared **`ToolRegistry`** afterward. Integration tests in **`boot.rs`** still assert that **`artifacts.tools`** contains a large, deduplicated set of department-declared tools—useful when evolving the composition root so runtime tool lists stay aligned with ADR-014 registrations.

**`dept-messaging`** is registered last in **`installed_departments()`** so outbound channel behavior sits after core departments, matching the comment in `boot.rs`.

---

## 5. Request Flow: Department Chat

The parameterized route lands in **`dept_chat`** in `crates/rusvel-api/src/department.rs`. The handler resolves the department id against **`AppState.registry`** (`validate_dept`). **Layered config** is loaded from the object store key `dept_config` / department id, then merged with registry defaults and **`UserProfile`** via **`resolve_dept_config`**, producing a **`ResolvedConfig`** and ultimately an outgoing **`DepartmentConfig`** shape for the client.

Messages are namespaced per department (`dept_msg_{engine}`) in **`ObjectStore`**: history is loaded with **`load_namespaced_history`**, the user message is stored with **`store_namespaced_message`**. Special paths short-circuit the LLM: **`!build`** flows through **`build_cmd`**, **`/skill`** through **`skills::resolve_skill`**, and **`@agent`** mentions can override prompts and tools from **`agents`** objects. **`rules::load_rules_for_engine`** appends enabled rules to the system prompt. If **`session_id`** is present, **`context_pack_for_chat`** uses **`assemble_context_pack`** (session name, goals from **`ForgeEngine::list_goals`**, recent **`EventPort::query`**, job/harvest summaries) with a small TTL cache in **`ContextPackCache`**. Optional RAG injects **`EmbeddingPort::embed_one`** and **`VectorStorePort::search`** snippets into the prompt. Department-specific blocks document real HTTP endpoints (code, content, harvest, forge, GTM).

The runtime path mirrors global chat: **`AgentConfig`** is built with **`parse_model_ref`**, **`RUSVEL_META_DEPARTMENT_ID`**, optional **`RUSVEL_META_MODEL_TIER`**, resolved tools and budget. **`AgentRuntime::create`** returns a **`RunId`**; **`run_streaming`** yields **`AgentEvent`** values on an mpsc channel. Axum returns **`Sse`**; **`sse_helpers::prelude_stream`** emits **`AgUiEvent::RunStarted`**, then each event is mapped through **`other_event_sse`** or, on **`AgentEvent::Done`**, **`run_completed_sse`**. A spawned task persists the assistant message, emits **`Event`** with kind **`{dept}.chat.completed`**, and calls **`hook_dispatch::dispatch_hooks`** for configured hooks.

God-agent chat in **`crates/rusvel-api/src/chat.rs`** is the same mechanical pattern with **`POST /api/chat`**, **`load_history`** / **`store_message`** under a global conversation namespace, **`load_and_migrate_chat_config`**, and metadata department id **`global`**.

**`build_router`** in **`rusvel-api/src/lib.rs`** assembles dozens of routes onto **`AppState`**: health, webhooks, cron, forge briefs, sessions, chat, config, departments, RusvelBase DB browser, knowledge, flows, jobs, approvals, engine-specific **`/api/dept/...`** modules, hooks, agents, skills, rules, MCP server CRUD, workflows, terminal, browser, and system utilities. Middleware stacks CORS, optional rate limiting, bearer auth from env, HTTP tracing, and request-id injection. Understanding department chat does not require memorizing every route; it requires knowing that **`AppState`** carries **`registry`**, **`agent_runtime`**, **`storage`**, **`sessions`**, **`events`**, **`jobs`**, optional engines, embedding/vector ports, and **`failed_departments`** from boot for diagnostics.

---

## 6. The Agent Runtime

`AgentRuntime` in `crates/rusvel-agent/src/lib.rs` owns **`Arc<dyn LlmPort>`**, **`Arc<dyn ToolPort>`**, **`Arc<dyn MemoryPort>`**, and a **`RwLock<HashMap<RunId, RunState>>`**. **`create`** registers **`AgentConfig`**; **`run_streaming`** flips status to running, spawns **`run_streaming_loop`** on a Tokio task, and sends **`AgentEvent::TextDelta`**, **`ToolCall`**, **`ToolResult`**, **`StateDelta`**, **`Done`**, or **`Error`** on the channel.

The loop builds **`LlmRequest`** from messages and **`ToolDefinition`** list (JSON Schema as `input_schema`), calls the LLM, parses **`Part::ToolCall`**, runs **`ToolPort::invoke`** after **pre-tool hooks** (`HookDecision::Allow | Modify | Deny`), emits streaming events, appends tool results, and repeats up to **`MAX_ITERATIONS`**. Compaction trims long histories (**`COMPACT_THRESHOLD`** / **`COMPACT_KEEP_RECENT`**). **`merge_llm_request_metadata`** injects **`RUSVEL_META_SESSION_ID`** for attribution. **`sse_helpers`** and **`agent_event_to_ag_ui`** translate to AG-UI-shaped SSE event names (`RUN_STARTED`, `TEXT_DELTA`, …).

**Example tool surface:** **`rusvel-builtin-tools/src/file_ops.rs`** registers **`read_file`** with **`ToolRegistry::register_with_handler`**, validates paths under the current working directory, and returns structured errors as **`RusvelError::Tool`**. Engine tools follow the same port contract but call **`CodeEngine`**, **`ContentEngine`**, or **`HarvestEngine`** methods inside their closures. From the LLM’s perspective every tool is a name, description, and schema; from the runtime’s perspective each maps to an async function on the shared registry.

---

## 7. Background Jobs

`JobPort` abstracts the queue; production uses SQLite via **`Database`**. The **`rusvel-jobs`** crate’s **`JobQueue`** is an in-memory alternative for tests. In **`main.rs`**, the worker loop calls **`dequeue`**, skips **`AwaitingApproval`**, then matches **`JobKind`**: **`CodeAnalyze`** → **`CodeEngine::analyze`**, **`ContentPublish`** → **`ContentEngine::publish`**, **`HarvestScan`** → **`HarvestEngine::scan`**, **`ProposalDraft`** may call **`hold_for_approval`**, **`ScheduledCron`** either runs **`ForgeEngine::generate_brief`** for the daily briefing kind or emits a generic cron **`Event`**, **`Custom("forge.pipeline")`** runs **`ForgeEngine::orchestrate_pipeline`** with **`HarvestContentPipelineRunner`**, **`OutreachSend`** delegates to **`GtmEngine::outreach`** with SMTP or mock email. Success paths call **`complete`**; failures call **`fail`**. Polling sleeps five seconds unless the shutdown watch fires.

Departments can register **`job_handlers`** in boot artifacts for string job kinds; the inline worker in **`main.rs`** is the central place where **`JobKind` enum variants** are executed today. **`dept-content`**, for example, registers a handler for **`content.publish`** in **`ctx.job_handlers.handle`**—the pattern is ready for a future unified dispatcher; until then, overlapping work may be triggered both from API routes and from this worker depending on **`JobKind`**.

**`CronScheduler`** ticks on an interval task spawned from **`main`**, enqueueing **`ScheduledCron`** jobs whose payloads carry **`event_kind`**, optional forge briefing hooks, and schedule ids—so scheduled work re-enters the same queue as user-initiated jobs.

---

## 8. The Frontend

SvelteKit 5 lives under **`frontend/`**. **`src/routes/+page.svelte`** is the dashboard: it subscribes to **`activeSession`**, **`departments`**, and pulls goals, events, analytics, and briefs via **`$lib/api`** helpers against **`/api/*`**. Department navigation uses registry-driven links (**`deptHref`**, **`resolveDeptId`**) so new booted departments appear without a static route table per id.

**`src/routes/dept/[id]/chat/+page.svelte`** resolves the department from **`page.params.id`** against the store-backed **`DepartmentDef`** list, requires an active session, and renders **`DepartmentChat`** inside **`{#key dept.id}`** so switching departments remounts chat state. That component (under **`$lib/components/chat`**) calls the API with the session id in the JSON body (alongside **`message`** and optional **`conversation_id`** / **`model_tier`**), consumes the SSE stream, and updates UI incrementally on **`TEXT_DELTA`**, tool start/end events, and **`RUN_COMPLETED`**. Errors surface through the same toast patterns as the rest of the app.

The API router in **`build_router_with_frontend`** mounts **`ServeDir`** for static files and falls back to **`index.html`** for client-side navigation, which is why deep links into **`/dept/forge/chat`** work when the server is the Rust binary rather than **`pnpm dev`** alone.

---

## 9. Data Model

**Sessions** (`Session`, `SessionId`, `SessionKind`, `SessionConfig`) live in **`SessionStore`** with **`metadata: serde_json::Value`** (ADR-007). **Events** append to **`EventStore`** with string **`kind`**, **`payload`**, optional **`session_id`**, **`run_id`**, **`source`**, and metadata. **Objects** are typed buckets in **`ObjectStore`** (`kind`, `id`, JSON document)—agents, skills, rules, department configs, namespaced chat transcripts, goals, tasks, opportunities, etc. **Jobs** carry **`JobKind`**, **`JobStatus`**, payload, schedule, retry fields, and optional approval metadata. **Metrics** (e.g. model spend) go through **`MetricStore`**. **Runs** record agent executions. The migration file is the ground truth for first-principles schema; later migrations extend the embedded list in **`migrations.rs`**.

The split is pragmatic: **relational tables** back query-heavy, structured entities (sessions, events, jobs, runs, metrics) while **objects** hold flexible JSON documents keyed by convention (`"agents"`, `"dept_config"`, **`dept_msg_{id}`** for chat). New features often start as object kinds to ship quickly, then promote fields into typed tables if indexing or joins demand it. **`Event.kind`** staying a string (ADR-005) means departments can emit new kinds without a core enum change—consumers match on prefixes or exact strings in subscriptions and hooks.

**`ForgeEngine`** ( **`forge-engine/src/lib.rs`** ) holds **`AgentPort`**, **`EventPort`**, **`MemoryPort`**, **`StoragePort`**, **`JobPort`**, **`SessionPort`**, and **`ConfigPort`** and implements the core **`Engine`** trait for capability reporting. Mission planning (**`mission_today`**, goals, reviews), pipeline orchestration, personas, and safety guardrails live in submodules; HTTP routes like **`/api/brief`** call into this engine through **`engine_routes`**, while tools registered from **`dept-forge`** call the same engine methods the UI would trigger via REST. That duplication of “entrypoints” (chat tools, REST, CLI, MCP) over one engine is intentional: each surface is thin; the engine remains the semantic center for forge behavior.

---

## 10. Adding a New Department

The recipe matches ADR-014: (1) A domain **`your-engine`** crate that depends only on **`rusvel-core`** ports and implements domain logic. (2) A **`dept-your`** wrapper crate implementing **`DepartmentApp`**: **`manifest()`** returning **`DepartmentManifest`** (id, **`depends_on`** if needed, UI, prompts, contributions), and **`register`** that instantiates the engine with **`ctx.agent`**, **`ctx.storage`**, etc., then calls **`ctx.tools.add`**, **`ctx.event_handlers.on`**, **`ctx.job_handlers.handle`** as appropriate. (3) Add **`Box::new(dept_your::YourDepartment::new())`** to **`installed_departments()`** in **`boot.rs`** in dependency-safe order. The API and frontend pick up the new id from **`GET /api/departments`** without hardcoding a new route tree for chat—parameterized **`/api/dept/{dept}/...`** handlers already exist.

---

## 11. Key Design Decisions

**ADR-007** — **`metadata` on domain types** as **`serde_json::Value`** allows forward-compatible fields without migrations for every new flag.

**ADR-009** — Engines use **`AgentPort`** ( **`AgentRuntime`** ) instead of **`LlmPort`**, centralizing tool loops, hooks, and run lifecycle.

**ADR-014** — **Departments as apps**: **`DepartmentManifest`** + **`DepartmentApp::register`** + **`RegistrationContext`** unify how capabilities join the host; **`DepartmentRegistry`** replaces ad hoc department lists for the UI.

Together with **ADR-003** (single job queue), **ADR-005** (event kinds as strings), and **ADR-008** (approval gates on sensitive jobs), these decisions explain most day-to-day constraints: where to add code (engine vs dept vs `rusvel-api` handler), how to thread session and cost metadata, and why **`main.rs`** reads like a wiring diagram while **`rusvel-core`** stays lean.

**ADR-010** (engines consume ports only) and the **“no engine imports adapters”** rule from project conventions reinforce the same boundary as ADR-014 for `dept-*` crates: they may depend on their engine and **`rusvel-core`**, not on sibling departments—cross-cutting reactions use **`EventPort`** with string **`kind`** values defined by each engine.

**ADR-008** shows up concretely in the job worker: rows in **`JobStatus::AwaitingApproval`** are never executed until a human approves via the approvals API and the job transitions back to a runnable state. **`ProposalDraft`** and **`OutreachSend`** use **`hold_for_approval`** so generated outreach or proposals do not leave the system without an explicit gate. Content publishing and similar flows share the same SQLite **`jobs`** table as analytics and UI listing endpoints, so operators see one pipeline rather than per-engine silos.

When you change behavior, ask which port is the authority: if it is durable state, it probably flows through **`StoragePort`** or **`JobPort`**; if it is ephemeral streaming, it flows through **`AgentPort`** and SSE; if it is cross-module reactions, prefer **`EventPort`** with a documented **`kind`** constant in the owning engine crate.

For verification, **`cargo test`** exercises engines with stub ports, **`rusvel-app`** boot tests validate department counts and tool registration invariants, and API tests cover handlers without starting a full LLM. That layered testing matches the hexagonal split: cheap tests near **`rusvel-core`**, heavier integration toward **`rusvel-api`** and **`rusvel-app`**. The frontend adds **`pnpm check`** and Playwright flows for UI regressions separately from Rust.

---

*Generated from the codebase layout described in `docs/prompts/generate-app-narrative.md`. For ADR text and roadmap context, see `docs/design/decisions.md` and `docs/design/architecture-v2.md`.*

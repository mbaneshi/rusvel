# RUSVEL: From Boot to Shutdown — A System Narrative

This document is written for a senior developer onboarding to the codebase. It follows data and control flow through the running process, names real types and file paths, and stays away from exhaustive API listing. Read `crates/rusvel-app/src/main.rs` once top-to-bottom alongside this text; almost every architectural choice is visible there.

---

## 1. What RUSVEL Is

RUSVEL is a single Rust binary that behaves as an AI-native “virtual agency” for a solo builder or small team. One process owns a shared SQLite database, a central job queue, an agent runtime that wraps language models and tools, and a SvelteKit frontend (served as static assets or embedded into the binary). Work is organized in **sessions**—logical workspaces with their own goals, events, and queued work. **Departments** (forge, code, content, harvest, flow, GTM, finance, and others registered in `boot::installed_departments`) are not separate processes; they are **DepartmentApp** implementations that declare manifests, register tools and event subscriptions at boot, and drive how the UI and HTTP API partition chat, config, and capabilities. The same binary can expose an Axum HTTP server (default), a Clap-driven CLI (one-shot commands, `shell` REPL, or `--tui` dashboard), or an MCP server over stdio (`--mcp`) or HTTP (`--mcp_http`). The product goal is to keep planning, execution, publishing, CRM-style outreach, and observability in one loop, with human approval gates where automation would otherwise touch the outside world.

---

## 2. Boot Sequence

When you run `cargo run` without a subcommand, `main` in `crates/rusvel-app/src/main.rs` builds the world in a strict order so that every port an adapter or department might need already exists.

Tracing initializes via `tracing_subscriber` with an `EnvFilter`. The data directory defaults to `~/.rusvel` (`rusvel_dir()`), created if missing. The primary store opens through `Database::open` in `crates/rusvel-db/src/store.rs`: SQLite gets WAL mode and foreign keys, then `migrations::run_migrations` in `crates/rusvel-db/src/migrations.rs` applies numbered DDL—events, objects, sessions, runs, threads, jobs, metrics—so schema evolution is centralized in one migration table.

Configuration loads from TOML via `TomlConfig`. LLM access starts as `MultiProvider`: `main` registers `ClaudeCliProvider` and `CursorAgentProvider`, then wraps the result in `CostTrackingLlm` with `MetricStore` implemented by the same `Database`, so token spend can be recorded per session and tier. The event pipeline is `EventBus::new` backed by the database’s `EventStore`. Session-scoped FTS memory uses `MemoryStore` on a separate `memory.db` file. A `ToolRegistry` (`crates/rusvel-tool/src/lib.rs`) is constructed; `rusvel_builtin_tools::register_all` and `tool_search::register` populate it, and the registry is aliased as `Arc<dyn ToolPort>` for injection. **`JobPort` is the same `Database` instance**—ADR-003’s single queue—so approvals, cron, and workers all see one logical queue.

`AgentRuntime::new(llm, tools, memory)` in `crates/rusvel-agent/src/lib.rs` is the **`AgentPort`** implementation (ADR-009). `SessionAdapter` in `main.rs` implements `SessionPort` by delegating to `StoragePort::sessions()`, keeping the session hierarchy on SQLite.

**Department boot (ADR-014)** runs next: `boot::installed_departments()` in `crates/rusvel-app/src/boot.rs` returns `Vec<Box<dyn DepartmentApp>>` in intentional order (core departments before messaging). `boot_departments` collects each `DepartmentManifest` via `manifest()`, validates unique IDs, runs `resolve_dependency_order` (Kahn topological sort on `depends_on`), then calls `register(&mut RegistrationContext)` in that order. `RegistrationContext` in `crates/rusvel-core/src/department/context.rs` carries shared `Arc` ports plus `ToolRegistrar`, `EventHandlerRegistrar`, and `JobHandlerRegistrar`. `finalize` yields `DepartmentsBootArtifacts`: a `DepartmentRegistry` built from manifests, vectors of tool registrations and event subscriptions, a map of job handler entries, and any failed department IDs.

`main` pattern-matches `DepartmentsBootArtifacts` but uses `..` to ignore the tool and job-handler collections; **what is wired today from that struct is the `registry`, `failed_departments`, and `event_subscriptions`**. `boot::spawn_department_event_dispatch` subscribes to the `EventBus` broadcast channel and, for each incoming `Event`, spawns matching department handlers when `event.kind` equals a registered `EventSubscription::event_kind`. That is the live bridge from append-only events to reactive department code.

After boot, `main` still constructs **concrete domain engines** for API and worker paths: a second-line `ForgeEngine::new` (fed the shared `AgentPort`), `CodeEngine`, `ContentEngine` with platform adapters, `HarvestEngine` with optional `BrowserPort` (`CdpClient`), `FlowEngine` with terminal and browser ports, and `GtmEngine`. Additional tools land on the same `ToolRegistry` (browser, delegate, terminal, artifact helpers, then `rusvel_engine_tools` for harvest, content, and code). `seed_defaults` fills `ObjectStore` buckets such as `agents`, `skills`, and `rules` when empty. `TriggerManager` loads `EventTrigger` JSON from the object store and subscribes to the bus for automated follow-ups.

A **`tokio::spawn`** job worker loop calls `jobs.dequeue(&[])` on the shared `JobPort`, dispatches by `JobKind` with explicit `match` arms (code analyze, content publish, harvest scan, proposal draft with `hold_for_approval`, cron, custom `forge.pipeline`, outreach send), then `complete`, `fail`, or leaves the job awaiting approval. A `watch` channel coordinates shutdown with `start_server`. Cron gets `CronScheduler::spawn_interval_ticker`; webhooks use `WebhookReceiver`.

For the default HTTP path, `main` may initialize `FastEmbedAdapter` and `LanceVectorStore` (RAG), calls `harvest_engine.configure_rag` when both exist, builds `AppState` in `crates/rusvel-api/src/lib.rs`, resolves `frontend_dir` or `extract_embedded_frontend()`, and runs `build_router_with_frontend` plus `start_server` on `127.0.0.1:3000`. A background task listens for first Ctrl+C, signals shutdown through the watch sender, and optionally force-exits on a second signal.

If **`Cli::parse()`** yields a subcommand, **`rusvel_cli::run`** in `crates/rusvel-cli/src/lib.rs` takes over after the same infrastructure spin-up: tier-one department verbs, **`session`**, **`forge`**, **`shell`**, and engine-specific actions receive **`Arc<ForgeEngine>`**, **`SessionPort`**, **`StoragePort`**, and selective engine refs (code, content, harvest) for commands that need real engines. The **`--tui`** branch loads objects for goals, tasks, opportunities, and recent events from **`ObjectStore`**, reads **`active_session`** from disk, pulls terminal pane metadata, and hands a **`TuiData`** bundle to **`rusvel_tui::run_tui`**. **`--mcp`** constructs **`RusvelMcp::new(forge, sessions)`** and blocks on **`rusvel_mcp::run_stdio`**, translating JSON-RPC tool invocations into forge operations without starting Axum. **`--mcp_http`** merges **`nest_mcp_http`** into the main router so streamable HTTP MCP shares the same **`AppState`** as the REST and SPA surface.

---

## 3. The Port System

Hexagonal architecture here means **`rusvel-core` defines boundaries; engines and departments never import adapter crates.** The inventory lives in `crates/rusvel-core/src/ports.rs`. `LlmPort` is raw model I/O: `generate`, default or overridden `stream`, `embed`, `list_models`. **`AgentPort`** is what orchestration code is supposed to call (ADR-009): `create` returns a `RunId`, `run` executes a full turn, `stop` and `status` manage lifecycle. `ToolPort` abstracts registration and `call` with optional `search` for deferred tool discovery. `EventPort` is append-only emit/query with **`Event::kind` as `String`** (ADR-005). `StoragePort` fans out to five focused sub-traits—`EventStore`, `ObjectStore`, `SessionStore`, `JobStore`, `MetricStore`—ADR-004’s split so each access pattern stays coherent. `MemoryPort` backs session-scoped recall; `JobPort` is the queue façade; `SessionPort` hides workspace CRUD; `ConfigPort` and `AuthPort` cover settings and credentials; `EmbeddingPort`, `VectorStorePort`, `ChannelPort`, `BrowserPort`, `TerminalPort`, and `DeployPort` extend the system when wired.

Engines like `ForgeEngine` in `crates/forge-engine/src/lib.rs` hold `Arc<dyn AgentPort>` and `Arc<dyn StoragePort>`, not `rusvel_db::Database`. That constraint keeps unit tests on mock ports and lets the composition root in `main.rs` swap SQLite for something else without recompiling forge. **`rusvel-tool::ToolRegistry`** implements `ToolPort` with interior mutability; **`ScopedToolRegistry`** wraps another `ToolPort` and filters `list`/`call`/`schema` by allowed prefixes or exact names—useful when a run should not see the entire tool surface.

`MemoryPort` sits beside the main database: chat and agent code can persist searchable snippets per session without overloading the generic object buckets. `EmbeddingPort` and `VectorStorePort` are optional; when both are present, department chat injects retrieved passages into the system prompt, and harvest can attach semantic hints to opportunities. `ChannelPort` (for example `TelegramChannel` from env) lets handlers notify humans out-of-band. `BrowserPort` wires CDP for listing-driven harvest; `TerminalPort` backs flow nodes and delegate tooling. None of these replace the core five-store contract—they are additional capabilities the composition root passes into engines or registers as tools.

---

## 4. Department Lifecycle

`DepartmentApp` in `crates/rusvel-core/src/department/app.rs` is the contract: `manifest()` must be side-effect free; `register(&mut RegistrationContext)` runs once at boot; `shutdown` defaults to no-op. Departments depend only on `rusvel-core` and must not import other department crates; cross-cutting reactions use `EventPort` and shared storage kinds.

The **manifest** (`DepartmentManifest` in `crates/rusvel-core/src/department/manifest.rs`, exemplified by `crates/dept-forge/src/manifest.rs`) declares `id`, display metadata, `system_prompt`, `capabilities`, UI tabs and quick actions, declarative tool and persona contributions for documentation, `requires_ports`, `depends_on`, `events_produced` / `events_consumed`, and default layered config. **`DepartmentRegistry::from_manifests`** in `context.rs` projects manifests into `DepartmentDef` rows the API serves at `GET /api/departments` and the Svelte app consumes for navigation.

During **`register`**, a typical department constructs its engine with ports from `ctx`, then calls `ctx.tools.add` for each callable surface, `ctx.event_handlers.on` for reactive work, and `ctx.job_handlers.handle` for custom job kinds. **Forge** (`crates/dept-forge/src/lib.rs`) instantiates `ForgeEngine::new` with `ctx.agent`, `ctx.events`, `ctx.storage`, and so on, stashes it in a `OnceLock`, and registers tools like `forge.mission.today` whose closures capture that `Arc<ForgeEngine>`. The manifest’s static tool list stays aligned with `mission_tool_contributions_for_manifest` in `forge-engine` so docs and runtime stay in sync.

---

## 5. Request Flow: Department Chat

The router is assembled in `build_router_with_frontend` in `crates/rusvel-api/src/lib.rs`. Department chat posts to a parameterized route (see the `.route` chains for `/api/dept/{dept}/chat`). `dept_chat` in `crates/rusvel-api/src/department.rs` is the spine.

The handler validates `dept` against `AppState::registry` via `validate_dept`. It loads persisted **`LayeredConfig`** from `ObjectStore` under key `dept_config` / department id, then **`resolve_dept_config`**: registry defaults, overlay from storage, merge with `UserProfile::to_system_prompt` when a profile exists. Conversation state lives in a namespaced object kind `dept_msg_{dept}`. The handler loads the last fifty `ChatMessage` rows for the conversation id, persists the user message immediately, and handles special paths: **`!build`** dispatches to `build_cmd` over a short SSE stream without the agent; **`/skill`** expansion goes through `skills::resolve_skill`; **`@agent`** can override system prompt and tools from an `AgentProfile` in the `agents` bucket.

Enabled **rules** from `rules::load_rules_for_engine` append to the system prompt. If the request includes `session_id`, **`context_pack_for_chat`** uses `ContextPackCache` and `assemble_context_pack` to pull session name, goal titles from `ForgeEngine::list_goals`, recent `EventPort::query` summaries, and quick job/harvest metrics, then **`to_prompt_section`** from `rusvel_agent` appends that block. Department-specific prose adds HTTP “cheat sheet” sections for code, content, harvest, forge, GTM. Optional RAG: **`EmbeddingPort::embed_one`** plus **`VectorStorePort::search`** append “Relevant Knowledge” paragraphs.

The handler builds **`AgentConfig`**: `model` from `sse_helpers::parse_model_ref`, `tools` from resolved `allowed_tools`, `instructions` as the accumulated system prompt, `metadata` carrying `RUSVEL_META_MODEL_TIER` and `RUSVEL_META_DEPARTMENT_ID` (from `domain.rs`). User content is a single `Content::text` bundling transcript lines (“User: …”, “Assistant: …”) plus the effective user message.

**`AgentRuntime::create`** allocates a `RunId` and stores `RunState`; **`run_streaming`** transitions status to `Running`, spawns **`run_streaming_loop`**, and returns an `mpsc::Receiver<AgentEvent>`. The Axum side wraps that receiver in **`ReceiverStream`**, prefixes **`sse_helpers::prelude_stream`** (a `RUN_STARTED` SSE event via `AgUiEvent`), then maps each **`AgentEvent`** through **`sse_helpers::other_event_sse`** or, on **`AgentEvent::Done`**, **`sse_helpers::run_completed_sse`**. On completion, a **`tokio::spawn`** persists the assistant `ChatMessage`, **`EventPort::emit`** sends `{dept}.chat.completed` with cost and length in the payload, and **`hook_dispatch::dispatch_hooks`** runs stored hooks against the same event kind.

Middleware on the stack (`crates/rusvel-api/src/lib.rs`) includes CORS for local dev origins, rate limiting, optional bearer auth (`auth::bearer_auth`), `TraceLayer`, and `request_id` middleware. **`start_server`** uses graceful shutdown so SSE clients get a bounded wind-down window.

---

## 6. The Agent Runtime

`AgentRuntime` implements **`AgentPort`** and also exposes **`run_streaming`** for HTTP SSE. Internally, **`run_streaming_loop`** seeds `Vec<LlmMessage>` with optional system instructions from `AgentConfig` and the user `Content`, then enters a loop bounded by **`MAX_ITERATIONS`**.

Tool definitions for the first LLM call come from **`tools.list()`** filtered to **non-`searchable`** tools—this is **deferred tool loading**: only a small baseline (including non-searchable builtins) enters the prompt; the model can call **`tool_search`**, and when that succeeds, **`ToolPort::search`** pulls additional `ToolDefinition` values into `tool_defs` for subsequent iterations (`crates/rusvel-agent/src/lib.rs`).

Each iteration builds an **`LlmRequest`** via **`AgentRuntime::build_request`**, merging **`AgentConfig::metadata`** with **`RUSVEL_META_SESSION_ID`** for cost attribution. The runtime calls **`LlmPort::stream`**, forwarding **`LlmStreamEvent::Delta`** as **`AgentEvent::TextDelta`**, surfacing tool use hints, and aggregating the final **`LlmResponse`**. On **`FinishReason::ToolUse`**, it extracts **`Part::ToolCall`**, runs **pre-hooks** (`run_hooks_pre`), calls **`ToolPort::call`**, increments tool call counts, emits **`ToolResult`** events, appends assistant and tool **`LlmMessage`** rows, and loops. **`compact_messages`** may summarize older turns with a fast-tier `generate` call when history exceeds thresholds—protecting context windows without silent loss of the most recent turns.

**`ToolRegistry::call`** in `crates/rusvel-tool/src/lib.rs` validates arguments against the JSON schema fragment, evaluates **tool permissions** (`ToolPermissionMode`: locked, supervised, auto) using optional `__department_id` in args, then runs the async handler. A representative builtin is **`read_file`** in `crates/rusvel-builtin-tools/src/file_ops.rs`: path canonicalization under the working directory, async file IO, structured **`ToolResult`** with line-numbered output.

The **god agent** path in `crates/rusvel-api/src/chat.rs` is the same mechanical pipeline with a global config object, profile-derived system prompt, namespace `chat_messages` instead of `dept_msg_*`, and without department-specific prompt appendices—still **`create` + `run_streaming` + SSE mapping**.

Cost routing deserves an explicit mention because it affects every call: **`LlmRequest::metadata`** can carry **`RUSVEL_META_MODEL_TIER`** and session identifiers; **`CostTrackingLlm`** in `rusvel-llm` interprets tier strings (fast, balanced, premium) to pick concrete models where the stack supports it and records spend into **`MetricStore`**. Frontend department chat can send **`model_tier`** on the JSON body; that value lands in **`AgentConfig::metadata`** and flows into **`merge_llm_request_metadata`** inside **`AgentRuntime::build_request`**. Analytics endpoints then aggregate by department id from **`RUSVEL_META_DEPARTMENT_ID`**, which department chat sets to the path parameter and god chat sets to `"global"`.

---

## 7. Background Jobs

`JobPort` abstracts enqueue, dequeue, complete, fail, list, and **`hold_for_approval`**—used when a job should pause with a proposed result until a human approves via the approvals API. `crates/rusvel-jobs/src/lib.rs` documents the trait and provides an in-memory **`JobQueue`** for tests; production uses **`Database`**’s `JobStore` implementation.

The worker in **`main.rs`** polls **`dequeue`** with an empty kind filter (all kinds), **skips `JobStatus::AwaitingApproval`** so held jobs are not stolen, and matches on **`JobKind`**: **`CodeAnalyze`** calls **`CodeEngine::analyze`**, **`ContentPublish`** parses payload into **`ContentId`** and **`Platform`** then **`ContentEngine::publish`**, **`HarvestScan`** runs **`HarvestEngine::scan`** with a mock source, **`ProposalDraft`** either resumes from `approval_pending_result` or generates a proposal then **`hold_for_approval`**, **`ScheduledCron`** either runs **`ForgeEngine::generate_brief`** for the daily briefing kind or emits a synthetic **`Event`**, **`Custom("forge.pipeline")`** runs **`ForgeEngine::orchestrate_pipeline`** with **`HarvestContentPipelineRunner`**, and **`OutreachSend`** delegates to GTM outreach with another approval branch. **`JobPort::complete`** / **`fail`** persist terminal state. This worker is separate from the **`job_handlers` map** produced by department boot—those entries are part of the ADR-014 artifact surface but are not dispatched by this loop today, so new job kinds still require either extending the **`match`** or wiring the map.

API handlers under **`crates/rusvel-api/src/jobs.rs`** and **`approvals.rs`** expose queue visibility and the approve/reject transitions that move a held job forward. Because **`Database`** implements both **`StoragePort`** and **`JobPort`**, the UI’s approval badge and the worker’s dequeue see identical rows—there is no shadow queue. Cron schedules stored through **`CronScheduler`** enqueue **`ScheduledCron`** jobs on tick; webhooks validated by **`WebhookReceiver`** can enqueue **`Custom`** work such as the forge pipeline when configured to do so.

---

## 8. The Frontend

SvelteKit 5 lives under `frontend/`. The dashboard entry **`frontend/src/routes/+page.svelte`** subscribes to **`activeSession`** and **`departments`** stores, then fans out parallel fetches—goals, events, latest brief, analytics—through **`$lib/api`** helpers against `/api/sessions/...`, `/api/analytics/spend`, and related routes. Department navigation uses manifest-driven ids from **`GET /api/departments`**.

**`frontend/src/routes/dept/[id]/chat/+page.svelte`** resolves the department row by `page.params.id`, requires an active session, and mounts **`DepartmentChat`** (`frontend/src/lib/components/chat/DepartmentChat.svelte`). That component loads conversations and config via **`getDeptConversations`**, **`getDeptConfig`**, **`getModels`**, **`getTools`**, and streams user input through **`streamDeptChat`** in **`frontend/src/lib/api.ts`**: a **`fetch`** `POST` to `/api/dept/${dept}/chat` with JSON body, then **`parseSSE`** over the response body, branching on AG-UI-style event types (`tool_call_start`, `tool_call_end`, `text_delta`, `run_completed`, `run_failed`) to update markdown streaming, **`ToolCallCard`**, and completion handlers. This mirrors the server’s **`sse_helpers`** / **`AgUiEvent`** naming.

The dashboard and chat assume the API is same-origin or CORS-allowed (`localhost:3000` and `localhost:5173` in **`build_router_with_frontend`**). Session selection in stores drives every authenticated-ish operation: without **`activeSession`**, department chat shows a blocking empty state. **`pnpm dev`** against Vite uses the proxy or full URL to reach the Rust server; the embedded **`frontend/build`** path is what ships inside the binary for single-file distribution.

---

## 9. Data Model

**`crates/rusvel-core/src/domain.rs`** is the shared vocabulary. **`Content`** and **`Part`** unify text, media, and tool call/result parts for LLM transcripts. **`LlmRequest`**, **`LlmMessage`**, **`LlmResponse`**, and **`LlmStreamEvent`** model provider I/O. **`Session`**, **`Run`**, and **`Thread`** form the hierarchy persisted by **`SessionStore`**. **`Event`** carries `source`, string **`kind`**, JSON **`payload`**, and **`metadata`** on every row—schema evolution defaults to **`metadata`** (ADR-007). **`Job`**, **`NewJob`**, **`JobKind`**, and **`JobStatus`** describe queue rows. **`Goal`**, **`Task`**, **`Opportunity`**, and many other structs power engines; CRUD for configurables like **`AgentProfile`** often serializes through **`ObjectStore`** as JSON blobs keyed by kind and id.

On disk, **`events`** and **`objects`** tables back the event log and document store; **`sessions`**, **`runs`**, **`threads`** back the chat hierarchy; **`jobs`** back the queue; metrics live in their own table set defined in migrations. **`MemoryStore`** is separate FTS storage for session recall, while **LanceDB** (when enabled) holds embedding-backed knowledge chunks for RAG.

Thinking in terms of access patterns helps when you add features. If you need append-only audit history, **`EventStore::append`** and **`EventPort::emit`** are the right abstraction—downstream UI reads **`query`** with **`EventFilter`**. If you need arbitrary JSON documents with CRUD, **`ObjectStore::put/get/list`** under a stable **`kind`** string namespaces your data without new tables; chat transcripts use this pattern per department. If you need transactional workflow state with dequeue semantics, **`JobStore`** is the path. Session-scoped chat threads that must align with **`AgentPort`** runs belong in **`SessionStore`**. If you are recording time series or token spend, **`MetricStore`** is where **`CostTrackingLlm`** already writes—reuse it before inventing a parallel table.

---

## 10. Adding a New Department

The repeatable recipe mirrors **forge**: (1) Add or extend a **domain engine crate** that depends only on **`rusvel-core`** ports and contains business logic (goals, pipelines, scoring—whatever the department owns). (2) Add a **`dept-*` crate** implementing **`DepartmentApp`**: `manifest()` returns a filled **`DepartmentManifest`**; **`register`** constructs the engine from **`RegistrationContext`** ports and calls **`ctx.tools.add`**, **`ctx.event_handlers.on`**, or **`ctx.job_handlers.handle`** as needed. (3) Expose a **`manifest.rs`** (or inline module) that stays the single source of truth for UI tabs, quick actions, port requirements, and declared events. (4) Append **`Box::new(dept_xxx::XDepartment::new())`** to **`installed_departments()`** in **`crates/rusvel-app/src/boot.rs`** in dependency order—if the department declares **`depends_on`**, topological sort will still order registration correctly. (5) Wire any **HTTP routes** through **`rusvel-api`** if the department needs engine-specific endpoints beyond generic dept CRUD. (6) For tools that must appear in the agent loop, ensure they end up on the process-wide **`ToolRegistry`** (today that means **builtin/engine registration in `main.rs`** or future merging of boot **`ToolRegistration`** entries—inspect **`main.rs`** when you add callable tools).

---

## 11. Key Design Decisions

**ADR-007 (metadata on domain types)** — Structured fields are stable; extensibility goes into **`serde_json::Value` metadata** first so adapters and engines can evolve without immediate migrations. When reading handlers, expect **`metadata`** on events, jobs, agents, and LLM responses to carry feature flags and tracing.

**ADR-009 (AgentPort, not LlmPort, for engines)** — **`ForgeEngine::hire_persona`** builds **`AgentConfig`** and would call **`AgentPort::run`**; mission planners never touch **`LlmPort`** directly. The API chat path uses **`AgentRuntime`**, which *does* own **`LlmPort`**, **`ToolPort`**, and **`MemoryPort`**—that is the sanctioned composition.

**ADR-014 (DepartmentApp)** — Departments are plugins with manifests, registrars, and a generated **`DepartmentRegistry`**. Boot validates IDs and dependency graphs. **Event subscriptions** from **`finalize`** are actively wired through **`spawn_department_event_dispatch`**. Tool and job-handler maps are part of the same **`DepartmentsBootArtifacts`** struct; **`main.rs` currently discards them with `..` when destructuring**, so the global tool surface is still dominated by explicit registration in **`main.rs`** (`rusvel_builtin_tools`, **`rusvel_engine_tools`**, etc.). When you add a department tool that must run inside chat, verify where it is registered relative to **`AgentRuntime`**.

Together, these ADRs explain day-to-day constraints: extend behavior through ports, keep engines pure, push cross-cutting reactions through **`EventPort`**, and treat manifests as the contract the UI and API both read—while recognizing that the composition root still owns a few legacy, explicit wires (standalone **`ForgeEngine`** for HTTP, the job worker **`match`**, and the shared **`ToolRegistry`**) that sit alongside the newer department boot path.

**ADR-003 and ADR-008** show up constantly in jobs and publishing: one queue, and human approval for sensitive outcomes. **ADR-005** means never assuming a closed set of event kinds in core—match on strings in subscribers and SQL filters. **ADR-010** is the engine boundary rule mirrored in **`ForgeEngine`’s** constructor signature: only **`Arc<dyn …Port>`** fields.

---

## Shutdown

When the shutdown future passed to **`start_server`** completes (first Ctrl+C toggles the watch sender in **`main.rs`**), Axum drains with graceful shutdown; a five-second guard in **`start_server`** then logs and **`process::exit(0)`** so stuck SSE connections cannot hang the process indefinitely. The job worker observes the same watch channel and breaks its loop, stopping dequeue. Department **`shutdown`** hooks are available on the trait but are not centrally orchestrated in **`main`** today—another place a contributor might tighten lifecycle symmetry.

---

*Paths cited: `crates/rusvel-app/src/main.rs`, `crates/rusvel-app/src/boot.rs`, `crates/rusvel-core/src/ports.rs`, `crates/rusvel-core/src/domain.rs`, `crates/rusvel-core/src/department/app.rs`, `crates/rusvel-core/src/department/context.rs`, `crates/rusvel-api/src/lib.rs`, `crates/rusvel-api/src/department.rs`, `crates/rusvel-api/src/chat.rs`, `crates/rusvel-api/src/sse_helpers.rs`, `crates/rusvel-agent/src/lib.rs`, `crates/rusvel-tool/src/lib.rs`, `crates/rusvel-builtin-tools/src/file_ops.rs`, `crates/forge-engine/src/lib.rs`, `crates/dept-forge/src/lib.rs`, `crates/dept-forge/src/manifest.rs`, `crates/rusvel-db/src/store.rs`, `crates/rusvel-db/src/migrations.rs`, `crates/rusvel-jobs/src/lib.rs`, `crates/rusvel-cli/src/lib.rs`, `crates/rusvel-mcp/src/lib.rs`, `frontend/src/routes/+page.svelte`, `frontend/src/routes/dept/[id]/chat/+page.svelte`, `frontend/src/lib/api.ts`.*

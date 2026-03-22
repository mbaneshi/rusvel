# Phase 0 — Foundation v2 (Post-Review)

> Incorporates Perplexity feedback. Supersedes phase-0-foundation.md.
> Goal: Prove hexagonal architecture with one vertical slice.
> Deliverable: `rusvel forge mission today` works from CLI, API, and Web.

---

## Changes from v1

- Mission is now part of Forge Engine (not a separate engine)
- Central job queue replaces AutomationPort + SchedulePort
- SessionPort has Session → Run → Thread hierarchy
- StoragePort has 5 canonical stores (not generic key-value)
- Event.kind is a String (not giant enum)
- All domain types have `metadata: serde_json::Value` for evolution
- Human approval model included from the start

---

## Milestone 0.1 — Core Traits + Types

**rusvel-core: 10 port traits + shared domain types. Zero framework deps.**

- [ ] Port traits: LlmPort, AgentPort, ToolPort, EventPort, StoragePort, MemoryPort, JobPort, SessionPort, AuthPort, ConfigPort
- [ ] StoragePort with 5 sub-stores: EventStore, ObjectStore, SessionStore, JobStore, MetricStore
- [ ] Session hierarchy: Session, Run, Thread + SessionKind, RunStatus, ThreadChannel
- [ ] Job types: Job, JobKind, JobStatus, NewJob, JobFilter, JobResult
- [ ] Approval model: ApprovalStatus, ApprovalPolicy
- [ ] Domain types: Content/Part, AgentProfile, Opportunity, ContentItem, Contact, Goal, Task, Event
- [ ] ID newtypes: SessionId, RunId, ThreadId, JobId, AgentProfileId, EventId, OpportunityId, ContentId, ContactId, GoalId, TaskId, SnapshotId, UserId, WorkspaceId
- [ ] Enums: ModelProvider, OpportunitySource, OpportunityStage, ContentKind, ContentStatus, Timeframe, GoalStatus, TaskStatus, Priority, EngineKind, SessionKind
- [ ] Error type: RusvelError
- [ ] Engine trait: name, capabilities, initialize, shutdown, health
- [ ] All types: Serialize + Deserialize + Clone + Debug
- [ ] All domain types have `metadata: serde_json::Value`
- [ ] `cargo build` succeeds
- [ ] `cargo test` passes (unit tests for type construction)

## Milestone 0.2 — First Adapters

**Minimum adapters to support forge mission.**

- [ ] `rusvel-config` — Load TOML from `~/.rusvel/config.toml`, per-session overrides
- [ ] `rusvel-db` — SQLite WAL, migration runner, implement 5 canonical stores
  - [ ] EventStore: append, query by session/time range
  - [ ] ObjectStore: CRUD for domain objects (typed by table prefix)
  - [ ] SessionStore: Session/Run/Thread CRUD
  - [ ] JobStore: enqueue, dequeue (FOR UPDATE SKIP LOCKED pattern), complete, fail
  - [ ] MetricStore: time-series insert + query
- [ ] `rusvel-event` — In-memory broadcast bus + persist to EventStore
- [ ] `rusvel-llm` — Ollama adapter: generate, generate_stream (no embed yet)
- [ ] `rusvel-memory` — SQLite FTS5 for text search, session-namespaced
- [ ] `rusvel-tool` — Tool registry, JSON Schema validation, basic tool calling
- [ ] `rusvel-auth` — Store/retrieve API keys from SQLite (encrypted at rest)
- [ ] `rusvel-jobs` — Job queue on top of JobStore, simple in-process worker pool
- [ ] Unit tests for each adapter
- [ ] Integration test: create session → store memory → search memory → verify

## Milestone 0.3 — Forge Engine (with Mission)

**Forge engine using only port traits. Includes mission sub-domain.**

- [ ] `ForgeEngine` struct with injected ports
- [ ] Implements `Engine` trait
- [ ] Mission sub-module:
  - [ ] `mission_today(session_id)` — read goals + recent events → LLM → daily plan → store as Tasks
  - [ ] `set_goal(session_id, goal)` — store goal, emit event
  - [ ] `list_goals(session_id)` — query goals
  - [ ] `review(session_id, period)` — summarize progress via LLM
- [ ] Agent sub-module:
  - [ ] `create_agent(profile)` — register an agent
  - [ ] `run_agent(agent_id, input, session_id)` — create Run, execute, stream output, complete Run
- [ ] Creates Run for every execution
- [ ] Emits events for all actions
- [ ] Respects session boundaries
- [ ] Integration test: create session → set goals → generate today → verify plan + events

## Milestone 0.4 — CLI Surface

**Wire forge engine to CLI.**

- [ ] `rusvel-cli` with Clap 4
- [ ] `rusvel session create <name>` → creates session, prints ID
- [ ] `rusvel session list` → lists sessions
- [ ] `rusvel session switch <id>` → sets active session
- [ ] `rusvel forge mission today` → daily plan for active session
- [ ] `rusvel forge mission goals` → list goals
- [ ] `rusvel forge mission goal add "..."` → create goal
- [ ] `rusvel forge mission review --weekly` → weekly review
- [ ] `rusvel-app` binary wires adapters → engine → CLI
- [ ] Pretty terminal output
- [ ] End-to-end test: binary produces output

## Milestone 0.5 — API + Web Surface

**HTTP API and minimal SvelteKit frontend.**

- [ ] `rusvel-api` — Axum server:
  - [ ] `GET /api/health`
  - [ ] `GET /api/sessions` + `POST /api/sessions`
  - [ ] `GET /api/sessions/:id`
  - [ ] `GET /api/sessions/:id/mission/today`
  - [ ] `GET /api/sessions/:id/mission/goals` + `POST`
  - [ ] `GET /api/sessions/:id/runs` (run history)
  - [ ] `GET /api/sessions/:id/events` (event timeline)
  - [ ] `WS /api/sessions/:id/stream` (live events)
- [ ] `frontend/` — SvelteKit 5:
  - [ ] Session switcher in sidebar
  - [ ] Dashboard: today's plan + recent events
  - [ ] Goals list with add form
  - [ ] Run history with status
  - [ ] Event timeline
- [ ] Build frontend → embed via rust-embed
- [ ] `rusvel` (no args) starts Axum + opens browser
- [ ] `rusvel --headless` starts server only

## Milestone 0.6 — MCP Surface

- [ ] `rusvel-mcp` — rmcp server:
  - [ ] `session_create`, `session_list`, `session_switch`
  - [ ] `mission_today`, `mission_goals`, `mission_add_goal`
- [ ] stdio mode (for Claude Code)
- [ ] SSE mode (for web clients)

---

## Definition of Done for Phase 0

- [ ] `cargo build --release` → single binary
- [ ] 20 crates compile (10 foundation + 5 engines (stubs) + 5 surfaces)
- [ ] `rusvel session create "my-project"` → creates session
- [ ] `rusvel forge mission today` → generates daily plan via Ollama
- [ ] `rusvel` → opens web dashboard with session view
- [ ] `rusvel mcp` → works as MCP server
- [ ] 10 port traits defined in rusvel-core
- [ ] Session → Run → Thread hierarchy working
- [ ] Central job queue processing jobs
- [ ] Human approval model in domain types
- [ ] All domain types have metadata field
- [ ] Events emitted and persisted for all actions
- [ ] ≥ 50 tests passing
- [ ] < 5 second cold start
- [ ] Binary size < 50MB
- [ ] CLAUDE.md with project conventions

---

## What Phase 0 Intentionally Skips

- Code Engine (Phase 1) — just a stub
- Harvest Engine (Phase 2) — just a stub
- Content Engine (Phase 2) — just a stub
- GoToMarket Engine (Phase 3) — just a stub
- TUI surface (Phase 2)
- Cloud LLM adapters: Claude/OpenAI/Gemini (Phase 1, Ollama only now)
- Semantic vector search in memory (Phase 1, FTS5 only now)
- Multi-language code parsing (Phase 1, Rust only)
- Browser extension (Phase 5)
- A2A protocol (Phase 5)

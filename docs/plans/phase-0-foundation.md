> **SUPERSEDED** — See phase-0-foundation-v2.md.

# Phase 0 — Foundation

> Goal: Prove the hexagonal architecture works end-to-end with one vertical slice.
> Deliverable: `rusvel mission today` works from CLI, API, and Web.

---

## Milestone 0.1 — Core Traits

**Create `rusvel-core` with all 13 port traits and shared domain types.**

- [ ] Define `LlmPort` trait
- [ ] Define `AgentPort` trait
- [ ] Define `AutomationPort` trait
- [ ] Define `MemoryPort` trait
- [ ] Define `ToolPort` trait
- [ ] Define `EventPort` trait
- [ ] Define `StoragePort` trait
- [ ] Define `SchedulePort` trait
- [ ] Define `HarvestPort` trait
- [ ] Define `PublishPort` trait
- [ ] Define `AuthPort` trait
- [ ] Define `ConfigPort` trait
- [ ] Define `SessionPort` trait
- [ ] Define `Engine` base trait
- [ ] Define shared types: `Content`, `Part`, `Opportunity`, `Contact`, `ContentPiece`, `Goal`, `AgentTask`, `Session`
- [ ] Define status enums: `TaskStatus`, `OpportunityStatus`, `WorkflowStatus`
- [ ] Define error types: `RusvelError`
- [ ] Define ID types: `AgentId`, `SessionId`, `EventId`, `TaskId`, etc.
- [ ] All types derive `Serialize`, `Deserialize`, `Clone`, `Debug`
- [ ] Zero external framework deps (only std, serde, async-trait, thiserror, chrono, uuid)
- [ ] `cargo test` passes

## Milestone 0.2 — First Adapters

**Implement the minimum adapters needed for mission-engine.**

- [ ] `rusvel-config` — Load TOML config from `~/.rusvel/config.toml`
- [ ] `rusvel-db` — SQLite WAL connection, migration runner, basic CRUD
- [ ] `rusvel-event` — In-memory broadcast event bus with SQLite persistence
- [ ] `rusvel-llm` — Ollama adapter (local, no API key) with `generate` and `generate_stream`
- [ ] `rusvel-memory` — Store/recall/search memory entries in SQLite with basic text matching
- [ ] `rusvel-tool` — Tool registry with JSON Schema declarations, basic tool calling
- [ ] `rusvel-auth` — Simple keyring (store/retrieve API keys from SQLite, encrypted at rest)
- [ ] Unit tests for each adapter
- [ ] `cargo test` passes for all adapter crates

## Milestone 0.3 — Mission Engine

**Build mission-engine using only port traits.**

- [ ] `MissionEngine` struct with injected ports (llm, memory, storage, schedule, events)
- [ ] `today()` method: read goals from memory → build prompt → call LLM → return daily plan
- [ ] `set_goal()` method: store goal in memory + emit GoalCreated event
- [ ] `list_goals()` method: query goals from memory
- [ ] `review()` method: summarize progress via LLM
- [ ] Integration test: create goals → generate today plan → verify output
- [ ] Engine implements `Engine` trait (name, capabilities, initialize, shutdown, health)

## Milestone 0.4 — First Surface (CLI)

**Wire mission-engine to CLI.**

- [ ] `rusvel-cli` with Clap 4 subcommands
- [ ] `rusvel mission today` → calls MissionEngine::today()
- [ ] `rusvel mission goals` → lists goals
- [ ] `rusvel mission goal add "..."` → creates goal
- [ ] `rusvel mission review --weekly` → weekly review
- [ ] `rusvel-app` binary that wires adapters → engine → CLI
- [ ] Pretty terminal output (colored, structured)
- [ ] End-to-end test: binary runs, produces output

## Milestone 0.5 — Second Surface (API + Web)

**Add HTTP API and minimal SvelteKit frontend.**

- [ ] `rusvel-api` — Axum server with routes:
  - `GET /api/health`
  - `GET /api/mission/today`
  - `GET /api/mission/goals`
  - `POST /api/mission/goals`
  - `WS /api/events` (stream events)
- [ ] `frontend/` — SvelteKit 5 project with:
  - Dashboard page showing today's plan
  - Goals list with add form
  - Real-time event stream display
- [ ] Build frontend → embed via rust-embed
- [ ] `rusvel` (no subcommand) starts Axum server + opens browser
- [ ] `rusvel --headless` starts server without opening browser

## Milestone 0.6 — MCP Surface

**Expose mission-engine via MCP.**

- [ ] `rusvel-mcp` — rmcp server with tools:
  - `mission_today` — get today's plan
  - `mission_goals` — list goals
  - `mission_add_goal` — add a goal
- [ ] Works via stdio (for Claude Code) and SSE (for web clients)
- [ ] Test from Claude Code: `rusvel mcp` as MCP server

## Definition of Done for Phase 0

- [ ] `cargo build --release` produces single binary
- [ ] `rusvel mission today` works (CLI)
- [ ] `rusvel` opens web dashboard with today's plan
- [ ] `rusvel mcp` works as MCP server
- [ ] All 13 port traits defined in rusvel-core
- [ ] Architecture proven: engine depends only on traits, adapters are injected
- [ ] At least 50 tests passing
- [ ] < 5 second cold start
- [ ] Binary size < 50MB

---

## What Phase 0 Does NOT Include

- No forge-engine (agent orchestration) — Phase 1
- No code-engine (code intelligence) — Phase 1
- No harvest-engine (opportunity discovery) — Phase 2
- No content-engine (publishing) — Phase 2
- No ops-engine (business ops) — Phase 3
- No connect-engine (outreach) — Phase 3
- No TUI — Phase 2
- No Claude/OpenAI/Gemini LLM adapters — Phase 1 (Ollama only in Phase 0)

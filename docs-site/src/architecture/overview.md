
## Design Philosophy

RUSVEL follows **hexagonal architecture** (ports and adapters). The core principle: domain logic never depends on infrastructure. Engines express what they need through port traits; adapters provide the implementations.

## Four Layers

```
┌─────────────────── SURFACES ───────────────────┐
│  CLI (Clap)  │  TUI (Ratatui)  │  Web (Svelte) │
│                  MCP Server                      │
└──────────────────────┬─────────────────────────┘
                       │
┌──────────────────────┴─────────────────────────┐
│              DOMAIN ENGINES (5)                  │
│                                                 │
│  Forge         │ Code       │ Harvest           │
│  (+ Mission)   │            │                   │
│                │            │                   │
│  Content       │ GoToMarket                     │
│                │ (Ops+Connect+Outreach)          │
└──────────────────────┬─────────────────────────┘
                       │ uses (traits only)
┌──────────────────────┴─────────────────────────┐
│              FOUNDATION                          │
│                                                 │
│  ┌──────────── rusvel-core ──────────────┐      │
│  │  Port Traits + Shared Domain Types    │      │
│  └───────────────────────────────────────┘      │
│                                                 │
│  ┌──────────── Adapters ─────────────────┐      │
│  │  rusvel-llm     (model providers)     │      │
│  │  rusvel-agent   (agent runtime)       │      │
│  │  rusvel-db      (SQLite + 5 stores)   │      │
│  │  rusvel-event   (event bus + persist)  │      │
│  │  rusvel-memory  (context + search)    │      │
│  │  rusvel-tool    (tool registry)       │      │
│  │  rusvel-jobs    (central job queue)   │      │
│  │  rusvel-auth    (credentials)         │      │
│  │  rusvel-config  (settings)            │      │
│  └───────────────────────────────────────┘      │
└─────────────────────────────────────────────────┘
```

### Layer 1: rusvel-core (Foundation)

The heart of the system. Contains:

- **10 port traits** -- interfaces that engines depend on
- **~40 shared domain types** -- Session, Goal, Event, Agent, Content, Opportunity, Contact, Task, etc.
- Zero framework dependencies

### Layer 2: Adapters

Concrete implementations of the port traits:

| Crate | Implements | Notes |
|-------|-----------|-------|
| `rusvel-llm` | LlmPort | 4 providers: Ollama, OpenAI, Claude API, Claude CLI |
| `rusvel-agent` | AgentPort | Wraps LLM + Tool + Memory into orchestration |
| `rusvel-db` | StoragePort | SQLite WAL with 5 sub-stores, migrations |
| `rusvel-event` | EventPort | Event bus with persistence |
| `rusvel-memory` | MemoryPort | FTS5 session-namespaced search |
| `rusvel-tool` | ToolPort | Tool registry with JSON Schema |
| `rusvel-jobs` | JobPort | Central SQLite job queue |
| `rusvel-auth` | AuthPort | In-memory credential storage from env |
| `rusvel-config` | ConfigPort | TOML config with per-session overrides |

### Layer 3: Engines

Domain logic crates. Each engine depends **only** on `rusvel-core` traits:

| Engine | Focus |
|--------|-------|
| `forge-engine` | Agent orchestration + Mission (goals, planning, reviews) |
| `code-engine` | Code intelligence: parser, dependency graph, BM25 search, metrics |
| `harvest-engine` | Opportunity discovery: scanning, scoring, proposals, pipeline |
| `content-engine` | Content creation: writer, calendar, platform adapters, analytics |
| `gtm-engine` | GoToMarket: CRM, outreach sequences, invoicing, deal stages |

### Layer 4: Surfaces

User-facing interfaces that wire adapters into engines:

| Surface | Technology | Status |
|---------|-----------|--------|
| `rusvel-api` | Axum HTTP | Active |
| `rusvel-cli` | Clap 4 | Active |
| `rusvel-mcp` | stdio JSON-RPC | Imported, not yet dispatched |
| `rusvel-tui` | Ratatui | Layout + widgets, not yet wired |
| `frontend/` | SvelteKit 5 + Tailwind 4 | Active |

### Composition Root: rusvel-app

The binary entry point. It constructs all adapters, injects them into engines, and starts the chosen surface (web server, CLI, or MCP). This is the only place where concrete types meet.

## Workspace Layout

```
all-in-one-rusvel/
├── crates/
│   ├── rusvel-core/        10 port traits + shared domain types
│   ├── rusvel-db/          SQLite WAL + 5 canonical stores
│   ├── rusvel-llm/         4 LLM providers
│   ├── rusvel-agent/       Agent runtime (LLM+Tool+Memory)
│   ├── rusvel-event/       Event bus + persistence
│   ├── rusvel-memory/      FTS5 session-namespaced search
│   ├── rusvel-tool/        Tool registry + JSON Schema
│   ├── rusvel-jobs/        Central job queue
│   ├── rusvel-auth/        Credential storage
│   ├── rusvel-config/      TOML config + overrides
│   ├── forge-engine/       Agent orchestration + Mission
│   ├── code-engine/        Code intelligence (Rust-only v0)
│   ├── harvest-engine/     Opportunity discovery
│   ├── content-engine/     Content creation + publishing
│   ├── gtm-engine/         GoToMarket (CRM + outreach)
│   ├── rusvel-api/         Axum HTTP API
│   ├── rusvel-cli/         Clap CLI
│   ├── rusvel-tui/         Ratatui TUI
│   ├── rusvel-mcp/         MCP server
│   └── rusvel-app/         Binary entry point
├── frontend/               SvelteKit 5 + Tailwind 4
├── Cargo.toml              Workspace manifest
└── CLAUDE.md               Project conventions
```

## Key Rules

1. **Engines never import adapter crates.** They receive port implementations via constructor injection.
2. **Engines never call LlmPort directly.** They use AgentPort, which wraps LLM + Tool + Memory (ADR-009).
3. **All domain types have `metadata: serde_json::Value`** for schema evolution without migrations (ADR-007).
4. **Event.kind is a String**, not an enum. Engines define their own constants (ADR-005).
5. **Single job queue** for all async work. No per-engine scheduling (ADR-003).
6. **Human approval gates** on content publishing and outreach sending (ADR-008).
7. **Each crate stays under 2000 lines.** Single responsibility.

## The 10 Core Ports

| Port | Responsibility |
|------|---------------|
| `LlmPort` | Raw model access: generate, stream, embed |
| `AgentPort` | Agent orchestration: create, run, stop, status |
| `ToolPort` | Tool registry + execution |
| `EventPort` | System-wide typed event bus (append-only) |
| `StoragePort` | 5 canonical stores: Event, Object, Session, Job, Metric |
| `MemoryPort` | Context, knowledge, semantic search |
| `JobPort` | Central job queue with approval support |
| `SessionPort` | Session hierarchy management |
| `AuthPort` | Opaque credential handles |
| `ConfigPort` | Settings and preferences |

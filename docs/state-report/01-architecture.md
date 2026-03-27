# Chapter 1 — Architecture & Boot Sequence

## Hexagonal Design

RUSVEL follows ports-and-adapters (hexagonal) architecture:

- **Core** (`rusvel-core`) defines 16 port traits + domain types. Zero framework dependencies.
- **Adapters** implement port traits against real infrastructure (SQLite, LanceDB, Claude CLI, etc.)
- **Engines** contain domain logic. Depend ONLY on `rusvel-core` traits — never import adapter crates.
- **Departments** (`dept-*`) wrap engines in the `DepartmentApp` trait, contributing tools, event handlers, and job handlers to the system.
- **Surfaces** (API, CLI, MCP, TUI) wire adapters into engines and expose them to users.

```
                    ┌──────────────────────┐
                    │     Surfaces         │
                    │  API · CLI · MCP     │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │   rusvel-app          │
                    │  (composition root)   │
                    └──────────┬───────────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
     ┌────────▼──────┐ ┌──────▼──────┐ ┌───────▼───────┐
     │  13 dept-*    │ │  Adapters    │ │   Engines     │
     │  crates       │ │  (concrete)  │ │  (domain)     │
     └───────────────┘ └─────────────┘ └───────────────┘
              │                │                │
              └────────────────┼────────────────┘
                               │
                    ┌──────────▼───────────┐
                    │    rusvel-core        │
                    │  16 port traits       │
                    │  domain types         │
                    │  DepartmentApp trait  │
                    └──────────────────────┘
```

## Composition Root (`rusvel-app/src/main.rs`)

The binary entry point instantiates every adapter and wires them into engines. Key sections:

### Adapter Instantiation Order

```
1. Tracing (env_logger)
2. Data directory (~/.rusvel)
3. Database (SQLite WAL: rusvel.db) → implements StoragePort (5 sub-stores)
4. TomlConfig (config.toml) → implements ConfigPort
5. MultiProvider LLM → ClaudeCliProvider + CursorAgentProvider
6. CostTrackingLlm wraps base LLM with MetricStore spend tracking
7. EventBus → in-memory broadcast + SQLite EventStore persistence
8. MemoryStore (SQLite: memory.db) → implements MemoryPort
9. ToolRegistry → register all builtin tools
10. InMemoryAuthAdapter (from env vars) → implements AuthPort
11. SessionAdapter (wraps StoragePort::sessions()) → implements SessionPort
12. AgentRuntime (LLM + Tool + Memory orchestration) → implements AgentPort
13. FastEmbedAdapter → implements EmbeddingPort (local embedding model)
14. LanceVectorStore → implements VectorStorePort
15. TerminalManager → implements TerminalPort
16. CdpClient → implements BrowserPort
17. FlyDeployPort → implements DeployPort
```

### Department Boot (ADR-014)

After adapters are live:

```
1. Collect all 13 DepartmentApp instances
2. Read manifests (no side effects — pure data)
3. Validate IDs + dependency order (topological sort)
4. Call register() in dependency order, passing RegistrationContext
5. Collect contributed tools, event subscriptions, job handlers
6. Build DepartmentRegistry for API /departments endpoint
7. Spawn event dispatch daemon
```

### Engine Construction

```
1. ForgeEngine (AgentPort, EventPort, MemoryPort, StoragePort, JobPort, SessionPort, ConfigPort)
2. CodeEngine (StoragePort, EventPort)
3. ContentEngine (AgentPort, EventPort, StoragePort, JobPort + platform adapters)
4. HarvestEngine (StoragePort, EventPort, AgentPort, BrowserPort)
5. FlowEngine (StoragePort, EventPort, AgentPort, TerminalPort, BrowserPort)
```

### Post-Boot

```
1. Seed defaults (first run): 5 agents, 5 skills, 3 rules, 4 self-improvement agents, 5 self-improvement skills, 1 workflow
2. Load EventTrigger definitions → register with TriggerManager
3. Spawn job queue worker (background polling loop)
4. Parse CLI args (clap)
5. Load user profile or run onboarding wizard
6. Dispatch: --command → CLI | --tui → TUI | --mcp → MCP stdio | --mcp-http → web+MCP | default → Axum API server
```

## The Four Surface Modes

| Flag | Surface | Crate | Transport |
|------|---------|-------|-----------|
| (default) | Web API | rusvel-api | HTTP/SSE on :3000 |
| `--mcp` | MCP server | rusvel-mcp | stdio JSON-RPC |
| `--mcp-http` | Web + MCP | rusvel-mcp | HTTP + SSE |
| `--tui` | TUI dashboard | rusvel-tui | ratatui terminal |
| `shell` | REPL | rusvel-cli | reedline interactive |
| `<dept> <action>` | CLI one-shot | rusvel-cli | stdout |

## Key Architectural Rules

| ADR | Rule | Enforced By |
|-----|------|-------------|
| ADR-003 | Single job queue for all async work | `rusvel-jobs` in-memory Vec |
| ADR-005 | Event.kind is String, not enum | `rusvel-core` Event type |
| ADR-007 | All domain types carry `metadata: serde_json::Value` | Domain type definitions |
| ADR-008 | Human approval gates on sensitive jobs | JobPort approval flow |
| ADR-009 | Engines use AgentPort, never LlmPort directly | Import restrictions |
| ADR-010 | Engines depend only on rusvel-core traits | Cargo.toml deps |
| ADR-014 | All departments implement DepartmentApp | 13 dept-* crates |

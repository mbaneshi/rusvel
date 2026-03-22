# Patterns to Steal from Reference Repos

> What specific design patterns, architectures, and ideas should RUSVEL adopt from each reference project.

---

## From Our Own Repos

### From forge-project / agentforge-hq → forge-engine
- **EventBus fan-out** — 43 event variants, typed enum, broadcast to all subscribers. Every action in RUSVEL should emit events.
- **Safety layer** — CircuitBreaker (error threshold → open → half-open → closed), RateLimiter (token bucket), CostTracker (per-model budget enforcement). These belong in rusvel-core as port traits.
- **Process spawning** — Stream JSON parsing from CLI stdout. The forge way of spawning Claude Code and parsing streaming output is battle-tested.
- **Git worktree isolation** — Each agent gets its own worktree. Critical for parallel agent safety.
- **Persona catalog** — 100+ personas organized by division (Engineering, QA, Security, Design, etc.). Each persona = system prompt + tool whitelist + budget.
- **UnitOfWork pattern** — Database transactions across multiple repositories. Apply to rusvel-db.
- **Middleware chain** — 7-stage Axum middleware (auth, logging, rate-limit, cors, compression, error-handling, metrics). Standardize for rusvel-api.

### From codeilus → code-engine
- **8-step pipeline** — parse → graph → metrics → analyze → narrate → learn → harvest → export. Each step is a crate. This pipeline-as-crates pattern applies to all engines.
- **tree-sitter incremental parsing** — Parse only what changed. 12+ language grammars.
- **Community detection** — Graph algorithms to find module clusters. Useful beyond code (e.g., finding opportunity clusters in harvest-engine).
- **BM25 search** — Full-text symbol search without external dependencies. Apply to all engines for search.
- **Narrative generation** — LLM takes structured data and generates human-readable stories. Pattern applies to mission-engine (daily briefings) and content-engine.

### From contentforge → content-engine
- **Platform adapter trait** — `publish(content, platform) → Result<PostId>`. Clean trait with per-platform implementations.
- **Content pipeline** — generate → adapt → split → review. Four-stage AI pipeline with human review gate.
- **Engagement tracking** — Polling platform APIs for metrics. Store time-series in SQLite.
- **Ratatui TUI** — Terminal dashboard pattern. Reuse for rusvel-tui.

### From smart-standalone-harvestor → harvest-engine
- **CDP passive interception** — Connect to Chrome on port 9222, intercept network responses without making direct API calls. Undetectable scraping.
- **DNA scanning** — Discover all available data sources on a page before scraping. Smart source discovery.
- **Job scoring** — Multi-factor AI scoring (relevance, budget fit, competition level, skill match). Generalize to "opportunity scoring."
- **Bid generation** — AI generates tailored proposals using profile + job data + past success patterns.
- **Extension + backend** — Browser extension captures data passively, backend processes it. Two-surface pattern.

### From solo-os (dir8) → ops-engine
- **Layered architecture** — Surfaces (web, native, CLI) → Apps (business domains) → Engines (invisible capabilities) → Core (infrastructure). This IS the RUSVEL architecture.
- **Schema-per-domain** — Each engine gets its own database schema (public, solo, throne, harvest, mana, windmill). Apply: each engine owns its SQLite tables via migrations.
- **Unified make orchestration** — `make up`, `make down`, `make status`, `make logs`. Simple operational commands.

### From mclic / Throne Protocol → ops-engine + harvest-engine
- **INBOUND taxonomy** — 23 categories of incoming signals (jobs, gigs, partnerships, social, events, etc.). Use this to structure HarvestPort source types.
- **PROCESS phases** — Capture → Qualify → Prioritize → Execute → Review. Apply to opportunity pipeline in ops-engine.
- **OUTBOUND modes** — 7 delivery modes (direct action, delegation, scheduling, notification, content, outreach, automation). Maps to RUSVEL's surface layer.

### From adk-rust → rusvel-core + rusvel-agent + rusvel-llm
- **Agent trait hierarchy** — `Agent` base trait, then LlmAgent, SequentialAgent, ParallelAgent, LoopAgent, GraphAgent. Steal the trait design.
- **Session + Artifact model** — Session groups conversation turns, artifacts are produced outputs. Add SessionPort to rusvel-core.
- **Content/Part model** — `Content` has `Part` variants (Text, Image, Audio, Video, FunctionCall, FunctionResponse). Universal message type.
- **Guardrails** — Input/output guardrails as middleware (PII redaction, content filtering, JSON schema validation). Add to AgentPort.
- **Tool declaration** — Tools declared with JSON Schema for parameters. Agent runtime validates before calling.
- **Provider abstraction** — `Llm` trait with `generate` and `generate_stream`. Each provider (Claude, OpenAI, Gemini, Ollama) implements it.

### From cine-combined → content-engine
- **Tool registry** — Named tools with categories, each with its own adapter. `register_tool("image_gen", ImageGenTool)`. Generalize for ToolPort.
- **Credit/billing model** — Per-action costs, user credits, tier-based access. Useful for ops-engine (tracking AI spend).
- **Workspace management** — Projects contain assets, settings, and history. Workspace = SessionPort context.

---

## From External Reference Repos

### From Windmill → rusvel-schedule + AutomationPort
- **PostgreSQL/SQLite job queue** — Jobs stored in DB, workers poll with `FOR UPDATE SKIP LOCKED`. Simple, no external queue needed.
- **Worker tags** — Jobs tagged with requirements, workers declare capabilities. Route jobs to appropriate workers.
- **50ms overhead** — Benchmark target. Job dispatch should add minimal latency.
- **Trigger diversity** — Cron, webhook, HTTP route, Kafka, WebSocket, email. Each trigger type is an adapter.
- **Script-to-UI** — Auto-generate form UI from function parameters. Apply: SvelteKit auto-generates forms from tool schemas.

### From cognee → rusvel-memory
- **Knowledge graph** — Entities and relationships extracted from text, stored as graph. Query with graph traversal.
- **Chunking strategies** — Different chunking for different content types (code, prose, conversations).

### From OpenViking → rusvel-memory
- **Tiered memory** — Hot (current session) → Warm (recent, indexed) → Cold (archived, searchable). Three-tier loading for context management.
- **Context window optimization** — Load only relevant memory into LLM context. Score by recency + relevance.

### From mem9 → rusvel-memory
- **Conversation-aware memory** — Memory that understands conversation flow, not just keyword matching.
- **Forgetting curve** — Memory importance decays over time unless reinforced. Prevents context bloat.

### From A2A Protocol → rusvel-agent
- **Agent cards** — Each agent publishes a card describing its capabilities, supported content types, and auth requirements. Apply to persona system.
- **Task lifecycle** — submitted → working → input-required → completed/failed. Standard state machine for agent tasks.
- **Streaming parts** — Tasks can stream partial results as they work. Already have this from forge's WebSocket pattern.

### From OpenClaw ecosystem → rusvel-tool
- **Skill discovery** — Agents discover and load skills dynamically. Skills are tools with context.
- **Skill composition** — Complex behaviors from simple skill chains.

### From CLI-Anything / mcp2cli → rusvel-mcp + rusvel-cli
- **Every capability as CLI** — If it exists in the system, it should be callable from CLI. No hidden features.
- **MCP as universal adapter** — Expose everything via MCP so any AI tool can use RUSVEL.

### From SpacetimeDB → rusvel-db
- **Database as server** — Queries are also subscriptions. When data changes, subscribers get updates. Apply to EventPort + StoragePort integration.

### From Scrapling → harvest-engine
- **Anti-detection** — Rotate user agents, respect rate limits, handle CAPTCHAs gracefully.
- **Structured extraction** — Define extraction schemas, get typed data back.

---

## Meta-Patterns (Apply Everywhere)

1. **Pipeline-as-crates** — Each processing step is its own crate (parse → analyze → transform → output). From codeilus.
2. **Trait-first design** — Define the port trait, then implement adapters. Never depend on concrete types across crate boundaries. From adk-rust.
3. **Event sourcing light** — Every mutation emits an event. Events are stored. State can be rebuilt from events. From forge.
4. **Embedded frontend** — SvelteKit builds to static, rust-embed compiles it into the binary. Zero deployment deps. From forge/codeilus/contentforge.
5. **SQLite WAL + migrations** — Every crate that stores data uses SQLite WAL mode with numbered migrations. From all Tier 1 repos.
6. **Safety by default** — Circuit breaker, rate limiter, cost tracker on every external call. From forge.
7. **Multi-surface** — Same core logic, multiple UIs (CLI, TUI, Web, MCP). From contentforge.
8. **Schema per engine** — Each engine owns its tables. No cross-engine table access. Communicate via ports. From solo-os.

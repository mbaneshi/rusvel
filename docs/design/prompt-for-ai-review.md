# RUSVEL — Design Brief for AI Review

> Send this prompt to Claude, Perplexity, and Gemini for competing design perspectives.

---

## What is RUSVEL?

A Rust + SvelteKit monorepo that serves as the **solo builder's AI-powered virtual agency**.
One binary, one human, infinite leverage.

A solo founder shouldn't need to hire 20 people. RUSVEL is an AI-powered agency that
handles everything — finding gigs, shipping code, publishing content, running the business —
all orchestrated from one system.

## Architecture: Hexagonal (Ports & Adapters)

**Core** = pure Rust traits, zero framework dependencies
**Adapters** = plug-in implementations (SQLite, Claude, Upwork, Twitter, etc.)
**Engines** = domain logic composed from core ports
**Surfaces** = CLI, TUI, Web (SvelteKit), MCP server

## The 13 Core Ports

| Port | Responsibility |
|------|---------------|
| LlmPort | Talk to any AI model (Claude, OpenAI, Gemini, Ollama) |
| AgentPort | Define, configure, and run AI agents |
| AutomationPort | Workflows, pipelines, multi-step execution |
| MemoryPort | Context, knowledge, notes, semantic search |
| ToolPort | Extensible tool system (functions agents can call) |
| EventPort | Everything emits events, system-wide bus |
| StoragePort | Persist anything (files, blobs, structured data) |
| SchedulePort | Cron, triggers, recurring tasks |
| HarvestPort | Scrape, discover, ingest from external sources |
| PublishPort | Push content to any platform |
| AuthPort | Identity, API keys, OAuth tokens |
| ConfigPort | Settings, preferences, per-workspace config |
| SessionPort | Group agents + memory + tools around a project/workspace context |

## The 7 Domain Engines

### 1. Forge Engine (Agent Orchestration)
- Spawn and orchestrate multiple AI agents
- Multi-agent workflows with safety controls (circuit breaker, rate limiter, budget)
- Git worktree isolation for parallel agent work
- 100+ agent personas across divisions
- Real-time WebSocket streaming of agent output
- MCP server mode

### 2. Code Engine (Code Intelligence)
- Parse any codebase via tree-sitter (12+ languages)
- Build dependency graphs, detect communities
- Compute metrics (complexity, churn, coupling)
- Detect anti-patterns and suggest refactors
- C-to-Rust transpilation
- Generate interactive learning paths from codebases
- Full-text BM25 symbol search

### 3. Harvest Engine (Opportunity Discovery)
- Passive job scraping via Chrome DevTools Protocol (CDP)
- Score and rank opportunities using AI
- Auto-generate tailored proposals/bids
- Sources: Upwork, Freelancer, LinkedIn, GitHub trending
- Browser extension for passive interception

### 4. Content Engine (Creation & Publishing)
- Write once in Markdown, AI adapts per platform
- Schedule and publish across: Twitter/X, LinkedIn, DEV.to, Medium, YouTube, Substack
- AI agents for: content generation, adaptation, splitting, review
- Track engagement metrics across platforms
- Video/multimedia content pipeline
- Personal branding system with templates

### 5. Ops Engine (Business Operations)
- CRM and sales pipeline
- Lead qualification and outreach sequences
- Invoice and payment tracking
- SOPs and knowledge base
- Organization/department modeling

### 6. Mission Engine (Goals & Planning)
- Set goals, track progress, daily planning
- `rusvel mission today` → AI generates daily focus from goals
- Decision logging and review
- Performance analytics
- Resource allocation for solo founder

### 7. Connect Engine (Networking & Outreach)
- Email sequences and follow-ups
- LinkedIn networking automation
- Messaging gateway integration
- Relationship tracking

## CLI UX

```
$ rusvel                    → SvelteKit dashboard (all domains)
$ rusvel forge run ...      → spin up AI agents
$ rusvel harvest scan       → find new gigs
$ rusvel content publish    → push content everywhere
$ rusvel ops pipeline       → see your business pipeline
$ rusvel mission today      → what to focus on today
$ rusvel connect outreach   → run outreach sequences
$ rusvel code analyze .     → analyze current codebase
```

## Workspace Structure

```
all-in-one-rusvel/
├── crates/
│   ├── rusvel-core/        ← ports (traits only, zero deps)
│   ├── rusvel-db/          ← StoragePort adapter (SQLite WAL)
│   ├── rusvel-llm/         ← LlmPort adapters (Claude/OpenAI/Gemini/Ollama)
│   ├── rusvel-agent/       ← AgentPort + runner
│   ├── rusvel-event/       ← EventPort + bus
│   ├── rusvel-memory/      ← MemoryPort + semantic search
│   ├── rusvel-tool/        ← ToolPort + registry
│   ├── rusvel-schedule/    ← SchedulePort adapter
│   ├── rusvel-auth/        ← AuthPort adapter
│   ├── rusvel-config/      ← ConfigPort adapter
│   │
│   ├── forge-engine/       ← agent orchestration domain
│   ├── code-engine/        ← code intelligence domain
│   ├── harvest-engine/     ← opportunity discovery domain
│   ├── content-engine/     ← content creation/publish domain
│   ├── ops-engine/         ← business operations domain
│   ├── mission-engine/     ← goals/planning domain
│   ├── connect-engine/     ← outreach/networking domain
│   │
│   ├── rusvel-api/         ← Axum HTTP + WebSocket + SSE
│   ├── rusvel-cli/         ← Clap CLI
│   ├── rusvel-tui/         ← Ratatui TUI
│   ├── rusvel-mcp/         ← MCP server (stdio + SSE)
│   └── rusvel-app/         ← single binary entry point
│
├── frontend/               ← SvelteKit 5 + Tailwind 4
└── Cargo.toml              ← workspace root
```

## Tech Stack

- **Language:** Rust (edition 2024)
- **Frontend:** SvelteKit 5 + Tailwind CSS 4 (embedded in binary via rust-embed)
- **Database:** SQLite with WAL mode
- **Web framework:** Axum
- **Async:** Tokio
- **CLI:** Clap 4
- **TUI:** Ratatui
- **Code parsing:** tree-sitter
- **LLM:** Multi-provider (Claude, OpenAI, Gemini, Ollama)
- **MCP:** rmcp
- **Distribution:** Single compiled binary

## Design Constraints

1. **Single binary** — everything compiles to one executable, frontend embedded
2. **Local-first** — works offline with SQLite + Ollama, cloud optional
3. **Solo founder optimized** — not enterprise, not team. One person running everything
4. **Rust + SvelteKit only** — no Python, no Node in production. Reference repos inspire but we rewrite in Rust
5. **Ports & adapters** — core has zero framework deps. Everything pluggable
6. **Each crate stays small** — single responsibility, <2000 lines ideally
7. **Shared domain types in core** — "Opportunity", "Content", "Agent" are core types, not engine-internal

## Reference Projects We've Studied

These are repos we've explored for inspiration. Extract patterns, not code:

### Agent Frameworks
- **ADK-Rust** (adk-rust.com) — Rust agent SDK with 30+ crates, multi-provider LLM, workflow agents (Sequential/Parallel/Loop/Graph), session management, A2A protocol, MCP support, vector memory, guardrails
- **OpenDevin** — autonomous AI agent for software development
- **OpenJarvis** — open-source personal assistant
- **ghost-os** — agent operating system concept
- **agency-agents** — multi-agent coordination patterns
- **claude-flow** — Claude agent orchestration
- **deer-flow** — agent workflow system
- **MetaClaw / OpenClaw** — agent skill/tool ecosystems
- **mem9** — memory system for agents
- **OpenViking** — tiered memory system
- **cognee** — knowledge graph engine

### Code Intelligence
- **Codeilus** — 16 Rust crates: parse→graph→metrics→analyze→narrate→learn→export
- **C2Rust** — C-to-Rust transpiler with AST transformation
- **emerge** — code structure analysis (Python)
- **PocketFlow-Tutorial-Codebase-Knowledge** — AI codebase tutorials
- **GitDiagram, GitNexus, GitVizz, CodeVisualizer** — code visualization tools
- **tree-sitter** — incremental parsing for 12+ languages

### Agent Orchestration
- **Claude Forge** — 9 crates: multi-agent Claude Code orchestrator with 10 presets, safety controls, WebSocket streaming, cron scheduling, analytics
- **AgentForge HQ** — 13 crates: 100+ personas, org chart, governance, approval workflows
- **Windmill** — open-source Rust+Svelte workflow engine with job queue, sandboxing, triggers (Kafka/MQTT/WebSocket/email), 50ms job overhead
- **n8n** — Node workflow automation (reference for workflow UX)

### Harvesting & Scraping
- **smart-standalone-harvestor** — CDP-based Upwork scraper + Gemini scoring + bid generation
- **Freelancer scraper** — Freelancer.com job interception
- **linkedin-captured** — Chrome CDP tab capture
- **Scrapling** — web scraping framework
- **apify-upwork** — Upwork scraping via Apify

### Content & Media
- **ContentForge** — 11 Rust crates: write Markdown → adapt per platform → schedule → publish
- **Mana** — AI real estate video: cloud agents (Python ADK) + macOS compositor (Swift AVFoundation)
- **Cinefilm** — AI creative studio: FastAPI + Angular + Vertex AI + ComfyUI
- **content-auto-machine** — content automation pipeline
- **MoneyPrinterV2** — automated video content generation
- **tome** — documentation generator

### Business/Solo Founder OS
- **Solo OS (v1)** — Next.js + FastAPI: 5 agent domains (Content, Sales, Marketing, Knowledge, Executive)
- **Solo OS (v2)** — Docker: unified OS with Windmill + n8n + Ollama + Supabase + Redis
- **Throne Protocol** — personal AI OS: INBOUND(23 categories) → PROCESS(5 phases) → OUTBOUND(7 modes)
- **founders-kit** — startup resources and playbooks
- **gstack** — Garry Tan's tech stack

### Infrastructure Patterns
- **Supabase self-hosted** — PostgreSQL + Auth + Realtime + Storage + Edge Functions
- **Tailscale** — mesh networking
- **SpacetimeDB** — database + server merged concept
- **Windmill** — Rust backend + Svelte frontend + PostgreSQL job queue

### Robotics
- **Carokia Brain** — Rust AI decision engine for robotics

### Developer Tools
- **CLI-Anything** — making software agent-native via CLI
- **mcp2cli** — one CLI for every API
- **chrome-cdp-skill** — AI agents + live Chrome sessions via CDP

## What I Want From You

1. **Critique this architecture** — what's missing, what's redundant, what will break at scale?
2. **Suggest the SessionPort design** — how should workspace/project/session context work across all engines?
3. **Define shared domain types** — what types belong in rusvel-core that engines share?
4. **Propose the first vertical slice** — the thinnest end-to-end path that proves the architecture works
5. **Identify patterns from reference repos** — what specific design patterns should we steal from the repos listed above?
6. **Risk assessment** — what are the top 3 things most likely to go wrong?
7. **What am I not seeing?** — blind spots, missing domains, architectural traps

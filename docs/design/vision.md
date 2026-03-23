# RUSVEL — Vision Document

> The Solo Builder's AI-Powered Virtual Agency
> One binary. One human. Infinite leverage.

---

## The Problem

A solo founder wears every hat: developer, marketer, sales, ops, recruiter, content creator, project manager. The tools exist to help — but they're scattered across dozens of SaaS platforms, languages, and workflows. Each tool solves one problem but creates integration overhead.

The result: a solo founder spends more time managing tools than doing work.

## The Solution

RUSVEL is a single Rust binary that replaces an entire agency. It combines AI agents, automation, and domain-specific intelligence into one system that a solo founder controls from a unified interface.

**Not a platform for teams. Not enterprise software. A personal superpower.**

## Core Philosophy

1. **One binary, zero ops** — `cargo install rusvel` or download a release. No Docker, no cloud, no configuration required. SQLite for storage, Ollama for local AI. Cloud services are optional adapters.

2. **Ports & adapters** — The core is pure Rust traits with zero framework dependencies. Everything pluggable. Swap Claude for Ollama. Swap SQLite for Supabase. Swap Twitter for LinkedIn. The core doesn't care.

3. **Agents do the work** — The founder decides what to do. AI agents figure out how and execute. The system orchestrates multiple agents working in parallel across domains.

4. **Local-first, cloud-optional** — Everything works offline. When online, cloud services enhance capabilities (better LLMs, more sources, broader publishing).

5. **Small crates, deep modules** — Each crate has a small interface and significant implementation. Under 2000 lines. Single responsibility. The workspace is the composition layer.

## Architecture

```
┌──────────────────────── SURFACES ─────────────────────────┐
│  CLI (Clap)  │  REPL (reedline)  │  TUI (Ratatui)        │
│  Web (Svelte)│  MCP Server                                │
└──────────────────────────┬────────────────────────────────┘
                       │
┌──────────────────────┴─────────────────────────┐
│              DOMAIN ENGINES                     │
│                                                 │
│  Forge    │ Code    │ Harvest  │ Content        │
│  Ops      │ Mission │ Connect                   │
└──────────────────────┬─────────────────────────┘
                       │ (depends on)
┌──────────────────────┴─────────────────────────┐
│              FOUNDATION (Ports + Adapters)       │
│                                                 │
│  rusvel-core   (traits only, zero deps)         │
│  rusvel-llm    (Claude/OpenAI/Gemini/Ollama)    │
│  rusvel-agent  (agent runtime + workflows)      │
│  rusvel-db     (SQLite WAL + migrations)        │
│  rusvel-event  (event bus + persistence)        │
│  rusvel-memory (context + semantic search)      │
│  rusvel-tool   (tool registry + execution)      │
│  rusvel-schedule (cron + triggers)              │
│  rusvel-auth   (API keys + OAuth)               │
│  rusvel-config (settings + workspace config)    │
└─────────────────────────────────────────────────┘
```

## The 13 Core Ports (in rusvel-core)

| Port | Responsibility | Key Methods |
|------|---------------|-------------|
| `LlmPort` | Talk to any AI model | `generate`, `generate_stream`, `embed` |
| `AgentPort` | Define and run AI agents | `create`, `run`, `stop`, `status` |
| `AutomationPort` | Multi-step workflows | `define_workflow`, `execute`, `pause`, `resume` |
| `MemoryPort` | Context and knowledge | `store`, `recall`, `search`, `forget` |
| `ToolPort` | Extensible tool system | `register`, `call`, `list`, `schema` |
| `EventPort` | System-wide event bus | `emit`, `subscribe`, `replay` |
| `StoragePort` | Persist anything | `put`, `get`, `delete`, `query`, `migrate` |
| `SchedulePort` | Cron and triggers | `schedule`, `cancel`, `list`, `trigger` |
| `HarvestPort` | Scrape and discover | `scan`, `extract`, `score`, `ingest` |
| `PublishPort` | Push to platforms | `publish`, `schedule_post`, `metrics` |
| `AuthPort` | Identity and keys | `authenticate`, `store_key`, `get_key`, `refresh` |
| `ConfigPort` | Settings and prefs | `get`, `set`, `watch`, `export` |
| `SessionPort` | Workspace/project context | `create_session`, `load`, `save`, `switch` |

## The 7 Domain Engines

### Forge Engine — Agent Orchestration
The meta-engine. Forge doesn't do domain work itself — it orchestrates agents that use other engines. "Find me 5 Rust gigs and draft pitches" → Forge spawns Harvest agent + Content agent, coordinates, returns results.

- 100+ agent personas organized by division
- Workflow patterns: Sequential, Parallel, Loop, Graph
- Safety: circuit breaker, rate limiter, budget enforcement
- Git worktree isolation for code-modifying agents
- Real-time streaming of agent work

### Code Engine — Code Intelligence
Understand, analyze, transform, and learn from any codebase.

- Parse 12+ languages via tree-sitter
- Build dependency graphs, detect module communities
- Compute complexity, churn, coupling metrics
- Detect anti-patterns, suggest refactors
- C-to-Rust transpilation pipeline
- Generate learning paths and interactive tutorials
- Full-text BM25 symbol search

### Harvest Engine — Opportunity Discovery
Find gigs, jobs, and opportunities from everywhere.

- Passive scraping via Chrome DevTools Protocol
- Source adapters: Upwork, Freelancer, LinkedIn, GitHub, custom
- AI-powered scoring (relevance, budget, competition, skill match)
- Automated proposal/bid generation
- Pipeline: discover → score → qualify → propose → track

### Content Engine — Creation & Publishing
Write once, publish everywhere with AI adaptation.

- Markdown-first authoring
- AI pipeline: generate → adapt → split → review
- Platform adapters: Twitter/X, LinkedIn, DEV.to, Medium, YouTube, Substack
- Scheduling with cron
- Engagement tracking and analytics
- Personal branding templates and voice consistency

### Ops Engine — Business Operations
Run your solo business.

- CRM: contacts, leads, deals, pipeline stages
- Invoice generation and payment tracking
- SOPs and knowledge base
- Organization modeling (even for a solo founder — track clients, contractors, partners)
- AI spend tracking and budget management

### Mission Engine — Goals & Planning
Know what to do and track progress.

- Set quarterly/monthly/weekly goals
- `rusvel mission today` → AI daily brief from goals + pipeline + opportunities
- Decision logging with context
- Progress analytics and velocity tracking
- Review cycles (weekly retro, monthly review)

### Connect Engine — Networking & Outreach
Build and maintain professional relationships.

- Contact management with relationship scoring
- Email sequence automation
- LinkedIn networking workflows
- Follow-up scheduling and reminders
- Outreach templates with personalization

## Shared Domain Types (in rusvel-core)

```rust
// Universal content type (from adk-rust's Content/Part model)
Content { parts: Vec<Part> }
Part::Text | Part::Image | Part::Audio | Part::Video | Part::File

// Cross-engine types
Opportunity { source, title, score, status, metadata }
Contact { name, channels, relationship_score, last_interaction }
ContentPiece { body, format, platform_variants, schedule, metrics }
Goal { description, deadline, progress, sub_goals }
AgentTask { agent_id, workflow, status, cost, events }
Session { id, workspace, agents, memory_scope, created_at }
```

## Three-Tier CLI UX

### Tier 1 — One-shot Commands
```bash
# Start the web dashboard
$ rusvel

# Session management
$ rusvel session create my-startup
$ rusvel session list
$ rusvel session switch <id>

# Forge engine
$ rusvel forge mission today
$ rusvel forge mission goals
$ rusvel forge mission review --period week

# Department commands (all 11 departments)
$ rusvel finance status
$ rusvel finance list --kind transactions
$ rusvel growth events
$ rusvel harvest status
$ rusvel content list
$ rusvel infra status
$ rusvel legal events
$ rusvel support list --kind tickets
$ rusvel product status
$ rusvel distro list
$ rusvel gtm status
```

### Tier 2 — Interactive REPL Shell
```bash
$ rusvel shell
RUSVEL Interactive Shell
Type 'help' for commands, 'exit' to quit.

rusvel> use finance              # Switch to department context
rusvel:finance> status           # Department-scoped commands
rusvel:finance> list transactions
rusvel:finance> events
rusvel:finance> back             # Leave department context
rusvel> session list             # Session management
rusvel> status                   # All departments overview
rusvel> exit
```
- Tab autocomplete for commands and departments
- Persistent history (`~/.rusvel/shell_history.txt`)
- Ctrl+R for history search

### Tier 3 — TUI Dashboard
```bash
$ rusvel --tui                   # Full-screen terminal dashboard
```
- 4-panel layout: Tasks, Goals, Pipeline, Events
- Loads live data from storage
- Press `q` or `Esc` to exit

## First Vertical Slice

The thinnest end-to-end proof that the architecture works:

1. **rusvel-core** — Define all 13 port traits + shared domain types
2. **rusvel-config** — Load TOML config file
3. **rusvel-db** — SQLite WAL with migration system
4. **rusvel-event** — In-memory event bus with persistence
5. **rusvel-llm** — Ollama adapter (local, no API key needed)
6. **rusvel-memory** — Store/recall with basic text search
7. **rusvel-tool** — Tool registry with JSON Schema declarations
8. **rusvel-agent** — Simple LLM agent that can use tools
9. **mission-engine** — `today` command: read goals from memory, generate daily plan via LLM
10. **rusvel-cli** — `rusvel mission today` command
11. **rusvel-api** — Single endpoint: GET /api/mission/today
12. **frontend** — One SvelteKit page showing today's plan

One command. End to end. Foundation proven.

## What RUSVEL Is NOT

- Not a SaaS platform (it's a personal tool)
- Not for teams (it's for one person)
- Not a framework (it's a product)
- Not cloud-dependent (local-first)
- Not another wrapper around ChatGPT (it's an agent system with domain intelligence)

## Name

**RUSVEL** = **Rus**t + S**vel**teKit

The stack IS the name. Simple. Memorable. Searchable.

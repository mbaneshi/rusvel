# RUSVEL — Complete Repository Inventory

> Cataloged: 2026-03-21
> Total repos scanned: ~60 original + ~200 reference/cloned

---

## Tier 1 — Direct Merge (Rust + SvelteKit, same architecture)

These repos share the exact same stack and patterns. Their code and design inform RUSVEL directly.

### forge-project
- **Path:** `/Users/bm/claude-parent/forge-project`
- **What:** Multi-agent Claude Code orchestrator
- **Stack:** Rust (Axum) + SvelteKit 5 + SQLite WAL
- **Crates (9):** forge-core, forge-agent, forge-db, forge-process, forge-safety, forge-api, forge-app, forge-git, forge-mcp-bin
- **Key patterns:** EventBus (35 event variants), 10 agent presets, WebSocket streaming, circuit breaker, rate limiter, cost tracker, cron scheduler, git worktree isolation, rust-embed frontend
- **Status:** v0.5.0 shipped, 150 tests
- **Harvest for RUSVEL:** Event system design, safety controls, process spawning, embedded frontend pattern

### agentforge-hq
- **Path:** `/Users/bm/cod/trend/10-march/agentforge-hq`
- **What:** Evolution of forge with organizational layer
- **Stack:** Rust (Axum) + SvelteKit 5 + SQLite WAL
- **Crates (13):** Everything in forge + forge-org, forge-persona, forge-governance, forge-mcp
- **Key patterns:** 100+ persona catalog (11 divisions), org chart builder, approval governance, UnitOfWork pattern, 13 migrations, 17 repositories
- **Status:** Epic 1 baseline, 43 event variants
- **Harvest for RUSVEL:** Persona system, org modeling, governance/approval layer

### codeilus
- **Path:** `/Users/bm/codeilus/codeilus`
- **What:** Gamified codebase learning tool
- **Stack:** Rust (Axum) + SvelteKit 5 + Tailwind 4 + SQLite
- **Crates (16):** codeilus-core, -parse, -graph, -metrics, -analyze, -diagram, -search, -llm, -narrate, -learn, -harvest, -export, -mcp, -api, -app, -db
- **Key patterns:** tree-sitter incremental parsing, BM25 search, dependency graph + community detection, complexity/churn/coupling metrics, LLM narrative generation, learning path generation, static site export
- **Status:** Working, Homebrew/Cargo distribution
- **Harvest for RUSVEL:** Code analysis pipeline, multi-language parsing, graph algorithms, search engine

### contentforge
- **Path:** `/Users/bm/contentforge`
- **What:** Content creation + multi-platform publishing
- **Stack:** Rust (Axum) + SvelteKit + SQLite WAL
- **Crates (11):** contentforge-core, -db, -publish, -agent, -schedule, -analytics, -api, -cli, -tui, -mcp, -app
- **Key patterns:** Platform adapters (DEV.to, Twitter/X, LinkedIn, Medium), AI content pipeline (generate→adapt→split→review), cron scheduling, Ratatui TUI, engagement metrics
- **Status:** Working
- **Harvest for RUSVEL:** PublishPort adapters, content pipeline, TUI patterns, analytics

### platform
- **Path:** `/Users/bm/cod/trend/17-march/platform`
- **What:** Already merges forge + codeilus into one workspace
- **Stack:** Rust + SvelteKit 5 + Tailwind 4 + SQLite
- **Crates (23):** Combined codeilus-* + forge-* crates
- **Key patterns:** Shared infrastructure layers (Code Intelligence, Agent Orchestration, Memory + Knowledge), 37 MCP tools, 409 tests, 14 planned commercial products
- **Status:** Sprint 0 complete
- **Harvest for RUSVEL:** Proof that merge works, workspace layout, shared dependency patterns

### c2rust
- **Path:** `/Users/bm/cod/dir6/c2rust`
- **What:** C-to-Rust transpiler
- **Stack:** Rust + Next.js (frontend planned)
- **Crates (6):** c2rust-parser, -ast, -transpiler, -cli, -ai, -server
- **Key patterns:** lang-c parser, AST transformation pipeline, planned LLM-powered repair loops
- **Status:** ~12% complete
- **Harvest for RUSVEL:** AST transformation patterns, transpilation pipeline design

---

## Tier 2 — Rewrite into Rust (valuable domain logic, wrong stack)

These repos have proven domain logic but are written in Python/TypeScript/etc. Their DESIGN informs RUSVEL engines.

### adk-rust
- **Path:** `/Users/bm/cod/dir6/adk-rust`
- **What:** Comprehensive Rust agent development kit
- **Stack:** Rust, 30+ crates
- **Key patterns:** Agent types (LLM, Sequential, Parallel, Loop, Graph), multi-provider LLM (Gemini/OpenAI/Claude/Ollama), session management, long-term memory with vector embeddings, A2A protocol, MCP support, guardrails (PII, filtering, schema validation), OpenTelemetry, 80+ examples
- **Status:** v0.3.0, production-ready
- **Harvest for RUSVEL:** LlmPort trait design, agent workflow patterns, session/artifact model, memory with semantic search, guardrails

### smart-standalone-harvestor
- **Path:** `/Users/bm/cod/dir7/smart-standalone-harvestor`
- **What:** Upwork job scraper + AI proposal generator
- **Stack:** TypeScript (CDP scraper) + Python (FastAPI agents) + PostgreSQL
- **Key patterns:** Chrome DevTools Protocol interception, passive scraping (no API needed), Drizzle ORM schema, Gemini-powered job scoring, automated bid generation, multi-agent orchestration (analyst + strategist)
- **Status:** Working
- **Harvest for RUSVEL:** HarvestPort design, CDP scraping pattern, scoring algorithms, proposal generation workflow

### solo-os (dir4)
- **Path:** `/Users/bm/cod/dir4/claude-sale/solo-os`
- **What:** AI business OS with 5 agent domains
- **Stack:** Next.js 16 + FastAPI + Google ADK + Firebase
- **Key patterns:** 5 domain agents (Content, Sales, Marketing, Knowledge, Executive), CopilotKit chat UI, Firestore CRUD, organization/user/conversation/lead/campaign models
- **Status:** Working
- **Harvest for RUSVEL:** Business domain models, agent domain structure, executive workflow patterns

### solo-os (dir8)
- **Path:** `/Users/bm/cod/dir8/solo-os`
- **What:** Unified personal OS integrating open-source tools
- **Stack:** Docker + Supabase + TypeScript + Makefile orchestration
- **Key patterns:** Layered architecture (Surfaces→Apps→Engines→Core), Throne/Harvest/Mana as apps, n8n+Windmill+Ollama+OpenWebUI as engines, single Supabase DB, Redis cache, MinIO storage, Traefik routing, per-component database schemas
- **Status:** Architecture defined
- **Harvest for RUSVEL:** Layered architecture concept, unified infrastructure model, schema-per-domain pattern

### mclic (Throne Protocol)
- **Path:** `/Users/bm/cod/dir7/mclic`
- **What:** Personal AI OS vision
- **Stack:** Next.js + FastAPI + Swift + Kotlin (planned)
- **Key patterns:** INBOUND (23 categories, 300+ sources) → PROCESS (5 phases, 100+ functions) → OUTBOUND (7 modes, 150+ channels), hexagonal architecture
- **Status:** Phase 0 (vision only, no implementation)
- **Harvest for RUSVEL:** INBOUND→PROCESS→OUTBOUND taxonomy, source categorization, channel mapping

### next-level-mcli (Throne OS)
- **Path:** `/Users/bm/cod/dir7/next-level-mcli`
- **What:** Iteration of Throne Protocol
- **Stack:** Same as mclic
- **Status:** Phase 0
- **Harvest for RUSVEL:** Same as mclic (pick the more refined version)

### mana
- **Path:** `/Users/bm/cod/dir7/mana`
- **What:** AI real estate video content platform
- **Stack:** Python (Google ADK on Vertex AI) + Swift (SwiftUI + AVFoundation)
- **Key patterns:** Director agent orchestrating 8 sub-agents (Vision Analyst, Script Writer, Veo Video Generator, TTS Narrator, Imagen Editor, Asset Scout, Quality Reviewer, Pipeline Manager), Google Cloud Storage artifacts, Firebase sync, property analysis workflow
- **Status:** Architecture defined
- **Harvest for RUSVEL:** Multi-agent creative pipeline design, director/worker agent pattern, asset management

### cine-combined
- **Path:** `/Users/bm/cod/in-progress/cine-combined`
- **What:** AI creative production studio
- **Stack:** Python (FastAPI) + Angular 18 + Vertex AI + ComfyUI
- **Key patterns:** Modular tool registry (image, video, character, fashion, music, speech, scene, branding), Stripe billing (Creator/Pro/Studio tiers), SSE streaming with AG-UI protocol, workspace management, credit system
- **Status:** Working
- **Harvest for RUSVEL:** Tool registry pattern, billing/credit model, workspace management, SSE streaming

### linkedin-captured
- **Path:** `/Users/bm/cod/in-progress/linkedin-captured`
- **What:** Browser tab capture via CDP
- **Stack:** TypeScript + Bun + Playwright
- **Status:** Basic skeleton
- **Harvest for RUSVEL:** CDP connection pattern (fold into harvest-engine)

---

## Tier 3 — Reference / Infrastructure (don't merge, learn from)

### carokia-brain
- **Path:** `/Users/bm/carokia-brain`
- **What:** Rust AI decision engine for robotics
- **Stack:** Rust
- **Harvest for RUSVEL:** Decision engine patterns, hardware agent abstraction (future AgentPort adapter)

### carokia-web
- **Path:** `/Users/bm/carokia-web`
- **What:** Robotics website
- **Stack:** SvelteKit + Rust
- **Harvest for RUSVEL:** SvelteKit + Rust integration patterns

### windmill (upstream)
- **Path:** `/Users/bm/cod/dir7/windmill`
- **What:** Open-source workflow engine
- **Stack:** Rust (backend) + Svelte 5 (frontend) + PostgreSQL
- **Key patterns:** PostgreSQL job queue with worker polling, worker tags for routing, 50ms overhead, sandboxed execution (nsjail), triggers (Kafka/MQTT/WebSocket/email/SQS), modular API crates
- **Harvest for RUSVEL:** Job queue design, worker patterns, trigger system, modular API crate splitting

### supabase-self-official
- **Path:** `/Users/bm/cod/dir7/subabase-self-official`
- **What:** Self-hosted Supabase
- **Stack:** Docker Compose + 12+ microservices
- **Harvest for RUSVEL:** Auth patterns, real-time subscription model, storage API design

### content-mbaneshi
- **Path:** `/Users/bm/content-mbaneshi`
- **What:** Content management docs + brand kit
- **Stack:** Pure Markdown + bash scripts
- **Harvest for RUSVEL:** Brand voice templates, editorial calendar structure, content workflow (replaced by content-engine)

### glass-ios-appl
- **Path:** `/Users/bm/glass-ios-appl`
- **What:** Monorepo with GlassForge + references
- **Stack:** Swift + links to Rust repos
- **Harvest for RUSVEL:** iOS modernization analysis rules (future code-engine feature)

### awave
- **Path:** `/Users/bm/awave/`
- **What:** Brainwave entrainment app
- **Stack:** Swift (iOS) + Kotlin (Android)
- **Harvest for RUSVEL:** Future mobile surface, audio processing patterns

### dripin
- **Path:** `/Users/bm/dripining/`
- **What:** Multi-tenant CMS + SiteProof AI
- **Stack:** JavaScript
- **Harvest for RUSVEL:** Multi-tenant patterns, CMS domain model

---

## Tier 4 — Cloned Reference Repos (inspiration only)

### Agent/AI References (~50 repos)
- `adk` (Google official), `adk-python`, `adk-js`, `adk-samples`, `adk-demos` — Google ADK ecosystem
- `A2A` (Google official), `a2a-samples`, `a2a-python`, `a2a-js` — Agent-to-Agent protocol
- `OpenDevin` — autonomous coding agent
- `openclaw`, `clawhub`, `clawra`, `zeroclaw`, `nanobot` — OpenClaw ecosystem
- `claude-code`, `claude-flow`, `Auto-claude-code-research-in-sleep` — Claude tooling
- `agency-agents`, `hermes-agent`, `deer-flow` — agent orchestration
- `ghost-os`, `OpenJarvis`, `OpenMAIC` — agent OS concepts
- `mem9`, `OpenViking`, `cognee` — memory/knowledge systems
- `MetaClaw`, `Agent-Skills-for-Context-Engineering` — agent skills
- `CLI-Anything`, `mcp2cli`, `onecli` — CLI agent patterns

### Code/Dev Tool References (~15 repos)
- `emerge` — code structure analysis
- `PocketFlow-Tutorial-Codebase-Knowledge` — AI codebase tutorials
- `GitDiagram`, `GitNexus`, `GitVizz`, `CodeVisualizer`, `GitHubTree` — code visualization
- `chrome-cdp-skill`, `chrome-devtools-mcp`, `playwright-mcp` — browser automation
- `Immunant/c2rust` — original C2Rust transpiler

### Business/Productivity References (~15 repos)
- `founders-kit`, `gstack` — startup resources
- `superpowers` — developer superpowers
- `MoneyPrinterV2` — automated content monetization
- `seomachine` — SEO automation
- `Scrapling` — web scraping
- `OctoBot` — trading bot
- `evershop` — e-commerce
- `posthog` — analytics
- `mindsdb` — AI database

### Infrastructure References (~15 repos)
- `tailscale`, `headscale-ui`, `tsdproxy` — mesh networking
- `SpacetimeDB` — database + server merged
- `Windmill` — workflow engine
- `baserow` — open-source Airtable
- `ComfyUI` — image generation UI
- `LocalAI` — local AI inference
- `coder` — remote dev environments

---

## Key Metrics

| Category | Count |
|----------|-------|
| Tier 1 (direct merge) | 6 repos, ~78 crates |
| Tier 2 (rewrite to Rust) | 10 repos |
| Tier 3 (reference/infra) | 8 repos |
| Tier 4 (cloned inspiration) | ~200 repos |
| **Total original projects** | **~35** |
| **Total reference repos** | **~200** |

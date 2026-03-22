# RUSVEL — Current State & Next Steps

> Last updated: 2026-03-22
> Phase: 0 (Foundation) — partially complete

---

## Where We Want to Reach (The Vision)

RUSVEL is a **solo builder's AI-powered virtual agency** — one binary that gives a single person the leverage of an entire agency. It orchestrates AI agents across five domains:

| Engine | What it does |
|--------|-------------|
| **Forge** | Agent orchestration + mission planning (goals, daily plans, reviews) |
| **Code** | Code intelligence (parse, analyze, search, metrics) |
| **Harvest** | Opportunity discovery (scrape CDPs, score gigs, generate proposals) |
| **Content** | Content creation & publishing (draft → adapt → schedule → publish) |
| **GoToMarket** | CRM, outreach sequences, invoicing, deal pipeline |

The end state: you say `rusvel forge mission today` and it plans your day across all five engines. You say `rusvel harvest scan` and it finds gigs. You say `rusvel content draft "topic"` and it writes, adapts for platforms, schedules with your approval. All from one binary, local-first, with AI agents doing the work.

---

## What's Built & Working (Verified 2026-03-22)

### Compiles & Tests Pass
- **20 crates**, ~13,700 lines of Rust
- **149 tests**, all passing, < 1 second total
- Compiles clean (1 dead-code warning in forge-engine)

### Fully Functional (tested end-to-end)

| Feature | CLI Command | Status |
|---------|------------|--------|
| Create session | `rusvel session create <name>` | Works — persists to SQLite |
| List sessions | `rusvel session list` | Works — shows active marker |
| Switch session | `rusvel session switch <id>` | Works |
| Add goal | `rusvel forge mission goal add <title>` | Works — persists + emits event |
| List goals | `rusvel forge mission goals` | Works |
| API server | `cargo run` (no subcommand) | Starts on :3000 |
| Health check | `GET /api/health` | Works |

### AI-Powered Features (via Claude Max subscription)

| Feature | CLI Command | Status |
|---------|------------|--------|
| Daily plan | `rusvel forge mission today` | Works — calls Claude via `claude -p` CLI |
| Weekly review | `rusvel forge mission review` | Works — same Claude CLI path |

**LLM wiring:** Uses `ClaudeCliProvider::max_subscription()` which spawns `claude -p` with env vars for Max subscription ($0 API credits). The Claude CLI JSON output (array format) is parsed correctly, including markdown code fence stripping for JSON responses.

### Built with Tests, but NO CLI/API Surface

These engines have real logic and passing tests, but you can't reach them from the CLI or API:

| Engine | Lines | Tests | What it can do (in code) |
|--------|-------|-------|-------------------------|
| **content-engine** | ~1,000 | 7 | Draft content, adapt for platforms (Twitter/LinkedIn/etc.), schedule via calendar, publish with approval gates, track analytics |
| **harvest-engine** | ~1,000 | 12 | Define sources (Upwork/LinkedIn/GitHub), scan with scoring pipeline, generate proposals, manage opportunity pipeline (Cold→Won/Lost) |
| **gtm-engine** | ~870 | 5 | CRM contacts, outreach sequences (multi-step with delays), invoicing, deal stage management |
| **code-engine** | ~880 | 6 | Parse Rust files, build dependency graphs, BM25 search across codebase, code metrics (lines, complexity) |

### Built but Not Wired

| Component | What it does | What's missing |
|-----------|-------------|----------------|
| **rusvel-agent workflow** | Sequential, Parallel, Loop execution patterns | No CLI/API to trigger workflows |
| **10 agent personas** | CodeWriter, Tester, SecurityAuditor, ContentWriter, etc. | `hire_persona()` exists but no CLI command |
| **rusvel-mcp** | MCP server (stdio JSON-RPC) | Imported but `--mcp` flag not dispatched in main |
| **rusvel-tui** | Terminal UI layout + widgets | Not wired into main |
| **Multi-provider LLM** | Ollama + Claude API + Claude CLI + OpenAI | Claude CLI wired; MultiProvider router unused |
| **Job queue worker** | `JobPort` with enqueue/dequeue/approve | No worker loop to process jobs |
| **rust-embed** | Embed frontend in binary | Not integrated (serving from filesystem instead) |

---

## Phase 0 Milestones (from `phase-0-foundation-v2.md`)

| Milestone | Description | Status |
|-----------|-------------|--------|
| **0.1** | rusvel-core: 10 port traits + domain types | DONE |
| **0.2** | Adapters: db, llm, event, memory, tool, auth, config, jobs | DONE |
| **0.3** | Forge Engine: agent orchestration + mission | DONE — Claude CLI wired, generates plans |
| **0.4** | CLI: `rusvel forge mission today` works | DONE — full flow works end-to-end |
| **0.5** | API + Web: Axum HTTP + SvelteKit dashboard | DONE — API + frontend served from same :3000 |
| **0.6** | MCP: stdio + SSE server | BUILT, not wired |

**Phase 0 Definition of Done (not yet met):**
- [x] `rusvel forge mission today` works end-to-end (CLI → Agent → Claude CLI → plan)
- [x] Same flow works via API (`GET /api/sessions/:id/mission/today`)
- [x] Same flow works via web dashboard (http://localhost:3000/forge)
- [ ] MCP server exposes forge commands (deferred)
- [ ] Central job queue processes at least one job type
- [ ] Human approval model wired (not just defined)
- [ ] Single binary < 50MB with embedded frontend (currently serves from filesystem)

---

## The 5-Phase Roadmap (from `roadmap-v2.md`)

| Phase | Focus | Status |
|-------|-------|--------|
| **0** | Foundation + Forge mission vertical slice | **IN PROGRESS** — 70% |
| **1** | Agent runtime (workflows, graph), Code intelligence, multi-LLM, embeddings, observability | Code engine built but not exposed |
| **2** | Harvest + Content revenue engines, TUI dashboard | Engines built, need CLI + approval flow |
| **3** | GoToMarket complete: CRM, outreach, invoicing, email/LinkedIn adapters | Engine built, need CLI + real adapters |
| **4** | Cross-engine intelligence: Forge orchestrates all engines, learning from outcomes | Not started |
| **5** | Ecosystem: plugins, A2A protocol, browser extension, community personas | Not started |

---

## Architecture Principles (from 10 ADRs)

These are the rules we committed to. The code follows them:

1. **5 engines, not 7** — Ops+Connect merged to GoToMarket, Mission folded into Forge (ADR-001, ADR-002)
2. **Single job queue** — no per-engine scheduling; one SQLite queue for all async work (ADR-003)
3. **5 canonical stores** — Events, Objects, Sessions, Jobs, Metrics (ADR-004)
4. **Event.kind is String** — engines define their own constants, core stays minimal (ADR-005)
5. **HarvestPort/PublishPort are engine-internal** — not core ports (ADR-006)
6. **metadata: serde_json::Value on everything** — schema evolution without migrations (ADR-007)
7. **Human-in-the-loop from day one** — publish and outreach require approval (ADR-008)
8. **Engines use AgentPort, never LlmPort** — clean boundary (ADR-009)
9. **Engines depend only on rusvel-core traits** — never import adapter crates (ADR-010)

---

## Immediate Next Steps (Priority Order)

### 1. Fix LLM wiring (unblocks everything)
Wire `MultiProvider` in `rusvel-app/main.rs` with Ollama as default provider. Change agent default model to one that's actually installed (e.g., `qwen3:14b`). This makes `forge mission today` and `forge mission review` work.

### 2. Complete Phase 0 vertical slice
- Test `mission today` end-to-end (CLI → Agent → Ollama → plan output)
- Test same via API endpoint
- Wire `--mcp` flag dispatch (one if-branch in main.rs)

### 3. Expose one more engine via CLI
Add `rusvel content draft "topic"` or `rusvel harvest scan` — prove the architecture works for a second engine, not just forge.

### 4. Wire job queue worker loop
The queue can enqueue jobs but nothing processes them. Add a simple worker that dequeues and dispatches to engines.

### 5. Build real frontend
The SvelteKit shell exists. Build the dashboard: session list, goal view, daily plan display, approval UI.

---

## Available LLM Models (local Ollama, verified running)

```
qwen3:30b-a3b     18 GB
qwen3:14b          9.3 GB
qwen2.5-coder:14b  9.0 GB
gemma3:27b         17 GB
gemma3:latest      3.3 GB
deepseek-r1:14b    9.0 GB
llama3.1:8b        4.9 GB
qwen2.5:7b         4.7 GB
```

---

## Reference Repos (key sources we're drawing from)

| Repo | What we took | Status |
|------|-------------|--------|
| **forge-project** | Event bus, safety controls, WebSocket | Partially integrated (events, safety guard) |
| **agentforge-hq** | 100+ personas, org chart, approval | Personas integrated (10 built-in) |
| **codeilus** | Parse→graph→metrics→search pipeline | Integrated into code-engine |
| **contentforge** | Platform adapters, AI pipeline, TUI | Integrated into content-engine |
| **smart-standalone-harvestor** | CDP scraping, scoring, proposals | Integrated into harvest-engine |
| **solo-os** | Business domains, CRM patterns | Integrated into gtm-engine |
| **adk-rust** | Agent traits, session model, multi-LLM | Influenced rusvel-core + rusvel-agent |
| **windmill** | SQLite job queue, worker patterns | Influenced rusvel-jobs |

# RUSVEL — Roadmap v2 (Post-Review)

> 5 engines (was 7). Central job queue. Session-centric everything.

---

## Phase 0 — Foundation ← YOU ARE HERE
> Prove the architecture with one vertical slice.

**Deliverable:** `rusvel forge mission today` via CLI, Web, MCP.

- rusvel-core (10 ports + shared types + approval model)
- Adapters: config, db (5 stores), event, llm (Ollama), memory (FTS5), tool, jobs, auth
- forge-engine (agent orchestration + mission goals/planning)
- Surfaces: CLI, API, Web (SvelteKit), MCP
- 4 stub engines (code, harvest, content, gtm)

**Proves:** Ports + adapters + session hierarchy + job queue + events + streaming LLM.

---

## Phase 1 — Agent Runtime + Code Intelligence
> Deep agent capabilities + first analytical engine.

**Deliverable:** Forge orchestrates multi-agent workflows that analyze code.

- rusvel-agent expanded: workflow patterns (Sequential, Parallel, Loop, Graph)
- rusvel-llm expanded: Claude + OpenAI adapters
- rusvel-memory expanded: vector embeddings for semantic search
- forge-engine: multi-agent orchestration, persona system, safety controls
- code-engine v0: Rust parsing (tree-sitter), symbol graph, BM25 search, complexity metrics
- Git worktree isolation for agent safety
- Observability: tracing + run IDs + replay view

**Proves:** `rusvel forge run "analyze this Rust repo"` spawns code agents, streams results.

---

## Phase 2 — Revenue Engines
> Find work and build audience.

**Deliverable:** Discover gigs, create content, publish — with human approval gates.

- harvest-engine: CDP scraping (Upwork adapter), AI scoring, proposal generation
- content-engine: Markdown authoring, AI adaptation, DEV.to + Twitter adapters, scheduling
- rusvel-tui: Ratatui terminal dashboard (pipeline + content calendar)
- Human approval workflow: agent proposes → human approves → system executes
- Inbox/Capture: funnel links, emails, docs into sessions

**Proves:** Find gig → score → draft proposal → approve → send. Write post → adapt → approve → publish.

---

## Phase 3 — GoToMarket Engine
> Complete the business stack.

**Deliverable:** Full CRM + outreach + invoicing.

- gtm-engine: contacts, deals, pipeline stages, invoicing, SOPs
- Outreach sequences with follow-up scheduling
- Email + LinkedIn adapters
- AI spend tracking and budget management
- Knowledge base

**Proves:** End-to-end: discover → qualify → propose → win → invoice → get paid.

---

## Phase 4 — Cross-Engine Intelligence
> Make it compound.

- Cross-engine workflows: Forge orchestrates Harvest + Content + GoToMarket in sequences
- Learning from outcomes (won/lost proposals → improve scoring)
- Context packs: standardized agent context per session
- Advanced agent workflows (Graph agents, best-of-N selection)
- Documentation as first-class artifact (agents maintain mission.md, architecture.md per session)
- Daily autonomous briefings from ALL engine states

---

## Phase 5 — Ecosystem
> Open it up.

- Plugin system (community engines and adapters)
- A2A protocol (agent-to-agent communication)
- Browser extension for passive harvesting
- More platform adapters (LinkedIn, Medium, YouTube, Substack, Freelancer)
- More LLM adapters (Gemini, DeepSeek, Groq)
- Community persona marketplace

---

## Crate Count by Phase

| Phase | New Crates | Total |
|-------|-----------|-------|
| 0 | 20 (10 foundation + 5 engines + 5 surfaces) | 20 |
| 1 | 0 (expand existing) | 20 |
| 2 | 0 (expand existing) | 20 |
| 3 | 0 (expand existing) | 20 |
| 4 | 0 (expand existing) | 20 |
| 5 | +N (plugins, new adapters) | 20+N |

**Key insight from Perplexity:** All the scope goes into expanding existing crates, not adding new ones. The 20-crate structure holds from Phase 0 through Phase 4.

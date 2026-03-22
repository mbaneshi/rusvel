# RUSVEL — Roadmap

---

## Phase 0 — Foundation (current)
> Prove the architecture with one vertical slice.

**Deliverable:** `rusvel mission today` works from CLI, API, Web, and MCP.

- rusvel-core (13 port traits + shared types)
- rusvel-config, rusvel-db, rusvel-event, rusvel-llm (Ollama), rusvel-memory, rusvel-tool, rusvel-auth
- mission-engine
- rusvel-cli, rusvel-api, rusvel-mcp, rusvel-app
- SvelteKit dashboard (mission page only)

**Key proof:** An engine depends ONLY on port traits. Adapters are injected at the app level.

---

## Phase 1 — Agent & Code Intelligence
> The two most technically deep engines.

**Deliverable:** Forge can orchestrate agents that analyze code.

- forge-engine (agent orchestration, personas, workflows, safety controls)
- code-engine (tree-sitter parsing, graphs, metrics, search)
- rusvel-llm expanded (Claude, OpenAI, Gemini adapters)
- rusvel-agent (agent runtime, workflow patterns: Sequential/Parallel/Loop)
- SvelteKit pages: forge (agent dashboard), code (codebase explorer)
- Git worktree isolation

**Key proof:** `rusvel forge run "analyze this repo and find anti-patterns"` spawns code-engine agents.

---

## Phase 2 — Harvest & Content
> Revenue-generating engines.

**Deliverable:** Find gigs and publish content.

- harvest-engine (CDP scraping, opportunity scoring, proposal generation)
- content-engine (Markdown→platform adaptation, scheduling, publishing, analytics)
- rusvel-tui (Ratatui terminal dashboard)
- PublishPort adapters (DEV.to, Twitter/X, LinkedIn)
- HarvestPort adapters (Upwork, GitHub trending)
- SvelteKit pages: harvest (pipeline), content (editor + calendar)

**Key proof:** `rusvel harvest scan` finds gigs → `rusvel content draft` creates pitch → `rusvel content publish` posts it.

---

## Phase 3 — Business & Outreach
> Complete the solo founder stack.

**Deliverable:** Full business operations.

- ops-engine (CRM, pipeline, invoicing, SOPs, knowledge base)
- connect-engine (outreach sequences, follow-ups, relationship tracking)
- Billing/credit tracking (AI spend management)
- SvelteKit pages: ops (CRM + pipeline), connect (outreach dashboard)

**Key proof:** End-to-end: discover gig → qualify → propose → win → invoice → get paid.

---

## Phase 4 — Intelligence & Autonomy
> Make it smarter.

- Cross-engine workflows: Forge orchestrates Harvest + Content + Connect in sequences
- Learning from outcomes (won/lost proposals → improve scoring)
- Semantic memory with vector embeddings
- Daily autonomous briefings (mission-engine pulls from all engines)
- Advanced agent workflows (Graph agents, best-of-N selection)

---

## Phase 5 — Ecosystem
> Extend the reach.

- Plugin system (community engines)
- A2A protocol (agent-to-agent communication)
- Mobile surface (API-driven, separate repo)
- Browser extension for passive harvesting
- Marketplace for agent personas and workflows

---

## Non-Goals (Won't Build)

- Team collaboration features
- Multi-tenant SaaS
- Enterprise SSO/RBAC
- Mobile native app (API-driven web app instead)
- Custom programming language or DSL

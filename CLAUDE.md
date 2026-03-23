# RUSVEL — Project Conventions

> The Solo Builder's AI-Powered Virtual Agency
> Rust + SvelteKit | One binary, one human, infinite leverage

## Quick Commands

```bash
cargo build                    # Build all crates (27 crates)
cargo test                     # Run all tests (192 tests)
cargo run                      # Start API server on :3000 (requires Ollama)
cargo run -- --help            # Show CLI help
cargo run -- --mcp             # Start MCP server (stdio JSON-RPC)
cargo run -- --tui             # Launch TUI dashboard (ratatui)
cargo run -- shell             # Launch interactive REPL shell (reedline)
cargo run -- session create X  # Create a session
cargo run -- forge mission today  # Generate daily plan
cargo run -- finance status    # Department one-shot command
cargo run -- growth list       # List department items
```

## Architecture

Hexagonal (ports & adapters). See `docs/design/architecture-v2.md`.

- **rusvel-core** — 15 port traits + 74 domain types. Zero framework deps.
- **Adapters** — Implement port traits (rusvel-db, rusvel-llm, rusvel-agent, etc.)
- **Engines** — Domain logic, depend ONLY on rusvel-core traits (12 engines, 5 fully wired + 7 domain stubs)
- **Surfaces** — CLI (3-tier: one-shot + REPL + TUI), API, MCP — wire adapters into engines
- **rusvel-app** — Composition root, the single binary

## Key Rules

1. **Engines never import adapter crates.** They depend only on `rusvel-core` port traits.
2. **Engines never call LlmPort directly.** They use AgentPort (ADR-009).
3. **All domain types have `metadata: serde_json::Value`** for schema evolution (ADR-007).
4. **Event.kind is a String**, not an enum (ADR-005). Engines define their own constants.
5. **Single job queue** for all async work — no per-engine scheduling (ADR-003).
6. **Human approval gates** on content publishing and outreach (ADR-008).
7. **Each crate < 2000 lines.** Single responsibility.

## Workspace Layout

```
crates/
├── rusvel-core/        Ports (15 traits) + 74 domain types + DepartmentRegistry (12 depts)
├── rusvel-db/          SQLite WAL + migrations (44 tests, largest suite)
├── rusvel-llm/         4 providers: Ollama, OpenAI, Claude API, Claude CLI + multi-router
├── rusvel-agent/       Agent runtime (wraps LLM+Tool+Memory) + persona + workflow
├── rusvel-event/       Event bus + persistence
├── rusvel-memory/      FTS5 session-namespaced search
├── rusvel-tool/        Tool registry + JSON Schema
├── rusvel-jobs/        Central job queue + approval
├── rusvel-auth/        In-memory credential storage (from env)
├── rusvel-config/      TOML config + per-session overrides
├── forge-engine/       Agent orchestration + Mission (goals, plans, reviews, 10 personas)
├── code-engine/        Code intelligence: parser, dependency graph, BM25 search, metrics
├── harvest-engine/     Opportunity discovery: source scanning, scorer, proposal gen, pipeline
├── content-engine/     Content creation: writer, calendar, platform adapters, analytics
├── gtm-engine/         GoToMarket: CRM, outreach sequences, invoicing, deal stages
├── finance-engine/     Ledger, runway calculator, tax estimation
├── product-engine/     Roadmap, pricing analysis, feedback aggregation
├── growth-engine/      Funnel analysis, cohort tracking, KPI dashboard
├── distro-engine/      SEO, marketplace listings, affiliate channels
├── legal-engine/       Contract drafting, compliance checks, IP management
├── support-engine/     Ticket management, knowledge base, NPS tracking
├── infra-engine/       Deployment, monitoring, incident response
├── rusvel-cli/         3-tier CLI: one-shot commands + REPL shell (reedline) + 11 dept subcommands
├── rusvel-api/         Axum HTTP: 44 routes, 16 modules (dept, chat, CRUD, analytics, capability)
├── rusvel-mcp/         MCP server (stdio JSON-RPC) — wired via --mcp flag
├── rusvel-tui/         TUI dashboard (ratatui) — wired via --tui flag
└── rusvel-app/         Binary entry point + composition root + rust-embed frontend
frontend/               SvelteKit 5 + Tailwind 4 (dept/[id], chat, settings, onboarding, workflow builder)
```

## Wired Features

- **MCP dispatch** — `--mcp` flag connects to `RusvelMcp::new()` in main.rs
- **Department Registry** — 12 departments, parameterized API routes, dynamic frontend route
- **Chat** — SSE streaming per department + God Agent chat
- **CRUD** — Agents, Skills, Rules, MCP Servers, Hooks, Workflows (all departments)
- **Skills execution** — `resolve_skill()` with `{{input}}` interpolation
- **Rules injection** — `load_rules_for_engine()` appended to system prompt
- **Hook dispatch** — `tokio::spawn` on chat completion events
- **Capability Engine** — AI-powered online discovery + JSON bundle install
- **`!build` command** — Generate agents/skills/rules/mcp/hooks from natural language
- **Frontend embedding** — rust-embed compiles `frontend/build/` into binary, ServeDir SPA fallback
- **Onboarding** — CommandPalette, OnboardingChecklist, ProductTour, DeptHelpTooltip components
- **Workflow Builder** — AgentNode + WorkflowBuilder visual components in frontend

## Three-Tier CLI Interface

```
rusvel <dept> <action>     # Tier 1: One-shot commands (11 departments)
rusvel shell               # Tier 2: Interactive REPL (reedline, autocomplete, history)
rusvel --tui               # Tier 3: TUI dashboard (ratatui, 4-panel layout)
```

**Tier 1 departments:** finance, growth, distro, legal, support, infra, product, code, harvest, content, gtm
**Tier 1 actions:** `status`, `list [--kind X]`, `events`
**Tier 2 REPL:** `use <dept>` to switch context, Tab completion, Ctrl+R history search
**Tier 3 TUI:** Tasks, Goals, Pipeline, Events panels — press `q` to exit

## Not Yet Wired

- **Approval workflow** — ApprovalStatus enum exists, no API/UI yet
- **7 domain engines** — Have real code but aren't instantiated in main.rs (chat still works via generic agent)
- **Job queue worker** — JobPort exists but no worker loop processes jobs

## Design Docs

- `docs/design/vision.md` — What RUSVEL is
- `docs/design/architecture-v2.md` — Current architecture (v2, post-review)
- `docs/design/decisions.md` — 10 ADRs with rationale
- `docs/design/audit-2026-03-23.md` — Comprehensive codebase audit
- `docs/plans/phase-0-foundation-v2.md` — Current phase milestones
- `docs/plans/roadmap-v2.md` — 5-phase roadmap

## Testing

```bash
cargo test                     # All 192 tests
cargo test -p rusvel-core      # Single crate
cargo test -p forge-engine     # Engine tests (15 tests, use mock ports)
cargo test -p content-engine   # Content engine (7 tests)
cargo test -p harvest-engine   # Harvest engine (12 tests)
cargo test -p rusvel-db        # DB store (largest suite)
cargo test -p rusvel-api       # API tests (19 tests)
```

## API Modules (rusvel-api, 44 routes)

agents, analytics, approvals, build_cmd, capability, chat, config, department,
help, hook_dispatch, hooks, mcp_servers, routes, rules, skills, workflows

## Stack

- Rust edition 2024, SQLite WAL, Axum, Clap 4, reedline, ratatui, tokio (~22k lines Rust)
- SvelteKit 5, Tailwind CSS 4
- LLM: Ollama (local), Claude API, Claude CLI, OpenAI — all implemented
- Frontend embedded in binary via rust-embed

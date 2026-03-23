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
cargo run -- session create X  # Create a session
cargo run -- forge mission today  # Generate daily plan
```

## Architecture

Hexagonal (ports & adapters). See `docs/design/architecture-v2.md`.

- **rusvel-core** — 13 port traits + ~67 domain types. Zero framework deps.
- **Adapters** — Implement port traits (rusvel-db, rusvel-llm, rusvel-agent, etc.)
- **Engines** — Domain logic, depend ONLY on rusvel-core traits (12 engines, 5 fully wired + 7 domain stubs)
- **Surfaces** — CLI, API, MCP — wire adapters into engines
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
├── rusvel-core/        Ports (13 traits) + shared domain types (~67 structs/enums)
├── rusvel-db/          SQLite WAL + 5 sub-stores + migrations
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
├── rusvel-cli/         Clap CLI (session + forge mission subcommands)
├── rusvel-api/         Axum HTTP: departments, chat, CRUD, analytics, capability engine
├── rusvel-mcp/         MCP server (stdio JSON-RPC) — wired via --mcp flag
├── rusvel-tui/         TUI surface: layout + widgets — not yet wired into main
└── rusvel-app/         Binary entry point + composition root
frontend/               SvelteKit 5 + Tailwind 4 (dynamic dept/[id] route, dashboard, chat)
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

## Not Yet Wired

- **TUI** — `rusvel-tui` has layout + widgets but isn't launched from main
- **Frontend embedding** — served from filesystem, not yet via rust-embed
- **Job queue worker** — JobPort exists but no worker loop processes jobs
- **Approval workflow** — ApprovalStatus enum exists, no API/UI yet
- **7 domain engines** — Have real code but aren't instantiated in main.rs (chat still works via generic agent)

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

## Stack

- Rust edition 2024, SQLite WAL, Axum, Clap 4, tokio
- SvelteKit 5, Tailwind CSS 4
- LLM: Ollama (local), Claude API, Claude CLI, OpenAI — all implemented
- Frontend embedded in binary via rust-embed (planned)

# RUSVEL — Project Conventions

> The Solo Builder's AI-Powered Virtual Agency
> Rust + SvelteKit | One binary, one human, infinite leverage

## Quick Commands

```bash
cargo build                    # Build all crates (30 crates)
cargo test                     # Run all tests (197 tests)
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
8. **NEVER use npm.** Use `pnpm` for all frontend/Node.js work. (`pnpm install`, `pnpm build`, `pnpm exec`).
9. **NEVER use pip/pip3/python -m pip.** Use `uv` for all Python work. (`uv run`, `uv add`, `uv sync`).

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
├── flow-engine/        DAG workflow engine: petgraph executor, code/condition/agent nodes
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
- **Domain engines wired** — CodeEngine, ContentEngine, HarvestEngine instantiated in main.rs
- **Engine API routes** — 9 engine-specific endpoints (`/api/dept/code/analyze`, `/api/dept/content/draft`, etc.)
- **Engine CLI commands** — `rusvel code analyze`, `rusvel code search`, `rusvel content draft`, `rusvel harvest pipeline`
- **Job queue worker** — Background worker processes CodeAnalyze, ContentPublish, HarvestScan jobs via real engines
- **Flow Engine** — DAG workflow engine (petgraph), 3 node types (code, condition, agent), 7 API routes at `/api/flows`

## Three-Tier CLI Interface

```
rusvel <dept> <action>     # Tier 1: One-shot commands (11 departments)
rusvel shell               # Tier 2: Interactive REPL (reedline, autocomplete, history)
rusvel --tui               # Tier 3: TUI dashboard (ratatui, 4-panel layout)
```

**Tier 1 departments:** finance, growth, distro, legal, support, infra, product, code, harvest, content, gtm
**Tier 1 actions:** `status`, `list [--kind X]`, `events`
**Engine-specific actions:** `code analyze [path]`, `code search <query>`, `content draft <topic>`, `harvest pipeline`
**Tier 2 REPL:** `use <dept>` to switch context, Tab completion, Ctrl+R history search
**Tier 3 TUI:** Tasks, Goals, Pipeline, Events panels — press `q` to exit

## Not Yet Wired

- **Approval workflow** — ApprovalStatus enum exists, no API/UI yet
- **8 domain engines** — GTM, Finance, Product, Growth, Distro, Legal, Support, Infra are stubs (chat works via generic agent)
- **OutreachSend jobs** — GTM engine not yet wired, job handler is placeholder

## Design Docs

- `docs/design/vision.md` — What RUSVEL is
- `docs/design/architecture-v2.md` — Current architecture (v2, post-review)
- `docs/design/decisions.md` — 10 ADRs with rationale
- `docs/design/audit-2026-03-23.md` — Comprehensive codebase audit
- `docs/plans/phase-0-foundation-v2.md` — Current phase milestones
- `docs/plans/roadmap-v2.md` — 5-phase roadmap

## Frontend (pnpm)

**Package manager: pnpm** (not npm). All frontend commands use `pnpm`.

```bash
cd frontend
pnpm install                   # Install dependencies
pnpm dev                       # Start dev server on :5173
pnpm build                     # Build for production (output: build/)
pnpm check                     # TypeScript + Svelte type checking
pnpm test:e2e                  # Run all E2E tests (Playwright)
pnpm test:visual               # Run visual regression tests only
pnpm test:e2e:update           # Update visual baselines
pnpm test:analyze              # AI-powered visual diff analysis (Claude Vision)
```

## Testing

```bash
cargo test                     # All 197 tests (Rust)
cargo test -p rusvel-core      # Single crate
cargo test -p forge-engine     # Engine tests (15 tests, use mock ports)
cargo test -p content-engine   # Content engine (7 tests)
cargo test -p harvest-engine   # Harvest engine (12 tests)
cargo test -p rusvel-db        # DB store (largest suite)
cargo test -p rusvel-api       # API tests (19 tests)
pnpm test:visual               # Visual regression tests (Playwright, 16 routes)
```

## Visual E2E Testing

Self-correction loop: screenshot -> compare -> Claude Vision analysis -> auto-generate fix skills/rules.

```bash
pnpm test:visual                                    # Run visual tests
pnpm test:analyze                                   # Analyze diffs with Claude Vision
curl -X POST http://localhost:3000/api/system/visual-test   # Run via API
curl -X POST http://localhost:3000/api/system/visual-report/self-correct  # Auto-fix
```

MCP tool: `visual_inspect` — run visual tests from Claude sessions.

## API Modules (rusvel-api, 56 routes)

agents, analytics, approvals, build_cmd, capability, chat, config, department,
engine_routes, help, hook_dispatch, hooks, mcp_servers, routes, rules, skills,
system, visual_report, workflows

## Python Scripts (uv)

**Package manager: uv** (not pip). Python is used for auxiliary scripts only — the app itself is Rust.

```bash
uv run <script.py>             # Run a script with managed deps
uv sync                        # Install all deps to .venv
uv add <package>               # Add a dependency
uv run --with anthropic ...    # One-off with extra deps
```

## Stack

- Rust edition 2024, SQLite WAL, Axum, Clap 4, reedline, ratatui, tokio (~22k lines Rust)
- SvelteKit 5, Tailwind CSS 4, **pnpm** package manager
- Python scripts: **uv** (pyproject.toml at workspace root)
- LLM: Ollama (local), Claude API, Claude CLI, OpenAI — all implemented
- Frontend embedded in binary via rust-embed
- E2E: Playwright visual regression + Claude Vision analysis

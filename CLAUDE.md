# RUSVEL — Project Conventions

> The Solo Builder's AI-Powered Virtual Agency
> Rust + SvelteKit | One binary, one human, infinite leverage

## Quick Commands

```bash
cargo build                    # Build all crates (48 crates)
cargo test                     # Run all tests (222 tests in 30 binaries)
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

- **rusvel-core** — 19 port traits (14 Port + 5 Store) + domain types + DepartmentApp trait + DepartmentManifest. Zero framework deps.
- **Adapters** — Implement port traits (rusvel-db, rusvel-llm, rusvel-agent, etc.)
- **Engines** — Domain logic, depend ONLY on rusvel-core traits (13 engines, all migrated to DepartmentApp pattern via ADR-014)
- **dept-* wrappers** — 13 `dept-*` crates (including dept-flow), each implementing DepartmentApp
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

## Workspace Layout (48 crates)

```
crates/
├── rusvel-core/          Ports (19 traits: 14 Port + 5 Store) + domain types + DepartmentApp trait + DepartmentManifest
├── rusvel-schema/        Database schema introspection (RusvelBase)
├── rusvel-db/            SQLite WAL + migrations
├── rusvel-llm/           4 providers: Ollama, OpenAI, Claude API, Claude CLI + multi-router + ModelTier routing
├── rusvel-agent/         Agent runtime (tool-use loop, streaming, AgentEvent) + persona + workflow
├── rusvel-event/         Event bus + persistence
├── rusvel-memory/        FTS5 session-namespaced search
├── rusvel-tool/          Tool registry + ScopedToolRegistry + JSON Schema validation
├── rusvel-builtin-tools/ 10 built-in tools (9 + tool_search meta-tool)
├── rusvel-engine-tools/  12 engine tools (harvest 5, content 5, code 2)
├── rusvel-mcp-client/    MCP client for external MCP servers
├── rusvel-jobs/          Central job queue + approval
├── rusvel-auth/          In-memory credential storage (from env)
├── rusvel-config/        TOML config + per-session overrides
├── rusvel-deploy/        Deployment port adapter
├── rusvel-embed/         Text embedding adapter
├── rusvel-vector/        Vector store (LanceDB) for semantic search
├── rusvel-terminal/      Terminal multiplexer (TerminalPort, PTY management)
├── forge-engine/         Agent orchestration + Mission (goals, plans, reviews, 10 personas)
├── code-engine/          Code intelligence: parser, dependency graph, BM25 search, metrics
├── harvest-engine/       Opportunity discovery: source scanning, scorer, proposal gen, pipeline
├── content-engine/       Content creation: writer, calendar, platform adapters (LinkedIn, Twitter, DEV.to), analytics
├── gtm-engine/           GoToMarket: CRM, outreach, invoicing
├── finance-engine/       Ledger, runway, tax estimation
├── product-engine/       Roadmap, pricing, feedback
├── growth-engine/        Funnel analysis, cohort tracking
├── distro-engine/        SEO, marketplace, affiliates
├── legal-engine/         Contracts, compliance, IP
├── support-engine/       Tickets, knowledge base, NPS
├── infra-engine/         Deployment, monitoring, incidents
├── flow-engine/          DAG workflow engine: petgraph, code/condition/agent nodes
├── dept-forge/           Forge department (DepartmentApp)
├── dept-code/            Code department (DepartmentApp)
├── dept-content/         Content department (DepartmentApp)
├── dept-harvest/         Harvest department (DepartmentApp)
├── dept-flow/            Flow department (DepartmentApp)
├── dept-gtm/             GTM department (DepartmentApp)
├── dept-finance/         Finance department (DepartmentApp)
├── dept-product/         Product department (DepartmentApp)
├── dept-growth/          Growth department (DepartmentApp)
├── dept-distro/          Distro department (DepartmentApp)
├── dept-legal/           Legal department (DepartmentApp)
├── dept-support/         Support department (DepartmentApp)
├── dept-infra/           Infra department (DepartmentApp)
├── rusvel-cli/           3-tier CLI: one-shot + REPL + TUI
├── rusvel-api/           Axum HTTP: 124 handlers across 23 modules
├── rusvel-mcp/           MCP server (stdio JSON-RPC) — wired via --mcp flag
├── rusvel-tui/           TUI dashboard (ratatui) — wired via --tui flag
└── rusvel-app/           Binary entry point + composition root + rust-embed frontend
frontend/                 SvelteKit 5 + Tailwind 4 (dept/[id], chat, database, flows, knowledge, settings)
```

## Wired Features

- **DepartmentApp pattern (ADR-014)** — All 12 departments migrated, each with a `dept-*` wrapper crate
- **EngineKind enum removed** — String IDs everywhere for department identification
- **AgentRuntime** — `run_streaming()` + `AgentEvent` replaced ClaudeCliStreamer
- **LlmStreamEvent** — `stream()` on LlmPort for character-by-character streaming
- **ModelTier routing** — Haiku/Sonnet/Opus + CostTracker in MetricStore
- **ScopedToolRegistry** — Per-department tool filtering
- **Deferred tool loading** — `tool_search` meta-tool (85% token savings)
- **22+ registered tools** — 10 built-in (read_file, write_file, edit_file, glob, grep, bash, git_status, git_diff, git_log + tool_search) + 12 engine tools (harvest 5, content 5, code 2)
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
- **Frontend** — ToolCallCard, ApprovalCard, ApprovalQueue with sidebar badge, manifest-aligned dept nav, /flows page
- **Domain engines wired** — All 12 departments via DepartmentApp pattern
- **Engine API routes** — 15 engine-specific endpoints (`/api/dept/code/analyze`, `/api/dept/content/draft`, `/api/dept/content/from-code`, etc.)
- **Engine CLI commands** — `rusvel code analyze`, `rusvel code search`, `rusvel content draft`, `rusvel content from-code`, `rusvel harvest pipeline`
- **Job queue worker** — Background worker processes CodeAnalyze, ContentPublish, HarvestScan jobs via real engines (session_id scoped)
- **Flow Engine** — DAG workflow engine (petgraph), 3 node types (code, condition, agent), 7 API routes at `/api/flows`
- **RusvelBase** — Database browser UI with schema introspection, table viewer, SQL runner at `/api/db/*`
- **Knowledge/RAG** — Vector-backed knowledge base with 5 API routes at `/api/knowledge`
- **Code-to-Content** — Pipeline from code analysis to content drafts via `/api/dept/content/from-code`
- **Real platform adapters** — LinkedIn, Twitter, DEV.to for content publishing
- **MCP Client** — Connect to external MCP servers for tool discovery
- **3 LLM streaming providers** — ClaudeCliProvider (real streaming), others (batch fallback)
- **TerminalPort** — `rusvel-terminal` crate for PTY management

## Three-Tier CLI Interface

```
rusvel <dept> <action>     # Tier 1: One-shot commands (12 departments)
rusvel shell               # Tier 2: Interactive REPL (reedline, autocomplete, history)
rusvel --tui               # Tier 3: TUI dashboard (ratatui, 4-panel layout)
```

**Tier 1 departments:** forge, finance, growth, distro, legal, support, infra, product, code, harvest, content, gtm
**Tier 1 actions:** `status`, `list [--kind X]`, `events`
**Engine-specific actions:** `code analyze [path]`, `code search <query>`, `content draft <topic>`, `harvest pipeline`
**Tier 2 REPL:** `use <dept>` to switch context, Tab completion, Ctrl+R history search
**Tier 3 TUI:** Tasks, Goals, Pipeline, Events panels — press `q` to exit

## Not Yet Wired

- **OutreachSend jobs** — GTM engine not yet wired, job handler is placeholder
- **Authentication/authorization** — rusvel-auth is in-memory from env vars; no middleware on API routes

## Design Docs

- `docs/design/vision.md` — What RUSVEL is
- `docs/design/architecture-v2.md` — Current architecture (v2, post-review)
- `docs/design/decisions.md` — 14 ADRs with rationale
- `docs/design/audit-2026-03-23.md` — Comprehensive codebase audit
- `docs/status/current-state.md` — Live codebase metrics snapshot
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
cargo test                     # All 222 tests in 30 binaries
cargo test -p rusvel-core      # Single crate
cargo test -p forge-engine     # Engine tests (15 tests, use mock ports)
cargo test -p content-engine   # Content engine (7 tests)
cargo test -p harvest-engine   # Harvest engine (12 tests)
cargo test -p rusvel-db        # DB store
cargo test -p rusvel-api       # API tests
cargo test -p rusvel-llm       # LLM provider tests
pnpm test:visual               # Visual regression tests (Playwright)

# Line coverage (llvm-cov): rustup component add llvm-tools-preview && cargo install cargo-llvm-cov
./scripts/coverage.sh          # HTML report + summary; see docs/testing/coverage-strategy.md
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

## API Modules (rusvel-api, 124 handlers across 23 modules)

agents, analytics, approvals, build_cmd, capability, chat, config, db_routes,
department, engine_routes, flow_routes, help, hook_dispatch, hooks, knowledge,
mcp_servers, routes, rules, skills, system, terminal, visual_report, workflows

## Python Scripts (uv)

**Package manager: uv** (not pip). Python is used for auxiliary scripts only — the app itself is Rust.

```bash
uv run <script.py>             # Run a script with managed deps
uv sync                        # Install all deps to .venv
uv add <package>               # Add a dependency
uv run --with anthropic ...    # One-off with extra deps
```

## Stack

- Rust edition 2024, SQLite WAL, Axum, Clap 4, reedline, ratatui, tokio (~43,670 lines Rust, 185 source files)
- SvelteKit 5, Tailwind CSS 4, **pnpm** package manager
- Python scripts: **uv** (pyproject.toml at workspace root)
- LLM: Ollama (local), Claude API, Claude CLI, OpenAI — all implemented
- Vector DB: LanceDB + Arrow for semantic search
- Frontend embedded in binary via rust-embed
- E2E: Playwright visual regression + Claude Vision analysis

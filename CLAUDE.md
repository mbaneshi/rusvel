# RUSVEL — Project Conventions

> The Solo Builder's AI-Powered Virtual Agency
> Rust + SvelteKit | One binary, one human, infinite leverage

## Quick Commands

```bash
cargo build                    # Build all crates
cargo test                     # Run all tests (149 tests)
cargo run                      # Start API server on :3000 (requires Ollama)
cargo run -- --help            # Show CLI help
cargo run -- session create X  # Create a session
cargo run -- forge mission today  # Generate daily plan
```

## Architecture

Hexagonal (ports & adapters). See `docs/design/architecture-v2.md`.

- **rusvel-core** — 10 port traits + ~40 domain types. Zero framework deps.
- **Adapters** — Implement port traits (rusvel-db, rusvel-llm, rusvel-agent, etc.)
- **Engines** — Domain logic, depend ONLY on rusvel-core traits (forge, code, harvest, content, gtm)
- **Surfaces** — CLI, API, MCP, TUI — wire adapters into engines
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
├── rusvel-core/     Ports (10 traits) + shared domain types (~40 structs/enums)
├── rusvel-db/       SQLite WAL + 5 sub-stores + migrations (largest crate, ~1500 lines)
├── rusvel-llm/      4 providers: Ollama, OpenAI, Claude API, Claude CLI + multi-router
├── rusvel-agent/    Agent runtime (wraps LLM+Tool+Memory) + persona + workflow
├── rusvel-event/    Event bus + persistence
├── rusvel-memory/   FTS5 session-namespaced search
├── rusvel-tool/     Tool registry + JSON Schema
├── rusvel-jobs/     Central job queue + approval
├── rusvel-auth/     In-memory credential storage (from env)
├── rusvel-config/   TOML config + per-session overrides
├── forge-engine/    Agent orchestration + Mission (goals, daily plan, reviews, 10 personas, safety guard)
├── code-engine/     Code intelligence: parser, dependency graph, BM25 search, metrics
├── harvest-engine/  Opportunity discovery: source scanning, scorer, proposal gen, pipeline
├── content-engine/  Content creation: writer, calendar, platform adapters, analytics
├── gtm-engine/      GoToMarket: CRM, outreach sequences, invoicing, deal stages
├── rusvel-cli/      Clap CLI (session + forge mission subcommands)
├── rusvel-api/      Axum HTTP: health, sessions, goals, mission, events (7 endpoints)
├── rusvel-mcp/      MCP server (stdio JSON-RPC) — imported but not yet dispatched
├── rusvel-tui/      TUI surface: layout + widgets — not yet wired into main
└── rusvel-app/      Binary entry point + composition root
frontend/            SvelteKit 5 + Tailwind 4 (layout + /forge route)
```

## Not Yet Wired

- **MCP dispatch** — `rusvel-mcp` is imported in `rusvel-app` but the `--mcp` flag isn't connected
- **TUI** — `rusvel-tui` has layout + widgets but isn't launched from main
- **Frontend** — minimal SvelteKit shell (layout + 2 routes), not yet embedded via rust-embed

## Design Docs

- `docs/design/vision.md` — What RUSVEL is
- `docs/design/architecture-v2.md` — Current architecture (v2, post-review)
- `docs/design/decisions.md` — 10 ADRs with rationale
- `docs/plans/phase-0-foundation-v2.md` — Current phase milestones
- `docs/plans/roadmap-v2.md` — 5-phase roadmap
- `docs/research/repo-inventory.md` — All source repos cataloged

## Testing

```bash
cargo test                     # All 149 tests
cargo test -p rusvel-core      # Single crate
cargo test -p forge-engine     # Engine tests (15 tests, use mock ports)
cargo test -p content-engine   # Content engine (7 tests)
cargo test -p harvest-engine   # Harvest engine (12 tests)
cargo test -p rusvel-db        # DB store (41 tests, largest suite)
```

## Stack

- Rust edition 2024, SQLite WAL, Axum, Clap 4, tokio
- SvelteKit 5, Tailwind CSS 4
- LLM: Ollama (local), Claude API, Claude CLI, OpenAI — all implemented
- Frontend embedded in binary via rust-embed (planned)

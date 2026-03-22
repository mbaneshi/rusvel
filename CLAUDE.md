# RUSVEL — Project Conventions

> The Solo Builder's AI-Powered Virtual Agency
> Rust + SvelteKit | One binary, one human, infinite leverage

## Quick Commands

```bash
cargo build                    # Build all crates
cargo test                     # Run all tests
cargo run                      # Start API server on :3000
cargo run -- --help            # Show CLI help
cargo run -- session create X  # Create a session
cargo run -- forge mission today  # Generate daily plan
```

## Architecture

Hexagonal (ports & adapters). See `docs/design/architecture-v2.md`.

- **rusvel-core** — 10 port traits + shared domain types. Zero framework deps.
- **Adapters** — Implement port traits (rusvel-db, rusvel-llm, rusvel-agent, etc.)
- **Engines** — Domain logic, depend ONLY on rusvel-core traits (forge, code, harvest, content, gtm)
- **Surfaces** — CLI, API, MCP, (TUI planned) — wire adapters into engines
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
├── rusvel-core/     Ports + shared types (the contract)
├── rusvel-db/       SQLite WAL + 5 stores + migrations
├── rusvel-llm/      Ollama adapter (Claude/OpenAI later)
├── rusvel-agent/    Agent runtime (wraps LLM+Tool+Memory)
├── rusvel-event/    Event bus + persistence
├── rusvel-memory/   FTS5 session-namespaced search
├── rusvel-tool/     Tool registry + JSON Schema
├── rusvel-jobs/     Central job queue + approval
├── rusvel-auth/     Credential storage
├── rusvel-config/   TOML config + session overrides
├── forge-engine/    Agent orchestration + Mission
├── code-engine/     Code intelligence (stub)
├── harvest-engine/  Opportunity discovery (stub)
├── content-engine/  Content creation (stub)
├── gtm-engine/      GoToMarket: CRM + outreach (stub)
├── rusvel-cli/      Clap CLI
├── rusvel-api/      Axum HTTP + WebSocket
├── rusvel-mcp/      MCP server (stdio JSON-RPC)
└── rusvel-app/      Binary entry point
frontend/            SvelteKit 5 + Tailwind 4
```

## Design Docs

- `docs/design/vision.md` — What RUSVEL is
- `docs/design/architecture-v2.md` — Current architecture (v2, post-review)
- `docs/design/decisions.md` — 10 ADRs with rationale
- `docs/plans/phase-0-foundation-v2.md` — Current phase milestones
- `docs/plans/roadmap-v2.md` — 5-phase roadmap
- `docs/research/repo-inventory.md` — All source repos cataloged

## Testing

```bash
cargo test                     # All tests
cargo test -p rusvel-core      # Single crate
cargo test -p forge-engine     # Engine tests (use mock ports)
```

## Stack

- Rust edition 2024, SQLite WAL, Axum, Clap 4, tokio
- SvelteKit 5, Tailwind CSS 4
- LLM: Ollama (local), Claude/OpenAI/Gemini (planned)
- Frontend embedded in binary via rust-embed

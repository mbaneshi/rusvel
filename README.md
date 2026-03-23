# RUSVEL

> The Solo Builder's AI-Powered Virtual Agency
> Rust + SvelteKit | One binary, one human, infinite leverage

## What is RUSVEL?

A single Rust binary that replaces an entire agency. AI agents handle code, content, sales, finance, legal, support, and more — all orchestrated from one unified interface. Not a SaaS platform. Not for teams. A personal superpower.

## Quick Start

```bash
cargo build                    # Build all 27 crates
cargo run                      # Start web server on http://localhost:3000
```

## Three Ways to Use the Terminal

### One-shot commands
```bash
rusvel session create my-startup    # Create a session
rusvel forge mission today          # AI-generated daily plan
rusvel finance status               # Department status
rusvel growth list                  # List department items
rusvel harvest events               # Recent department events
```

All 11 departments available: `finance`, `growth`, `distro`, `legal`, `support`, `infra`, `product`, `code`, `harvest`, `content`, `gtm`.

### Interactive REPL shell
```bash
rusvel shell
```
Drop into a persistent prompt with autocomplete, history, and department context switching:
```
rusvel> use finance
rusvel:finance> status
rusvel:finance> list transactions
rusvel:finance> back
rusvel> status          # All departments overview
```

### TUI dashboard
```bash
rusvel --tui
```
Full-screen terminal dashboard with Tasks, Goals, Pipeline, and Events panels.

## Other Surfaces

```bash
rusvel                  # Web server (Axum + SvelteKit) on :3000
rusvel --mcp            # MCP server (stdio JSON-RPC) for Claude Code
```

## Architecture

Hexagonal (ports & adapters). 27 crates, 12 department engines, single binary.

```
SURFACES: CLI (Clap) | REPL (reedline) | TUI (Ratatui) | Web (Svelte) | MCP
    |
ENGINES:  Forge | Code | Harvest | Content | GTM | Finance | Product
          Growth | Distro | Legal | Support | Infra
    |
FOUNDATION: rusvel-core (traits) + 9 adapter crates (DB, LLM, Agent, Events, ...)
```

See [docs/design/architecture-v2.md](docs/design/architecture-v2.md) for full details.

## Stack

- **Backend:** Rust 2024, SQLite WAL, Axum, Clap 4, reedline, ratatui, tokio
- **Frontend:** SvelteKit 5, Tailwind CSS 4
- **AI:** Ollama (local), Claude API, Claude CLI, OpenAI

## Testing

```bash
cargo test              # All tests
cargo test -p rusvel-core
cargo test -p forge-engine
cargo test -p rusvel-db
cargo test -p rusvel-api
```

## Documentation

- [Vision](docs/design/vision.md) — What RUSVEL is
- [Architecture](docs/design/architecture-v2.md) — Hexagonal architecture, 12 departments
- [Decisions](docs/design/decisions.md) — 10 ADRs with rationale
- [Roadmap](docs/plans/roadmap-v2.md) — 5-phase plan

## License

MIT

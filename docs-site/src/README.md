# RUSVEL

> **The Solo Builder's AI-Powered Virtual Agency**
> One binary, one human, infinite leverage.

RUSVEL is an AI-powered virtual agency built with **Rust + SvelteKit**. It gives a single person the leverage of an entire agency through **12 autonomous departments**, each powered by AI agents.

## Quick Start

```bash
git clone https://github.com/mbaneshi/rusvel
cd rusvel
cargo run
# Open http://localhost:3000
```

## What's Inside

- **God Agent** — Your AI companion that knows your identity, products, and mission
- **12 Departments** — Forge, Code, Content, Harvest, GTM, Finance, Product, Growth, Distribution, Legal, Support, Infra
- **Knowledge/RAG** — fastembed + lancedb for semantic search over your documents
- **Self-Improvement** — The app can analyze and improve its own codebase
- **54 workspace members** — Hexagonal architecture; **20** port traits in `rusvel-core/src/ports.rs` (including five `*Store` subtraits and `BrowserPort`)
- **~476 tests** (~61 test targets); full `cargo test` passes in a normal dev environment

See **[Repository status](./reference/repository-status.md)** for canonical metrics and links to `docs/status/current-state.md` on GitHub.

## Architecture

```
God Agent (Chat — full authority + visibility)
├── Forge      — Mission planning, goals, reviews
├── Code       — Full Claude Code capabilities
├── Content    — Draft, adapt, publish across platforms
├── Harvest    — Find opportunities, score, propose
├── GTM        — CRM, outreach, invoicing, deals
├── Finance    — Ledger, runway, tax estimation
├── Product    — Roadmap, pricing, feedback
├── Growth     — Funnel, cohorts, KPIs
├── Distribution — SEO, marketplace, affiliates
├── Legal      — Contracts, compliance, IP
├── Support    — Tickets, knowledge base, NPS
└── Infra      — Deploy, monitor, incidents
```

Each department is **autonomous** — own config, own chat, own agents, own events. God sees everything and can orchestrate any combination.

## Stack

- **Rust** edition 2024, SQLite WAL, Axum, Clap 4, tokio
- **SvelteKit 5**, Tailwind CSS 4
- **LLM**: Claude CLI (Max subscription), Ollama, OpenAI, Claude API
- **RAG**: fastembed (local ONNX embeddings) + lancedb (vector search)


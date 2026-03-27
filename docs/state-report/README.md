# RUSVEL State Report — Code Truth Edition

> Generated 2026-03-27 from actual source code, not documentation.
> Every claim verified against disk artifacts under `crates/` and `frontend/src/`.

## Metrics at a Glance

| Dimension | Count |
|-----------|-------|
| Rust lines | 52,560 |
| Frontend lines | 10,623 |
| Combined | ~63,183 |
| Workspace crates | 50 |
| `.rs` source files | 215 |
| `.svelte` components | 66 |
| Test cases | 399 |
| Cargo.lock packages | 795 |
| Port traits | 16 |
| Engines | 13 |
| Departments (ADR-014) | 13 |
| API routes | ~105 |
| MCP tools | 6 |
| Built-in agent tools | 22+ |
| LLM providers | 4 |

## Chapters

1. **[Architecture & Boot Sequence](01-architecture.md)** — Hexagonal design, composition root, wiring order
2. **[Ports & Adapters](02-ports-and-adapters.md)** — All 16 port traits, concrete adapter types, data flow
3. **[Engines & Departments](03-engines-and-departments.md)** — Per-department implementation status, tool wiring gaps
4. **[API Surface](04-api-surface.md)** — Complete route map, handler signatures, AppState
5. **[Frontend](05-frontend.md)** — Route tree, components, stores, communication patterns
6. **[Data Flows](06-data-flows.md)** — Agent loop, tool dispatch, streaming, MCP, RAG
7. **[Metrics & Gaps](07-metrics-and-gaps.md)** — Crate sizes, wiring status matrix, actionable gaps

## How to Read

Start with Chapter 1 for the big picture. Jump to Chapter 3 for the honest department-by-department status. Chapter 7 is the action-oriented summary of what's wired and what's not.

## Methodology

Six parallel exploration agents scanned every `lib.rs`, `main.rs`, route file, component, and store in the repository. No documentation files were consulted — only compiled source code and `Cargo.toml` manifests.

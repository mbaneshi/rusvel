# Manual Testing Playbook

> Complete feature-by-feature testing guide with expected behavior for every RUSVEL surface.

RUSVEL exposes **6 surfaces** — each needs manual verification:

| Surface | Entry point | Requires Ollama |
|---------|------------|-----------------|
| [CLI One-Shot](./cli.md) | `cargo run -- <command>` | Some commands |
| [REPL Shell](./cli.md#interactive-repl-shell) | `cargo run -- shell` | No |
| [TUI Dashboard](./cli.md#tui-dashboard) | `cargo run -- --tui` | No |
| [Web API](./api.md) | `curl localhost:3000/api/*` | Chat endpoints |
| [Web Frontend](./frontend.md) | Browser at `localhost:3000` | Chat pages |
| [MCP Server](./api.md#mcp-server) | `cargo run -- --mcp` | Some tools |

## Testing Strategy

1. **Start with the [Smoke Test Checklist](./smoke-checklist.md)** — 30 steps, covers all surfaces in ~15 minutes
2. **Deep-dive per surface** — Use the dedicated pages for thorough testing
3. **Engine-specific tests** — [Code, Content, Harvest, GTM, Flow](./engines.md) each have unique endpoints
4. **Cross-surface verification** — Perform an action in one surface, verify it in another:
   - Create a session via CLI -> verify via `curl /api/sessions` -> verify in browser UI
   - Chat via web UI -> verify conversation via API -> check events in TUI

## Quick Reference

| What | Where |
|------|-------|
| Build & prerequisites | [Setup](./setup.md) |
| CLI, REPL, TUI | [CLI Testing](./cli.md) |
| REST API (130+ routes) | [API Testing](./api.md) |
| Browser UI (30+ pages) | [Frontend Testing](./frontend.md) |
| Code, Content, Harvest, GTM, Flow | [Engine Testing](./engines.md) |
| 30-step release gate | [Smoke Checklist](./smoke-checklist.md) |

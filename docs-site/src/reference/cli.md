
## Overview

RUSVEL provides a **three-tier CLI** using [Clap 4](https://docs.rs/clap). The binary is `rusvel` (or `cargo run --` during development).

```
rusvel <dept> <action>     # Tier 1: One-shot commands (12 departments)
rusvel shell               # Tier 2: Interactive REPL (reedline, autocomplete, history)
rusvel --tui               # Tier 3: TUI dashboard (ratatui, 4-panel layout)
```

## Global Options

| Flag | Description |
|------|------------|
| `--mcp` | Start the MCP server (stdio JSON-RPC) instead of the web server |
| `--tui` | Launch the TUI dashboard (ratatui, 4-panel layout) |
| `--help` | Show help information |
| `--version` | Show version |

## Starting the Server

### `rusvel` (no subcommand)

Starts the API web server on `0.0.0.0:3000`. Serves the REST API and the embedded SvelteKit frontend.

```bash
cargo run
```

### `rusvel --mcp`

Starts the MCP (Model Context Protocol) server over stdio. Used for integration with Claude Desktop and other MCP clients.

```bash
cargo run -- --mcp
```

### `rusvel --tui`

Launches the TUI dashboard with 4 panels: Tasks, Goals, Pipeline, Events. Press `q` to exit.

```bash
cargo run -- --tui
```

## Tier 1: Department One-Shot Commands

All 12 departments support these actions:

```bash
rusvel <dept> status              # Show department status summary
rusvel <dept> list [--kind X]     # List department items
rusvel <dept> events              # Show recent events for department
```

**Departments:** `forge`, `code`, `content`, `harvest`, `gtm`, `finance`, `product`, `growth`, `distro`, `legal`, `support`, `infra`

### Engine-Specific Commands

Some departments have additional commands powered by their wired engines:

```bash
# Code department
rusvel code analyze [path]        # Analyze code: parser, dependency graph, metrics
rusvel code search <query>        # BM25 search across codebase

# Content department
rusvel content draft <topic>      # Draft content on a topic
rusvel content from-code          # Generate content from code analysis

# Harvest department
rusvel harvest pipeline           # Show opportunity pipeline
```

### Forge Commands

```bash
rusvel forge mission today        # Generate a prioritized daily plan
rusvel forge mission goals        # List all goals for active session
rusvel forge mission goal add <title>  # Add a new goal
rusvel forge mission review       # Generate a periodic review
```

**Options for `goal add`:**

| Flag | Default | Description |
|------|---------|------------|
| `--description` | `""` | Goal description |
| `--timeframe` | `month` | One of: `day`, `week`, `month`, `quarter` |

**Options for `review`:**

| Flag | Default | Description |
|------|---------|------------|
| `--period` | `week` | One of: `day`, `week`, `month`, `quarter` |

## Tier 2: Interactive REPL

```bash
rusvel shell
```

Launches an interactive shell powered by [reedline](https://docs.rs/reedline) with:

- **Tab completion** for commands and department names
- **Ctrl+R** history search
- **`use <dept>`** to switch department context
- All Tier 1 commands available without the `rusvel` prefix

## Session Management

```bash
rusvel session create <name>      # Create a new session
rusvel session list               # List all sessions
rusvel session switch <id>        # Switch active session
```

The CLI stores the active session ID in `~/.rusvel/active_session`. All `forge` commands operate on this session.

If no active session is set, commands that require one will error:

```
Error: No active session. Run `rusvel session create <name>` first.
```

## Active Session

The CLI stores the active session ID in `~/.rusvel/active_session`. All `forge` commands operate on this session. Change it with `session switch`.

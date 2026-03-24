
## Overview

RUSVEL uses [Clap 4](https://docs.rs/clap) for its CLI. The binary is `rusvel` (or `cargo run --` during development).

```bash
rusvel [OPTIONS] [COMMAND]
```

## Global Options

| Flag | Description |
|------|------------|
| `--mcp` | Start the MCP server (stdio JSON-RPC) instead of the web server |
| `--help` | Show help information |
| `--version` | Show version |

## Commands

### `rusvel` (no subcommand)

Starts the API web server on `0.0.0.0:3000`. Serves the REST API and, if built, the SvelteKit frontend.

```bash
cargo run
```

### `rusvel --mcp`

Starts the MCP (Model Context Protocol) server over stdio. Used for integration with Claude Desktop and other MCP clients.

```bash
cargo run -- --mcp
```


### `rusvel forge`

Forge engine commands for mission planning and goals.

### `rusvel forge mission`

Mission planning subcommands.

#### `rusvel forge mission today`

Generate a prioritized daily plan for the active session. The AI reads your goals, checks engine states, and produces a task list.

```bash
cargo run -- forge mission today
```

**Output:**
```
Generating daily plan...

Daily Plan -- 2026-03-23
==================================================
  1. [High] Finalize API documentation
  2. [Medium] Draft landing page copy
  3. [Low] Review dependency updates

Focus areas:
  - Ship the auth feature

Notes: Focus on high-priority items first.
```

#### `rusvel forge mission goals`

List all goals for the active session.

```bash
cargo run -- forge mission goals
```

**Output:**
```
ID                                      TITLE                      TIMEFRAME   STATUS      PROGRESS
----------------------------------------------------------------------------------------------------
a1b2c3d4-...                            Launch MVP                 Month       Active      25%
```

#### `rusvel forge mission goal add <title>`

Add a new goal to the active session.

```bash
cargo run -- forge mission goal add "Launch MVP" \
  --description "Ship the minimum viable product" \
  --timeframe month
```

**Options:**
| Flag | Default | Description |
|------|---------|------------|
| `--description` | `""` | Goal description |
| `--timeframe` | `month` | One of: `day`, `week`, `month`, `quarter` |

#### `rusvel forge mission review`

Generate a periodic review summarizing accomplishments, blockers, insights, and next actions.

```bash
cargo run -- forge mission review --period week
```

**Options:**
| Flag | Default | Description |
|------|---------|------------|
| `--period` | `week` | One of: `day`, `week`, `month`, `quarter` |

**Output:**
```
Generating Week review...

Review (Week)
==================================================

Accomplishments:
  - Completed API auth flow
  - Deployed staging environment

Blockers:
  - Waiting on SSL certificate

Next actions:
  - Follow up on SSL certificate
  - Schedule design review
```

## Active Session

The CLI stores the active session ID in `~/.rusvel/active_session`. All `forge` commands operate on this session. Change it with `session switch`.

If no active session is set, commands that require one will error:

```
Error: No active session. Run `rusvel session create <name>` first.
```

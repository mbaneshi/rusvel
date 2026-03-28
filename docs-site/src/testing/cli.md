# CLI, REPL & TUI Testing

## CLI One-Shot Commands

### Help

```bash
cargo run -- --help
```

**Expected:** Shows usage, subcommands (session, forge, shell, brief, browser, finance, growth, distro, legal, support, infra, product, code, harvest, content, gtm), and flags (`--mcp`, `--mcp-http`, `--tui`).

### Session Management

```bash
# Create a session
cargo run -- session create "test-project"
```

**Expected:** Prints session ID (UUID). Session is now active.

```bash
# List sessions
cargo run -- session list
```

**Expected:** Table showing session ID, name, kind, created_at. Should include "test-project".

```bash
# Switch session
cargo run -- session switch <session-id>
```

**Expected:** Confirms switch to the specified session.

### Forge / Mission

```bash
# Daily mission plan (requires Ollama)
cargo run -- forge mission today
```

**Expected:** AI-generated daily plan with prioritized tasks. Streams text to stdout. May take 5-30s depending on model.

```bash
# List goals
cargo run -- forge mission goals
```

**Expected:** Table of goals (may be empty on fresh install).

```bash
# Add a goal
cargo run -- forge mission goal add "Launch MVP" \
  --description "Ship v1 to production" --timeframe month
```

**Expected:** Prints goal ID. Goal appears in subsequent `goals` listing.

```bash
# Periodic review
cargo run -- forge mission review --period week
```

**Expected:** AI-generated weekly review (requires Ollama).

### Executive Brief

```bash
cargo run -- brief
```

**Expected:** Generates an executive daily digest summarizing activity across departments.

### Department Commands (Generic)

Every department supports `status`, `list`, and `events`:

```bash
# Status (repeat for each department)
cargo run -- finance status
cargo run -- growth status
cargo run -- code status
cargo run -- content status
cargo run -- harvest status
cargo run -- gtm status
cargo run -- product status
cargo run -- distro status
cargo run -- legal status
cargo run -- support status
cargo run -- infra status
```

**Expected:** Each prints a status summary. May show "No active items" on fresh install.

```bash
# List with kind filter
cargo run -- finance list --kind transactions
cargo run -- support list --kind tickets
cargo run -- legal list --kind contracts
cargo run -- growth list --kind funnel_stages
cargo run -- product list --kind features
```

**Expected:** Table of domain objects. Empty on fresh install, but no error.

```bash
# Events
cargo run -- finance events --limit 5
```

**Expected:** Recent events for that department. Empty on fresh install.

### Engine-Specific Commands

```bash
# Code: analyze current directory
cargo run -- code analyze .
```

**Expected:** Prints analysis results (file count, complexity metrics, dependency info). Works without Ollama.

```bash
# Code: search symbols
cargo run -- code search "DepartmentApp"
```

**Expected:** BM25 search results showing matching symbols/definitions.

```bash
# Content: draft a topic (requires Ollama)
cargo run -- content draft "Why Rust is great for CLI tools"
```

**Expected:** AI-generated markdown draft. Streams to stdout.

```bash
# Harvest: pipeline stats
cargo run -- harvest pipeline
```

**Expected:** Pipeline stage counts (Cold, Contacted, Qualified, etc.). All zeros on fresh install.

### Browser Commands

```bash
# Check browser status (no Chrome needed)
cargo run -- browser status
```

**Expected:** Shows "Not connected" or connection info.

```bash
# Connect to Chrome (requires Chrome with --remote-debugging-port=9222)
cargo run -- browser connect
```

**Expected:** Connects to Chrome DevTools Protocol endpoint.

---

## Interactive REPL Shell

```bash
cargo run -- shell
```

**Expected:** Launches interactive prompt: `rusvel> `

### REPL Commands

| Command | Expected Behavior |
|---------|-------------------|
| `help` | Lists all available commands |
| `status` | Overview across all departments |
| `use finance` | Switches context -> prompt becomes `rusvel:finance> ` |
| `status` (in dept) | Shows finance-specific status |
| `list transactions` | Lists finance transactions |
| `events` | Shows finance events |
| `back` | Returns to top-level `rusvel> ` |
| `use code` | Switches to code department |
| `session list` | Lists all sessions |
| `session create foo` | Creates a new session |
| `exit` or Ctrl+D | Exits the REPL |

### REPL Features

| Feature | How to test | Expected |
|---------|------------|----------|
| Tab completion | Type `us` then press Tab | Completes to `use` |
| Department completion | Type `use ` then Tab | Shows all 14 departments |
| History | Type a command, exit, re-enter shell | Up arrow recalls previous commands |
| History search | Press Ctrl+R, type partial command | Finds matching history entry |

---

## TUI Dashboard

```bash
cargo run -- --tui
```

**Expected:** Full-screen terminal dashboard with 4 panels.

### TUI Panels

| Panel | Content | Expected |
|-------|---------|----------|
| Tasks (top-left) | Active tasks with priority markers | Shows tasks or "No active tasks" |
| Goals (top-right) | Goals with progress bars | Shows goals or "No goals" |
| Pipeline (bottom-left) | Opportunity counts by stage | Shows stages with counts |
| Events (bottom-right) | Recent system events | Shows recent events or empty |

### TUI Keybindings

| Key | Expected |
|-----|----------|
| `q` or `Esc` | Exits TUI cleanly, returns to terminal |
| `t` | Toggles terminal pane focus |
| Arrow keys (in terminal mode) | Navigate between terminal panes |

### TUI Verification

- **No crash on empty data** -- Fresh install should render all panels without panic
- **Resize handling** -- Resize terminal window; panels should reflow
- **Clean exit** -- Terminal should restore to normal state after `q`

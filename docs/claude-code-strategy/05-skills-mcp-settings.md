# Strategy: Skills, MCP Protocol & Settings in RUSVEL

> How custom workflows, model context protocol, and hierarchical config create RUSVEL's operational backbone.

---

## Part A: Skills — Reusable RUSVEL Workflows

### Bundled Skills We Should Use

| Skill | RUSVEL Application |
|-------|-------------------|
| `/simplify` | After building a new engine feature — review for quality, remove duplication |
| `/security-review` | Before every PR that touches auth, outreach, or API endpoints |
| `/batch` | Large cross-crate refactors (rename a domain type, update all implementations) |
| `/loop 5m cargo test` | Continuous testing during multi-crate changes |
| `/init` | Already done — CLAUDE.md exists |
| `/insights` | End of week — analyze how development sessions went |

### Custom Skills to Build

**`.claude/skills/wire-engine.md`:**
```yaml
---
name: wire-engine
description: Scaffold complete surface exposure for a RUSVEL engine (CLI + API + MCP + tests).
user_invocable: true
---
Given an engine name, create:
1. CLI subcommands in rusvel-cli (Clap 4 struct + handler)
2. API endpoints in rusvel-api (Axum handler + route)
3. MCP tool definitions in rusvel-mcp (JSON-RPC method)
4. Integration tests for each surface

Follow the forge-engine wiring pattern. Read:
- crates/rusvel-cli/src/lib.rs (existing CLI pattern)
- crates/rusvel-api/src/lib.rs (existing API pattern)
- crates/rusvel-mcp/src/lib.rs (existing MCP pattern)
- crates/forge-engine/src/lib.rs (engine method signatures)

Engine name is passed as argument: /wire-engine harvest
```

**`.claude/skills/new-component.md`:**
```yaml
---
name: new-component
description: Create a new design system component following RUSVEL conventions.
user_invocable: true
---
Create a new Svelte 5 component in frontend/src/lib/components/ui/:
1. ComponentName.svelte with $props(), cn(), snippets
2. Add to index.ts barrel export
3. Follow existing patterns (see Button.svelte, Card.svelte)
4. Use semantic tokens (--r-*), not raw Tailwind colors
5. Full TypeScript types for all props
6. Accessibility: aria attributes, keyboard handling

Component name is passed as argument: /new-component Dropdown
```

**`.claude/skills/new-engine.md`:**
```yaml
---
name: new-engine
description: Scaffold a new RUSVEL engine crate with proper hexagonal structure.
user_invocable: true
---
Create a new engine crate:
1. Create crates/<name>-engine/Cargo.toml (depends only on rusvel-core)
2. Create crates/<name>-engine/src/lib.rs with:
   - Engine struct taking port trait objects
   - Constructor: new(ports) -> Self
   - Domain-specific methods
   - #[cfg(test)] mod tests with mock ports
3. Add to workspace Cargo.toml members list
4. Add to rusvel-app/Cargo.toml dependencies
5. Wire in rusvel-app/src/main.rs composition root
6. Run cargo build && cargo test -p <name>-engine

Engine name is passed as argument: /new-engine analytics
```

**`.claude/skills/daily-review.md`:**
```yaml
---
name: daily-review
description: End-of-day development review — what was built, what broke, what's next.
user_invocable: true
---
Generate a daily development review:
1. Run `git log --oneline --since="8 hours ago"` to see today's commits
2. Run `cargo test` to verify everything passes
3. Check `docs/status/current-state.md` against actual state
4. List any new gaps or TODOs discovered
5. Suggest tomorrow's priorities based on roadmap

Output format: brief markdown summary with sections: Done, Issues, Tomorrow.
```

**`.claude/skills/deploy-check.md`:**
```yaml
---
name: deploy-check
description: Pre-deployment verification for RUSVEL.
user_invocable: true
---
Run pre-deployment checks:
1. `cargo build --release` — clean release build
2. `cargo test` — all tests pass
3. `cd frontend && pnpm build` — frontend builds
4. `cargo clippy -- -D warnings` — no lint warnings
5. Check for TODO/FIXME/HACK in changed files
6. Verify no .env or credentials in tracked files
7. Run /security-review on API endpoints

Report: PASS/FAIL with details.
```

---

## Part B: MCP Protocol — RUSVEL as an MCP Server

### The Self-Building Loop

This is where it gets powerful. RUSVEL **already has an MCP server** (`rusvel-mcp`). Once wired, Claude Code can **directly manage RUSVEL sessions, goals, and missions** through MCP tools.

### Current MCP Tools (built, not wired)

```
session_list    → List all RUSVEL sessions
session_create  → Create a new session
session_switch  → Switch active session
mission_today   → Generate daily plan
mission_goals   → List goals
mission_add_goal → Add a new goal
```

### How to Wire It

**Step 1: Wire `--mcp` flag in main.rs** (one if-branch):
```rust
if args.mcp {
    rusvel_mcp::serve(app_state).await?;
    return Ok(());
}
```

**Step 2: Register RUSVEL as MCP server in `.mcp.json`:**
```json
{
  "mcpServers": {
    "rusvel": {
      "command": "cargo",
      "args": ["run", "--", "--mcp"],
      "cwd": "/Users/bm/all-in-one-rusvel"
    }
  }
}
```

**Step 3: Use RUSVEL tools from Claude Code:**
```
Claude: I'll check your current goals...
[MCP: rusvel.mission_goals] → returns goals list

Claude: Let me create a task for that...
[MCP: rusvel.mission_add_goal] → creates goal in RUSVEL's DB
```

### Future MCP Tools to Add

| Tool | Engine | Description |
|------|--------|-------------|
| `harvest_scan` | harvest | Trigger opportunity scan |
| `harvest_score` | harvest | Score an opportunity |
| `harvest_pipeline` | harvest | View pipeline status |
| `content_draft` | content | Generate content draft |
| `content_schedule` | content | Schedule content publication |
| `content_approve` | content | Approve content for publishing |
| `gtm_contacts` | gtm | List/search contacts |
| `gtm_outreach` | gtm | Create outreach sequence |
| `gtm_invoice` | gtm | Generate invoice |
| `code_analyze` | code | Analyze code structure |
| `code_search` | code | Semantic code search |
| `events_query` | core | Query event history |
| `jobs_status` | core | Check job queue status |

### External MCP Servers for RUSVEL

Connect these to enhance RUSVEL's capabilities:

| Server | RUSVEL Use |
|--------|-----------|
| **GitHub** | Code management, PRs, issues for all 3 products |
| **Notion** | Already connected — knowledge base, project tracking |
| **Slack** | Team communication (when RUSVEL scales beyond solo) |
| **PostgreSQL** | If RUSVEL migrates from SQLite for multi-user |
| **Playwright** | harvest-engine web scraping, content-engine site testing |
| **Gmail** | gtm-engine outreach email sending |
| **Google Calendar** | content-engine calendar scheduling |

### MCP Configuration Hierarchy

```
~/.claude/.mcp.json          → User-global (GitHub, Notion)
.mcp.json                    → Project (RUSVEL's own MCP server)
.claude/.mcp.local.json      → Local-only (dev credentials, test servers)
```

---

## Part C: Settings Hierarchy — RUSVEL's Configuration Layers

### Settings Architecture

```
~/.claude/settings.json           → User-wide defaults
.claude/settings.json             → RUSVEL project settings (checked in)
.claude/settings.local.json       → Machine-local overrides (gitignored)
```

### Project Settings (`.claude/settings.json`)

```json
{
  "model": "opus",
  "effort": "high",
  "permissions": {
    "allow": [
      "Bash(cargo build)",
      "Bash(cargo test*)",
      "Bash(cargo run*)",
      "Bash(cargo clippy*)",
      "Bash(pnpm *)",
      "Bash(cd frontend*)",
      "Bash(git log*)",
      "Bash(git status*)",
      "Bash(git diff*)",
      "Read",
      "Glob",
      "Grep",
      "WebSearch"
    ],
    "deny": [
      "Bash(rm -rf*)",
      "Bash(git push --force*)",
      "Bash(git reset --hard*)",
      "Bash(cargo publish*)"
    ]
  },
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "command": "if [[ \"$FILE_PATH\" == *.rs ]]; then rustfmt \"$FILE_PATH\" 2>/dev/null; fi"
      }
    ],
    "Stop": [
      {
        "command": "osascript -e 'display notification \"Claude finished\" with title \"RUSVEL\"'"
      }
    ]
  },
  "env": {
    "RUSVEL_DIR": "~/.rusvel",
    "RUST_LOG": "info"
  }
}
```

### Local Settings (`.claude/settings.local.json`)

```json
{
  "env": {
    "CLAUDE_CODE_MAX_SUBSCRIPTION_FIX": "true",
    "ANTHROPIC_MODEL": "claude-sonnet-4-6-20250514"
  }
}
```

### Permission Modes by Task

| Task | Permission Mode | Why |
|------|----------------|-----|
| Architecture design | `plan` | Read-only exploration, no accidental changes |
| Feature development | `default` | Normal guardrails |
| Rapid iteration | `acceptEdits` | Trust file edits, prompt for bash |
| Automated pipeline | `dontAsk` | CI/CD, no human in loop |

---

## Concrete Actions

### Immediate

1. **Wire MCP `--mcp` flag** — one if-branch in main.rs
2. **Create `.mcp.json`** — register RUSVEL as MCP server
3. **Create `/wire-engine` skill** — scaffold engine surfaces

### Short-term

4. **Create 5 custom skills** (wire-engine, new-component, new-engine, daily-review, deploy-check)
5. **Set up project settings.json** — permissions, hooks, env vars
6. **Add Notion MCP server** — already connected, use for project tracking

### Medium-term

7. **Add 15+ MCP tools** — expose all engines via MCP
8. **Add Playwright MCP** — web scraping for harvest-engine
9. **Add Gmail/Calendar MCP** — outreach and content scheduling
10. **Use `/loop`** for automated opportunity scanning

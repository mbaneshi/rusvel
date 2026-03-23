# Strategy: Subagents & Hooks in RUSVEL

> How specialized AI workers and lifecycle automation create a self-building, self-guarding development system.

---

## Part A: Subagents — RUSVEL's AI Development Team

### The Concept

RUSVEL has 10 built-in **personas** in forge-engine (CodeWriter, Tester, ContentWriter, SecurityAuditor, etc.). Claude Code has **subagents**. These are the same idea at different layers:

| Layer | System | Purpose |
|-------|--------|---------|
| **Development-time** | Claude Code subagents | Build RUSVEL itself with specialized agents |
| **Runtime** | RUSVEL personas (forge-engine) | RUSVEL's AI agents that serve the end user |

### Built-in Subagents — How RUSVEL Uses Each

**Explore agent** — Codebase research without changes:
- "How does the event system flow from forge-engine to rusvel-db?"
- "Find all places where Session is created across all crates"
- "What ports does harvest-engine actually use?"

**Plan agent** — Architecture before implementation:
- Design a new engine's port requirements
- Plan a cross-crate refactor (e.g., rename a domain type)
- Evaluate trade-offs (WebSocket vs SSE for streaming)

**General-purpose** — Complex multi-step tasks:
- "Build the full vertical slice for harvest-engine: CLI + API + MCP + tests"
- "Migrate all raw Tailwind colors to design system tokens in frontend"
- "Add approval flow across content-engine and gtm-engine"

### Custom Subagents to Create

**`.claude/agents/rust-engine.md`:**
```yaml
---
name: rust-engine
description: Build and modify Rust engine crates. Enforces hexagonal architecture.
tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Bash
model: opus
---
You build RUSVEL engine crates. Rules:
- Engines depend ONLY on rusvel-core port traits
- Never import adapter crates
- Use AgentPort for LLM calls
- All domain types have metadata: serde_json::Value
- Event.kind is String
- Each crate < 2000 lines
- Write tests with mock ports

Always run `cargo test -p <crate>` after changes.
```

**`.claude/agents/svelte-ui.md`:**
```yaml
---
name: svelte-ui
description: Build SvelteKit 5 frontend pages and components using the RUSVEL design system.
tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Bash
model: sonnet
---
You build RUSVEL's SvelteKit 5 frontend. Rules:
- Import from $lib/components barrel
- Use design system: Button, Card, Badge, Input, Icon, Heading, Text, Stack
- Use semantic tokens: var(--r-bg-surface), var(--r-fg-default)
- Svelte 5 patterns: $props(), snippets, $bindable(), $derived
- No raw Tailwind colors — always design tokens

Always run `npm run check` after changes.
```

**`.claude/agents/test-writer.md`:**
```yaml
---
name: test-writer
description: Write comprehensive tests for RUSVEL crates and frontend.
tools:
  - Read
  - Edit
  - Glob
  - Grep
  - Bash
model: sonnet
---
You write tests for RUSVEL. Patterns:
- Engine tests: mock ports, test happy path + error cases
- DB tests: in-memory SQLite (:memory:)
- API tests: integration tests with test AppState
- Frontend: svelte-check type validation

Run tests after writing: `cargo test -p <crate>` or `npm run check`.
```

**`.claude/agents/security-auditor.md`:**
```yaml
---
name: security-auditor
description: Security review for auth, credentials, outreach gates, and API endpoints.
tools:
  - Read
  - Glob
  - Grep
disallowedTools:
  - Write
  - Edit
  - Bash
model: opus
---
You are a read-only security auditor. Review:
- AuthPort implementation (credential handling)
- API endpoint authorization
- Human approval gates (ADR-008)
- Outreach sequence safety (rate limits, consent)
- Content publishing gates
- Input validation on all API endpoints
- SQL injection prevention in rusvel-db

Report findings without making changes.
```

**`.claude/agents/api-builder.md`:**
```yaml
---
name: api-builder
description: Expose engine functionality via CLI, API, and MCP surfaces.
tools:
  - Read
  - Write
  - Edit
  - Glob
  - Grep
  - Bash
model: sonnet
---
You wire RUSVEL engines to surfaces (CLI, API, MCP). Pattern:
1. Add Clap subcommand to rusvel-cli
2. Add Axum handler + route to rusvel-api
3. Add MCP tool definition to rusvel-mcp
4. All three call the same engine method
5. Update CLAUDE.md with new endpoints

Follow existing patterns in forge-engine's wiring.
```

### Subagent Workflows

**Parallel engine development:**
```
User: "Wire harvest-engine and content-engine to CLI and API"

→ Agent(rust-engine): Add harvest CLI commands
→ Agent(rust-engine): Add content CLI commands
→ Agent(api-builder): Add harvest API endpoints
→ Agent(api-builder): Add content API endpoints
→ Agent(test-writer): Write integration tests
→ Agent(security-auditor): Review new endpoints (background)
```

**Full vertical slice:**
```
User: "Build the opportunity pipeline page"

→ Agent(rust-engine): Verify harvest-engine has all needed methods
→ Agent(api-builder): Expose harvest via API endpoints
→ Agent(svelte-ui): Build /harvest page with design system
→ Agent(test-writer): Write tests
```

### Worktree Isolation for Risky Changes

```
Agent(rust-engine, isolation: "worktree"):
  "Refactor StoragePort to split ObjectStore into typed stores"
  → Works in isolated copy
  → If it breaks, no damage to main
  → If it works, merge the worktree branch
```

---

## Part B: Hooks — Automated Quality Gates

### Hook Strategy for RUSVEL

Hooks enforce RUSVEL's architecture rules **automatically**, without relying on Claude remembering them.

### Essential Hooks to Configure

**1. Auto-format on file save (PostToolUse):**
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit",
        "command": "if [[ \"$TOOL_INPUT\" == *.rs ]]; then rustfmt \"$FILE_PATH\"; fi"
      },
      {
        "matcher": "Write|Edit",
        "command": "if [[ \"$TOOL_INPUT\" == *.svelte || \"$TOOL_INPUT\" == *.ts ]]; then npx prettier --write \"$FILE_PATH\"; fi"
      }
    ]
  }
}
```

**2. Architecture guard (PreToolUse):**
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write",
        "command": "python3 .claude/hooks/check-engine-imports.py \"$FILE_PATH\"",
        "description": "Block engine crates from importing adapter crates"
      }
    ]
  }
}
```

**3. Auto-test after changes (PostToolUse):**
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "command": "if [[ \"$FILE_PATH\" == */crates/* ]]; then cargo test -p $(basename $(dirname $(dirname \"$FILE_PATH\"))) 2>/dev/null; fi"
      }
    ]
  }
}
```

**4. Protect critical files (PreToolUse):**
```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Edit|Write",
        "command": "echo \"$FILE_PATH\" | grep -qE '(rusvel-core/src/ports\\.rs|rusvel-core/src/domain\\.rs)' && echo 'BLOCKED: Core types require explicit approval' && exit 1 || exit 0"
      }
    ]
  }
}
```

**5. Desktop notification on completion (Stop):**
```json
{
  "hooks": {
    "Stop": [
      {
        "command": "osascript -e 'display notification \"Claude finished working\" with title \"RUSVEL\"'"
      }
    ]
  }
}
```

**6. Context re-injection after compaction (PostCompact):**
```json
{
  "hooks": {
    "PostCompact": [
      {
        "command": "cat docs/status/current-state.md"
      }
    ]
  }
}
```

**7. Session audit (SessionStart):**
```json
{
  "hooks": {
    "SessionStart": [
      {
        "command": "echo '=== RUSVEL Session Start ===' && cargo test --quiet 2>&1 | tail -1 && echo 'Last commit:' && git log --oneline -1"
      }
    ]
  }
}
```

### Hook for Self-Building

**Emit RUSVEL events from Claude Code sessions (PostToolUse):**
```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "command": "curl -s -X POST http://localhost:3000/api/events -d '{\"kind\":\"dev.file_changed\",\"source\":\"claude-code\",\"payload\":{\"file\":\"'$FILE_PATH'\"}}'"
      }
    ]
  }
}
```

This means RUSVEL's event bus tracks its own development — the self-building loop closes.

---

## Concrete Actions

### Immediate

1. **Create 5 subagent files** in `.claude/agents/`
2. **Add PostToolUse hook for rustfmt/prettier** — auto-format on save
3. **Add Stop hook for desktop notification** — know when Claude finishes

### Short-term

4. **Add architecture guard hook** — prevent engine→adapter imports
5. **Add PostCompact hook** — re-inject current-state.md after compaction
6. **Add SessionStart hook** — run tests + show last commit on session start

### Medium-term

7. **Wire RUSVEL event emission hook** — Claude Code changes feed into RUSVEL's event bus
8. **Add security review hook** — auto-trigger for auth/outreach file changes
9. **Use worktree isolation** for all rusvel-core refactors

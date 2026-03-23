# Strategy: Cloud/CI/CD, Security/Privacy & Agent Teams in RUSVEL

> How deployment automation, security posture, and multi-agent collaboration shape RUSVEL's operational maturity.

---

## Part A: Cloud & CI/CD

### Current State

RUSVEL is **local-first by design** — single binary, SQLite WAL, no cloud dependencies. But CI/CD and deployment automation are essential for a solo builder.

### GitHub Actions — Automated Quality Gates

**`.github/workflows/ci.yml`:**
```yaml
name: RUSVEL CI
on: [push, pull_request]
jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo clippy -- -D warnings
      - run: cargo test
      - run: cargo build --release

  frontend:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: frontend
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
      - run: npm ci
      - run: npm run check
      - run: npm run build
```

### Claude in CI — Automated PR Reviews

**Install GitHub App:**
```bash
/install-github-app
```

**What Claude reviews in RUSVEL PRs:**
- Architecture violations (engine importing adapter)
- Missing tests for new engine methods
- Raw Tailwind colors instead of design tokens
- Missing event emissions after state changes
- Security concerns in API endpoints
- Approval gates bypassed in content/outreach code

### Deployment Strategy

**Phase 1 (now): Local development**
```bash
cargo build --release
# Single binary at target/release/rusvel-app
# Frontend embedded via rust-embed (planned)
```

**Phase 2: Docker for reproducibility**
```dockerfile
FROM rust:1.80-slim AS builder
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /target/release/rusvel-app /usr/local/bin/
COPY frontend/build /app/frontend/
CMD ["rusvel-app"]
```

**Phase 3: Cloud deployment (optional)**
- Single VPS (Hetzner/Fly.io) for always-on RUSVEL
- SQLite WAL works fine for single-user
- RUSVEL scans opportunities, publishes content, sends outreach 24/7
- Claude Code Remote for development from any device

### Claude Code on the Web

**Use case:** Work on RUSVEL from any device via claude.ai:
```bash
claude --remote          # Create web session
claude --teleport        # Resume web session in terminal
claude --remote-control  # Control terminal session from web
```

### Third-Party LLM Providers

RUSVEL's `rusvel-llm` already supports 4 providers. Map to Claude Code's supported backends:

| RUSVEL Provider | Claude Code Backend | Use Case |
|----------------|--------------------|---------|
| ClaudeCliProvider | Claude Max subscription | Primary — $0 per call |
| ClaudeApiProvider | Anthropic API / Bedrock / Vertex | Production — metered billing |
| OllamaProvider | N/A (local) | Offline development, privacy |
| OpenAiProvider | N/A | Fallback, comparison |

**Multi-provider routing:**
```
Tier 1: Ollama (free, local, fast) → classification, tagging, simple tasks
Tier 2: Claude CLI Max ($0, rate-limited) → planning, analysis, content
Tier 3: Claude API (metered) → high-priority, customer-facing, production
```

---

## Part B: Security & Privacy

### RUSVEL's Security Surface

RUSVEL handles sensitive data across multiple engines:

| Engine | Sensitive Data | Risk Level |
|--------|---------------|------------|
| **gtm** | Client contacts, email addresses, invoices | HIGH |
| **harvest** | Scraped opportunity data, proposals | MEDIUM |
| **content** | Unpublished content, API keys for platforms | MEDIUM |
| **auth** | Credentials, API tokens | CRITICAL |
| **forge** | Agent prompts, business strategy | MEDIUM |

### Permission Mode Strategy

| Context | Mode | Rationale |
|---------|------|-----------|
| Architecture design | `plan` | Read-only exploration |
| Normal development | `default` | Safe defaults, prompt for risky ops |
| Rapid UI iteration | `acceptEdits` | Trust file edits, still prompt for shell |
| CI/CD automation | `dontAsk` | No human in loop, but scoped permissions |
| Production deployment | `default` | Extra caution |

### Security Reviews — When to Run

**Always run `/security-review` before:**
- Merging changes to `rusvel-auth/`
- Adding new API endpoints
- Modifying `rusvel-api/` middleware
- Changing gtm-engine outreach logic
- Updating content-engine publishing flow
- Any change that touches credentials or tokens

### Sandbox Mode for RUSVEL

**Enable sandbox** when:
- Testing untrusted content from harvest-engine scrapers
- Running LLM-generated code from code-engine
- Processing external input from API endpoints

**Sandbox configuration:**
```json
{
  "sandbox": {
    "enabled": true,
    "allowPaths": [
      "/Users/bm/all-in-one-rusvel/",
      "/Users/bm/.rusvel/"
    ]
  }
}
```

### RUSVEL's Built-in Security (ADR-008)

Already designed:
- **Human approval gates** on content publishing
- **Human approval gates** on outreach sequences
- **Rate limiting** in forge-engine SafetyGuard
- **Circuit breaking** to prevent runaway agents
- **Event audit trail** — every action is logged

**What's missing:**
- Approval UI in frontend
- API endpoints for approve/reject
- Rate limit enforcement in production
- Input validation on all API endpoints

### Privacy Considerations

RUSVEL is local-first, which is a privacy advantage:
- SQLite on disk — no cloud database
- Ollama for local LLM — no data leaves machine
- Claude CLI with Max — Anthropic's privacy policy applies
- No telemetry built into RUSVEL

**When RUSVEL goes multi-user:**
- Per-user data isolation
- Session-namespaced memory (already designed)
- Encrypted credential storage (replace in-memory AuthPort)
- HTTPS for API (currently HTTP)

---

## Part C: Agent Teams (Experimental)

### Concept for RUSVEL

Agent Teams = multiple Claude sessions working in parallel on different aspects. This maps perfectly to RUSVEL's multi-engine architecture.

### Team Configuration for RUSVEL Development

**Team 1: "Engine Builder"**
```
Teammate A: Rust backend — works on engine logic + adapters
Teammate B: Surface layer — works on CLI + API + MCP
Teammate C: Frontend — builds SvelteKit pages
```

**Shared task list:**
```
□ Implement harvest-engine opportunity scoring
□ Add harvest CLI commands (assigned: B)
□ Add harvest API endpoints (assigned: B)
□ Build /harvest page with pipeline view (assigned: C)
□ Write integration tests (assigned: A)
```

**Quality gates:**
- All teammates must pass `cargo test` before merging
- Frontend teammate must pass `npm run check`
- Architecture violations block all teammates

### When to Use Agent Teams

| Scenario | Team Setup | Why |
|----------|-----------|-----|
| New engine vertical slice | Backend + Surface + Frontend | Three independent workstreams |
| Cross-crate refactor | Multiple isolated worktrees | Each teammate handles subset of crates |
| Bug triage | Explorer + Fixer + Tester | Find → Fix → Verify pipeline |
| Release prep | Builder + Tester + Deployer | Build → Test → Package |

### Team + Subagent Synergy

Agent Teams are **parallel sessions**. Subagents are **within a session**. Combine them:

```
Team Session A (Backend):
  ├── Subagent: rust-engine (builds engine logic)
  ├── Subagent: test-writer (writes tests)
  └── Subagent: security-auditor (reviews, read-only)

Team Session B (Frontend):
  ├── Subagent: svelte-ui (builds components)
  └── Subagent: test-writer (runs svelte-check)

Team Session C (Integration):
  ├── Subagent: api-builder (wires surfaces)
  └── Subagent: test-writer (integration tests)
```

### RUSVEL Agents ↔ Claude Code Agents Alignment

RUSVEL's 10 built-in personas map to Claude Code agents:

| RUSVEL Persona | Claude Code Agent | Connection |
|---------------|-------------------|------------|
| CodeWriter | rust-engine subagent | Same role, different layer |
| Tester | test-writer subagent | Same role |
| ContentWriter | N/A (content-engine does this at runtime) | Runtime persona |
| SecurityAuditor | security-auditor subagent | Same role |
| ProjectManager | Plan agent | Same role |
| DataAnalyst | Explore agent | Similar (codebase vs data) |

**Future vision:** RUSVEL's runtime personas use the same agent patterns as Claude Code's subagents. The development tool and the product converge.

---

## Concrete Actions

### Immediate

1. **Create `.github/workflows/ci.yml`** — Rust + Frontend CI
2. **Run `/security-review`** on current API endpoints
3. **Set up permission deny rules** for destructive operations

### Short-term

4. **Install GitHub App** — automated PR reviews
5. **Create Docker deployment** for always-on RUSVEL
6. **Add input validation** to all API endpoints
7. **Set up sandbox mode** for development

### Medium-term

8. **Try Agent Teams** for next vertical slice (harvest-engine full stack)
9. **Wire approval UI** — frontend buttons that call approve/reject API endpoints
10. **Add HTTPS** to API server (reqwest with rustls)
11. **Implement multi-provider routing** — Ollama → Claude CLI → Claude API tiering

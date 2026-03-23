# Strategy: Authentication, Debugging & Performance in RUSVEL

> How auth management, diagnostic tools, and cost optimization sustain RUSVEL's development velocity.

---

## Part A: Authentication

### Claude Code Auth — RUSVEL's LLM Access

**Current setup:** Claude Max subscription via CLI. No API key, no cost.

```
Method: ClaudeCliProvider::max_subscription()
Cost: $0 (included in Max subscription)
Env vars: CLAUDE_CODE_MAX_SUBSCRIPTION_FIX=true
Limit: Rate-limited, not cost-limited
```

**Auth hierarchy for RUSVEL's LLM backends:**

| Priority | Provider | Auth Method | Cost | Use Case |
|----------|----------|-------------|------|----------|
| 1 | Ollama | None (local) | $0 | Fast tasks, offline, privacy |
| 2 | Claude CLI Max | Max subscription | $0 | Complex reasoning, content gen |
| 3 | Claude API | ANTHROPIC_API_KEY | Metered | Production, high availability |
| 4 | OpenAI | OPENAI_API_KEY | Metered | Fallback, comparison |

### Credential Management in RUSVEL

**Current:** `rusvel-auth` stores credentials in-memory from env vars.

**Secure credential flow:**
```
1. User sets ANTHROPIC_API_KEY, OPENAI_API_KEY in shell profile
2. RUSVEL's AuthPort loads from env on startup
3. Credentials stored in-memory only — never persisted to disk
4. Per-engine credential scoping (gtm gets email creds, content gets platform creds)
```

**Platform credentials for engines:**

| Engine | Credentials Needed | Storage |
|--------|-------------------|---------|
| content | DEV.to API key, Twitter OAuth, LinkedIn OAuth | AuthPort |
| harvest | Upwork cookies/session, GitHub token | AuthPort |
| gtm | SMTP credentials, LinkedIn session | AuthPort |
| code | GitHub token (for dependency analysis) | AuthPort |

**What to build:**
- Encrypted credential storage (replace in-memory with keyring/keychain)
- Per-session credential scoping
- OAuth flow for platform integrations
- Credential rotation reminders

### MCP OAuth for External Services

When connecting MCP servers (GitHub, Gmail, Notion):
```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_PERSONAL_ACCESS_TOKEN": "${GITHUB_TOKEN}"
      }
    }
  }
}
```

Claude Code handles OAuth callbacks for MCP servers that support it. No manual token management.

---

## Part B: Debugging & Diagnostics

### Debug Mode for RUSVEL Development

**When things go wrong:**

```bash
claude --debug                    # Full debug output
claude --debug "llm,mcp"          # Filter to specific categories
claude --verbose                  # See Claude's reasoning
```

**Common RUSVEL debug scenarios:**

| Problem | Debug Approach |
|---------|---------------|
| LLM call fails | `--debug llm` — see raw request/response |
| MCP tools not showing | `--debug mcp` — verify server startup |
| Hook not firing | `/hooks` — verify configuration |
| Permission blocked | `--debug permissions` — see rule evaluation |
| Context too large | `/context` — see what's consuming space |
| Subagent stuck | `/tasks` + `Ctrl+T` — check background status |

### `/doctor` — Installation Verification

Run regularly to verify:
- Claude Code version is current
- Authentication is valid
- MCP servers are reachable
- Settings files are valid JSON
- Hooks are properly configured

### RUSVEL's Own Diagnostic Tools

**`/api/health` endpoint:**
```json
{
  "status": "ok",
  "db": "connected",
  "llm": "claude-cli",
  "sessions": 3,
  "active_session": "uuid",
  "uptime": "2h 34m"
}
```

**Event trail for debugging:**
```bash
# Query RUSVEL's event log
curl http://localhost:3000/api/sessions/{id}/events
# → Shows every action, LLM call, state change
```

**Planned diagnostics:**
- Token usage tracking per engine
- LLM latency monitoring
- Error rate by engine
- Job queue depth and processing time

### Debugging the Self-Building Loop

When Claude Code is both the developer and the LLM backend:

```
Problem: "mission_today returns empty plan"

Debug flow:
1. Check Claude Code debug: --debug llm
2. Check RUSVEL API: curl /api/sessions/{id}/mission/today
3. Check RUSVEL events: curl /api/sessions/{id}/events
4. Check raw Claude CLI call: claude -p "test prompt" --verbose
5. Check Ollama: curl http://localhost:11434/api/tags
```

**Logging strategy:**
```rust
// In ClaudeCliProvider
tracing::debug!(prompt = %prompt, model = %model, "Spawning claude -p");
tracing::debug!(response = %output, "Claude CLI response");
tracing::warn!(error = %err, "Claude CLI failed, falling back to Ollama");
```

---

## Part C: Performance & Cost Optimization

### Token & Cost Management

**RUSVEL's cost structure:**

| Component | Cost | Optimization |
|-----------|------|-------------|
| Claude CLI (Max) | $0 (subscription) | Rate-limited → batch calls, cache responses |
| Claude API | ~$3/M input, $15/M output (Opus) | Use Haiku for simple tasks |
| Ollama | $0 (local compute) | Free but slower, use for non-critical |
| Development sessions | Max subscription covers | Use `/fast`, `/effort low` for simple tasks |

### Cost Optimization Strategies

**1. Model routing by task complexity:**
```rust
match task.complexity {
    Complexity::Trivial => Provider::Ollama("qwen2.5:7b"),     // $0, fast
    Complexity::Simple => Provider::Ollama("qwen3:14b"),        // $0, good
    Complexity::Medium => Provider::ClaudeCli("sonnet"),         // $0, rate-limited
    Complexity::Complex => Provider::ClaudeCli("opus"),          // $0, rate-limited
    Complexity::Critical => Provider::ClaudeApi("opus"),         // $$, reliable
}
```

**2. Caching LLM responses:**
```rust
// In rusvel-memory
// Cache identical prompts for 1 hour
if let Some(cached) = memory_port.search(&prompt_hash).await? {
    if cached.created_at > Utc::now() - Duration::hours(1) {
        return Ok(cached.content);
    }
}
```

**3. Budget caps per engine:**
```toml
# ~/.rusvel/config.toml
[budgets]
forge_daily_usd = 5.0
harvest_per_scan_usd = 0.50
content_per_article_usd = 1.0
gtm_per_outreach_usd = 0.25
```

**4. Effort level optimization:**

| Task | Effort | Model | Why |
|------|--------|-------|-----|
| Tag content with category | low | Haiku | Classification only |
| Score opportunity relevance | low | Sonnet | Numeric output |
| Generate daily plan | high | Opus | Multi-step reasoning |
| Write blog post | high | Opus | Creative output |
| Code review | max | Opus | Deep analysis |
| Proposal generation | max | Opus | Business-critical |

### Context Window Optimization

**For development sessions:**
- `/compact` proactively — don't wait for auto-compaction
- Use subagents — each gets isolated context, main stays lean
- Use `@imports` in CLAUDE.md instead of pasting long docs
- Rules load conditionally — only relevant file types consume context

**For RUSVEL's LLM calls:**
- `--bare` flag — skip CLAUDE.md loading for API calls
- `--no-session-persistence` — don't accumulate session history
- Prompt templates — reuse structured prompts, leverage caching
- Chunk large inputs — process opportunities in batches of 10, not 100

### Performance Monitoring

**Development velocity metrics:**
```bash
/cost        # How much this session cost
/stats       # Usage patterns, session streaks
/insights    # What went well, what took too long
```

**RUSVEL runtime metrics (MetricStore):**
- Token usage per engine per day
- LLM latency p50/p95/p99
- Cache hit rate
- Job queue processing time
- Error rate by provider

### Prompt Caching

Claude Code automatically caches repeated context. Maximize this:
- Stable system prompts (don't vary unnecessarily)
- CLAUDE.md stays consistent → cached across sessions
- Per-persona prompts are fixed templates → cached
- Only the user query varies → minimal uncached tokens

---

## Concrete Actions

### Immediate

1. **Set up multi-provider routing** — Ollama first, Claude CLI fallback
2. **Add `--bare --no-session-persistence`** to ClaudeCliProvider
3. **Use effort levels** — map task complexity to effort flag

### Short-term

4. **Implement response caching** in rusvel-memory
5. **Add per-engine budget caps** in config.toml
6. **Set up tracing/logging** in LLM provider calls
7. **Run `/doctor`** to verify installation health

### Medium-term

8. **Build MetricStore dashboard** — track costs, latency, errors
9. **Implement prompt templates** — stable system prompts for cache hits
10. **Add encrypted credential storage** — replace in-memory AuthPort
11. **Set up model routing** — Ollama → CLI → API tiering based on task

# Plan: Agent SDK Features → RUSVEL Integration

> Source: [anthropics/claude-agent-sdk-python](https://github.com/anthropics/claude-agent-sdk-python)
> Goal: Steal the best patterns from Anthropic's Agent SDK and wire them into RUSVEL's Rust agent runtime
> Date: 2026-03-25
> Status: Proposed
> Related docs:
>   - `docs/plans/agent-orchestration.md` — delegate_agent, event triggers, workflow templates (P9)
>   - `docs/plans/next-level-proposals.md` — P1-P12 enhancements with ROI analysis
>   - `docs/plans/next-level-inspiration-2026-03-25.md` — Playbooks, Executive Brief, Starter Kits (GenAICircle)
>   - `docs/design/architecture-v2.md` — Hexagonal architecture, 19 ports
>   - `docs/design/decisions.md` — 13 ADRs (esp. ADR-008 approval, ADR-009 AgentPort, ADR-014 DepartmentApp)
> In-progress code:
>   - `crates/rusvel-core/src/department/` — DepartmentApp trait + DepartmentManifest (ADR-014)
>   - `crates/dept-content/`, `crates/dept-forge/` — DepartmentApp migrations started
>   - `frontend/src/routes/flows/` — Flow builder UI started
>   - `crates/rusvel-agent/src/lib.rs` — AgentRuntime with run_streaming() + AgentEvent (just shipped)
>   - `crates/rusvel-engine-tools/` — 12 engine tools + ScopedToolRegistry (just shipped)

---

## Context: What Just Shipped (Phases 1-6)

Before this plan, we already implemented:
- **AgentRuntime wired to chat** — replaced ClaudeCliStreamer with Rust-side agent loop
- **21 tools registered** — 9 built-in (file ops, shell, git) + 12 engine (harvest 5, content 5, code 2)
- **ScopedToolRegistry** — per-department tool filtering
- **LLM text streaming** — character-by-character deltas from Claude CLI provider
- **Frontend tool call UI** — ToolCallCard, ApprovalCard, inline rendering
- **Real platform adapters** — DEV.to HTTP adapter (LinkedIn + Twitter already existed)

This plan builds on that foundation. Features below assume `run_streaming()`, `AgentEvent`, `ToolRegistry`, and `ScopedToolRegistry` exist and work.

---

## Feature Map: Agent SDK → RUSVEL → Broader Roadmap

Each feature below shows how it connects to the Agent SDK source, the RUSVEL implementation, and which downstream features it unlocks across all plan docs.

---

### 1. PreToolUse / PostToolUse Hooks

**Agent SDK pattern:** Deterministic guardrails intercepting tool calls before/after execution. Can validate, modify, or deny. Shell hooks with exit code 2 = blocked (no negotiation).

**RUSVEL gap:** `hook_dispatch` fires *after* chat completes. ADR-008 approval gates live in the job queue, not the agent loop.

**What to build:**
- `HookPoint` enum: `PreToolUse`, `PostToolUse`, `PreChat`, `PostChat`
- `ToolHook` trait: `async fn check(name, args) -> HookDecision { Allow | Deny(reason) | Modify(new_args) }`
- Wire into `AgentRuntime::run_streaming()` — before `self.tools.call()`, run registered hooks
- Default hooks: block `bash rm -rf`, require approval for `content_publish` / `harvest_propose`
- Store hook configs per department in DB (reuse existing `hooks` CRUD)
- Emit `AgentEvent::HookBlocked { name, reason }` → SSE `hook_blocked` event → frontend shows inline

**Files:** `rusvel-agent/src/hooks.rs` (~120 lines), `rusvel-agent/src/lib.rs`, `rusvel-api/src/department.rs`

**Unlocks:**
- **P5 Self-Correction** (next-level-proposals) — PostToolUse hooks trigger critique pass
- **Agent Orchestration Phase 1a** — safety gates on `delegate_agent` calls
- **Hierarchical Permissions** (Feature 5 below) — hooks implement the `can_use_tool` callback
- **ADR-008 compliance** — approval gates move from job queue into real-time agent loop

---

### 2. Context Compaction (Auto-Summarization)

**Agent SDK pattern:** Automatic conversation summarization when approaching context limits. Enables work beyond 200k tokens. Paired with memory tool for critical info preservation.

**RUSVEL gap:** Chat loads last 50 messages raw. No summarization. Long sessions lose early context.

**What to build:**
- `ContextCompactor` triggers when message count > threshold (30 messages)
- Oldest N messages → LLM summary → 1 compact message replaces them
- Preserves: system prompt, last 10 messages, tool call results from current task
- Triggered in `dept_chat()` before building agent input
- Store compacted summaries in object store for audit

**Files:** `rusvel-agent/src/compaction.rs` (~100 lines), `rusvel-api/src/department.rs`

**Unlocks:**
- **Executive Brief** (GenAICircle inspiration) — brief history stays compact, older briefs auto-compacted
- **Long-running Playbooks** (agent-orchestration) — multi-step pipelines don't blow context
- **Progress Docs** (Feature 7 below) — compaction + progress docs = seamless multi-session work

---

### 3. Memory Tool (Cross-Session Persistence)

**Agent SDK pattern:** Agents have `memory` tool with `view`, `create`, `str_replace`, `delete`. Agents auto-check memory before starting. Bridges context between sessions.

**RUSVEL gap:** `rusvel-memory` has FTS5 search but agents can't use it as a tool. No auto-load into context.

**What to build:**
- 4 tools: `memory_read`, `memory_write`, `memory_search`, `memory_delete`
- Wrap `MemoryPort` (already implemented in `rusvel-memory`)
- Add "Check memory for relevant context" instruction to department system prompts
- Auto-inject top 5 recent memories into system prompt on session start

**Files:** `rusvel-engine-tools/src/memory.rs` (~100 lines), `rusvel-engine-tools/src/lib.rs`, `rusvel-app/src/main.rs`

**Unlocks:**
- **Starter Kits** (GenAICircle) — kit installation seeds memory entries ("You are configured for Indie SaaS founder...")
- **Self-Improving Knowledge Base** (GenAICircle Feature 6) — agents auto-write discoveries to memory, memory vectorized and searchable cross-department
- **Progress Docs** (Feature 7) — progress summaries stored via memory_write, loaded via memory_read
- **Agent Orchestration** — sub-agents share memory namespace with parent (same session)
- **P2 Hybrid RAG** (next-level-proposals) — memory_search enhanced by RRF fusion when P2 ships

---

### 4. Multi-Agent Handoffs → `delegate_agent` + `invoke_flow`

**Agent SDK pattern:** `handoff` (sync), `assign` (async parallel), `send_message` (direct agent-to-agent).

**RUSVEL gap:** Departments isolated. No cross-department agent communication.

**What to build:** Defined in detail in `docs/plans/agent-orchestration.md` Phase 2-4:

- **`delegate_agent` tool** (~100 lines) — spawn sub-agent with persona, prompt, tools, rules, department
  - `wait: true` = sync (parent waits), `wait: false` = async (job queue)
  - Cross-department: `department` param resolves via `DepartmentManifest` (ADR-014)
  - Recursion depth guard: max 3 levels
  - Budget scoping: sub-agent cost tracked and capped
- **`invoke_flow` tool** (~50 lines) — trigger FlowEngine DAG from within agent loop
- **`send_message` tool** — post event to another department's bus

**Files:** `rusvel-builtin-tools/src/delegate.rs`, `rusvel-builtin-tools/src/flow.rs`, `rusvel-agent/src/lib.rs` (depth guard)

**Unlocks (critical path):**
- **Playbooks** (GenAICircle Priority 1) — playbook steps use `delegate_agent` for cross-department chaining
- **Executive Brief** (GenAICircle Priority 2) — God Agent uses `delegate_agent` to query each department
- **Roundtable UI** (GenAICircle Feature 5) — multi-persona discussion via inline delegation
- **Event Triggers** (agent-orchestration Phase 4) — completion events trigger next delegation
- **Autonomous Code Pipeline** (agent-orchestration example) — Plan → Build → Test → Review → Report
- **Agent Workforce** (docs/design/agent-workforce.md) — 14 sub-agents for building RUSVEL itself

---

### 5. Hierarchical Tool Permissions

**Agent SDK pattern:** 4-level: `allowed_tools` → `permission_mode` → `can_use_tool` hook → `disallowed_tools`.

**RUSVEL gap:** `ScopedToolRegistry` — binary prefix/name filtering. No dynamic decisions.

**What to build:**
- `ToolPermissionMode` enum: `Auto` (all allowed), `Supervised` (ask for dangerous), `Locked` (only allowlisted)
- Per-tool metadata: `{ "dangerous": true, "requires_approval": true }`
- `can_use_tool` callback on `ScopedToolRegistry` — dynamic permission check via hooks (Feature 1)
- `Supervised` mode: dangerous tools emit `approval_required` SSE event, agent pauses

**Files:** `rusvel-tool/src/lib.rs`, `rusvel-core/src/domain.rs`, `rusvel-api/src/department.rs`

**Unlocks:**
- **Agent Orchestration AgentScope** — each sub-agent gets its own `permission_mode`
- **P4 Approval UI** (next-level-proposals) — permission mode drives which tools surface approval cards
- **Starter Kits** — kits configure permission_mode per department ("Freelancer" kit = supervised for outreach)

---

### 6. Verification Loops (Self-Correction)

**Agent SDK pattern:** Rules-based (lint), visual (screenshots), LLM judgment (secondary model review). Agents self-correct before marking done.

**RUSVEL gap:** Content engine has `review()` scoring, visual E2E tests exist. Not wired into agent loop.

**What to build:** Aligns with **P5 Self-Correction Loop** (next-level-proposals):
- `CritiqueStep` in `AgentRuntime` — after main agent output, invoke critique agent (Haiku, cheap)
- Per-engine evaluation dimensions:
  - Content: factual accuracy, tone match, platform fit, SEO score
  - Code: correctness, complexity, test coverage
  - Harvest: relevance score, source quality, opportunity viability
- If critique score < threshold → auto-generate Rule via `!build` pattern → append to engine rules
- Store critique results in MetricStore for trend analysis
- Emit SSE events: "reviewing..." → score → "approved" / "revising..."

**Files:** `rusvel-agent/src/verification.rs` (~200 lines), `rusvel-agent/src/critique.rs`

**Unlocks:**
- **Playbook critique steps** — each playbook step can optionally include verification
- **Agent Orchestration** — CritiqueAgent evaluates sub-agent output, auto-retries if below threshold
- **Self-improving Knowledge Base** — critique results feed back into rules, making agents better over time

---

### 7. Progress Docs (Session Continuity)

**Agent SDK pattern:** Auto-generated `progress.txt` at end of session. Next session loads it. Bridges context resets.

**What to build:**
- On chat session end (`Done` event), auto-generate progress summary via LLM
- Store via `memory_write` under `progress/{dept}/{date}` (depends on Feature 3)
- Next session auto-loads latest progress doc into system prompt
- Format: what was done, what's pending, key decisions, blockers

**Files:** `rusvel-api/src/department.rs` (post-chat hook), `rusvel-engine-tools/src/memory.rs`

**Unlocks:**
- **Executive Brief** — aggregates progress docs from all departments into daily digest
- **Multi-session Playbooks** — complex playbooks spanning multiple sessions resume seamlessly

---

### 8. Agentic Search (Dynamic Context Retrieval)

**Agent SDK pattern:** Agents search dynamically via tools instead of pre-embedding all data.

**RUSVEL gap:** RAG injected upfront into system prompt. Built-in `grep` + `glob` tools exist but RAG is pre-loaded.

**What to build:**
- Make RAG injection optional per department config
- Add instruction "search for relevant context when needed" to system prompt
- Agents use existing `grep`, `glob`, `code_search`, `memory_search` tools on-demand
- Pairs with **P1 Deferred Tool Loading** (next-level-proposals): agents discover tools via `tool_search` meta-tool

**Files:** `rusvel-api/src/department.rs` — config flag, prompt adjustment

**Unlocks:**
- **P1 Deferred Tool Loading** — 85% token reduction by not injecting all tool schemas upfront
- **Knowledge Search tool** — when P2 Hybrid RAG ships, `knowledge_search` becomes the primary search tool

---

### 9. In-Process MCP Bridge

**Agent SDK pattern:** Custom tools as in-process MCP servers. No subprocess overhead.

**RUSVEL gap:** `rusvel-mcp-client` connects via stdio subprocess.

**What to build:**
- In-process MCP handler routing `tools/call` directly to `ToolRegistry`
- Already partially there via `rusvel-mcp` server mode
- Full implementation deferred to **P6 Streamable HTTP MCP** (next-level-proposals) which adds HTTP transport

---

## Unified Implementation Order

Phasing aligned with `agent-orchestration.md` and `next-level-inspiration-2026-03-25.md`:

```
Phase A — Agent Intelligence Foundation (parallel, 1 session)
  ├── A1: PreToolUse/PostToolUse Hooks        (~120 lines)  → Feature 1
  ├── A2: Memory Tools (4 tools)              (~100 lines)  → Feature 3
  └── A3: Context Compaction                  (~100 lines)  → Feature 2

Phase B — Orchestration Primitives (parallel, 1 session)
  ├── B1: delegate_agent + invoke_flow        (~150 lines)  → Feature 4
  ├── B2: Hierarchical Permissions            (~80 lines)   → Feature 5
  └── B3: Event Trigger System                (~200 lines)  → agent-orchestration Phase 4

Phase C — Intelligence & Continuity (sequential)
  ├── C1: Verification / Self-Correction      (~200 lines)  → Feature 6 + P5
  ├── C2: Progress Docs                       (~60 lines)   → Feature 7
  └── C3: Agentic Search (prompt adjustment)  (~20 lines)   → Feature 8
```

**Total: ~1,030 lines of new code**

## Dependency Map (across ALL plan docs)

```
                    ┌─────────────────────────────────────────┐
                    │      THIS PLAN (Agent SDK Features)      │
                    └──────────────┬──────────────────────────┘
                                   │
  Phase A (Foundation)             │
  ┌─────────────┐                  │
  │ A1: Hooks   │──────────────────┼──→ Hierarchical Perms (B2)
  │ A2: Memory  │──────────────────┼──→ Progress Docs (C2)
  │ A3: Compact │──────────────────┼──→ Long-running Playbooks
  └──────┬──────┘                  │
         │                         │
  Phase B (Orchestration)          │         ┌─────────────────────────┐
  ┌──────┴──────┐                  │         │  agent-orchestration.md  │
  │ B1: delegate│──────────────────┼────────→│  Playbooks, Triggers,   │
  │ B2: perms   │                  │         │  Workflow Templates      │
  │ B3: triggers│──────────────────┼────────→│  TriggerManager          │
  └──────┬──────┘                  │         └────────────┬────────────┘
         │                         │                      │
  Phase C (Intelligence)           │                      ▼
  ┌──────┴──────┐                  │         ┌─────────────────────────┐
  │ C1: verify  │──────────────────┼────────→│  next-level-proposals   │
  │ C2: progress│                  │         │  P5 Self-Correction     │
  │ C3: search  │──────────────────┼────────→│  P1 Deferred Tools      │
  └─────────────┘                  │         │  P8 Durable Execution   │
                                   │         └────────────┬────────────┘
                                   │                      │
                                   │                      ▼
                                   │         ┌─────────────────────────┐
                                   │         │  GenAICircle Inspiration │
                                   └────────→│  Playbooks, Exec Brief, │
                                             │  Starter Kits, Leveling  │
                                             └─────────────────────────┘
```

### Critical Path

```
Hooks (A1) → delegate_agent (B1) → Playbooks (agent-orch) → Starter Kits (GenAI)
Memory (A2) → Progress Docs (C2) → Executive Brief (GenAI)
Compaction (A3) → Long-running sessions → Playbook execution
Triggers (B3) → Event-driven pipelines → Self-improving KB (GenAI)
```

## What NOT to Steal

- **CLI subprocess architecture** — Agent SDK wraps Claude Code CLI. We already replaced that with direct `AgentRuntime`.
- **Python-specific patterns** — Decorators, async generators. Rust has traits + async.
- **Client SDK message format** — Our `LlmMessage` / `Content` / `Part` types work fine.
- **Their error hierarchy** — `RusvelError` + `thiserror` is cleaner.
- **Session management** — Agent SDK uses CLI sessions. We have `rusvel-db` session store with proper persistence.

## What to Steal That's NOT in Agent SDK

These come from the broader ecosystem research in `next-level-proposals.md`:

| Feature | Source | Why |
|---------|--------|-----|
| Deferred Tool Loading (P1) | Anthropic advanced-tool-use beta | 85% token savings |
| Hybrid RAG + Reranking (P2) | Industry best practice | 48% retrieval quality boost |
| Batch API for Jobs (P3) | Claude Batch API | 50-95% cost savings on async work |
| AG-UI Protocol (P7) | Microsoft/Google agent UI standard | Ecosystem compatibility |
| Durable Execution (P8) | Temporal/Cloudflare patterns | Crash-resilient workflows |
| LLM Cost Intelligence (P12) | Multi-tier model routing | 60-70% cost reduction |

These complement the SDK features and should be implemented in parallel sprints per `next-level-proposals.md` roadmap.

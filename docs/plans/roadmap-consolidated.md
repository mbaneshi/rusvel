# RUSVEL Consolidated Roadmap

> Single source of truth. Updated 2026-03-30.
> Supersedes: sprint-agent-intelligence.md, sprint-6-pattern-extraction.md, implementation-adr-015.md (archived)

## Status: Sprint 0-1 Complete (17/60 tasks)

**Done today (this session):**
- CLI `--help` instant (no boot noise), descriptive help text
- Expandable sidebar with labels + Alt+N shortcuts
- God Agent system prompt includes all department capabilities
- Terminal page fix (was stuck on "Starting terminal...")
- Terminal resize route registered, full-width layout, Nerd Font support
- Graceful shutdown (OS thread 3s force-exit)
- Tool descriptions enhanced (WHEN TO USE, TIPS, cross-references)
- Max iterations raised to 50, configurable per-run via AgentConfig
- Error recovery: 3-strike reflection on consecutive tool failures
- Dev hot-reload workflow (cargo-watch + Vite proxy)
- 11 new API integration tests (65 total, all passing)

---

## Phase 1: Quick Wins (1-2 days each, no dependencies)

These can be done in any order. All HIGH value, SMALL effort.

| # | Task | Effort | What |
|---|------|--------|------|
| 1 | `invoke_flow` tool | 1d | Agents can trigger DAG workflows |
| 2 | Claude Code hooks | 1d | 6 hooks in `.claude/hooks/` (quality gate, auto-format, session save) |
| 3 | Tool permissions | 2d | `PermissionMode` enum per-dept (auto/supervised/locked) |
| 4 | MiniJinja expressions | 2d | `{{ inputs.field }}` templates in flow node parameters |
| 5 | Cost tracking domain | 2d | `CostEvent` struct, per-operation recording in MetricStore |
| 6 | Cost API + dashboard | 2d | `/api/analytics/costs`, spend-by-dept/model charts |

## Phase 2: Core Enablers (unlock everything else)

| # | Task | Effort | Unlocks |
|---|------|--------|---------|
| 7 | `delegate_agent` tool | 2d | Playbooks, Executive Brief, multi-agent workflows |
| 8 | Event triggers | 2d | Auto-start agents/flows on event patterns |
| 9 | PreToolUse/PostToolUse hooks | 3d | Quality gates, auto-formatting, approval injection |
| 10 | ChannelPort expansion | 2d | Richer messaging trait (ADR-016) |
| 11 | ChannelRouter | 2d | Pattern-based dept→channel routing |
| 12 | Context compaction improvements | 2d | Token-based (not message count), preserve pinned messages |
| 13 | Memory tools | 2d | `memory_read/write/search/delete` as agent tools |

## Phase 3: Platform Features (the big ones)

| # | Task | Effort | Depends |
|---|------|--------|---------|
| 14 | Self-correction / Verification chain | 4d | Wire existing VerificationChain into agent loop |
| 15 | Flow node Tier 1 | 3d | LoopNode, DelayNode, HttpRequestNode, ToolCallNode |
| 16 | Discord adapter | 3d | #10 ChannelPort |
| 17 | Slack adapter | 3d | #10 ChannelPort |
| 18 | Email adapter (SMTP) | 2d | #10 ChannelPort |
| 19 | Hybrid RAG (FTS5 + LanceDB) | 3d | Better knowledge search |
| 20 | Playbooks | 5d | #7 delegate_agent |
| 21 | Executive Brief enhanced | 3d | #7 delegate_agent, #5 cost tracking |

## Phase 4: Production Hardening

| # | Task | Effort | Depends |
|---|------|--------|---------|
| 22 | Durable execution (checkpoint/resume/retry) | 5d | Production-grade flows |
| 23 | Flow node Tier 2 | 3d | SwitchNode, MergeNode, SubFlowNode, NotifyNode |
| 24 | Session persistence + `/learn` | 3d | #2 hooks |
| 25 | Continuous learning pipeline | 3d | #24 session persistence |
| 26 | Inbound channel routing | 3d | #11 ChannelRouter |
| 27 | Credential encryption | 3d | AES-256 at rest |
| 28 | Flow versioning + templates | 4d | Version counter, starter flows |
| 29 | Design tokens + variants | 3d | CSS custom properties, tailwind-variants |
| 30 | Batch API | 3d | 50% discount on async LLM jobs |

## Backlog (unscheduled)

- AG-UI Protocol mapping (4d)
- Streamable HTTP MCP (5d)
- CDP Browser Bridge (10d)
- Starter Kits / Capability Marketplace (3d)
- Self-Improving Knowledge Base (3d)
- Desktop distribution via Tauri (10d)

---

## Reference Docs (kept)

| Doc | Purpose |
|-----|---------|
| `docs/proposals/reference-repos-minibook.md` | Strategic analysis of 6 reference repos |
| `docs/proposals/agent-intelligence-minibook.md` | Agent gap analysis (partially implemented) |
| `docs/design/implementation-design-v3.md` | Code samples for flow nodes, channels, cost tracking |
| `docs/design/decisions.md` | ADR-001 through ADR-019 |
| `docs/plans/sprints.md` | Original sprint breakdown (S0-S6) |
| `docs/plans/pattern-extraction-design.md` | Code samples for session persistence, learning |

## Archived Docs (superseded)

Moved to `docs/archive/`:
- agent-runtime-enhancement.md
- agent-intelligence-implementation.md
- pattern-extraction-from-repos.md
- sprint-agent-intelligence.md
- sprint-6-pattern-extraction.md
- implementation-adr-015.md
- comprehensive-refactoring-plan.md

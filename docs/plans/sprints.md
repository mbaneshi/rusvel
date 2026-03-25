# RUSVEL — Consolidated Sprint Plan

> Single source of truth. All other plan docs are reference material.
> Date: 2026-03-25 | Status: Active | Updated: 2026-03-26

---

## Where We Are (verified 2026-03-26)

**Build:** `cargo build` clean (0 errors). `cargo test` 0 failures across 98 test suites. **49 crates**, ~43k lines Rust, 185 source files.

**Git:** Clean. 15 uncommitted files (docs/frontend in-progress).

**What shipped since original plan:**

| Commit | What | Sprint Task |
|--------|------|-------------|
| `7fadf75` | AgentRuntime wired to chat, 21 tools, ScopedToolRegistry, ToolCallCard + ApprovalCard | Pre-sprint (Phases 1-4) |
| `452e730` | LLM text streaming, DEV.to real adapter | Pre-sprint (Phases 5-6) |
| `7108933` | ADR-014 DepartmentApp architecture | Sprint 1 #1-5 |
| `0bb4dfe` | dept-content complete (tools, events, jobs) | Sprint 1 #1 |
| `94a7eb1` | dept-forge complete (manifest, mission, safety) | Sprint 1 #2 |
| `119af14` | Remove EngineKind, use department ID strings | Sprint 1 #6 |
| `4c387f1` | Deferred tool loading (`tool_search` meta-tool) | Sprint 1 #7 |
| `2ba3872` | ModelTier routing + MetricStore cost tracking | Sprint 1 #8 |
| `17e82ac` | Approval queue UI, sidebar badge, approve/reject API | Sprint 1 #9 |
| `856b75d` | TerminalPort + rusvel-terminal Phase 1 | Sprint 1 #10 |
| `087e414` | Frontend manifest-aligned departments (nav + tabs) | Sprint 2 #15 |
| `b1df03f` | Flow REST client + /flows page | Sprint 2 (partial) |

**Summary: Sprint 0 done. Sprint 1 done. Sprint 2 partially done.**

---

## What's Left

### Sprint 2 Remaining (~8 days)

> Theme: Agent intelligence + Terminal web bridge

| # | Task | Effort | Status | Depends On |
|---|------|--------|--------|------------|
| 11 | **Context Compaction** — auto-summarize at 30 messages, keep recent 10 | 2d | Not started | — |
| 12 | **Memory Tools** — `memory_read/write/search/delete` as agent tools, auto-inject top 5 | 2d | Not started | — |
| 13 | **Batch API** — `submit_batch()/poll_batch()` on LlmPort, 50% discount on async jobs | 3d | Not started | #8 ✅ (done) |
| 14 | **Hybrid RAG** — fuse FTS5 + LanceDB via RRF, rerank top-N with Haiku | 3d | Not started | — |
| ~~15~~ | ~~Frontend manifest alignment~~ | — | ✅ Done | — |
| 16 | **Terminal Phase 2: Web Bridge** — WebSocket route, xterm.js, `/terminal` page | 3d | Not started | #10 ✅ (done) |
| 17 | **Terminal Phase 3: Dept Integration** — "Terminal" tab in dept panel, lazy panes | 2d | Not started | #10 ✅, #15 ✅ |

**Parallelizable:** 11+12 parallel, 13+14 parallel, 16+17 sequential. **~8 working days.**

---

### Sprint 3: Orchestration + Agent Visibility (~12 days)

> Theme: Agents controlling agents — and you can watch them.

| # | Task | Effort | Status | Depends On |
|---|------|--------|--------|------------|
| 18 | **`delegate_agent` tool** — spawn sub-agent with persona, tools, budget cap, max depth 3 | 2d | Not started | — |
| 19 | **`invoke_flow` tool** — agents trigger FlowEngine DAGs, get results | 1d | Not started | — |
| 20 | **PreToolUse/PostToolUse hooks** — `HookPoint` enum, `ToolHook` trait, wire into agent loop | 3d | Not started | ADR-014 ✅ |
| 21 | **Tool permissions** — `ToolPermissionMode` per-dept (auto/supervised/locked) | 2d | Not started | ADR-014 ✅ |
| 22 | **Event triggers** — `EventTrigger` subscribes to patterns, auto-starts agents/flows | 2d | Not started | #18, #19 |
| 23 | **Self-correction loops** — `VerificationStep` trait, per-tool critique, auto-fix rules | 4d | Not started | #8 ✅ (done) |
| 24 | **Terminal Phase 4: Agent Visibility** — `DelegationTerminal.svelte`, `terminal_open/watch` tools | 3d | Not started | #16, #18 |

**Sprint 3 total:** ~12 days (18-21 parallel, 22-23 after deps, 24 after 16+18)

---

### Sprint 4: Platform & Interop (~12 days)

> Theme: Crash-resilient workflows, ecosystem compatibility, full pipeline visibility.

| # | Task | Effort | Status | Depends On |
|---|------|--------|--------|------------|
| 25 | **Durable Execution** — checkpoint/resume for FlowEngine, retry per node | 5d | Not started | — |
| 26 | **Playbooks** — predefined multi-step JSON pipelines, UI at `/playbooks`, run/history | 5d | Not started | #18, #19, #22 |
| 27 | **AG-UI Protocol** — map SSE events to AG-UI schema, add RUN_STARTED/STATE_DELTA | 4d | Not started | — |
| 28 | **Terminal Phase 5: Flow/Playbook Visibility** — node-per-pane in DAG execution | 3d | Not started | #16, #25, #26 |

**Sprint 4 total:** ~12 days (25+27 parallel, 26 after deps, 28 after 25+26)

---

### Sprint 5: Product Features (~12 days)

> Theme: Framework → Product. Features users see and love.

| # | Task | Effort | Status | Depends On |
|---|------|--------|--------|------------|
| 29 | **Executive Brief** — enhanced `forge mission today` + delegate per dept | 3d | Not started | #18 |
| 30 | **Starter Kits** — pre-built dept bundles via `!build` + Capability Engine | 3d | Not started | ADR-014 ✅ |
| 31 | **Self-Improving KB** — auto-index engine outputs, cross-dept insights | 3d | Not started | #14 |
| 32 | **Streamable HTTP MCP** — Axum handlers, session mgmt, OAuth, keep stdio fallback | 5d | Not started | — |
| 33 | **Terminal Phase 6-7: CDP + TUI** — browser panes, ratatui panel | 4d | Not started | #16, CDP bridge |

**Sprint 5 total:** ~12 days (29-31 parallel, 32 independent, 33 after CDP)

---

## Backlog (unscheduled)

| Task | Effort | Depends On | Source Doc |
|------|--------|------------|-----------|
| CDP Browser Bridge (phases 1-6) | 10d | ADR-014 ✅ | cdp-browser-bridge.md |
| AI SDK 6 Frontend | 8d | AG-UI (#27) | next-level-proposals.md |
| Leveling/Progression system | 4d | ADR-014 ✅ | next-level-inspiration.md |
| Roundtable/Strategy Review UI | 5d | #18 (delegate_agent) | next-level-inspiration.md |
| Agent Workforce dogfooding | Ongoing | #18, ADR-014 ✅ | agent-workforce.md |
| A2UI Component Registry | 4d | AG-UI (#27) | next-level-proposals.md |
| Inter-department messaging | 3d | ADR-014 ✅ | agent-orchestration.md |
| In-Process MCP Bridge | 3d | — | agent-sdk-features.md |
| Progress Docs (session continuity) | 2d | #12 (memory tools) | agent-sdk-features.md |
| Agentic Search (prompt adjustment) | 1d | #7 ✅ (deferred tools) | agent-sdk-features.md |

---

## Completed Work Log

### Pre-Sprint (Phases 1-6, session 2026-03-25)
- ✅ AgentRuntime::run_streaming() with AgentEvent channel
- ✅ 9 built-in tools registered (file ops, shell, git)
- ✅ 12 engine tools (harvest 5, content 5, code 2)
- ✅ ScopedToolRegistry for per-department tool filtering
- ✅ Chat handlers rewired from ClaudeCliStreamer → AgentRuntime
- ✅ LLM text streaming (LlmStreamEvent + stream() on LlmPort)
- ✅ Frontend ToolCallCard + ApprovalCard components
- ✅ DEV.to real HTTP adapter (LinkedIn + Twitter already existed)

### Sprint 0: Unbreak ✅
- ✅ Build compiles clean
- ✅ All tests pass
- ✅ `searchable` field added to all ToolDefinition sites

### Sprint 1: Foundation ✅
- ✅ **#1** dept-content DepartmentApp (routes, tools, personas, skills, handlers, events, jobs)
- ✅ **#2** dept-forge DepartmentApp (mission tools, 10 personas, safety)
- ✅ **#3** Boot sequence — main.rs iterates installed_departments()
- ✅ **#4** Code, Harvest, Flow → dept-* crates
- ✅ **#5** 8 stub depts → dept-* crates (all 12 departments now DepartmentApp)
- ✅ **#6** EngineKind removed, string IDs everywhere
- ✅ **#7** Deferred Tool Loading — `tool_search` meta-tool, 85% token savings
- ✅ **#8** LLM Cost Intelligence — ModelTier routing + CostTracker
- ✅ **#9** Approval UI — ApprovalQueue, inline chat approve/reject, sidebar badge
- ✅ **#10** Terminal Phase 1 — TerminalPort trait, rusvel-terminal crate

### Sprint 2: Partial ✅
- ✅ **#15** Frontend manifest alignment — dept manifest drives UI tabs/routes
- ✅ Flows page — REST client + /flows route

---

## Terminal Multiplexer Threading

```
Sprint 0   Sprint 1          Sprint 2            Sprint 3              Sprint 4             Sprint 5
--------   ----------        ----------          ----------            ----------           ----------
✅ done    ✅ TerminalPort   ⬜ Web Bridge       ⬜ Agent Visibility   ⬜ Flow Panes        ⬜ CDP + TUI
           ✅ PTY core       ⬜ xterm.js         ⬜ DelegationTerm    ⬜ Playbook dash     ⬜ Browser panes
           ✅ TerminalMgr    ⬜ /terminal route  ⬜ terminal_open     ⬜ Checkpoint UI     ⬜ ratatui panel
                             ⬜ Dept tab         ⬜ /runs/:id/panes   ⬜ /flows/:id/panes
```

---

## Critical Path (updated)

```
✅ DONE                     Sprint 2              Sprint 3              Sprint 4         Sprint 5
---------                   ---------             ---------             ---------        ---------
✅ ADR-014 ──────────────────────────────────→ Hooks + Permissions
                                                    |
✅ Cost routing ──────────→ Batch API               |
                                                    +──→ Self-correction
                                                    |
✅ Terminal PTY ──────────→ Terminal Web ───────→ Terminal Agent Vis ──→ Terminal Flow ──→ Terminal CDP
                                                    |
                            Memory + RAG            |
                                                    |
                            Compaction          delegate_agent ────────→ Playbooks ──→ Executive Brief
                                                    |                       |
                                                Event triggers              |
                                                    |                  Durable Exec
                                                invoke_flow
                                                                       AG-UI ──────────→ (AI SDK 6)
```

**Two remaining unlock points:**
1. **delegate_agent (Sprint 3 #18)** — unlocks playbooks, executive brief, roundtable, workforce dogfooding
2. **Terminal Web Bridge (Sprint 2 #16)** — unlocks all agent visibility features

---

## Metrics

| Sprint | Tasks | Effort | Status | Cumulative |
|--------|-------|--------|--------|------------|
| 0 | 5 | ~1d | ✅ Done | 1d |
| 1 | 10 | ~10d | ✅ Done | 11d |
| 2 | 6 remaining | ~8d | **In Progress** | 19d |
| 3 | 7 | ~12d | Planned | 31d |
| 4 | 4 | ~12d | Planned | 43d |
| 5 | 5 | ~12d | Planned | 55d |
| **Total** | **32 tasks remaining** | **~44 working days** | | **~9 weeks** |

Backlog: 10 items, ~45 additional days if all scheduled.

**Progress: 17/49 tasks done (35%). ~21 working days completed. ~44 remaining.**

---

## Codebase Snapshot (2026-03-26)

| Metric | Value |
|--------|-------|
| Crates | 49 (16 foundation + 13 engines + 15 dept-* + 5 surfaces) |
| Rust source files | 185 |
| Rust lines | ~43,276 |
| API handler functions | ~115 |
| Test suites passing | 98 (0 failures) |
| Department crates | 15 (all 12 + dept-code, dept-flow, dept-harvest) |
| Registered tools | 21+ (9 built-in + 12 engine + tool_search) |
| Frontend routes | 16+ |

---

## Reference Documents

| Doc | What It Contains | Relevant Sprints |
|-----|-----------------|-----------------|
| `design/department-as-app.md` | DepartmentApp trait, manifest types, migration steps | ✅ Sprint 1 (done) |
| `plans/native-terminal-multiplexer.md` | TerminalPort, PTY, xterm.js, pane sources | Sprints 2-5 |
| `plans/agent-sdk-features.md` | 9 features from Anthropic SDK + cross-references | Sprints 2-3 |
| `plans/agent-orchestration.md` | delegate_agent, invoke_flow, triggers, templates | Sprint 3-4 |
| `plans/next-level-proposals.md` | 12 proposals (P1-P12), dependency graph | All |
| `plans/next-level-inspiration-2026-03-25.md` | GenAICircle-inspired product features | Sprint 5 + Backlog |
| `plans/cdp-browser-bridge.md` | BrowserPort, CDP, platform extractors | Sprint 5 + Backlog |
| `design/agent-workforce.md` | 14 builder agents for self-building | Backlog |
| `plans/flow-engine.md` | DAG executor design, node types, triggers | Sprint 4 |
| `design/decisions.md` | 13 ADRs + ADR-014 | Reference |
| `design/architecture-v2.md` | Hexagonal architecture, port traits | Reference |

---

## Superseded Documents

- `plans/sprint-current.md` → superseded by this document
- `plans/phase-0-foundation.md` → superseded by `phase-0-foundation-v2.md`
- `plans/roadmap.md` → superseded by `roadmap-v2.md`
- `plans/refactor-2026-03-23.md` → work completed
- `plans/ui-overhaul-2026-03-23.md` → work completed
- `plans/ui-enhancements-2026-03-23.md` → work completed
- `plans/refactor-and-enhance-2026-03-23.md` → work completed

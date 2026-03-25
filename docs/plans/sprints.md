# RUSVEL — Consolidated Sprint Plan

> Single source of truth. All other plan docs are reference material.
> Date: 2026-03-25 | Status: Active | Updated: 2026-03-25 (Phase 0 + Sprint 1 #7)

---

## Where We Are (verified 2026-03-25)

**Phase 0 complete.** ~40 workspace members (including 12 `dept-*` crates), 79+ API routes, 3-tier CLI, embedded SvelteKit frontend.

**Build status: green.** `cargo check --workspace` succeeds on `main` + deferred-tool WIP.

**ADR-014 (Department-as-App):**
- `rusvel-core/src/department/` — `DepartmentApp`, `DepartmentManifest`, `RegistrationContext` — **shipped**
- `crates/rusvel-app/src/boot.rs` + `main.rs` — `installed_departments()` + `boot_departments()` — **shipped**
- All **12** `crates/dept-*` — present in workspace; **partial** depth (content/forge richer; stubs wrap engines)
- **Next hard cut:** Task **#6** — remove `EngineKind`, string IDs everywhere (wide refactor)

**Sprint 1 Track B — Task #7 (Deferred Tool Loading):** `tool_search` registered in `rusvel-builtin-tools`; `AgentRuntime` filters `searchable` tools from initial prompt and injects tools after `tool_search` — **landing with next commit**.

**Next focus:** Track A **#1–#2** polish (full manifest + routes/tools), **#6**, then **#8–#10** per table below.

---

## Priority Framework

1. **Must — Architectural blockers** (ADR-014 completion, `EngineKind` removal)
2. **Should — High-ROI features** (cost routing, approval UI, terminal core)
3. **Could — Ambitious features** (valuable but can wait — see Backlog)

---

## Sprint 0: Unbreak — **complete**

> Historical: `searchable` on `ToolDefinition`, call sites updated, workspace green.

| # | Task | Effort | Status |
|---|------|--------|--------|
| 0a | Add `searchable` to `ToolDefinition` + all call sites | — | **Done** |
| 0b | API / compile fixes | — | **Done** |
| 0c | `cargo check --workspace` | — | **Done** |
| 0d | `cargo test` (spot + CI) | — | **Done** |
| 0e | Commit department + tool plumbing | — | **Done** |

---

## Sprint 1: Foundation (2 weeks)

> Theme: Complete the architectural shift + ship independent quick wins in parallel.

### Track A: ADR-014 Completion (blocking)

| # | Task | Effort | Status |
|---|------|--------|--------|
| 1 | Complete dept-content impl (routes, tools, personas, skills, handlers, events, jobs) | 2d | Partial |
| 2 | Complete dept-forge impl (mission tools, 10 personas, forge routes, safety) | 2d | Partial |
| 3 | Wire boot sequence — `main.rs` iterates `installed_departments()`, calls `manifest()` then `register()` | 1d | **Done** |
| 4 | Convert Code, Harvest, Flow engines → dept-* crates | 3d | **Done** (crates + boot) |
| 5 | Convert 8 stub depts (finance, growth, distro, legal, support, infra, product, gtm) → dept-* crates | 1d | **Done** (wrappers) |
| 6 | Remove `EngineKind` enum, use string IDs everywhere | 1d | Not started |

**Exit criteria:** All 12 departments boot via manifest. No hardcoded department wiring in main.rs. `EngineKind` removed.

### Track B: Quick Wins (parallel, no ADR-014 dependency)

| # | Task | Effort | Impact |
|---|------|--------|--------|
| 7 | **Complete Deferred Tool Loading** — `tool_search` meta-tool, split always/searchable in AgentRuntime | 1d | **Done** (register + filter + inject) |
| 8 | **LLM Cost Intelligence** — ModelTier routing (Haiku/Sonnet/Opus) + CostTracker in MetricStore | 3d | 60% cost savings |
| 9 | **Approval UI** — `ApprovalQueue.svelte`, inline approve/reject in chat, sidebar badge | 3d | Biggest UX gap |
| 10 | **Terminal Multiplexer Phase 1** — `TerminalPort` trait, `rusvel-terminal` crate, `TerminalManager` with `portable-pty` | 3d | Foundation for all observability |

**Why terminal in Sprint 1:** Terminal is a platform service. Phase 1 (PTY core) has zero dependencies on ADR-014 — it only needs `EventPort` and `StoragePort` which already exist. Getting the PTY layer working early means Sprint 2 can wire the web bridge, and Sprint 3 can show agent delegation live.

**Sprint 1 total:** ~10 days Track A + ~10 days Track B (parallel) = **~10 working days**

---

## Sprint 2: Agent Intelligence + Terminal Web (2 weeks)

> Theme: Make agents smarter, cheaper, persistent. Terminal visible in browser.

| # | Task | Effort | Depends On |
|---|------|--------|------------|
| 11 | **Context Compaction** — auto-summarize at 30 messages, keep recent 10 | 2d | — |
| 12 | **Memory Tools** — `memory_read/write/search/delete` as agent tools, auto-inject top 5 | 2d | — |
| 13 | **Batch API** — `submit_batch()/poll_batch()` on LlmPort, 50% discount on async jobs | 3d | #8 (cost routing) |
| 14 | **Hybrid RAG** — fuse FTS5 + LanceDB via RRF, rerank top-N with Haiku | 3d | — |
| 15 | **Frontend manifest alignment** — dept manifest drives UI tabs/routes, remove hardcoded dept UI | 2d | Sprint 1 ADR-014 |
| 16 | **Terminal Phase 2: Web Bridge** — WebSocket route, xterm.js component, `/terminal` page, paneforge layout | 3d | #10 (terminal core) |
| 17 | **Terminal Phase 3: Department Integration** — "Terminal" tab in dept panel, lazy pane creation, `TerminalContribution` in manifests | 2d | #10, #15 |

**Sprint 2 total:** ~12 days (11-14 + 16 parallelizable, 15 + 17 sequential after ADR-014)

---

## Sprint 3: Orchestration + Agent Visibility + Browser Foundation (2 weeks)

> Theme: Agents controlling agents, watching them work, and reaching the real world.

| # | Task | Effort | Depends On |
|---|------|--------|------------|
| 18 | **`delegate_agent` tool** — spawn sub-agent with persona, tools, budget cap, max depth 3 | 2d | — |
| 19 | **`invoke_flow` tool** — agents trigger FlowEngine DAGs, get results | 1d | — |
| 20 | **PreToolUse/PostToolUse hooks** — `HookPoint` enum, `ToolHook` trait, wire into agent loop | 3d | Sprint 1 ADR-014 |
| 21 | **Tool permissions** — `ToolPermissionMode` per-dept (auto/supervised/locked) | 2d | Sprint 1 ADR-014 |
| 22 | **Event triggers** — `EventTrigger` subscribes to patterns, auto-starts agents/flows | 2d | #18, #19 |
| 23 | **Self-correction loops** — `VerificationStep` trait, per-tool critique, auto-fix rules | 4d | #8 (cost routing) |
| 24 | **Terminal Phase 4: Agent Visibility** — `PaneSource::Delegation`, `terminal_open/watch` tools, `DelegationTerminal.svelte`, `/api/terminal/runs/:id/panes` | 3d | #16 (web bridge), #18 (delegate_agent) |
| 25 | **BrowserPort + rusvel-cdp** — `BrowserPort` trait (port #20) in rusvel-core, CDP WebSocket adapter (`rusvel-cdp` crate <1500 lines via `tokio-tungstenite`), tab discovery, network interception, Passive mode only. Follows `cdp-browser-bridge.md` Phase 1. Browser tools guarded by tool permissions (#21). | 3d | Sprint 1 ADR-014, #21 (tool permissions) |

**This is the big unlock.** After Sprint 3: agents delegate to sub-agents, each sub-agent runs in a visible terminal pane, hooks guard dangerous calls, event triggers chain departments, CDP captures browser data passively, and the user watches it all happen live.

**Sprint 3 total:** ~15 days (18-21 + 24 + 25 parallel, 22-23 sequential after deps)

---

## Sprint 4: Platform & Interop (2 weeks)

> Theme: Crash-resilient workflows, ecosystem compatibility, full pipeline visibility.

| # | Task | Effort | Depends On |
|---|------|--------|------------|
| 26 | **Durable Execution** — checkpoint/resume for FlowEngine, retry per node | 5d | — |
| 27 | **Playbooks** — predefined multi-step JSON pipelines, UI at `/playbooks`, run/history | 5d | #18, #19, #22 |
| 28 | **AG-UI Protocol** — map SSE events to AG-UI schema, add RUN_STARTED/STATE_DELTA | 4d | — |
| 29 | **Terminal Phase 5: Flow/Playbook Visibility** — `PaneSource::FlowNode/PlaybookStep`, node-per-pane in DAG execution, checkpoint resume in terminal UI | 3d | #16, #26, #27 |
| 30 | **Claude Computer Use + Browser Tools** — `computer_20250124` tool type in `ClaudeProvider` (beta header, screenshot action handling, base64 image in tool results), `browser_observe/search/act` built-in tools wrapping BrowserPort, vision fallback when CDP selectors fail. Hybrid strategy: CDP primary (fast/cheap), computer use fallback (flexible). `BrowsingMode::Vision` added alongside Passive/Assisted/Autonomous. AG-UI streams screenshots to frontend. | 3d | #25 (BrowserPort), #28 (AG-UI) |

**After Sprint 4:** Playbooks execute as visible DAGs with each step in its own terminal pane. Flows survive crashes. AG-UI events stream to any compatible client. Agents can take screenshots and interact with unknown UIs via Claude computer use as fallback to CDP.

**Sprint 4 total:** ~16 days (26 + 28 + 30 parallel, 27 after deps, 29 after 26 + 27)

---

## Sprint 5: Product Features (2 weeks)

> Theme: Framework → Product. Features users see and love.

| # | Task | Effort | Depends On |
|---|------|--------|------------|
| 29 | **Executive Brief** — enhanced `forge mission today` + delegate per dept, terminal panes show each dept query | 3d | #18 |
| 30 | **Starter Kits** — pre-built dept bundles via `!build` + Capability Engine | 3d | Sprint 1 ADR-014 |
| 31 | **Self-Improving KB** — auto-index engine outputs, cross-dept insights | 3d | #14 (hybrid RAG) |
| 32 | **Streamable HTTP MCP** — Axum handlers, session mgmt, OAuth, keep stdio fallback | 5d | — |
| 33 | **Terminal Phase 6-7: CDP Browser Panes + TUI Surface** — `PaneSource::Browser`, read-only capture log panes, ratatui terminal panel | 4d | #16, CDP bridge |

**Sprint 5 total:** ~14 days (29-31 parallel, 32 independent, 33 after CDP)

---

## Backlog (unscheduled)

| Task | Effort | Depends On | Source Doc |
|------|--------|------------|-----------|
| CDP Browser Bridge (phases 1-6) | 10d | Sprint 1 ADR-014 | cdp-browser-bridge.md |
| AI SDK 6 Frontend | 8d | AG-UI (#27) | next-level-proposals.md |
| Leveling/Progression system | 4d | Sprint 1 ADR-014 | next-level-inspiration.md |
| Roundtable/Strategy Review UI | 5d | #18 (delegate_agent) | next-level-inspiration.md |
| Agent Workforce dogfooding | Ongoing | #18, Sprint 1 ADR-014 | agent-workforce.md |
| A2UI Component Registry | 4d | AG-UI (#27) | next-level-proposals.md |
| Inter-department messaging | 3d | Sprint 1 ADR-014 | agent-orchestration.md |
| In-Process MCP Bridge | 3d | — | agent-sdk-features.md |

---

## Terminal Multiplexer Threading (across sprints)

The terminal is not a single sprint item — it threads through Sprints 1-5 as a platform service that gains capabilities with each orchestration feature:

```
Sprint 0   Sprint 1          Sprint 2            Sprint 3              Sprint 4             Sprint 5
--------   ----------        ----------          ----------            ----------           ----------
Fix build  TerminalPort      Web Bridge          Agent Visibility      Flow Panes           CDP + TUI
           + PTY core        + xterm.js          + DelegationTerminal  + Playbook dashboard  + Browser panes
           + TerminalManager + /terminal route   + terminal_open tool  + Checkpoint UI        + ratatui panel
                             + Dept tab          + /runs/:id/panes    + /flows/:id/panes
```

Each phase is 2-3 days, integrated with the sprint's theme. No separate "terminal sprint" needed.

---

## Critical Path

```
Sprint 0        Sprint 1                Sprint 2              Sprint 3              Sprint 4         Sprint 5
--------        ---------               ---------             ---------             ---------        ---------
Fix build --> ADR-014 ----------------> Frontend align --> Hooks + Permissions
                                                              |
              Cost routing ----------> Batch API              |
                                                              +---> Self-correction
                                                              |
              Terminal PTY ----------> Terminal Web -----> Terminal Agent Vis --> Terminal Flow --> Terminal CDP
                                                              |
                                       Memory + RAG           |
                                                              |
                                       Compaction            delegate_agent -------> Playbooks --> Executive Brief
                                                              |                         |
                                                         Event triggers                 |
                                                              |                    Durable Exec
                                                         invoke_flow
                                                                                   AG-UI ----------> (AI SDK 6)
```

**Three unlock points:**
1. **Build fix (Sprint 0)** — nothing works until compilation is green
2. **ADR-014 (Sprint 1)** — unlocks hooks, permissions, dept-scoped everything, manifest-driven UI
3. **delegate_agent (Sprint 3)** — unlocks playbooks, executive brief, workforce dogfooding, roundtable, and terminal agent visibility

---

## Reference Documents

| Doc | What It Contains | Relevant Sprints |
|-----|-----------------|-----------------|
| `design/department-as-app.md` | DepartmentApp trait, manifest types, migration steps | Sprint 1 |
| `plans/native-terminal-multiplexer.md` | TerminalPort, PTY, xterm.js, pane sources, integration map | Sprints 1-5 |
| `plans/agent-sdk-features.md` | 9 features from Anthropic SDK | Sprints 2-3 |
| `plans/agent-orchestration.md` | delegate_agent, invoke_flow, triggers, templates | Sprint 3-4 |
| `plans/next-level-proposals.md` | 12 proposals (P1-P12), dependency graph, ecosystem signals | All |
| `plans/next-level-inspiration-2026-03-25.md` | GenAICircle-inspired product features | Sprint 5 + Backlog |
| `plans/cdp-browser-bridge.md` | BrowserPort, CDP, platform extractors | Sprint 5 + Backlog |
| `design/agent-workforce.md` | 14 builder agents for self-building | Backlog |
| `plans/flow-engine.md` | DAG executor design, node types, triggers | Sprint 4 |
| `design/decisions.md` | 13 ADRs + ADR-014 | Reference |
| `design/architecture-v2.md` | Hexagonal architecture, port traits | Reference |
| `plans/roadmap-v2.md` | 5-phase high-level roadmap | Reference |

---

## Superseded Documents

- `plans/phase-0-foundation.md` → superseded by `phase-0-foundation-v2.md`
- `plans/roadmap.md` → superseded by `roadmap-v2.md`
- `plans/sprint-current.md` → superseded by this document
- `plans/refactor-2026-03-23.md` → work completed
- `plans/ui-overhaul-2026-03-23.md` → work completed
- `plans/ui-enhancements-2026-03-23.md` → work completed
- `plans/refactor-and-enhance-2026-03-23.md` → work completed
- `design/architecture.md` → superseded by `architecture-v2.md`
- `design/capability-gaps-and-wiring.md` → gaps filled
- `design/config-hierarchy-v3.md` → integrated into ADR-014
- `design/department-scaling-proposal.md` → superseded by ADR-014
- `design/browser-port-proposal.md` → superseded by `cdp-browser-bridge.md`

---

## Metrics

| Sprint | Tasks | Effort (days) | Cumulative | Terminal Phase |
|--------|-------|---------------|------------|----------------|
| 0 | 5 | ~1 | 1 | — |
| 1 | 10 | ~10 (parallel tracks) | 11 | Phase 1: PTY core |
| 2 | 7 | ~12 | 23 | Phase 2-3: Web + Dept |
| 3 | 7 | ~14 | 37 | Phase 4: Agent visibility |
| 4 | 4 | ~14 | 51 | Phase 5: Flow panes |
| 5 | 5 | ~14 | 65 | Phase 6-7: CDP + TUI |
| **Total** | **38 tasks** | **~65 working days** | **~13 weeks** | **Full multiplexer** |

Backlog: 8 items, ~40 additional days if all scheduled.

# RUSVEL Next-Level Proposals — Research & Reasoning

> Ecosystem research, tradeoff analysis, and detailed rationale for all enhancements.
> Date: 2026-03-25 | Revision: 3
> **Execution plan:** [`sprints.md`](sprints.md) is the single source of truth for sequencing and task tracking.
>
> **2026-03-28:** ADR-014 (`DepartmentApp`, string IDs, **`EngineKind` removed**) is **shipped**. Workstream A tables below retain useful ideas but **migration steps 4–6 are done** — verify any task against [`../status/current-state.md`](../status/current-state.md) before acting.

---

## Executive Summary

This document provides the **research backing and detailed reasoning** for RUSVEL's enhancement roadmap. It draws from 12 plan/design documents, 3 new crates in progress, and extensive ecosystem research (2025-2026).

**Companion docs:**
- [`sprints.md`](sprints.md) — Authoritative 6-sprint execution plan (33 tasks, ~78 working days, 19 backlog items)
- [`a2ui-department-apps.md`](a2ui-department-apps.md) — AG-UI + A2UI generative UI, capability catalog, self-extension loop
- [`claude-ecosystem-integration.md`](claude-ecosystem-integration.md) — Inter-dept messaging, context health, TUI activity, git-aware code
- [`machine-awareness-fs-integration.md`](machine-awareness-fs-integration.md) — `fs` tool as machine sensory system (MachinePort)

**The key insight:** ADR-014 (Department-as-App) is the foundational refactor. It changes how departments register routes, tools, personas, skills, and rules. Nearly every other proposal either **depends on it** or **becomes easier after it**. It must go first.

**Three unlock points:** ADR-014 (Sprint 1) → delegate_agent (Sprint 3) → AG-UI + A2UI (Sprint 4-6)

---

## Dependency Graph

```
ADR-014 (Department-as-App) ─────────────────────────────────────────────┐
  │ unlocks                                                               │
  ├── Agent SDK Hooks (PreToolUse/PostToolUse) ─── per-dept hook config   │
  ├── Hierarchical Tool Permissions ─── per-dept permission modes         │
  ├── Dept-Scoped Tools ─── manifest declares tool contributions          │
  ├── Playbooks + Starter Kits ─── manifest declares playbook bundles     │
  │                                                                       │
  ├── delegate_agent ─────┐                                               │
  │                        ├── Event Triggers ─── reactive orchestration   │
  │   invoke_flow ────────┘    │                                          │
  │                             ├── Workflow Templates                     │
  │                             └── Executive Brief (cross-dept queries)   │
  │                                                                       │
  ├── CDP Browser Bridge ─── dept-harvest manifest declares browser tools  │
  └── Terminal Multiplexer ─── dept-scoped terminal windows               │
                                                                          │
Independent (no ADR-014 dependency):                                      │
  ├── P1  Deferred Tool Loading                                           │
  ├── P2  Hybrid RAG + Reranking                                          │
  ├── P3  Batch API for Jobs                                              │
  ├── P4  Approval Workflow UI                                            │
  ├── P5  Self-Correction Loop ─── needs P12 (model routing)              │
  ├── P6  Streamable HTTP MCP                                             │
  ├── P7  AG-UI Protocol ─── enables P10 (AI SDK 6)                       │
  ├── P8  Durable Execution (Flow Engine)                                 │
  ├── P10 AI SDK 6 Frontend ─── needs P7                                  │
  ├── P12 LLM Cost Intelligence                                          │
  └── Context Compaction + Memory Tool (Agent SDK)                        │
```

---

## Workstream A: Architecture Foundation (ADR-014)

### Status: FOUNDATION COMPLETE (depth work continues per `sprints.md`)

**Already done (shipped):**
- `rusvel-core/src/department/` — `DepartmentApp`, `DepartmentManifest`, `RegistrationContext` (complete)
- All **`dept-*`** crates + **`boot::installed_departments()`** + `EngineKind` **removed** from codebase (ADR-014)
- `dept-content` / `dept-forge` and remaining departments registered at boot

**ADR-014 migration checklist (historical):**

| Step | What | Status | Effort |
|------|------|--------|--------|
| 1. Define contract | `DepartmentApp` trait + manifest types | **Done** | -- |
| 2. Convert content-engine | Full impl: routes, tools, personas, skills in manifest | **Partial** | 2 days |
| 3. Convert forge-engine | Full impl: mission tools, 10 personas, forge-specific routes | **Partial** | 2 days |
| 4. Wire boot sequence | `rusvel-app` boots `DepartmentApp` list | **Done** | -- |
| 5. Convert remaining depts | All departments → `dept-*` | **Done** | -- |
| 6. Remove EngineKind | String IDs everywhere | **Done** | -- |
| 7. Align frontend | Department manifest drives UI tabs/routes | **Partial** | 2 days |

**Total remaining (rough):** polish for rows 2–3 and 7 only — see `current-state.md` / `sprints.md`

**Why first:** Every dept-specific proposal (hooks, permissions, scoped tools, playbooks, CDP, terminal) is cleaner when departments declare their own contributions via manifest instead of hardcoding in `main.rs`.

---

## Workstream B: Agent Intelligence

These proposals make agents smarter, cheaper, and more autonomous. **All independent of ADR-014** — can run in parallel.

### B1. Deferred Tool Loading (P1)

**What:** `tool_search` meta-tool instead of injecting all tools into every prompt.

**Ecosystem validation:** Anthropic `advanced-tool-use-2025-11-20` (85% token reduction, 49%→74% accuracy). Vercel AI SDK 6 uses regex + BM25.

**Implementation:** Add `searchable: bool` to `ToolDefinition`. Split tools in `AgentRuntime::build_request()` into `always_include` vs `searchable`. Inject `tool_search` tool that queries `ToolRegistry`.

**Effort:** 2 days | **Impact:** 85% token savings per agent run

---

### B2. LLM Cost Intelligence (P12)

**What:** Smart model routing — Haiku for simple tasks, Sonnet for standard, Opus for complex.

**Implementation:** `ModelTier` enum + `complexity_hint` on `AgentConfig` + per-department defaults + `CostTracker` in `MetricStore` + prompt caching with `cache_control` breakpoints.

**Cost projection:**
| Scenario | Monthly (10k runs) |
|----------|-------------------|
| All Opus | ~$500 |
| Smart routing | ~$120 |
| + caching + batching | ~$35 |

**Effort:** 3 days | **Impact:** 60-70% cost savings

---

### B3. Batch API for Jobs (P3)

**What:** Route async jobs through Claude Batch API (50% discount) + prompt caching (90% savings).

**Implementation:** Add `submit_batch()` / `poll_batch()` to `LlmPort`. Batch-collect jobs in worker (30s or 10 jobs). Poll results, complete/fail individual jobs.

**Effort:** 3 days | **Impact:** 50-95% cost savings on background work

---

### B4. Context Compaction (from agent-sdk-features.md)

**What:** Auto-summarize old messages when approaching context limits. Trigger at 30 messages, keep recent 10.

**Anthropic SDK pattern:** Server-side compaction enables work beyond 200k tokens.

**RUSVEL gap:** Chat loads last 50 messages raw. No summarization. Long sessions lose early context.

**Implementation:** `ContextCompactor::compact(messages, llm) -> Vec<LlmMessage>` in `rusvel-agent/src/compaction.rs`. Call before building agent input in `dept_chat()`.

**Effort:** 2 days | **Impact:** Unlimited conversation length, cheaper long sessions

---

### B5. Memory Tool (from agent-sdk-features.md)

**What:** Agents use `memory_read`, `memory_write`, `memory_search`, `memory_delete` as tools. Auto-inject top 5 relevant memories into system prompt.

**RUSVEL gap:** `rusvel-memory` has FTS5 search but agents can't use it as a tool. No cross-session persistence.

**Implementation:** 4 tools in `rusvel-builtin-tools` wrapping `MemoryPort`. Add "Check memory for relevant context" to system prompts. Auto-inject recent memories.

**Effort:** 2 days | **Impact:** Cross-session agent intelligence

---

### B6. Hybrid RAG + Reranking (P2)

**What:** Fuse FTS5 (BM25) + LanceDB (vector) with Reciprocal Rank Fusion + reranking.

**Implementation:** `hybrid_search()` queries both in parallel, merges via RRF (`score = 1/(k+rank)`), reranks top-N with Haiku or local cross-encoder. Add contextual chunking on ingest.

**Comparison:**
| Approach | Precision@10 | Latency |
|----------|-------------|---------|
| BM25 only (current) | ~0.45 | <10ms |
| Hybrid + Reranking | ~0.82 | ~250ms |

**Effort:** 3 days | **Impact:** 48% retrieval quality improvement

---

### B7. Self-Correction Loop (P5) — merges with agent-sdk-features Verification Loops

**What:** Critique agent evaluates output + auto-generates fix rules. Maps directly to `agent-sdk-features.md` Feature #6 (Verification Loops).

**Combined implementation:**
1. `VerificationStep` trait in `rusvel-agent/src/verification.rs`
2. Per-tool verification config: `content_draft` → auto-call `content_review()`, `code_analyze` → `cargo check`
3. If score < threshold → auto-generate Rule via `!build` pattern
4. Store critique results in `MetricStore`
5. SSE events: "reviewing..." → "approved" / "revising..."

**Effort:** 4 days | **Impact:** 35-50% output quality gain

**Depends on:** B2 (cost intelligence) for cheap critique calls via Haiku

---

## Workstream C: Agent Orchestration

Merges `agent-orchestration.md` + `agent-sdk-features.md` (handoffs, permissions) + P9 from original proposals. **Phase 1-2 independent of ADR-014. Phase 3+ benefits from it.**

### C1. `delegate_agent` Tool (Phase 1 — the 80% primitive)

**What:** Any agent spawns a sub-agent with specific persona, prompt, tools, max_iterations.

**Already designed in:** `agent-orchestration.md` Phase 1 + `agent-sdk-features.md` Feature #4

**Implementation:** ~100 lines in `rusvel-builtin-tools/src/delegate.rs`. Calls `AgentPort::run()` with child config. Returns output to parent. Safety: max depth 3, budget cap per chain.

**Effort:** 2 days | **Impact:** Unlocks all multi-agent workflows

---

### C2. `invoke_flow` Tool (Phase 2)

**What:** Agents trigger FlowEngine DAG workflows and get results.

**Implementation:** ~50 lines. Call `FlowEngine::execute()`, return flow output as tool result.

**Effort:** 1 day | **Impact:** Bridges agent intelligence + workflow automation

---

### C3. PreToolUse / PostToolUse Hooks (from agent-sdk-features.md)

**What:** Deterministic guardrails that intercept tool calls before/after execution.

**Anthropic SDK pattern:** Shell hooks with exit code 2 = blocked. Can validate, modify, or deny.

**RUSVEL gap:** `hook_dispatch` fires *after* chat completes. No pre-execution interception. ADR-008 approval gates are in job queue, not agent loop.

**Implementation:** `HookPoint` enum + `ToolHook` trait + `HookRegistry`. Wire into `AgentRuntime::run_streaming()` before `tools.call()`. Default hooks: block `bash rm -rf`, require approval for `content_publish`.

**After ADR-014:** Each department declares its hooks in manifest → loaded during `register()`.

**Effort:** 3 days | **Impact:** Safety + deterministic guardrails in agent loop

---

### C4. Hierarchical Tool Permissions (from agent-sdk-features.md)

**What:** 4-level permission hierarchy: `allowed_tools` → `permission_mode` → `can_use_tool` hook → `disallowed_tools`.

**Implementation:** Add `ToolPermissionMode` (auto/supervised/locked) to department config. In supervised mode, dangerous tools emit `approval_required` SSE event.

**After ADR-014:** Permission mode declared in `DepartmentManifest`.

**Effort:** 2 days | **Impact:** Per-department tool safety

---

### C5. Event Triggers (Phase 3)

**What:** Subscribe to event patterns, auto-start agents/flows when matching events fire.

**Implementation:** `EventTrigger { pattern, filter, action: RunAgent | RunFlow }`. Subscribe to `EventBus`. ~150 lines in `rusvel-event`.

**Depends on:** C1 (delegate_agent), C2 (invoke_flow)

**Effort:** 2 days | **Impact:** Reactive automation — agents trigger agents without human intervention

---

### C6. Workflow Templates + Playbooks (Phase 4)

**What:** Predefined JSON pipelines combining personas + steps + rules + failure handling. User-facing as "Playbooks" (from `next-level-inspiration.md`).

**Templates from agent-orchestration.md:**
- Autonomous Code Pipeline: Architect → CodeWriter → Tester → Debugger (retry) → SecurityAuditor → Documenter
- Research & Answer: Researcher → Draft → Validate → Respond

**User-facing Playbooks (from inspiration doc):**
- Content-from-Code, Weekly Growth Review, Opportunity Scout
- UI at `/playbooks` with run buttons, parameter forms, execution history
- Shareable as JSON exports (foundation for marketplace)

**Depends on:** C1, C2, C5 + ADR-014 (manifests declare playbook contributions)

**Effort:** 5 days | **Impact:** Reusable multi-agent recipes. Framework → Product.

---

## Workstream D: Interoperability & Infrastructure

### D1. Approval Workflow UI (P4)

**What:** Build the missing approval flow frontend.

**RUSVEL gap:** API exists (`GET /api/approvals`, approve/reject), jobs block at `AwaitingApproval`. No UI. **Biggest UX gap.**

**Implementation:**
1. `ApprovalQueue.svelte` — polls `/api/approvals`, shows pending items
2. Inline approval in chat — approve/reject buttons in message stream
3. Sidebar badge — count of pending approvals
4. SSE event `approval_required` for real-time surfacing

**Effort:** 3 days | **Impact:** Core safety model becomes visible

---

### D2. AG-UI Protocol (P7)

**What:** Standardize SSE events to AG-UI schema (Microsoft + Google adopted).

**Mapping:**
| RUSVEL Current | AG-UI |
|----------------|-------|
| `TextDelta` | `TEXT_MESSAGE_CONTENT` |
| `ToolCall` | `TOOL_CALL_START` + `TOOL_CALL_ARGS` |
| `ToolResult` | `TOOL_CALL_END` |
| `Done` | `RUN_FINISHED` |
| (missing) | `RUN_STARTED`, `STATE_SNAPSHOT`, `STATE_DELTA` |

**Why:** Directly enables the observability UX you want (feedback_ux_observability.md): showing tool calls, reasoning steps, timing in real-time. Also makes RUSVEL compatible with third-party agent UIs.

**Effort:** 4 days | **Impact:** Ecosystem compatibility + rich observability

---

### D3. Durable Execution for Flows (P8)

**What:** Checkpoint/resume for `flow-engine` DAG execution.

**Implementation:** `FlowCheckpoint` domain type, persist after each node, `FlowEngine::resume(exec_id)` skips completed nodes, retry policy per node, `POST /api/flows/{id}/executions/{exec_id}/resume`.

**Synergy with C5 (Event Triggers):** Failed flows emit event → trigger retry flow or alert.

**Effort:** 5 days | **Impact:** Crash-resilient workflows, approval waits don't block threads

---

### D4. Streamable HTTP MCP (P6)

**What:** Upgrade from stdio to MCP 2025-11-25 Streamable HTTP transport.

**Implementation:** Axum HTTP handlers (`POST /mcp`, `GET /mcp` for SSE). Session management via `Mcp-Session-Id`. Map MCP async Tasks to `rusvel-jobs`. OAuth 2.1 middleware. Keep stdio as fallback.

**Effort:** 5 days | **Impact:** Remote deployment, multi-client, 12,480+ MCP ecosystem compatibility

---

### D5. AI SDK 6 Frontend (P10)

**What:** Vercel AI SDK 6 Svelte bindings for chat, replacing custom SSE handling.

**What you gain:** `useChat()` hook, `ToolInvocation` rendering, artifact panels, DevTools, `needsApproval` primitive.
**What you lose:** ~50KB bundle increase. Custom SSE code removed (~500 lines).

**Depends on:** D2 (AG-UI) — AI SDK 6 has first-party AG-UI adapter.

**Effort:** 8 days | **Impact:** Rich agent UI with less custom code

---

## Workstream E: New Capabilities

These are **net-new features** from your plan docs. They're ambitious but have detailed designs ready.

### E1. CDP Browser Bridge (from cdp-browser-bridge.md)

**What:** Chrome DevTools Protocol integration for passive data capture from Upwork, LinkedIn, Freelancer + two-way agent actions.

**Architecture:** New `rusvel-cdp` crate (~1500 lines) → `BrowserPort` trait in `rusvel-core` → harvest-engine + gtm-engine consume. Three modes: Passive (observe), Assisted (suggest), Autonomous (agent acts with approval).

**Key capabilities:**
- Intercept API responses as user browses (no scraping, no credential management)
- Platform extractors normalize data to domain types (Opportunity, Lead)
- Agent tools: `browser_observe`, `browser_search`, `browser_act`
- Data persistence: SQLite + LanceDB + FTS5

**After ADR-014:** dept-harvest manifest declares `browser_observe`, `browser_search` as tool contributions.

**Effort:** 2-3 weeks (6 phases) | **Impact:** Real-world data pipeline for harvest + GTM engines

**Tradeoff:** Platform extractors break when UIs change. Mitigate with: JSON API interception (more stable than DOM), fallback to generic extraction, user-editable selectors.

---

### E2. Native Terminal Multiplexer (from native-terminal-multiplexer.md)

**What:** Built-in terminal multiplexer — no tmux/zellij dependency. Per-department windows, web UI via xterm.js, agent-visible panes.

**Architecture:** New `rusvel-terminal` crate (~1250 lines) → `TerminalPort` trait → `portable-pty` for PTY allocation → `@xterm/xterm` + `paneforge` (already installed) for frontend → WebSocket `/api/terminal/ws/:pane_id`.

**Key insight:** Agent bash executions spawn visible panes — user watches in real-time. This is the observability you want: not just showing "agent ran bash" in chat, but the actual terminal output streaming live.

**After ADR-014:** dept manifest declares terminal layout preferences (auto-create window per dept on first terminal tab click).

**Effort:** 2-3 weeks (5 phases) | **Impact:** Full observability + native terminal experience

**Tradeoff:** Adds `portable-pty` dependency + WebSocket routes. But reuses existing patterns (tokio channels, broadcast, crossterm, paneforge).

---

### E3. Agent Workforce — Dogfooding (from agent-workforce.md)

**What:** Use 14 specialized Claude Code sub-agents to build RUSVEL itself. Three-layer hierarchy: Human → Architect Agent → Builder Agents.

**This is not a code proposal — it's an execution strategy.** The 14 agents map directly to RUSVEL's runtime patterns:
- Agent A0 (Sprint Architect) = Forge Engine mission planning
- Agents B1-B4 (Core/Dept/Feature/Integration Builders) = Department engines with scoped tools

**When to execute:** After C1 (delegate_agent) + ADR-014 Step 4 (boot sequence). Then RUSVEL can orchestrate its own development — the ultimate dogfood.

**Effort:** Ongoing | **Impact:** Proves the orchestration system works by building with it

---

### E4. Product Features (from next-level-inspiration.md)

| Feature | Maps To | Depends On | Effort |
|---------|---------|------------|--------|
| Executive Brief | Enhanced `forge mission today` + delegate_agent per dept | C1, C6 | 3 days |
| Starter Kits | Pre-built dept bundles via `!build` + Capability Engine | ADR-014 | 3 days |
| Leveling/Progression | Per-dept milestone tracker, uses MetricStore | ADR-014 | 4 days |
| Roundtable/Strategy Review | Multi-persona structured discussion UI | C1 | 5 days |
| Self-Improving Knowledge Base | Auto-index all engine outputs → cross-dept insights | B6 (RAG) | 3 days |

---

## Execution Plan

> **See [`sprints.md`](sprints.md) for the authoritative sprint plan.**
> 38 tasks across Sprint 0-5, ~65 working days (~13 weeks), 8 backlog items.

Key changes from this doc's original roadmap (reflected in sprints.md):
- **Sprint 0 added** — Build is broken (25 compile errors from P1 `searchable` field). Must fix first.
- **Terminal Multiplexer threaded across all sprints** (not a single-sprint item). PTY core in Sprint 1, web bridge Sprint 2, agent visibility Sprint 3, flow panes Sprint 4, CDP + TUI Sprint 5.
- **A2UI/Capability Catalog** deferred to backlog — AG-UI (Sprint 4) is the prerequisite.
- **CDP Browser Bridge** moved to Sprint 5 (connected to terminal panes).

---

## Cross-Reference: Proposals ↔ All Plan/Design Docs

| Proposal | Primary Doc | Secondary Docs |
|----------|-------------|----------------|
| ADR-014 | `design/department-as-app.md` | Crates: dept-content, dept-forge, rusvel-core/department |
| C1-C6 | `plans/agent-orchestration.md` | `plans/agent-sdk-features.md` (Features 4, 5) |
| C3-C4 | `plans/agent-sdk-features.md` | Features 1, 5 |
| B4-B5 | `plans/agent-sdk-features.md` | Features 2, 3 (compaction, memory) |
| B7 | `plans/agent-sdk-features.md` | Feature 6 (verification loops) + P5 |
| E1 | `plans/cdp-browser-bridge.md` | -- |
| E2 | `plans/native-terminal-multiplexer.md` | Threaded across Sprints 1-5 in sprints.md |
| E3 | `design/agent-workforce.md` | 14 builder agents for self-building |
| E4 | `plans/next-level-inspiration-2026-03-25.md` | Playbooks, brief, kits, leveling, roundtable |
| D2 (AG-UI) | `plans/a2ui-department-apps.md` | AG-UI + A2UI generative UI, capability catalog, 3 UI modes |
| Inter-dept | `plans/claude-ecosystem-integration.md` | AgentBroker, DepartmentPeer, context health, TUI activity, git-aware code |
| Machine awareness | `plans/machine-awareness-fs-integration.md` | MachinePort, `fs` tool integration, project/system discovery |
| B1-B3, B6, D1-D5 | This doc (ecosystem research) | -- |
| All | `plans/sprints.md` | **Authoritative execution plan** (38 tasks, 6 sprints) |

---

## What We Decided NOT to Adopt (and Why)

| Pattern | Source | Why Skip |
|---------|--------|----------|
| Full Temporal integration | Durable execution research | Overkill for single-binary. Our checkpoint/resume (D3) is sufficient. |
| WASM department loading | ADR-014 mentions as future | Premature. In-process trait objects work fine now. Revisit at 50+ departments. |
| Python Agent SDK wrapping | Anthropic SDK | We already moved past CLI subprocess architecture. Rust-native is better. |
| CopilotKit/assistant-ui | Frontend research | AI SDK 6 is more aligned with SvelteKit. One frontend framework dependency is enough. |
| Full Knowledge Graph (Neo4j) | RAG research | External dependency. Hybrid search (B6) covers 90% of use cases. |
| Agentic Search replacing RAG | Agent SDK Feature 8 | Too aggressive. Keep RAG + add dynamic search as fallback. |
| WebSocket replacing SSE | Transport research | SSE works, AG-UI uses it, AI SDK 6 uses it. No reason to switch. |

---

## Ecosystem Signals (Updated)

### Validated by Our Architecture
- **Graph-based workflows** (LangGraph) → flow-engine petgraph
- **Role-based personas** (CrewAI) → forge-engine 10 personas
- **Approval gates** (Vercel AI SDK 6) → ADR-008
- **Single binary** → rust-embed, zero Docker
- **MCP as universal tool protocol** → rusvel-mcp + rusvel-mcp-client
- **Department-as-App** (Django AppConfig) → ADR-014

### New Signals to Watch
- **AG-UI/A2UI** — Microsoft + Google convergence on agent-frontend protocol
- **MCP Streamable HTTP** — stdio is legacy, HTTP is the future
- **Deferred tool loading** — Anthropic's own cost optimization
- **Durable execution** — Temporal at $5B, every framework adding it
- **Generative UI** — agents emitting UI components (AI SDK 6 artifacts)
- **Solo builder economics** — 38% of 7-figure businesses are solopreneurs with AI

### Solo Builder Market
- Anthropic CEO: 70-80% odds first $1B one-person company by 2026
- Base44: 1 person → $80M exit in 6 months
- **RUSVEL's content-engine should build RUSVEL's distribution** (dogfooding)
- Distribution > Development — building is easy, getting noticed is hard

---

## Sources

- [Anthropic Advanced Tool Use](https://www.anthropic.com/engineering/advanced-tool-use)
- [Claude Agent SDK](https://platform.claude.com/docs/en/agent-sdk/overview)
- [MCP Spec 2025-11-25](https://modelcontextprotocol.io/specification/2025-11-25)
- [2026 MCP Roadmap](http://blog.modelcontextprotocol.io/posts/2026-mcp-roadmap/)
- [AG-UI Protocol](https://docs.ag-ui.com/)
- [Vercel AI SDK 6](https://vercel.com/blog/ai-sdk-6)
- [Temporal Durable Execution](https://temporal.io/blog/durable-execution-meets-ai-why-temporal-is-the-perfect-foundation-for-ai)
- [Trigger.dev AI Agents](https://trigger.dev/product/ai-agents)
- [Rig (Rust LLM Framework)](https://rig.rs/)
- [PulseMCP Registry](https://www.pulsemcp.com/servers)
- [Hybrid RAG with Reranking](https://superlinked.com/vectorhub/articles/optimizing-rag-with-hybrid-search-reranking)
- [Prompt Caching](https://platform.claude.com/docs/en/build-with-claude/prompt-caching)
- [Extended Thinking](https://platform.claude.com/docs/en/build-with-claude/extended-thinking)
- [Computer Use](https://platform.claude.com/docs/en/agents-and-tools/tool-use/computer-use-tool)
- [AutoAgents Rust](https://github.com/liquidos-ai/AutoAgents)
- [Pydantic AI](https://ai.pydantic.dev/)

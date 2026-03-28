# Chapter 7 — Metrics & Gaps

> **Obsolete snapshot (pre-2026-03-28 refresh).** Tables and counts below do not match the current workspace. **Canonical metrics:** [`../status/current-state.md`](../status/current-state.md). This file is retained for narrative context only.

## Crate Size Distribution

### Top 15 by Lines of Rust

| Rank | Crate | Lines | Role |
|------|-------|-------|------|
| 1 | rusvel-api | 8,932 | HTTP handlers (26 modules) |
| 2 | rusvel-core | 4,160 | Port traits + domain types |
| 3 | rusvel-llm | 3,908 | 4 LLM providers + streaming |
| 4 | content-engine | 2,416 | Content creation + publishing |
| 5 | rusvel-agent | 2,352 | Agent runtime + tool-use loop |
| 6 | rusvel-db | 2,148 | SQLite adapter (5 stores) |
| 7 | flow-engine | 2,038 | DAG executor + checkpointing |
| 8 | rusvel-builtin-tools | 2,005 | 10 built-in tools |
| 9 | harvest-engine | 1,662 | Opportunity pipeline |
| 10 | rusvel-app | 1,621 | Composition root + binary |
| 11 | rusvel-cli | ~1,500 | 3-tier CLI (one-shot + REPL + TUI) |
| 12 | forge-engine | ~1,400 | Mission planning + personas |
| 13 | rusvel-tool | ~1,200 | Tool registry + dispatch |
| 14 | code-engine | ~1,100 | Code parser + BM25 search |
| 15 | gtm-engine | ~1,000 | CRM + outreach + invoicing |

### Smallest Crates

| Crate | Lines | Note |
|-------|-------|------|
| rusvel-deploy | 103 | Fly.io stub |
| rusvel-embed | 105 | FastEmbed wrapper |
| rusvel-auth | 138 | In-memory from env |
| dept-* (most) | ~180 | Manifest + OnceLock engine |
| dept-forge | ~630 | 5 tool handlers |
| dept-content | ~400 | Tools + event + job handlers |

### Adherence to <2000-Line Rule

All crates comply except:
- `rusvel-api` at 8,932 lines (26 handler modules — acceptable as aggregate)
- `rusvel-core` at 4,160 lines (central definition crate — structurally necessary)
- `rusvel-llm` at 3,908 lines (4 provider implementations — could be split)
- `content-engine` at 2,416 lines (slightly over — real domain complexity)
- `rusvel-agent` at 2,352 lines (complex runtime — acceptable)
- `rusvel-db` at 2,148 lines (5 store impls — could be split)
- `flow-engine` at 2,038 lines (DAG executor — barely over)
- `rusvel-builtin-tools` at 2,005 lines (10 tools — barely over)

---

## Test Distribution

**Total: 399 test cases**

Key crate test counts (from exploration):
- forge-engine: 15 tests
- content-engine: 7 tests
- harvest-engine: 12 tests
- All dept-* crates: manifest + port requirement tests
- rusvel-db: store operation tests
- rusvel-api: handler tests
- rusvel-llm: provider tests

All engines have tests covering core happy paths using mock ports.

---

## Wiring Status Matrix

### Full Stack Wiring (API route + CLI + Agent tools + Events)

| Feature | API Route | CLI | Agent Tool | Event | Job |
|---------|:---------:|:---:|:----------:|:-----:|:---:|
| Session CRUD | done | done | -- | -- | -- |
| Mission (goals, plan) | done | done | done (forge) | done | -- |
| Code analyze | done | done | done (code) | done | done |
| Code search | done | done | done (code) | -- | -- |
| Content draft | done | done | done (content) | done | -- |
| Content publish | done | -- | done (content) | done | done |
| Content from-code | done | -- | -- | done | -- |
| Harvest scan | done | done | -- | done | done |
| Harvest pipeline | done | done | -- | -- | -- |
| Harvest proposal | done | -- | -- | done | done |
| Flow CRUD + run | done | -- | -- | done | -- |
| Flow checkpoint | done | -- | -- | -- | -- |
| Approval queue | done | -- | -- | -- | -- |
| Knowledge RAG | done | -- | -- | auto-index | -- |
| Database browser | done | -- | -- | -- | -- |
| Terminal panes | done | -- | done (tool) | -- | -- |
| Browser CDP | done | done | done (tool) | -- | -- |
| Chat (God Agent) | done | -- | -- | done | -- |
| Chat (per-dept) | done | -- | -- | done | -- |
| Agents CRUD | done | -- | -- | -- | -- |
| Skills CRUD | done | -- | -- | -- | -- |
| Rules CRUD | done | -- | -- | -- | -- |
| Hooks CRUD | done | -- | -- | -- | -- |
| MCP servers | done | -- | -- | -- | -- |
| Workflows CRUD | done | -- | -- | -- | -- |
| Playbooks | done | -- | -- | -- | -- |
| Starter Kits | done | -- | -- | -- | -- |
| Capability Builder | done | -- | -- | -- | -- |
| Visual Testing | done | -- | done (MCP) | -- | -- |
| Analytics | done | -- | -- | -- | -- |

### Department Tool Wiring Gap

**3 departments fully wired:**
- **forge:** mission_today, list_goals, set_goal, review, hire_persona
- **code:** analyze, search
- **content:** draft, adapt, publish, schedule, approve + event handler (code.analyzed) + job handler (content.publish)

**10 departments need tool wiring:**
- harvest, flow, gtm, finance, product, growth, distro, legal, support, infra

Each needs:
1. `ToolDefinition` for each engine method
2. Handler closure calling engine method
3. Registration in `register()` via `RegistrationContext::tools`

---

## Actionable Gaps

### Priority 1: Department Tool Wiring (10 departments)

The biggest leverage point. All engines are implemented but agents can't invoke them. Pattern is established by dept-forge and dept-code:

```rust
// Example from dept-forge register():
ctx.tools.register(
    ToolDefinition {
        name: "mission_today".into(),
        description: "Generate today's mission plan".into(),
        parameters: json!({ "type": "object", "properties": { "session_id": { "type": "string" } } }),
        searchable: false,
    },
    |args| {
        let engine = ENGINE.get().unwrap();
        Box::pin(async move { engine.mission_today(args).await })
    },
);
```

### Priority 2: Job Queue Persistence

Currently `Vec<Job>` behind Mutex — volatile. Jobs are lost on restart. The `JobStore` trait exists in `rusvel-db` (SQLite) but the in-memory implementation in `rusvel-jobs` doesn't use it for the worker.

### Priority 3: OutreachSend Job Handler

GTM engine is fully implemented but the `OutreachSend` job kind handler is a placeholder in the job worker. Needs wiring to `GtmEngine.outreach`.

### Priority 4: Visual Flow Editor

`@xyflow/svelte` is installed. `WorkflowBuilder.svelte` and `AgentNode.svelte` components exist. But the `/flows` page uses JSON input for flow creation instead of the visual builder. Connecting xyflow to the flow CRUD API would enable drag-and-drop DAG construction.

### Priority 5: Auth Middleware

`rusvel-auth` stores credentials in memory from env vars. No API route middleware enforces authentication. `AuthPort` trait exists but isn't checked on requests.

---

## Architecture Health

| Principle | Status | Evidence |
|-----------|--------|----------|
| Hexagonal boundaries | Healthy | No engine imports any adapter crate |
| Single responsibility (<2000 LOC) | Mostly | 5 crates slightly over, all justifiable |
| ADR-014 DepartmentApp | Complete | All 13 departments implement trait |
| ADR-009 no direct LlmPort | Compliant | Engines use AgentPort only |
| ADR-007 metadata field | Compliant | All domain types carry `metadata: serde_json::Value` |
| ADR-005 String event kinds | Compliant | No Event enum anywhere |
| ADR-003 single job queue | Compliant | One JobPort, one worker |
| ADR-008 approval gates | Partial | Content uses it; other departments not yet |
| Test coverage | Moderate | 399 tests, all engines covered, mock ports used |

---

## Summary Numbers

```
52,560  lines of Rust across 215 files in 50 crates
10,623  lines of frontend (66 Svelte + TypeScript)
~63,183 total lines
399     tests
~105    API routes
22+     agent tools
16      port traits
13      engines (all substantially implemented)
13      departments (3 fully wired, 10 skeleton)
4       LLM providers
6       MCP tools
795     Cargo.lock dependencies
```

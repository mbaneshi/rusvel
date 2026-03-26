# RUSVEL Gap Analysis: What old/ Has That We Don't

> **Status 2026-03-27:** Historical comparison to `old/` monoliths. **Current shipped behavior** is in [`current-state.md`](current-state.md). Tables below are not auto-updated when RUSVEL gains features — re-verify before acting.

> Audited 2026-03-22 — compared working code in old/ against current RUSVEL crates

---

## The Big Picture

Your old projects have **~2.4M lines of battle-tested code**. RUSVEL distilled the architecture into clean hexagonal ports — but many working implementations didn't make the cut yet. Here's what's missing, organized by priority.

---

## HIGH PRIORITY — Safety & Agent Reliability

These are in `old/forge-project/crates/forge-safety/` and `forge-process/`. Without them, agents can loop forever, burn budget, or overwhelm APIs.

| Pattern | Old Source | RUSVEL Status | What to Do |
|---------|-----------|---------------|------------|
| **Circuit Breaker** (3-state FSM) | forge-safety/lib.rs:20-127 | Safety guard exists but basic | Port the Closed→Open→HalfOpen FSM with atomic ops |
| **Rate Limiter** (token bucket) | forge-safety/lib.rs:129-171 | Missing | Port — prevents API quota exhaustion |
| **Cost Tracker** (budget enforcement) | forge-safety/lib.rs:182-228 | **MetricStore** tracks usage/cost in current app; **hard budget enforcement** vs old forge-safety may still differ | Compare `MetricStore` + job approval with old Warning/Hard Limit UX |
| **Loop Detector** (exit gate) | forge-process/loop_detect.rs | Missing | Port — sliding window hash to detect repeating output |
| **Context Pruner** (token-aware) | forge-process/context_pruner.rs | Missing | Port — truncate/drop old messages to fit token budget |

All have full test suites in the old code. These are pure functions — can be ported in hours.

---

## HIGH PRIORITY — Content Publishing (Real APIs)

**Update 2026-03-27:** `content-engine` ships **HTTP adapters** for **DEV.to**, **Twitter/X**, and **LinkedIn** under `crates/content-engine/src/adapters/` (`devto.rs`, `twitter.rs`, `linkedin.rs`), using `ConfigPort` keys (e.g. API keys / bearer tokens). **Bluesky** and **Mastodon** are still not present vs old `contentforge`.

| Platform | Old Source | RUSVEL as of 2026-03-27 |
|----------|-----------|-------------------------|
| **DEV.to** | adapters/devto.rs | Implemented — `DevToAdapter` |
| **Twitter/X** | adapters/twitter.rs | Implemented — `TwitterAdapter` (bearer token) |
| **LinkedIn** | adapters/linkedin.rs | Implemented — `LinkedInAdapter` |
| **Bluesky** | adapters/bluesky.rs | Missing |
| **Mastodon** | adapters/mastodon.rs | Missing |

Routing is via content-engine’s platform adapter wiring (not necessarily identical to old **PublisherRegistry** naming).

---

## MEDIUM PRIORITY — Agent Orchestration Patterns

In `old/forge-project/crates/forge-process/`. RUSVEL has `rusvel-agent` with Sequential/Parallel/Loop but not these advanced patterns.

| Pattern | Old Source | What It Does | RUSVEL Status |
|---------|-----------|-------------|---------------|
| **Pipeline Engine** | forge-process/pipeline.rs | Sequential + Fanout steps with output chaining | Planned (Phase 1) |
| **Best-of-N Runner** | forge-process/best_of_n.rs | Run N strategy variants, score results, pick best | Planned |
| **Strategy Set** | forge-agent/strategy.rs | 3 defaults: minimal_changes, modular_refactor, thorough_with_tests | Missing |
| **Concurrent Runner** | forge-process/concurrent.rs | Semaphore-limited parallel sub-agents | Partially in rusvel-agent |
| **Stream JSON Parser** | forge-process/stream_event.rs | Parse Claude CLI streaming output (system, assistant, result) | Partially in rusvel-llm |

---

## MEDIUM PRIORITY — Persona & Org System

In `old/agentforge-hq/crates/forge-persona/` and `forge-org/`. RUSVEL has 10 hardcoded personas — old system has 112 from markdown files.

| Feature | Old Source | What Works | RUSVEL Status |
|---------|-----------|------------|---------------|
| **112 Personas in 11 divisions** | personas/*.md | Markdown files auto-parsed at startup | 10 hardcoded in PersonaManager |
| **Persona Parser** | forge-persona/parser.rs | Walk filesystem, extract name/description/sections | Missing |
| **Org Chart** | forge-org/service.rs | OrgPosition with reports_to, tree builder, MAX_DEPTH=50 | Missing |
| **Hire Persona** | forge-api/routes/personas.rs | POST endpoint → creates Agent + OrgPosition | hire_persona() exists, no org |
| **Budget per Company** | forge-org model | Company.budget_limit + budget_used | In domain types, not wired |

---

## MEDIUM PRIORITY — Code Intelligence (Codeilus)

In `old/codeilus/codeilus/crates/`. RUSVEL's code-engine has basic parse + search — codeilus has the full 8-step pipeline.

| Feature | Old Source | What Works | RUSVEL Status |
|---------|-----------|------------|---------------|
| **Multi-language parsing** | codeilus-parse | 12 languages via tree-sitter | Rust only |
| **Incremental parsing** | codeilus-parse | mtime tracking, skip unchanged | Missing |
| **Knowledge graph** | codeilus-graph | Call/import/extends edges, Louvain communities | Basic graph, no communities |
| **Git metrics** | codeilus-metrics | Churn, contributors per file | Missing |
| **Pattern detection** | codeilus-analyze | God class, long method, circular deps, security | Missing |
| **Mermaid diagrams** | codeilus-diagram | Auto architecture diagrams from call graph | Missing |
| **8-step checkpoint pipeline** | codeilus-app/main.rs | Parse→Store→Graph→Metrics→Analyze→Diagram→Narrate→Learn | Missing |

---

## MEDIUM PRIORITY — Approval/Governance

**Update 2026-03-27:** RUSVEL has **job-based approvals** (`GET /api/approvals`, approve/reject) and approval-gated jobs (see [`current-state.md`](current-state.md)). Old forge-governance had a richer **governance** model; parity is not 1:1.

| Feature | Old Source | RUSVEL Status (2026-03-27) |
|---------|-----------|----------------------------|
| **Approval model** | forge-governance/model.rs | Domain + **Job** approval states; API lists pending jobs |
| **Approval API** | forge-governance routes | **Present** — REST under `/api/approvals/*` |
| **Approval gates** | orchestration | **Wired** for gated job kinds (e.g. publish) per ADR-008 |

---

## LOWER PRIORITY — WebSocket, TUI, Scheduling

| Feature | Old Source | What Works | RUSVEL Status |
|---------|-----------|------------|---------------|
| **WebSocket streaming** | forge-api/routes/ws.rs | EventBus → WS → live UI updates | Missing (axum WS feature exists) |
| **TUI Dashboard** | contentforge-tui | Ratatui tabs: Drafts, Adapt, Publish, Platforms | rusvel-tui has layout, not wired |
| **Cron scheduling** | contentforge-schedule | cron expression parsing + tick loop | JobPort exists, no cron parsing |
| **Batch event writer** | forge-core event_bus | Batch 50 events, flush every 2s | Direct write per event |
| **HTML export** | codeilus-export | Single self-contained HTML with embedded data | Missing |

---

## What RUSVEL Already Does Better

Not everything in old/ is better. RUSVEL improved on:

| Area | Old Approach | RUSVEL Approach | Why Better |
|------|-------------|-----------------|------------|
| **Architecture** | Each project has own DB, events, CLI | 10 shared port traits | No duplication, one kernel |
| **Event typing** | Giant enum (35+ variants per project) | Event.kind as String (ADR-005) | Extensible without recompile |
| **Storage** | Different schemas per project | 5 canonical stores (ADR-004) | Uniform access pattern |
| **LLM routing** | Hardcoded provider per project | LlmPort trait + MultiProvider | Swap providers without code change |
| **Session model** | Varied per project | Session→Run→Thread hierarchy | Consistent across all engines |
| **Schema evolution** | Manual migrations | metadata: serde_json::Value (ADR-007) | Add fields without migrations |

---

## Recommended Adoption Order

### Sprint 1: Safety (from forge-project)
- [ ] Port CircuitBreaker, RateLimiter, CostTracker into forge-engine safety module
- [ ] Port LoopDetector into rusvel-agent
- [ ] Port ContextPruner into rusvel-agent
- **Source:** `old/forge-project/crates/forge-safety/src/lib.rs`

### Sprint 2: Approval Flow (from agentforge-hq)
- [ ] Add approval API endpoints (POST + PATCH + GET)
- [ ] Wire approval gate into content publish + outreach send
- **Source:** `old/agentforge-hq/crates/forge-governance/`

### Sprint 3: Real Platform Publishing (from contentforge)
- [ ] Port DEV.to adapter (real HTTP)
- [ ] Port Twitter adapter (OAuth + threads)
- [ ] Add PublisherRegistry + PlatformCredential
- [ ] Wire into content-engine publish flow
- **Source:** `old/contentforge/crates/contentforge-publish/src/adapters/`

### Sprint 4: Persona System (from agentforge-hq)
- [ ] Create personas/ markdown directory with 11 divisions
- [ ] Port PersonaParser (filesystem walker)
- [ ] Add "hire persona" API endpoint
- **Source:** `old/agentforge-hq/crates/forge-persona/`

### Sprint 5: Code Intelligence (from codeilus)
- [ ] Add multi-language tree-sitter support
- [ ] Build proper knowledge graph with communities
- [ ] Add pattern detection (god class, circular deps)
- **Source:** `old/codeilus/codeilus/crates/codeilus-{parse,graph,analyze}/`

### Sprint 6: WebSocket + TUI (from forge-project + contentforge)
- [ ] Add /api/ws endpoint forwarding EventBus
- [ ] Wire rusvel-tui with engine data
- [ ] Add cron expression parsing to job queue
- **Source:** `old/forge-project/crates/forge-api/src/routes/ws.rs`

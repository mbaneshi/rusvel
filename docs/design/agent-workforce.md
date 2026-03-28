# Agent Workforce — Sub-Agent Orchestration for RUSVEL Development

> Date: 2026-03-25
> Status: Proposed
> Depends on: ADR-014 (Department-as-App), all proposal documents
> Purpose: Design the agent system that builds RUSVEL itself
>
> **2026-03-28:** Shipped code follows **ADR-014** — **`EngineKind` is removed**; use string department IDs. Any instruction below to keep or avoid removing `EngineKind` is **obsolete** (treat as historical proposal text).

---

## Concept

One human (you) orchestrates a workforce of specialized Claude Code sub-agents.
Each agent has a **bounded scope**, **clear inputs/outputs**, **validation criteria**,
and **knows what it must NOT touch**. They run in isolated worktrees where possible,
produce PRs or patches, and the human reviews + merges.

This is dogfooding: we're using the same orchestration pattern (delegate_agent, scoped tools,
rules, skills) that P9 proposes for RUSVEL's runtime. The agents below are the blueprint
for what RUSVEL departments will eventually do autonomously.

---

## Architecture: Three-Layer Agent Hierarchy

```
                    ┌──────────────────────┐
                    │   HUMAN (you)        │
                    │   Reviews, merges,   │
                    │   resolves conflicts │
                    └──────────┬───────────┘
                               │
                    ┌──────────▼───────────┐
                    │   ARCHITECT AGENT    │
                    │   Plans sprints,     │
                    │   sequences work,    │
                    │   validates deps     │
                    └──────────┬───────────┘
                               │
         ┌─────────────────────┼─────────────────────┐
         │                     │                     │
    ┌────▼────┐          ┌────▼────┐          ┌────▼────┐
    │ BUILDER │          │ BUILDER │          │ BUILDER │
    │ AGENTS  │          │ AGENTS  │          │ AGENTS  │
    │ (Rust)  │          │ (Svelte)│          │ (Infra) │
    └─────────┘          └─────────┘          └─────────┘
```

**Human**: Reviews PRs, resolves merge conflicts, makes architecture calls.
**Architect**: Reads all plans, sequences work, generates prompts for builders, validates outputs.
**Builders**: Execute scoped tasks in isolated worktrees, run tests, produce commits.

---

## The 14 Sub-Agents

### Layer 0: Architect

#### Agent A0: Sprint Architect

**Role**: Reads all plan documents, the dependency matrix, and current codebase state.
Generates sequenced sprint plans with concrete prompts for each builder agent.

**Scope**: Read-only on codebase. Writes only to `docs/sprints/`.

**Inputs**:
- All docs in `docs/plans/` and `docs/design/`
- Current `cargo test` results
- Current `git log --oneline -20`
- Human's priority override (e.g., "focus on P1 and ADR-014 step 1")

**Outputs**:
- `docs/sprints/sprint-N.md` with:
  - Ordered task list
  - Per-task: agent assignment, input files, output files, validation command
  - Dependency arrows between tasks
  - Estimated line counts

**Validation**: Human reviews sprint plan before any builder starts.

**Prompt template**:
```
You are the Sprint Architect for RUSVEL. Your job is to read all plan documents
and the current codebase state, then produce a sequenced sprint plan.

Read these documents:
- docs/design/department-as-app.md (ADR-014)
- docs/plans/next-level-proposals.md (P1-P12)
- docs/plans/agent-sdk-features.md (SDK 1-9)
- docs/plans/agent-orchestration.md
- docs/status/current-state.md
- CLAUDE.md (project conventions)

The human's priority for this sprint is: {{priority}}

Produce a sprint plan in docs/sprints/sprint-{{n}}.md with:
1. Tasks ordered by dependency (what must complete before what)
2. Tasks that can run in parallel grouped together
3. For each task: which builder agent, input files, output files, test command
4. No task should touch more than 3 crates
5. Every task must end with `cargo test` or `pnpm check` passing

Do NOT write any code. Only produce the plan document.
```

---

### Layer 1: Core Builders

#### Agent B1: Core Contract Builder

**Role**: Implements ADR-014 Step 1 — define `DepartmentApp` trait, `DepartmentManifest`,
`RegistrationContext`, and all contribution types in `rusvel-core`.

**Scope**: Only touches `crates/rusvel-core/src/`.

**Must NOT touch**: Any engine crate, `rusvel-app`, `rusvel-api`, frontend.

**Inputs**:
- `docs/design/department-as-app.md` (the full ADR-014 spec)
- `crates/rusvel-core/src/ports.rs` (existing port traits)
- `crates/rusvel-core/src/engine.rs` (existing Engine trait)
- `crates/rusvel-core/src/registry.rs` (existing DepartmentRegistry)
- `crates/rusvel-core/src/domain.rs` (existing domain types)

**Outputs**:
- `crates/rusvel-core/src/department/mod.rs`
- `crates/rusvel-core/src/department/app.rs` — `DepartmentApp` trait
- `crates/rusvel-core/src/department/manifest.rs` — `DepartmentManifest` + all contribution types
- `crates/rusvel-core/src/department/context.rs` — `RegistrationContext` + registrar stubs
- `crates/rusvel-core/src/lib.rs` — re-export `department` module
- Tests in each new file

**Validation**:
```bash
cargo test -p rusvel-core
cargo check --workspace  # Ensure no downstream breakage
```

**Rules**:
- ~~Keep `EngineKind` temporarily~~ — **N/A:** removed per ADR-014
- Keep existing `DepartmentRegistry` — new code is additive
- All structs must derive `Debug, Clone, Serialize, Deserialize`
- All structs must have `metadata: serde_json::Value` where appropriate (ADR-007)
- `DepartmentManifest` must be constructable with no side effects
- `RegistrationContext` registrars are initially stubs (just collect data)
- Total new code < 600 lines

**Prompt template**:
```
You are the Core Contract Builder for RUSVEL. Your task is to implement
ADR-014 Step 1: define the DepartmentApp contract in rusvel-core.

Read docs/design/department-as-app.md for the full specification.
Read crates/rusvel-core/src/ports.rs for the existing port traits.
Read crates/rusvel-core/src/engine.rs for the existing Engine trait.
Read crates/rusvel-core/src/registry.rs for the existing DepartmentRegistry.

Create the following files:
- crates/rusvel-core/src/department/mod.rs
- crates/rusvel-core/src/department/app.rs
- crates/rusvel-core/src/department/manifest.rs
- crates/rusvel-core/src/department/context.rs

Update crates/rusvel-core/src/lib.rs to re-export the department module.

Rules:
- Do NOT modify any existing traits or structs
- ~~Do NOT remove EngineKind~~ — **N/A:** `EngineKind` removed; registry is manifest-driven
- All new types must derive Debug, Clone, Serialize, Deserialize
- RegistrationContext registrars are stubs that collect Vec<T>
- Include unit tests for manifest construction and validation
- Total new code must be under 600 lines

After writing, run: cargo test -p rusvel-core && cargo check --workspace
```

---

#### Agent B2: Department Migrator (Content)

**Role**: ADR-014 Step 2 — convert `content-engine` to `dept-content` using the
new `DepartmentApp` contract. This is the proof-of-concept migration.

**Scope**: `crates/content-engine/` → `crates/dept-content/`, plus minimal wiring in `rusvel-app`.

**Must NOT touch**: Other engine crates, `rusvel-core` (contract is frozen), frontend.

**Inputs**:
- `crates/rusvel-core/src/department/` (the contract from B1)
- `crates/content-engine/src/` (existing content engine code)
- `crates/rusvel-api/src/engine_routes.rs` (existing content routes)
- `docs/design/department-as-app.md` (manifest example for content)

**Outputs**:
- `crates/dept-content/Cargo.toml`
- `crates/dept-content/src/lib.rs` — `DepartmentApp` impl
- `crates/dept-content/src/manifest.rs` — static manifest
- `crates/dept-content/src/engine.rs` — domain logic (moved from content-engine)
- `crates/dept-content/src/handlers.rs` — HTTP route handlers
- `crates/dept-content/src/tools.rs` — agent tool definitions
- `crates/dept-content/src/events.rs` — event constants
- `crates/dept-content/src/jobs.rs` — job handlers
- `crates/dept-content/src/platform/` — platform adapters (DevTo, Twitter, LinkedIn)
- `crates/dept-content/seeds/` — default agents, skills, rules JSON
- Tests passing

**Validation**:
```bash
cargo test -p dept-content
cargo test -p content-engine  # Old crate still compiles (transitional)
cargo check --workspace
```

**Rules**:
- `dept-content/Cargo.toml` depends ONLY on `rusvel-core` (not rusvel-db, rusvel-llm, etc.)
- Platform adapters use only `ConfigPort` for API keys (not raw env vars)
- Event constants are `pub const` strings (ADR-005)
- manifest() must be pure — no I/O, no side effects
- All existing content-engine tests must pass in the new location
- Keep old `content-engine` crate temporarily (dual existence during migration)

**Prompt template**:
```
You are the Department Migrator. Convert content-engine to the dept-content
DepartmentApp pattern defined in ADR-014.

Read docs/design/department-as-app.md for the target architecture.
Read crates/rusvel-core/src/department/ for the DepartmentApp trait.
Read crates/content-engine/src/ for the existing code to migrate.
Read crates/rusvel-api/src/engine_routes.rs for the existing content routes.

Create crates/dept-content/ following the convention layout:
  src/lib.rs        — impl DepartmentApp
  src/manifest.rs   — static DepartmentManifest (see ADR-014 example)
  src/engine.rs     — ContentEngine domain logic (moved from content-engine)
  src/handlers.rs   — Axum route handlers (moved from engine_routes.rs)
  src/tools.rs      — content.draft, content.adapt tool definitions
  src/events.rs     — event kind constants
  src/jobs.rs       — content.publish job handler
  src/platform/     — DevTo, Twitter, LinkedIn adapters

Rules:
- Cargo.toml depends ONLY on rusvel-core + standard library crates
- Do NOT import rusvel-db, rusvel-llm, or any adapter crate
- Keep old content-engine compiling (don't delete it yet)
- All existing tests must pass in the new crate
- manifest() is pure — no I/O

After writing, run: cargo test -p dept-content && cargo check --workspace
```

---

#### Agent B3: Department Migrator (Forge)

**Role**: ADR-014 Step 3 — convert `forge-engine` to `dept-forge`. Validates the
contract generalizes (Forge uses 7 ports, the most of any engine).

**Scope**: `crates/forge-engine/` → `crates/dept-forge/`.

**Same pattern as B2**, adapted for Forge:
- Forge has personas (10 defaults) → `PersonaContribution` in manifest
- Forge has mission sub-module → stays as engine internal
- Forge uses 7 ports → `requires_ports` in manifest lists all 7
- Forge has no platform adapters → simpler than content

**Validation**: `cargo test -p dept-forge && cargo check --workspace`

---

#### Agent B4: Department Migrator (Remaining 10)

**Role**: Convert code, harvest, flow, gtm, finance, product, growth, distro, legal, support, infra.

**Can run in parallel** for independent departments. Each gets its own worktree.

**For stub departments** (finance, product, growth, distro, legal, support, infra):
- Minimal `register()` — empty or just seeds
- Full manifest with UI, personas, quick_actions
- No routes, tools, or jobs yet
- ~200 lines each

**For real departments** (code, harvest, flow):
- Full migration like B2/B3
- ~400 lines each

**Validation**: `cargo test --workspace && cargo check --workspace`

---

#### Agent B5: Boot Sequence Builder

**Role**: ADR-014 Steps 4-5 — refactor `rusvel-app/src/main.rs` to use the
`installed_departments()` + `boot()` pattern.

**Scope**: `crates/rusvel-app/src/`.

**Must NOT touch**: Department crates (they're already migrated), `rusvel-core`.

**Inputs**:
- All `dept-*` crates (from B2, B3, B4)
- Current `crates/rusvel-app/src/main.rs`
- `docs/design/department-as-app.md` (boot sequence section)

**Outputs**:
- `crates/rusvel-app/src/main.rs` — refactored with `installed_departments()` + `boot()`
- `crates/rusvel-app/src/boot.rs` — boot logic (manifest validation, dependency resolution, registration)
- Old hardcoded engine instantiation removed

**Validation**:
```bash
cargo test --workspace
cargo run -- --help  # CLI still works
cargo run &; sleep 3; curl http://localhost:3000/api/health; kill %1  # API still serves
```

**Rules**:
- `installed_departments()` returns `Vec<Box<dyn DepartmentApp>>`
- Dependency order: forge first, then code, then content/harvest/flow, then stubs
- All ports created before any department registers
- seed_defaults() moved into department `seeds/` directories
- Keep `DepartmentRegistry` generated from manifests (for backward compat with frontend)

---

### Layer 2: Feature Builders

#### Agent F1: Deferred Tool Loading (P1)

**Role**: Implement deferred tool loading — split tools into always-include vs searchable,
add `tool_search` built-in tool.

**Scope**: `rusvel-tool`, `rusvel-builtin-tools`, `rusvel-agent`.

**Must NOT touch**: Engine crates, API, frontend, core port traits.

**Inputs**:
- `docs/plans/next-level-proposals.md` (P1 section)
- `crates/rusvel-tool/src/lib.rs` (ToolRegistry)
- `crates/rusvel-agent/src/lib.rs` (AgentRuntime — where tools are injected into LLM request)
- `crates/rusvel-builtin-tools/src/` (existing tools)

**Outputs**:
- `crates/rusvel-tool/src/lib.rs` — add `searchable: bool` to `ToolDefinition`, add `search()` method
- `crates/rusvel-builtin-tools/src/tool_search.rs` — new `tool_search` tool
- `crates/rusvel-agent/src/lib.rs` — filter tools: inject only `searchable=false` + `tool_search` into prompt
- Tests for search matching, tool filtering

**Validation**:
```bash
cargo test -p rusvel-tool
cargo test -p rusvel-builtin-tools
cargo test -p rusvel-agent
```

**Rules**:
- Default `searchable = false` (backward compatible — existing tools stay in prompt)
- `tool_search` returns JSON array of matching `ToolDefinition`s
- Search by name substring + description keyword match
- Agent loop: when LLM calls `tool_search`, inject returned tools into next request
- ~150 lines total

**Prompt template**:
```
You are implementing P1: Deferred Tool Loading for RUSVEL.

Goal: 85% token savings by not injecting all tools into every LLM prompt.

Read docs/plans/next-level-proposals.md (P1 section) for context.
Read crates/rusvel-tool/src/lib.rs for ToolRegistry.
Read crates/rusvel-agent/src/lib.rs for AgentRuntime tool injection.
Read crates/rusvel-builtin-tools/src/lib.rs for existing tools.

Changes:
1. Add `searchable: bool` field to ToolDefinition (default false)
2. Create crates/rusvel-builtin-tools/src/tool_search.rs:
   - Tool name: "tool_search"
   - Params: { query: String }
   - Returns: matching ToolDefinition array (name + description + schema)
   - Search: case-insensitive substring on name + keyword match on description
3. In AgentRuntime::run_streaming(), when building LlmRequest:
   - Include only tools where searchable == false
   - Always include tool_search
   - When agent calls tool_search and gets results, add those tools to next LlmRequest

Tests:
- tool_search finds tools by name substring
- tool_search finds tools by description keyword
- AgentRuntime excludes searchable tools from initial prompt
- AgentRuntime includes found tools in subsequent requests

Run: cargo test -p rusvel-tool && cargo test -p rusvel-builtin-tools && cargo test -p rusvel-agent
```

---

#### Agent F2: Smart Model Routing (P12)

**Role**: Implement model tier routing — Haiku for simple, Sonnet for standard, Opus for complex.

**Scope**: `rusvel-core` (domain types only), `rusvel-llm`.

**Must NOT touch**: Engine crates, API handlers, frontend.

**Inputs**:
- `docs/plans/next-level-proposals.md` (P12 section)
- `crates/rusvel-core/src/domain.rs` (AgentConfig, ModelRef)
- `crates/rusvel-llm/src/` (providers, MultiProvider)

**Outputs**:
- `crates/rusvel-core/src/domain.rs` — add `ModelTier` enum, `complexity_hint` to `AgentConfig`
- `crates/rusvel-llm/src/router.rs` (new) — routing logic: complexity_hint → model selection
- Tests for tier mapping, fallback behavior

**Validation**:
```bash
cargo test -p rusvel-core
cargo test -p rusvel-llm
cargo check --workspace
```

**Rules**:
- `ModelTier`: Fast (Haiku), Balanced (Sonnet), Powerful (Opus)
- `ComplexityHint`: Simple, Standard, Complex
- If `AgentConfig.model` is explicitly set, skip routing (respect user override)
- If `complexity_hint` is None, default to Balanced
- Provider mapping is configurable (not hardcoded to Claude model names)
- ~150 lines total

---

#### Agent F3: Agent Hooks (SDK Feature 1)

**Role**: Implement PreToolUse/PostToolUse hooks in the agent runtime.

**Scope**: `rusvel-agent`.

**Must NOT touch**: Tool registry, engines, API.

**Inputs**:
- `docs/plans/agent-sdk-features.md` (Feature 1)
- `crates/rusvel-agent/src/lib.rs` (AgentRuntime tool call loop)

**Outputs**:
- `crates/rusvel-agent/src/hooks.rs` — `ToolHook` trait, `HookRegistry`, `HookDecision` enum
- `crates/rusvel-agent/src/lib.rs` — inject hook check before `self.tools.call()`
- Tests: hook blocks tool, hook modifies args, hook allows

**Validation**: `cargo test -p rusvel-agent`

**Rules**:
- `HookDecision`: Allow, Deny(reason), Modify(new_args)
- Hooks run in registration order, first Deny wins
- Default: no hooks registered (backward compatible)
- Hook check is async (hooks may call external services)
- Emit `AgentEvent::HookBlocked { tool, reason }` when denied
- ~120 lines total

---

#### Agent F4: Memory Tools (SDK Feature 3)

**Role**: Implement 4 agent-callable memory tools that wrap MemoryPort.

**Scope**: `rusvel-builtin-tools`.

**Must NOT touch**: MemoryPort trait, rusvel-memory adapter.

**Inputs**:
- `docs/plans/agent-sdk-features.md` (Feature 3)
- `crates/rusvel-core/src/ports.rs` (MemoryPort trait)
- `crates/rusvel-builtin-tools/src/lib.rs` (registration pattern)

**Outputs**:
- `crates/rusvel-builtin-tools/src/memory.rs` — 4 tools: `memory_read`, `memory_write`, `memory_search`, `memory_delete`
- Registration in `crates/rusvel-builtin-tools/src/lib.rs`
- Tests for each tool

**Validation**: `cargo test -p rusvel-builtin-tools`

**Rules**:
- Tools are session-scoped (use session_id from agent context)
- `memory_write` params: `{ key: String, content: String, tags: Vec<String> }`
- `memory_search` params: `{ query: String, limit?: u32 }`
- `memory_read` params: `{ id: String }` or `{ key: String }`
- `memory_delete` params: `{ id: String }`
- Tools need `Arc<dyn MemoryPort>` — passed during registration
- ~100 lines total

---

#### Agent F5: Context Compaction (SDK Feature 2)

**Role**: Auto-summarize conversation history when it exceeds a threshold.

**Scope**: `rusvel-agent`.

**Must NOT touch**: LlmPort trait, chat API handlers.

**Inputs**:
- `docs/plans/agent-sdk-features.md` (Feature 2)
- `crates/rusvel-agent/src/lib.rs` (message building in run_streaming)

**Outputs**:
- `crates/rusvel-agent/src/compaction.rs` — `ContextCompactor` struct
- Integration point in `AgentRuntime::run_streaming()` — compact before building LlmRequest
- Tests for compaction logic

**Validation**: `cargo test -p rusvel-agent`

**Rules**:
- Threshold: configurable, default 30 messages
- Keep: system prompt + last 10 messages + all tool results from current iteration
- Summarize: everything else into a single summary message
- Summary LLM call uses Fast tier (Haiku) — cheap
- Compacted messages tagged with `is_summary: true` for debugging
- ~100 lines total

---

#### Agent F6: Delegate Agent Tool (P9 Phase 1)

**Role**: Implement `delegate_agent` built-in tool — agents can spawn sub-agents.

**Scope**: `rusvel-builtin-tools`, `rusvel-agent`.

**Must NOT touch**: Flow engine, event bus, API.

**Inputs**:
- `docs/plans/agent-orchestration.md` (Phase 1)
- `docs/plans/next-level-proposals.md` (P9)
- `crates/rusvel-agent/src/lib.rs` (AgentRuntime)
- `crates/rusvel-builtin-tools/src/lib.rs` (registration pattern)

**Outputs**:
- `crates/rusvel-builtin-tools/src/delegate.rs` — `delegate_agent` tool
- Recursion depth tracking in `AgentRuntime`
- Tests: successful delegation, depth limit, budget tracking

**Validation**: `cargo test -p rusvel-builtin-tools && cargo test -p rusvel-agent`

**Rules**:
- Tool params: `{ persona: String, task: String, tools?: Vec<String>, max_iterations?: u32 }`
- Returns: sub-agent's text output as tool result
- Max recursion depth: 3 (configurable)
- Sub-agent inherits session_id but gets own run_id
- Budget tracking: sub-agent cost added to parent's cost_estimate
- If sub-agent fails, return error as tool result (don't crash parent)
- ~100 lines total

---

#### Agent F7: Hybrid RAG (P2)

**Role**: Implement BM25 + vector search fusion with RRF reranking.

**Scope**: `rusvel-memory`, `rusvel-vector`, `rusvel-api/knowledge.rs`.

**Must NOT touch**: MemoryPort/VectorStorePort traits, engine crates.

**Inputs**:
- `docs/plans/next-level-proposals.md` (P2)
- `crates/rusvel-memory/src/lib.rs` (FTS5 search)
- `crates/rusvel-vector/src/lib.rs` (LanceDB search)
- `crates/rusvel-api/src/knowledge.rs` (knowledge routes)

**Outputs**:
- `crates/rusvel-memory/src/hybrid.rs` (new) — `hybrid_search()` function, RRF fusion
- `crates/rusvel-api/src/knowledge.rs` — update search endpoint to use hybrid
- Tests for RRF score calculation, result merging

**Validation**: `cargo test -p rusvel-memory && cargo test -p rusvel-api`

**Rules**:
- RRF formula: `score = Σ(1 / (k + rank_i))` where k=60
- Query both FTS5 (BM25) and LanceDB (vector) in parallel (tokio::join!)
- Merge results by document ID, sum RRF scores
- Optional rerank: if > 20 results, call Haiku to reorder top 10
- Return top N with scores and source attribution
- ~200 lines total

---

#### Agent F8: Approval Workflow UI (P4)

**Role**: Build the frontend approval queue — the biggest UX gap.

**Scope**: Frontend only (`frontend/src/`).

**Must NOT touch**: Rust crates, API routes (they already exist).

**Inputs**:
- `docs/plans/next-level-proposals.md` (P4)
- Existing API: `GET /api/approvals`, `POST /api/approvals/{id}/approve`, `POST /api/approvals/{id}/reject`
- `frontend/src/lib/components/` (existing component patterns)

**Outputs**:
- `frontend/src/lib/components/approval/ApprovalQueue.svelte` — main component
- `frontend/src/lib/components/approval/ApprovalCard.svelte` — individual approval item
- `frontend/src/lib/components/shell/Sidebar.svelte` — approval badge count
- Integration in department panel and/or home dashboard

**Validation**:
```bash
cd frontend && pnpm check
```

**Rules**:
- Poll `GET /api/approvals` every 10 seconds (or use SSE if available)
- Each approval shows: job type, department, payload summary, created_at
- Approve button: `POST /api/approvals/{id}/approve` → remove from queue
- Reject button: `POST /api/approvals/{id}/reject` with optional reason
- Badge in sidebar shows pending count
- Use existing UI patterns (Tailwind, design tokens)
- ~300 lines Svelte total

---

#### Agent F9: Durable Execution (P8)

**Role**: Add checkpoint/resume to FlowEngine for crash-resilient workflows.

**Scope**: `flow-engine`.

**Must NOT touch**: rusvel-core domain types (use metadata), other engines.

**Inputs**:
- `docs/plans/next-level-proposals.md` (P8)
- `crates/flow-engine/src/lib.rs` (FlowEngine executor)
- `crates/rusvel-core/src/domain.rs` (FlowExecution, FlowNodeResult)

**Outputs**:
- `crates/flow-engine/src/checkpoint.rs` (new) — checkpoint persistence, resume logic
- `crates/flow-engine/src/lib.rs` — integrate checkpoint after each node
- Tests: checkpoint save, resume from checkpoint, skip completed nodes

**Validation**: `cargo test -p flow-engine`

**Rules**:
- After each node completes, persist checkpoint to ObjectStore
- Checkpoint key: `flow_checkpoint:{execution_id}:{node_id}`
- On resume: load checkpoints, skip nodes with status=Completed, start from first Pending
- Per-node retry: max_retries in FlowNodeDef.metadata, exponential backoff
- Approval nodes: checkpoint as AwaitingApproval, resume when approved via job queue
- ~250 lines total

---

#### Agent F10: Event Triggers (P9 Phase 3)

**Role**: Implement event-driven triggers — events auto-spawn agent runs or flows.

**Scope**: `rusvel-event`.

**Must NOT touch**: Engine crates, API routes.

**Inputs**:
- `docs/plans/agent-orchestration.md` (Phase 3)
- `docs/plans/next-level-proposals.md` (P9)
- `crates/rusvel-event/src/lib.rs` (EventBus)

**Outputs**:
- `crates/rusvel-event/src/triggers.rs` (new) — `TriggerManager`, `EventTrigger`, pattern matching
- `crates/rusvel-event/src/lib.rs` — wire TriggerManager into EventBus.emit()
- Tests: exact match, prefix match, filter match, action dispatch

**Validation**: `cargo test -p rusvel-event`

**Rules**:
- `EventTrigger { id, name, pattern, action, enabled }`
- Patterns: Exact(kind), Prefix(prefix), Filter(kind + field + value)
- Actions: StartAgent(persona, prompt), StartFlow(flow_id, input), EmitEvent(kind, payload), EnqueueJob(kind, payload)
- TriggerManager runs as background task, subscribes to EventBus
- Triggers stored in ObjectStore (CRUD via API later)
- Rate limiting: max 1 trigger fire per kind per 60 seconds (configurable)
- ~200 lines total

---

#### Agent F11: Self-Correction Loop (P5)

**Role**: Add critique step after agent completion — auto-evaluate and generate fix rules.

**Scope**: `rusvel-agent`.

**Must NOT touch**: Engine crates, frontend.

**Inputs**:
- `docs/plans/next-level-proposals.md` (P5)
- `docs/plans/agent-sdk-features.md` (Feature 6 — verification)
- `crates/rusvel-agent/src/lib.rs` (AgentRuntime)

**Outputs**:
- `crates/rusvel-agent/src/critique.rs` (new) — `CritiqueAgent`, evaluation dimensions
- `crates/rusvel-agent/src/lib.rs` — optional critique after `Done` event
- Tests: critique scores, rule generation, threshold behavior

**Validation**: `cargo test -p rusvel-agent`

**Rules**:
- Critique is optional (enabled per AgentConfig or department config)
- Uses Fast tier (Haiku) — cheap evaluation
- Evaluation dimensions: accuracy, relevance, tone, completeness
- If score < threshold (default 0.7), emit `AgentEvent::CritiqueResult` with suggestions
- Auto-generated rules stored via ObjectStore if `auto_generate_rules: true`
- ~200 lines total

**Dependencies**: F2 (model routing for Haiku selection)

---

### Layer 3: Integration Builders

#### Agent I1: Boot Integration

**Role**: After all department migrations (B2-B4) and boot builder (B5) are done,
validate the full system works end-to-end.

**Scope**: Integration testing only. No code changes.

**Tests**:
```bash
# Full workspace compiles
cargo check --workspace

# All tests pass
cargo test --workspace

# Binary starts and serves API
cargo run &
sleep 3
curl -s http://localhost:3000/api/health | jq .
curl -s http://localhost:3000/api/departments | jq '.[].id'
curl -s http://localhost:3000/api/dept/content/chat -X POST -H 'Content-Type: application/json' -d '{"message":"test"}'
kill %1

# CLI works
cargo run -- --help
cargo run -- session create test-session

# Frontend builds
cd frontend && pnpm build
```

---

#### Agent I2: Frontend Shell Alignment

**Role**: ADR-014 Step 7 — update frontend to read from manifests instead of hardcoded DepartmentDef.

**Scope**: Frontend only.

**Must NOT touch**: Rust crates.

**Inputs**:
- New `GET /api/departments` response (manifest-based, from B5)
- `frontend/src/lib/stores/departments.ts`
- `frontend/src/routes/dept/[id]/+page.svelte`
- `frontend/src/lib/components/department/DepartmentPanel.svelte`

**Outputs**:
- Updated stores and components to consume manifest format
- Dashboard cards from `manifest.ui.dashboard_cards`
- Custom component lazy loading from `manifest.ui.custom_components`

**Validation**: `cd frontend && pnpm check && pnpm build`

---

## Execution Order & Dependency Graph

```
Phase 1: Foundation (can start immediately)
├── B1: Core Contract ──────────────────────────────┐
├── F1: Deferred Tool Loading (independent)         │
├── F2: Smart Model Routing (independent)           │
├── F3: Agent Hooks (independent)                   │
├── F4: Memory Tools (independent)                  │
└── F5: Context Compaction (independent)            │
                                                    │
Phase 2: Migration (after B1 completes)             │
├── B2: Content Dept Migration ◄────────────────────┘
├── B3: Forge Dept Migration ◄──────────────────────┘
│   (B2 and B3 can run in parallel)
├── B4: Remaining 10 Depts ◄─── (after B2+B3 validate the pattern)
│   (all 10 can run in parallel, each in own worktree)
└── F8: Approval UI (independent, parallel with migrations)

Phase 3: Orchestration (after F3, F4, F6 complete)
├── F6: Delegate Agent ◄──── (needs agent hooks for safety)
├── F7: Hybrid RAG (independent)
├── F9: Durable Execution (independent)
└── F10: Event Triggers (independent)

Phase 4: Intelligence (after F2, F6 complete)
├── F11: Self-Correction ◄──── (needs model routing for Haiku)
└── (F6 enables orchestration patterns for F11)

Phase 5: Integration (after all migrations + features)
├── B5: Boot Sequence ◄──── (needs all dept-* crates)
├── I1: Boot Integration ◄──── (needs B5)
└── I2: Frontend Shell ◄──── (needs B5 serving manifests)
```

### Parallelization Matrix

```
Week 1:  B1 ────────────  F1 ──  F2 ──  F3 ──  F4 ──  F5 ──
Week 2:  B2 ────  B3 ────  F8 ──────────────────────────────
Week 3:  B4 (×10, parallel) ──────────────────────────────────
Week 4:  F6 ──  F7 ──  F9 ──  F10 ──────────────────────────
Week 5:  F11 ──  B5 ──────────────────────────────────────────
Week 6:  I1 ──  I2 ──────────────────────────────────────────
```

**Maximum parallel agents**: 10 (during B4 when all stub departments migrate simultaneously)
**Typical parallel agents**: 3-5

---

## Agent Prompt Structure (Template)

Every builder agent receives this structure:

```markdown
# Role
You are the {{agent_name}} for RUSVEL.

# Task
{{task_description}}

# Context Files (read these first)
{{list of files to read}}

# Output Files (create/modify these)
{{list of files to produce}}

# Rules
{{numbered list of constraints}}

# Validation
{{exact commands to run after writing code}}

# Anti-Rules (what you must NOT do)
- Do NOT touch files outside your scope
- Do NOT modify existing port traits in rusvel-core
- Do NOT add dependencies on adapter crates from engine/dept crates
- Do NOT remove backward-compatible code without explicit instruction
- Do NOT add comments to code you didn't write
- Do NOT add error handling for impossible scenarios
```

---

## Validation Protocol

Every agent's work is validated at three levels:

### Level 1: Compile Check
```bash
cargo check --workspace  # All crates compile
```

### Level 2: Test Suite
```bash
cargo test --workspace   # All tests pass (existing + new)
```

### Level 3: Integration Smoke Test
```bash
cargo run &
sleep 3
# Health check
curl -sf http://localhost:3000/api/health
# Department list
curl -sf http://localhost:3000/api/departments | jq 'length'
# Chat works
curl -sf http://localhost:3000/api/dept/forge/chat \
  -X POST -H 'Content-Type: application/json' \
  -d '{"message":"hello","session_id":"test"}' \
  --max-time 10
kill %1
```

### Level 4: Human Review
- Code review of each agent's diff
- Merge only after validation passes
- Resolve conflicts between parallel agents

---

## Cost Estimate

Assuming Claude Opus for architect, Sonnet for builders:

| Agent | Model | Est. Tokens | Est. Cost |
|-------|-------|-------------|-----------|
| A0 (Architect) | Opus | ~50K | ~$0.75 |
| B1 (Core Contract) | Sonnet | ~100K | ~$0.90 |
| B2-B3 (Content/Forge) | Sonnet | ~80K each | ~$1.44 |
| B4 (10 depts) | Sonnet | ~40K each | ~$3.60 |
| F1-F11 (Features) | Sonnet | ~60K each | ~$5.94 |
| I1-I2 (Integration) | Sonnet | ~30K each | ~$0.54 |
| **Total** | | ~**990K** | ~**$13.17** |

---

## How This Maps to RUSVEL's Own Agent System

This design IS the blueprint for RUSVEL's production agent orchestration:

| Workforce Pattern | RUSVEL Runtime Equivalent |
|-------------------|--------------------------|
| Agent A0 (Architect) | ForgeEngine `mission_today()` with delegate_agent |
| Agent B* (Builders) | Department-scoped agents with persona + tools + rules |
| Isolated worktrees | Session-scoped agent runs with own memory namespace |
| Validation protocol | PostToolUse hooks + verification loops (SDK Feature 6) |
| Dependency graph | FlowEngine DAG with dependency edges |
| Human review | ApprovalStatus gates (ADR-008) |
| Parallel execution | Job queue with worker pool |
| Cost tracking | CostTracker (P12) per run |

When RUSVEL ships delegate_agent (P9), event triggers (P9 Phase 3), and
durable execution (P8), these 14 agents can be defined as workflow templates
and run autonomously — with human approval gates at merge points.

---

## Quick Start

To run the first sprint:

```bash
# 1. Human reviews and approves sprint plan
# 2. Launch parallel agents (in Claude Code):

# Core contract (blocks migrations)
# → Agent B1 in worktree

# Independent features (run in parallel)
# → Agent F1: Deferred Tool Loading
# → Agent F2: Smart Model Routing
# → Agent F3: Agent Hooks
# → Agent F4: Memory Tools
# → Agent F5: Context Compaction

# After B1 merges:
# → Agent B2: Content Dept Migration
# → Agent B3: Forge Dept Migration

# After B2+B3 merge:
# → Agent B4: Remaining 10 Depts (10 parallel worktrees)
# → Agent F8: Approval UI (parallel)
```

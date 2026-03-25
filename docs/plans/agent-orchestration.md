# Agent Orchestration — Agents Controlling Agents

> Hierarchical agent composition: delegate, chain, and orchestrate autonomous multi-agent pipelines with predefined workflows, rules, skills, and search.
>
> Date: 2026-03-25
> Status: Proposed
> Depends on: ADR-014 (Department-as-App), Agent SDK Features, Next-Level Proposals (P9)
> Related:
> - `docs/design/department-as-app.md` — DepartmentApp contract, manifest, registration
> - `docs/design/agent-workforce.md` — 14 sub-agents blueprint (dogfooding this pattern)
> - `docs/plans/agent-sdk-features.md` — SDK features 1-9 (hooks, handoffs, memory, compaction)
> - `docs/plans/next-level-proposals.md` — P9 (delegate_agent), P5 (self-correction), P8 (durable execution)
> - `docs/plans/next-level-inspiration-2026-03-25.md` — Playbooks, Roundtable UI
> - `docs/plans/flow-engine.md` — DAG workflow engine spec

---

## Motivation

RUSVEL has single-agent execution (AgentRuntime), DAG workflows (FlowEngine), and 10 personas (ForgeEngine). But agents can't spawn other agents, events don't trigger agents, and there's no way to define reusable multi-agent pipelines. The goal: one human triggers a workflow, agents handle the rest autonomously — planning, building, testing, reviewing, reporting — each with their own tools, rules, and skills.

This plan is the **runtime counterpart** to the Agent Workforce design (`docs/design/agent-workforce.md`). The workforce doc defines 14 sub-agents for building RUSVEL itself — this plan defines the primitives that make those agents possible as first-class RUSVEL features.

---

## What We Already Have

| Component | Location | Status |
|-----------|----------|--------|
| AgentRuntime (tool-use loop, streaming, 10 iterations) | `rusvel-agent/src/lib.rs` | Working |
| Workflow patterns (Sequential, Parallel, Loop) | `rusvel-agent/src/workflow.rs` | Working |
| Flow Engine (petgraph DAG, 3 node types) | `flow-engine/src/` | Working |
| 10 Personas (CodeWriter, Reviewer, Tester, etc.) | `forge-engine/src/personas.rs` | Working |
| Event Bus (broadcast + persistence) | `rusvel-event/src/lib.rs` | Working |
| Built-in Tools (file, shell, git) | `rusvel-builtin-tools/src/` | Working |
| Tool Registry + JSON Schema | `rusvel-tool/src/` | Working |
| Skills (resolve + `{{input}}` interpolation) | `rusvel-api/src/skills.rs` | Working |
| Rules (load per engine, inject into system prompt) | `rusvel-api/src/rules.rs` | Working |
| Job Queue (central, async) | `rusvel-jobs/src/` | Working |

### In-Progress (ADR-014 migration)

| Component | Location | Status |
|-----------|----------|--------|
| DepartmentApp trait + manifest | `rusvel-core/src/department/` | **In progress** |
| dept-content (DepartmentApp migration) | `crates/dept-content/` | **In progress** |
| dept-forge (DepartmentApp migration) | `crates/dept-forge/` | **In progress** |
| Flow builder UI | `frontend/src/routes/flows/` | **In progress** |

---

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    ORCHESTRATION LAYER                         │
│                                                                │
│  ┌─────────────────────────────────────────────────────────┐  │
│  │              Orchestrator Agent                          │  │
│  │  (God Agent / Forge persona / custom)                   │  │
│  │  Tools: delegate_agent, invoke_flow, send_message, ...  │  │
│  └──────────────────────┬──────────────────────────────────┘  │
│                          │                                     │
│         ┌────────────────┼────────────────┐                   │
│         ▼                ▼                ▼                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐           │
│  │ Plan Agent  │  │ Build Agent │  │ Test Agent  │  → ...    │
│  │ (Architect) │  │ (CodeWriter)│  │ (Tester)    │           │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘           │
│         │emit            │emit            │emit               │
├──────────────────────────────────────────────────────────────┤
│  Event Bus — completion events trigger next step              │
│  + TriggerManager (event pattern → action)                    │
├──────────────────────────────────────────────────────────────┤
│  Flow Engine — DAG orchestration for complex graphs           │
│  + Durable execution (checkpoint/resume, P8)                  │
├──────────────────────────────────────────────────────────────┤
│  Job Queue — async execution + approval gates (ADR-008)       │
├──────────────────────────────────────────────────────────────┤
│  PLATFORM: AgentRuntime + ToolRegistry + MemoryPort + LLM    │
│  + PreToolUse/PostToolUse hooks (SDK Feature 1)               │
│  + Context compaction (SDK Feature 2)                         │
│  + Memory tools (SDK Feature 3)                               │
│  + Hierarchical permissions (SDK Feature 5)                   │
├──────────────────────────────────────────────────────────────┤
│  KERNEL: rusvel-core ports + DepartmentApp contract           │
└──────────────────────────────────────────────────────────────┘
```

### Three Orchestration Modes

**Mode 1: Inline Delegation (synchronous)**
An orchestrator agent uses the `delegate_agent` tool within its tool-use loop. It decides what to delegate, to whom, and what to do with the result — all within a single agent run.

```
Orchestrator calls delegate_agent("CodeWriter", "implement X")
  → CodeWriter runs, returns code
Orchestrator calls delegate_agent("Tester", "test this code: ...")
  → Tester runs, returns test results
Orchestrator calls delegate_agent("Reviewer", "review code + tests")
  → Reviewer runs, returns review
Orchestrator synthesizes final report
```

**Mode 2: Event-Driven Pipeline (asynchronous)**
A flow definition chains agents via events. Each agent runs independently, emits a completion event, which triggers the next agent. Enhanced with durable execution (P8) for crash resilience.

```
Flow: plan → build → test → review → report
  - "plan" AgentNode completes → checkpoint → event "flow.node.completed.plan"
  - Flow engine advances to "build" node
  - "build" completes → checkpoint → event → "test" node starts
  - If "test" fails → condition node routes to "fix" branch
  - "fix" agent runs → loops back to "test" (max 3 retries)
  - All pass → "report" node generates summary
  - Crash at any point → resume from last checkpoint
```

**Mode 3: Cross-Department Handoffs (async, multi-department)**
Agents in one department delegate to agents in another via the event bus. Per SDK Feature 4 (Multi-Agent Handoffs):

```
Harvest discovers opportunity → emits "harvest.opportunity.qualified"
  → TriggerManager matches → StartAgent("ContentWriter", "Write case study for: {{payload}}")
  → Content agent completes → emits "content.drafted"
  → TriggerManager matches → StartAgent("CodeWriter", "Create project scaffold for: {{payload}}")
```

This mode aligns with ADR-014's principle: departments communicate via events, never direct imports.

---

## Implementation

### Phase 1: Agent Infrastructure (Prerequisites)

These are defined in `docs/plans/agent-sdk-features.md` and must land first:

| Feature | What | Where | Ref |
|---------|------|-------|-----|
| **PreToolUse/PostToolUse Hooks** | Intercept + validate tool calls before execution | `rusvel-agent/src/hooks.rs` | SDK Feature 1 |
| **Context Compaction** | Auto-summarize when messages > threshold | `rusvel-agent/src/compaction.rs` | SDK Feature 2 |
| **Memory Tools** | `memory_read/write/search/delete` as agent tools | `rusvel-builtin-tools/src/memory.rs` | SDK Feature 3 |
| **Hierarchical Permissions** | `auto/supervised/locked` permission modes per dept | `rusvel-tool/src/lib.rs` | SDK Feature 5 |

These enable safe, context-aware, memory-equipped agents — the foundation for delegation.

### Phase 2: `delegate_agent` Tool (the core primitive)

**Ref:** P9 Phase 1, Agent Workforce Agent F6

Add to `rusvel-builtin-tools/src/`:

```rust
// delegate.rs — Agent delegation tool

pub fn tool_def() -> ToolDef {
    ToolDef {
        name: "delegate_agent".into(),
        description: "Delegate a task to a specialized sub-agent. The sub-agent runs \
                      to completion and returns its output.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "persona": {
                    "type": "string",
                    "description": "Agent persona to delegate to"
                },
                "task": {
                    "type": "string",
                    "description": "What the sub-agent should accomplish"
                },
                "context": {
                    "type": "string",
                    "description": "Additional context (file contents, prior outputs, etc.)"
                },
                "department": {
                    "type": "string",
                    "description": "Target department (for cross-dept delegation). Defaults to current."
                },
                "tools": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Tool allowlist (optional, defaults to persona's tools)"
                },
                "rules": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Additional rules injected into sub-agent's system prompt"
                },
                "max_iterations": {
                    "type": "integer",
                    "description": "Max tool-use iterations (default: 10)"
                },
                "wait": {
                    "type": "boolean",
                    "description": "Wait for result (true) or fire-and-forget via job queue (false). Default: true"
                }
            },
            "required": ["persona", "task"]
        }),
    }
}
```

**Key design decisions:**
- Sub-agent gets its own AgentConfig (persona instructions, tool allowlist, rules)
- `department` param enables cross-department delegation (SDK Feature 4)
- `wait: false` enqueues to job queue for async handoffs
- `rules` param lets parent inject context-specific rules into sub-agent
- Parent agent sees the full output + metadata about what happened
- Sub-agent runs in the same session (shares memory namespace)
- Recursion depth tracked in AgentRuntime (max 3 levels)
- Budget tracking: sub-agent cost added to parent's cost_estimate
- PreToolUse hook (SDK Feature 1) can gate dangerous delegations

**Integration with DepartmentApp (ADR-014):**
When departments are migrated, `delegate_agent` resolves the target department's persona from its manifest:
```rust
// In delegate.rs execute():
let dept = department_registry.get(dept_id)?;
let manifest = dept.manifest();
let persona = manifest.personas.iter().find(|p| p.name == persona_name)?;
```

### Phase 3: `invoke_flow` Tool

**Ref:** P9 Phase 2

Let agents trigger DAG flows from within their tool-use loop:

```rust
pub fn tool_def() -> ToolDef {
    ToolDef {
        name: "invoke_flow".into(),
        description: "Trigger a predefined workflow (flow) and wait for results.".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "flow_id": { "type": "string", "description": "Flow ID to execute" },
                "flow_name": { "type": "string", "description": "Flow name (alternative to ID)" },
                "input": { "type": "object", "description": "Input data for the flow" },
                "wait": { "type": "boolean", "description": "Wait for completion (default: true)" }
            },
            "required": ["input"]
        }),
    }
}
```

With durable execution (P8), flows survive crashes. With checkpointing, the agent can poll for progress.

### Phase 4: Event-Driven Triggers

**Ref:** P9 Phase 3, Agent Workforce Agent F10

Extend `rusvel-event` with trigger subscriptions:

```rust
// New: EventTrigger — watches for patterns and starts agents/flows

pub struct EventTrigger {
    pub id: TriggerId,
    pub name: String,
    pub pattern: EventPattern,       // What events to match
    pub action: TriggerAction,       // What to do when matched
    pub enabled: bool,
    pub rate_limit_secs: u64,        // Max 1 fire per kind per N seconds
    pub department_id: Option<String>, // Scoped to department (ADR-014)
}

pub enum EventPattern {
    Exact { kind: String },                                    // "agent.run.completed"
    Prefix { prefix: String },                                 // "agent.run.*"
    Filter { kind: String, field: String, value: Value },      // kind == X AND payload.persona == "CodeWriter"
    Compound { all: Vec<EventPattern> },                       // All patterns must match
}

pub enum TriggerAction {
    StartAgent { persona: String, prompt_template: String, department: Option<String> },
    StartFlow { flow_id: FlowId, input_mapping: Value },
    EmitEvent { kind: String, payload_template: Value },
    EnqueueJob { job_kind: String, payload: Value },
}
```

**TriggerManager** runs as a background task:

```rust
impl TriggerManager {
    pub async fn run(&self, mut rx: broadcast::Receiver<Event>) {
        while let Ok(event) = rx.recv().await {
            for trigger in self.triggers.read().await.values() {
                if trigger.enabled && trigger.pattern.matches(&event) {
                    if self.rate_limiter.allow(&trigger.id).await {
                        self.execute_action(&trigger.action, &event).await;
                    }
                }
            }
        }
    }
}
```

**Integration with DepartmentApp (ADR-014):**
Departments declare triggers in their manifest:
```rust
// In DepartmentManifest
pub events_produced: Vec<String>,   // What this dept emits
pub events_consumed: Vec<String>,   // What this dept listens to
```
TriggerManager auto-registers triggers from manifests at boot time.

### Phase 5: Workflow Templates (Playbooks)

**Ref:** P11, next-level-inspiration Playbooks

Stored as JSON, loadable via API, combinable with rules and skills. These are the user-facing "Playbooks" described in the inspiration doc — named, versioned, shareable multi-step recipes.

```json
{
    "name": "autonomous-code-pipeline",
    "description": "Plan → Build → Test → Review → Report",
    "version": "1.0.0",
    "trigger": { "kind": "manual" },
    "departments": ["code", "forge"],
    "variables": {
        "project_context": "",
        "quality_bar": "production"
    },
    "steps": [
        {
            "id": "plan",
            "persona": "Architect",
            "department": "forge",
            "prompt": "Analyze the request and create a detailed implementation plan.\n\nRequest: {{input.request}}\nContext: {{variables.project_context}}",
            "tools": ["file_read", "shell_execute", "delegate_agent"],
            "rules": ["Always consider edge cases", "Prefer simple solutions"],
            "output_key": "plan"
        },
        {
            "id": "build",
            "persona": "CodeWriter",
            "department": "code",
            "prompt": "Implement the following plan:\n\n{{steps.plan.output}}",
            "tools": ["file_read", "file_write", "shell_execute", "git"],
            "skills": ["rust-idiomatic-code", "error-handling-patterns"],
            "depends_on": ["plan"],
            "output_key": "code"
        },
        {
            "id": "test",
            "persona": "Tester",
            "department": "code",
            "prompt": "Write and run tests for:\n\n{{steps.build.output}}",
            "tools": ["file_read", "file_write", "shell_execute"],
            "depends_on": ["build"],
            "output_key": "test_results",
            "on_failure": {
                "action": "delegate",
                "persona": "Debugger",
                "prompt": "Fix these test failures:\n\n{{steps.test.error}}",
                "retry_step": "test",
                "max_retries": 3
            }
        },
        {
            "id": "review",
            "persona": "Reviewer",
            "department": "forge",
            "prompt": "Review code quality and security:\n\nPlan: {{steps.plan.output}}\nCode: {{steps.build.output}}\nTests: {{steps.test.output}}",
            "tools": ["file_read"],
            "rules": ["Flag any security issues", "Check test coverage"],
            "depends_on": ["test"],
            "output_key": "review"
        },
        {
            "id": "report",
            "persona": "Documenter",
            "department": "forge",
            "prompt": "Summarize what was accomplished:\n\n{{steps.plan.output}}\n{{steps.review.output}}",
            "tools": ["file_write"],
            "depends_on": ["review"],
            "output_key": "report"
        }
    ]
}
```

**Template → FlowDef conversion:** Templates compile down to FlowEngine DAGs. Each step becomes an AgentNode. `depends_on` becomes edges. `on_failure` becomes condition nodes with retry loops. This reuses the existing flow-engine petgraph executor rather than creating a parallel execution engine.

**Additional template examples:**

```json
{
    "name": "research-and-answer",
    "description": "Research → Draft → Validate → Respond with best answer",
    "departments": ["forge"],
    "steps": [
        {
            "id": "research",
            "persona": "Researcher",
            "prompt": "Search for the best possible answer to: {{input.question}}",
            "tools": ["knowledge_search", "memory_search", "code_search", "file_read"],
            "skills": ["deep-research"],
            "output_key": "research"
        },
        {
            "id": "draft",
            "persona": "ContentWriter",
            "prompt": "Draft a clear, accurate answer based on:\n\n{{steps.research.output}}",
            "depends_on": ["research"],
            "output_key": "draft"
        },
        {
            "id": "validate",
            "persona": "Reviewer",
            "prompt": "Fact-check and validate:\n\n{{steps.draft.output}}\n\nAgainst research:\n{{steps.research.output}}",
            "depends_on": ["draft"],
            "output_key": "validation",
            "on_failure": {
                "action": "retry_step",
                "step": "draft",
                "prompt_append": "\n\nFix these issues: {{steps.validate.error}}",
                "max_retries": 2
            }
        },
        {
            "id": "respond",
            "persona": "ContentWriter",
            "prompt": "Finalize the answer incorporating feedback:\n\n{{steps.draft.output}}\n{{steps.validate.output}}",
            "depends_on": ["validate"],
            "output_key": "final_answer"
        }
    ]
}
```

```json
{
    "name": "cross-department-opportunity",
    "description": "Harvest finds opportunity → Content writes case study → Code scaffolds project",
    "departments": ["harvest", "content", "code"],
    "trigger": { "kind": "event", "pattern": "harvest.opportunity.qualified" },
    "steps": [
        {
            "id": "analyze",
            "persona": "Researcher",
            "department": "harvest",
            "prompt": "Deep-dive on this opportunity:\n\n{{trigger.payload}}",
            "tools": ["knowledge_search", "file_read"],
            "output_key": "analysis"
        },
        {
            "id": "case_study",
            "persona": "ContentWriter",
            "department": "content",
            "prompt": "Write a case study based on:\n\n{{steps.analyze.output}}",
            "tools": ["file_write"],
            "depends_on": ["analyze"],
            "output_key": "case_study"
        },
        {
            "id": "scaffold",
            "persona": "CodeWriter",
            "department": "code",
            "prompt": "Create project scaffold for:\n\n{{steps.analyze.output}}",
            "tools": ["file_write", "shell_execute", "git"],
            "depends_on": ["analyze"],
            "output_key": "scaffold"
        }
    ]
}
```

### Phase 6: Scoped Context per Sub-Agent

Each sub-agent gets isolated but inheritable context:

```rust
pub struct AgentScope {
    pub persona: PersonaConfig,
    pub department_id: String,        // Department context (ADR-014)
    pub tools: Vec<String>,           // Allowlisted tools
    pub permission_mode: ToolPermissionMode, // auto/supervised/locked (SDK Feature 5)
    pub rules: Vec<Rule>,             // Injected into system prompt
    pub skills: Vec<Skill>,           // Available skill templates
    pub memory_namespace: String,     // Isolated memory scope
    pub parent_run_id: Option<RunId>, // For tracing delegation chains
    pub depth: u8,                    // Current nesting depth (max 3)
    pub budget: TokenBudget,          // Token/cost limit for this agent
    pub model_tier: ModelTier,        // Fast/Balanced/Powerful (P12)
}
```

**Integration with Self-Correction (P5):** After a sub-agent completes, the CritiqueAgent (SDK Feature 6) can evaluate output quality. If score < threshold, the orchestrator auto-retries with the critique feedback appended.

### Phase 7: Search & Knowledge Integration

Sub-agents can search for the best answer using existing infrastructure, enhanced by Hybrid RAG (P2):

```rust
// Tools available to agents for knowledge retrieval:

"knowledge_search"  → VectorStorePort + FTS5 hybrid (P2: RRF fusion + reranking)
"memory_search"     → MemoryPort (FTS5 search over session memory)
"memory_read/write" → MemoryPort (cross-session persistence, SDK Feature 3)
"code_search"       → CodeEngine (BM25 search over codebase)
"file_read"         → FileOps (read any file for context)
"tool_search"       → Deferred tool discovery (P1)
```

With P1 (Deferred Tool Loading), sub-agents start with only essential tools and discover specialized ones on demand — cutting token costs by ~85%.

---

## API Routes

```
# Workflow Templates / Playbooks
GET    /api/playbooks                    # List predefined workflow templates
POST   /api/playbooks                    # Create template
GET    /api/playbooks/{id}               # Get template
PUT    /api/playbooks/{id}               # Update template
DELETE /api/playbooks/{id}               # Delete template
POST   /api/playbooks/{id}/run           # Execute a template
GET    /api/playbooks/{id}/executions    # Execution history

# Event Triggers
GET    /api/triggers                     # List event triggers
POST   /api/triggers                     # Create trigger
PUT    /api/triggers/{id}                # Update trigger
DELETE /api/triggers/{id}                # Delete trigger
POST   /api/triggers/{id}/enable         # Enable trigger
POST   /api/triggers/{id}/disable        # Disable trigger

# Delegation Observability
GET    /api/runs/{id}/delegation-tree    # Full tree of parent → child agent runs
GET    /api/runs/{id}/stream             # SSE: real-time delegation progress
```

---

## Frontend Components

```
frontend/src/lib/components/orchestration/
├── PlaybookBuilder.svelte      # Visual template builder (drag personas into sequence)
├── PlaybookRunner.svelte       # Execute template, show live progress
├── DelegationTree.svelte       # Tree view of agent → sub-agent chains
├── TriggerManager.svelte       # Configure event triggers
├── PersonaPicker.svelte        # Select persona + configure tools/rules/dept
└── StepConfigPanel.svelte      # Configure individual pipeline step

# Roundtable UI (from next-level-inspiration)
frontend/src/lib/components/orchestration/
├── RoundtableView.svelte       # Multi-persona conversation display
└── RoundtableLauncher.svelte   # Pick topic + departments, start strategy review
```

**Integration with existing in-progress work:**
- `frontend/src/routes/flows/+page.svelte` — Flow builder UI already started, becomes the canvas for visual playbook editing
- `WorkflowBuilder.svelte` + `AgentNode.svelte` — Existing @xyflow components, extended with persona/dept/rules config panels

---

## Cross-Reference: How This Connects to Everything

| This Plan | Related Doc | Integration Point |
|-----------|-------------|-------------------|
| `delegate_agent` tool | Agent Workforce (F6) | Same implementation, workforce agents are the first users |
| Event triggers | Agent Workforce (F10) | Same implementation |
| Cross-dept delegation | ADR-014 (department-as-app) | `department` param resolves via manifest |
| Playbook templates | Inspiration doc (Priority 1) | Playbooks = templates + UI |
| Roundtable mode | Inspiration doc (Priority 5) | Multi-persona orchestration UI |
| Scoped permissions | SDK Features (Feature 5) | `permission_mode` per sub-agent |
| Memory tools | SDK Features (Feature 3) | Sub-agents persist + recall across runs |
| Self-correction | P5 (self-correction loop) | CritiqueAgent evaluates sub-agent output |
| Durable execution | P8 (durable execution) | Flows checkpoint after each agent node |
| Hybrid RAG | P2 (hybrid search) | Sub-agents get better search results |
| Deferred tools | P1 (deferred tool loading) | Sub-agents save tokens via tool_search |
| Smart routing | P12 (cost intelligence) | Sub-agents use appropriate model tier |
| PreToolUse hooks | SDK Feature 1 | Safety gates on delegation + dangerous tools |
| Approval UI | P4 (approval workflow) | Delegation chains can pause for approval |
| CDP browser bridge | `docs/plans/cdp-browser-bridge.md` | Sub-agents can browse + act on platforms |
| Native terminal | `docs/plans/native-terminal-multiplexer.md` | Sub-agents get isolated terminal panes |

---

## Implementation Priority

Aligned with Agent Workforce execution order (`docs/design/agent-workforce.md`):

| Phase | What | Where | Effort | Dependencies | Agent Workforce Ref |
|-------|------|-------|--------|--------------|---------------------|
| **1a** | PreToolUse/PostToolUse hooks | `rusvel-agent/src/hooks.rs` | ~120 lines | None | F3 |
| **1b** | Memory tools (4 tools) | `rusvel-builtin-tools/src/memory.rs` | ~100 lines | None | F4 |
| **1c** | Context compaction | `rusvel-agent/src/compaction.rs` | ~100 lines | None | F5 |
| **2** | **`delegate_agent` tool** | `rusvel-builtin-tools/src/delegate.rs` | ~100 lines | 1a (hooks for safety) | **F6** |
| 3 | Recursion depth guard | `rusvel-agent/src/lib.rs` | ~20 lines | 2 | F6 |
| 4 | `invoke_flow` tool | `rusvel-builtin-tools/src/flow.rs` | ~50 lines | 2 | — |
| 5 | Event trigger system | `rusvel-event/src/triggers.rs` | ~200 lines | None | **F10** |
| 6 | TriggerManager background task | `rusvel-app/src/main.rs` | ~30 lines | 5 | F10 |
| 7 | Durable execution (checkpoint/resume) | `flow-engine/src/checkpoint.rs` | ~250 lines | None | **F9** |
| 8 | Self-correction loop | `rusvel-agent/src/critique.rs` | ~200 lines | P12 (model routing) | **F11** |
| 9 | Playbook templates (JSON schema + storage) | `rusvel-core` + `rusvel-db` | ~200 lines | 2, 5 | — |
| 10 | Template → FlowDef compiler | `flow-engine/src/template.rs` | ~200 lines | 9, 7 | — |
| 11 | Scoped agent context | `rusvel-agent/src/scope.rs` | ~100 lines | 2, SDK Feature 5 | — |
| 12 | API routes (playbooks, triggers, delegation tree) | `rusvel-api` | ~200 lines | 9, 5 | — |
| 13 | Frontend (PlaybookBuilder, DelegationTree) | `frontend/src/` | ~500 lines | 12, flows UI | F8, I2 |

**Total: ~2,070 lines of new code across ~13 files**

---

## Design Principles

1. **Agents are composable** — any agent can delegate to any other agent via tool call
2. **Departments communicate via events** — no direct imports (ADR-014, ADR-005)
3. **Flows are composable** — flows can invoke sub-flows via `invoke_flow` node type
4. **Events are reactive** — any completion event can trigger the next step
5. **Templates are portable** — JSON definitions, shareable, versionable (= Playbooks)
6. **Depth is bounded** — max 3 levels of delegation prevents runaway recursion
7. **Budget is scoped** — each sub-agent has its own token/cost budget (P12)
8. **Everything is observable** — delegation trees, event trails, execution logs (AG-UI events, P7)
9. **Rules and skills travel** — each step can carry its own rules and skills
10. **Safety gates at every level** — PreToolUse hooks, approval gates, permission modes

---

## Example: Full Autonomous Code Pipeline

```
Human: "Add rate limiting to the API"

Orchestrator (God Agent, Forge dept):
  ├── delegate_agent("Architect", dept="forge", "Plan rate limiting for Axum API")
  │   └── Returns: implementation plan with middleware approach
  ├── delegate_agent("CodeWriter", dept="code", "Implement: {plan}", tools: [file_write, shell])
  │   └── Returns: code changes in 3 files
  ├── delegate_agent("Tester", dept="code", "Write tests for: {code}", tools: [file_write, shell])
  │   └── Returns: 5 tests, 4 pass, 1 fails
  │       → Self-correction (P5): CritiqueAgent scores 0.6 < 0.7 threshold
  ├── delegate_agent("Debugger", dept="code", "Fix failing test: {test_error}")
  │   └── Returns: fix applied
  ├── delegate_agent("Tester", dept="code", "Re-run tests")  ← retry
  │   └── Returns: 5/5 pass
  │       → Self-correction: CritiqueAgent scores 0.95 ✓
  ├── delegate_agent("SecurityAuditor", dept="forge", "Review rate limiting for bypass vulnerabilities")
  │   └── Returns: audit report, no issues
  └── delegate_agent("Documenter", dept="forge", "Summarize changes")
      └── Returns: changelog entry + API docs update
```

Total: 1 human message → 7 autonomous agent runs → complete feature with tests and docs.

**The same pipeline as a Playbook:** Save the above as a template, trigger it with `POST /api/playbooks/code-pipeline/run` with `{ "request": "Add rate limiting" }`. The FlowEngine executes the DAG with checkpointing. The UI shows real-time delegation tree progress. The approval queue catches anything flagged by hooks.

---

## How This Dogfoods Into Agent Workforce

The Agent Workforce design (`docs/design/agent-workforce.md`) defines 14 sub-agents for building RUSVEL itself. When this orchestration plan ships:

| Workforce Pattern | Runtime Equivalent |
|-------------------|---------------------|
| Agent A0 (Sprint Architect) | ForgeEngine `mission_today()` + `delegate_agent` |
| Agent B* (Builders) | Department-scoped agents with persona + tools + rules |
| Isolated worktrees | Session-scoped agent runs with own memory namespace |
| Validation protocol | PostToolUse hooks + self-correction (P5) |
| Dependency graph | FlowEngine DAG / Playbook template `depends_on` |
| Human review | ApprovalStatus gates (ADR-008) + Approval UI (P4) |
| Parallel execution | Job queue + `wait: false` delegation |
| Cost tracking | CostTracker (P12) per run + ModelTier routing |
| 10 parallel dept migrations | 10 parallel `delegate_agent` calls with different worktrees |

These 14 agents become Playbook templates — runnable from the UI, observable in real-time, resumable on crash.

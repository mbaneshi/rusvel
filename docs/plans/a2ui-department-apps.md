# A2UI + Department App Store — Dynamic Agent UI for RUSVEL

> Date: 2026-03-25
> Status: Proposed
> Depends on: ADR-014 (Department-as-App), Agent Workforce, P7 (AG-UI), P9 (Agent Orchestration)
> Purpose: Design the agent-driven UI system that makes each department a self-extending app with on-demand UI generation

---

## Problem

RUSVEL's frontend today is **static per department**. Every department gets the same layout: DepartmentPanel (tabs) + DepartmentChat (SSE streaming). Tool calls render as collapsible JSON cards. The UI doesn't know what the agent is doing — it just shows text + tool call blobs.

Three gaps:

1. **No generative UI.** When the Content agent drafts a blog post, the user sees a text blob — not a structured draft card with sections, metadata, platform targets, and an approval button. When the Harvest agent scores opportunities, there's no pipeline table — just chat text.

2. **No capability catalog.** Each department ships with agents, skills, rules, workflows, hooks, MCP servers — but they're hidden in separate tabs. There's no unified "app store" view showing what a department CAN do, with discoverable cards that can be browsed, installed, configured, and extended by AI.

3. **No self-extension loop.** The `!build` command generates entities from natural language, but the UI doesn't close the loop. There's no visual flow where the agent creates a new skill and it appears as a card in the catalog, or where the agent generates a dashboard and it renders inline.

---

## Research Summary

We evaluated 9 protocols/frameworks for agent-driven UI:

| Protocol | UI Protocol? | Streaming | Rust | Svelte | Production |
|----------|-------------|-----------|------|--------|------------|
| **AG-UI** (CopilotKit) | ✅ Core purpose | SSE/WS/HTTP | Community crate | Custom consumer | ✅ |
| **A2UI** (Declarative UI) | ✅ JSON component trees | Via AG-UI | Emit JSON | Render JSON | New (2026) |
| **MCP Apps** | ✅ Iframe-based | Streamable HTTP | Your MCP crate | Iframe embed | ✅ Jan 2026 |
| **A2A** (Google) | ❌ Agent-to-agent | SSE, gRPC | No SDK | N/A | ✅ |
| **Vercel AI SDK** | Partial (React RSC paused) | HTTP chunks | No | Hooks only | RSC paused |
| **CopilotKit** | ✅ via AG-UI | AG-UI events | Backend-agnostic | No (React) | ✅ |
| **LangGraph** | ❌ Uses AG-UI | SSE | No | Via CopilotKit | ✅ |
| **OpenAI Agents SDK** | ❌ | SSE | Via rusvel-llm | N/A | ✅ |
| **Claude Agent SDK** | ❌ | Async stream | No | N/A | ✅ |

### Verdict

**AG-UI + A2UI is the right protocol stack.** Reasoning:

1. **AG-UI** is the only production-ready, transport-agnostic, framework-agnostic protocol designed specifically for agent-to-UI communication. It maps cleanly onto RUSVEL's existing SSE streaming — our `AgentEvent` enum is already 80% of AG-UI's event types.

2. **A2UI** is the declarative layer ON TOP of AG-UI that lets agents describe UI components as JSON trees. The frontend maps these to Svelte components. This is how "UI on demand" works — the agent says "render a DataTable with these columns" and the frontend renders it natively.

3. **MCP Apps** is valuable but complementary — it serves external MCP clients (Claude, ChatGPT, VS Code) with rich UI. Our own frontend uses AG-UI/A2UI natively.

4. **A2A** is the future inter-agent protocol. RUSVEL's `delegate_agent` tool is a local version. A2A would let departments talk to external agents. But it's not a UI protocol — it runs alongside AG-UI.

5. **CopilotKit** is React-only, but its architectural patterns (CoAgents, shared state, three generative UI modes) are directly applicable. We build the Svelte equivalent.

---

## Architecture

### The Five-Layer Stack

```
┌──────────────────────────────────────────────────────────────┐
│  Layer 5: UI Surface (Svelte)                                 │
│  ┌───────────────────────┐  ┌──────────────────────────────┐ │
│  │  Static Cards          │  │  Dynamic A2UI Components     │ │
│  │  (from manifests)      │  │  (from agent tool calls)     │ │
│  │  - Capability catalog  │  │  - DataTable, Form, Chart    │ │
│  │  - Skill/Rule/Flow     │  │  - DraftCard, PipelineView   │ │
│  │  - Browse/Edit/CRUD    │  │  - ApprovalPanel, Diff       │ │
│  └───────────────────────┘  └──────────────────────────────┘ │
├──────────────────────────────────────────────────────────────┤
│  Layer 4: AG-UI Event Stream (SSE)                            │
│  RUN_STARTED → TEXT_MESSAGE_* → TOOL_CALL_* → STATE_DELTA    │
│  → CUSTOM (a2ui.render) → RUN_FINISHED                       │
├──────────────────────────────────────────────────────────────┤
│  Layer 3: Workforce (Self-Extension)                          │
│  Agent creates skill → emits a2ui.capability_created →       │
│  frontend adds card to catalog. Agent generates dashboard →  │
│  emits a2ui.render(DataTable) → frontend renders inline.     │
├──────────────────────────────────────────────────────────────┤
│  Layer 2: Platform (Rust)                                     │
│  AgentRuntime emits AgUiEvent, ToolPort, A2UI registry,      │
│  STATE_SNAPSHOT/DELTA for shared state                        │
├──────────────────────────────────────────────────────────────┤
│  Layer 1: Kernel                                              │
│  Port traits, domain types, DepartmentApp contract            │
└──────────────────────────────────────────────────────────────┘
```

### How It Works End-to-End

**Scenario: User opens the Content department**

1. Frontend loads `GET /api/dept/content/manifest` → receives `DepartmentManifest` with contributions (3 skills, 2 rules, 1 workflow, 1 persona, 2 quick_actions, UI declarations)
2. **Capability card grid** renders immediately — each contribution is a `CapabilityCard` component with name, description, type badge, status, hover details, edit/delete/toggle
3. User clicks "Draft Blog Post" quick action → chat sends message
4. Backend creates agent run → starts streaming AG-UI events:
   ```
   RUN_STARTED { run_id, thread_id }
   TEXT_MESSAGE_START { message_id }
   TEXT_MESSAGE_CONTENT { "Analyzing topic..." }
   TOOL_CALL_START { tool_call_id, tool_name: "content.draft" }
   TOOL_CALL_ARGS { "topic": "Rust async patterns", "platform": "devto" }
   TOOL_CALL_END {}
   STATE_DELTA { op: "add", path: "/draft", value: { title, sections, tags } }
   CUSTOM { name: "a2ui.render", data: {
     component: "DraftCard",
     props: { title: "...", sections: [...], platform: "devto", actions: ["approve", "edit", "adapt"] }
   }}
   TOOL_CALL_RESULT { output: "Draft ready for review" }
   TEXT_MESSAGE_CONTENT { "Here's your draft. Review and approve when ready." }
   TEXT_MESSAGE_END {}
   RUN_FINISHED { run_id }
   ```
5. Frontend receives `a2ui.render` → looks up `DraftCard` in the A2UI component registry → renders a structured card inline in the chat stream with title, sections, tags, platform badge, and Approve/Edit/Adapt buttons
6. User clicks "Approve" → frontend sends approval → agent resumes with `content.publish` tool call → `STATE_DELTA` updates draft status to "published"

**Scenario: User says "create a skill for LinkedIn carousel posts"**

1. Agent runs `!build`-style generation → produces a SkillDefinition JSON
2. Agent calls `create_skill` tool → stores in ObjectStore
3. Agent emits:
   ```
   CUSTOM { name: "a2ui.capability_created", data: {
     type: "skill",
     id: "linkedin-carousel",
     name: "LinkedIn Carousel",
     definition: { ... }
   }}
   ```
4. Frontend receives event → adds new `CapabilityCard` to the catalog grid with a "New" badge animation
5. The catalog is now self-extended without page reload

---

## Design: AG-UI Event Types

### Mapping Current → AG-UI

Our `AgentEvent` enum maps directly to AG-UI events with minimal changes:

| Current `AgentEvent` | AG-UI Event | Notes |
|----------------------|-------------|-------|
| `TextDelta { text }` | `TEXT_MESSAGE_CONTENT` | Add `message_id` |
| (missing) | `TEXT_MESSAGE_START` | Add before first delta |
| (missing) | `TEXT_MESSAGE_END` | Add after last delta |
| `ToolCall { name, args }` | `TOOL_CALL_START` + `TOOL_CALL_ARGS` | Split into start + args streaming |
| `ToolResult { name, output }` | `TOOL_CALL_RESULT` | Add `tool_call_id` |
| `Done { output }` | `RUN_FINISHED` | Extract cost, run_id |
| `Error { message }` | `RUN_ERROR` | Add error code |
| (missing) | `RUN_STARTED` | Add at stream start |
| (missing) | `STATE_SNAPSHOT` | Full state on connect |
| (missing) | `STATE_DELTA` | JSON Patch updates |
| (missing) | `CUSTOM` | A2UI render events |
| (missing) | `STEP_STARTED/FINISHED` | Multi-step workflow visibility |

### New `AgUiEvent` Enum

```rust
// In rusvel-core/src/domain.rs or new rusvel-agui crate

/// AG-UI protocol events for agent-to-frontend communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AgUiEvent {
    RunStarted {
        run_id: String,
        thread_id: String,
    },
    TextMessageStart {
        message_id: String,
    },
    TextMessageContent {
        message_id: String,
        delta: String,
    },
    TextMessageEnd {
        message_id: String,
    },
    ToolCallStart {
        tool_call_id: String,
        tool_name: String,
        parent_message_id: String,
    },
    ToolCallArgs {
        tool_call_id: String,
        delta: String, // Streamed JSON chunks
    },
    ToolCallEnd {
        tool_call_id: String,
    },
    ToolCallResult {
        tool_call_id: String,
        result: String,
        is_error: bool,
    },
    StateSnapshot {
        snapshot: serde_json::Value,
    },
    StateDelta {
        delta: Vec<JsonPatchOp>,
    },
    StepStarted {
        step_name: String,
        metadata: serde_json::Value,
    },
    StepFinished {
        step_name: String,
        metadata: serde_json::Value,
    },
    Custom {
        name: String,
        data: serde_json::Value,
    },
    RunFinished {
        run_id: String,
        cost_usd: Option<f64>,
    },
    RunError {
        message: String,
        code: Option<String>,
    },
}

/// RFC 6902 JSON Patch operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPatchOp {
    pub op: String,    // "add", "remove", "replace", "move", "copy", "test"
    pub path: String,  // JSON Pointer (e.g., "/draft/title")
    pub value: Option<serde_json::Value>,
}
```

---

## Design: A2UI Component Protocol

### How Agents Declare UI

Agents don't render HTML. They emit `Custom { name: "a2ui.render", data: ... }` events with a declarative JSON component tree. The frontend maps component type names to registered Svelte components.

```json
{
  "component": "DataTable",
  "props": {
    "columns": ["Name", "Score", "Budget", "Status"],
    "rows": [
      { "Name": "Rust CLI Tool", "Score": 0.87, "Budget": "$5000", "Status": "open" },
      { "Name": "SvelteKit App", "Score": 0.72, "Budget": "$3000", "Status": "applied" }
    ],
    "actions": [
      { "label": "Apply", "tool": "harvest.apply", "args_template": { "opportunity_id": "{{row.id}}" } },
      { "label": "Dismiss", "tool": "harvest.dismiss", "args_template": { "opportunity_id": "{{row.id}}" } }
    ]
  }
}
```

### Built-in A2UI Component Types

| Component | Description | Used By |
|-----------|-------------|---------|
| `DataTable` | Sortable table with row actions | Harvest pipeline, Finance ledger, Growth metrics |
| `DraftCard` | Structured content draft with sections + approval | Content drafting |
| `PipelineView` | Kanban/stage pipeline (deal stages, content calendar) | GTM deals, Content calendar, Harvest pipeline |
| `FormCard` | Dynamic form from JSON Schema | Entity CRUD, config editing |
| `MetricsGrid` | KPI cards with sparklines | Growth dashboard, Finance runway |
| `DiffView` | Side-by-side diff | Code analysis, content revision |
| `DagView` | DAG visualization (wraps XYFlow) | Flow execution, agent orchestration |
| `ApprovalPanel` | Approve/reject with payload preview | Any approval gate |
| `MarkdownCard` | Rich markdown with collapsible sections | Reports, analysis results |
| `StatusCard` | Status indicator with progress | Job tracking, deployment status |
| `ChartCard` | Simple chart (bar, line, pie) | Analytics, metrics |
| `CapabilityCard` | Skill/Rule/Workflow/Agent card with CRUD | Capability catalog |

### A2UI Action Protocol

Components can declare actions that map back to agent tool calls:

```json
{
  "actions": [
    {
      "label": "Approve",
      "tool": "content.publish",
      "args_template": { "draft_id": "{{props.draft_id}}" },
      "style": "primary"
    },
    {
      "label": "Edit",
      "action": "edit_inline",
      "target": "props.sections"
    },
    {
      "label": "Adapt for Twitter",
      "tool": "content.adapt",
      "args_template": { "draft_id": "{{props.draft_id}}", "platform": "twitter" },
      "style": "secondary"
    }
  ]
}
```

When a user clicks an action button:
1. If `tool` is specified: frontend sends a follow-up chat message that invokes the tool
2. If `action` is "edit_inline": frontend enables inline editing, then sends the diff as a tool call
3. The agent loop continues with the action result

---

## Design: Capability Card Catalog

### What Each Department Ships

Every department declares its capabilities in `DepartmentManifest.contributions`:

```rust
pub struct DepartmentManifest {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub color: String,
    // ...existing fields...

    /// Pre-built capabilities that ship with this department
    pub contributions: DepartmentContributions,
}

pub struct DepartmentContributions {
    pub agents: Vec<AgentSeed>,
    pub skills: Vec<SkillSeed>,
    pub rules: Vec<RuleSeed>,
    pub workflows: Vec<WorkflowSeed>,
    pub playbooks: Vec<PlaybookSeed>,
    pub personas: Vec<PersonaSeed>,
    pub tools: Vec<ToolContribution>,
    pub mcp_servers: Vec<McpServerSeed>,
    pub hooks: Vec<HookSeed>,
    pub ui_components: Vec<UiComponentDeclaration>,
    pub quick_actions: Vec<QuickAction>,
    pub dashboard_cards: Vec<DashboardCardDecl>,
}
```

### Seed Types

Each seed type carries enough info to render as a card AND to instantiate the entity:

```rust
pub struct SkillSeed {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,       // "drafting", "analysis", "automation"
    pub icon: Option<String>,
    pub tags: Vec<String>,
    pub template: String,       // The actual prompt template with {{input}}
    pub schema: Option<serde_json::Value>,  // JSON Schema for input params
    pub version: String,
    pub status: SeedStatus,     // Active, Draft, Disabled, Builtin
}

pub enum SeedStatus {
    Builtin,   // Ships with department, non-deletable
    Active,    // User-created or activated
    Draft,     // Work in progress
    Disabled,  // Deactivated by user
}
```

### Frontend: CapabilityGrid Component

```svelte
<!-- CapabilityGrid.svelte -->
<!-- Renders all contributions for a department as browsable cards -->

Props:
  dept: string
  contributions: DepartmentContributions
  filter: string (search text)
  category: string | "all"

Layout:
  - Category tabs: All | Agents | Skills | Rules | Workflows | Playbooks | MCP | Hooks
  - Search bar with instant filter
  - Card grid (responsive, 2-4 columns)
  - Each card: CapabilityCard component
  - "Create with AI" button → opens agent conversation scoped to entity generation
  - "Import" button → paste JSON or install from capability bundle
```

### Frontend: CapabilityCard Component

```svelte
<!-- CapabilityCard.svelte -->

Props:
  type: "agent" | "skill" | "rule" | "workflow" | "playbook" | "persona" | "mcp" | "hook"
  seed: AgentSeed | SkillSeed | ... (union)
  dept: string
  deptColor: string

Renders:
  ┌─────────────────────────────────┐
  │ [icon]  Skill Name        [⋯]  │  ← type badge + overflow menu (edit/delete/duplicate)
  │                                  │
  │ Short description that fits     │
  │ in two lines max...             │
  │                                  │
  │ [tag1] [tag2] [category]        │  ← clickable filter tags
  │                                  │
  │ ● Active    v1.2    Built-in    │  ← status dot + version + origin
  └─────────────────────────────────┘

Hover/Click:
  - Hover: show full description tooltip
  - Click: expand to detail view (slide-over or modal):
    - Full description
    - Template/definition (syntax highlighted)
    - Schema (if applicable)
    - Usage stats (times used, last used)
    - Edit button (opens inline editor or AI conversation)
    - Duplicate button
    - Delete button (disabled for Builtin)
    - Version history (if applicable)
  - For skills: "Run" button that sends template as chat message
  - For workflows: "Run" button + DAG preview
  - For rules: "Toggle" to enable/disable
  - For agents: "Chat" button to open department chat with that persona
```

---

## Design: Shared State (STATE_SNAPSHOT / STATE_DELTA)

### Per-Department State Store

Each department chat session maintains a typed state object. The agent can read and write it. The frontend renders from it. Changes flow bidirectionally via AG-UI state events.

```typescript
// Frontend: per-department reactive state
interface DepartmentState {
  // Agent-managed state (pushed via STATE_DELTA)
  draft?: ContentDraft;
  pipeline?: Opportunity[];
  analysis?: CodeAnalysis;
  metrics?: Record<string, number>;

  // UI-managed state (pushed to agent via context)
  selectedItems?: string[];
  filterCriteria?: Record<string, string>;
  viewMode?: "grid" | "list" | "kanban";
}
```

**Backend → Frontend:** Agent emits `StateDelta` events during tool execution. Example: after `content.draft` completes, agent pushes `{ op: "add", path: "/draft", value: { title: "...", sections: [...] } }`.

**Frontend → Backend:** When user interacts with A2UI components (selects rows, changes filters), frontend includes the current state snapshot in the next chat message's metadata. Agent receives it as context.

---

## Design: Three Modes of UI (CopilotKit Pattern, Adapted for Svelte)

### Mode 1: Controlled (Tool → Component Mapping)

Pre-registered tool-to-component mappings. When the agent calls a known tool, the frontend renders the associated component.

```typescript
// Frontend: tool-component registry
const toolRenderers: Record<string, SvelteComponent> = {
  'content.draft': DraftCard,
  'harvest.score': PipelineView,
  'code.analyze': DiffView,
  'finance.runway': MetricsGrid,
  'flow.execute': DagView,
};
```

**When to use:** Department-specific tools with well-known output shapes. High control, predictable UI.

### Mode 2: Declarative (A2UI JSON → Native Components)

Agent emits `a2ui.render` custom events with component type + props. Frontend maps to built-in A2UI components.

```json
{
  "component": "DataTable",
  "props": { "columns": [...], "rows": [...] }
}
```

**When to use:** Generic data display, agent-generated dashboards, dynamic reports. Medium control, flexible.

### Mode 3: Open-Ended (MCP Apps)

For external MCP clients. Agent returns interactive iframe UIs via MCP Apps protocol.

**When to use:** When RUSVEL tools are consumed by Claude, ChatGPT, or VS Code. Our own frontend uses Modes 1+2 instead.

---

## Implementation Plan

### Phase 1: AG-UI Event Migration (Backend) — 3 days

**Goal:** Replace current `AgentEvent` → SSE mapping with AG-UI typed events.

**Tasks:**

1. **Define `AgUiEvent` enum** in `rusvel-core/src/domain.rs`
   - 15 event types (listed above)
   - `JsonPatchOp` for state deltas
   - `Serialize/Deserialize` with `#[serde(tag = "type")]`
   - ~80 lines

2. **Update `AgentRuntime`** in `rusvel-agent/src/lib.rs`
   - Change `Sender<AgentEvent>` → `Sender<AgUiEvent>`
   - Emit `RunStarted` at stream start
   - Emit `TextMessageStart/Content/End` (replacing bare `TextDelta`)
   - Split `ToolCall` into `ToolCallStart` + `ToolCallArgs` + `ToolCallEnd`
   - Emit `RunFinished` with cost
   - Add message_id and tool_call_id tracking
   - ~60 lines changed

3. **Update `chat.rs`** in `rusvel-api`
   - Map `AgUiEvent` → SSE `Event` (1:1 — each variant becomes an SSE event with JSON data)
   - Remove old `AgentEvent` mapping
   - Add `id` field to SSE events for client-side dedup
   - ~40 lines changed

4. **Add `StateDelta` emission** to engine tool handlers
   - When `content.draft` completes → emit `StateDelta { op: "add", path: "/draft", value: ... }`
   - When `harvest.score` completes → emit `StateDelta { op: "replace", path: "/pipeline", value: ... }`
   - ~30 lines per engine tool, 5 tools = ~150 lines

5. **Tests:** Verify all existing chat tests pass with new event types. Add tests for state delta serialization.

**Validation:** `cargo test -p rusvel-core && cargo test -p rusvel-agent && cargo test -p rusvel-api`

---

### Phase 2: AG-UI Svelte Consumer (Frontend) — 3 days

**Goal:** Build a Svelte AG-UI event consumer that replaces the current `parseSSE` implementation.

**Tasks:**

1. **Create `agui-consumer.ts`** in `frontend/src/lib/`
   - TypeScript types for all 15 AG-UI event types
   - `AgUiStream` class that wraps SSE and emits typed events
   - Svelte 5 runes-compatible: `$state` for current run, messages, tool calls
   - State store with JSON Patch application (use `fast-json-patch` or implement RFC 6902 subset)
   - ~200 lines

2. **Create `agui-store.svelte.ts`** — reactive AG-UI state
   - `createAgUiStore(dept: string)` factory
   - Exposes: `messages`, `toolCalls`, `state` (department state), `isRunning`, `currentStep`
   - Auto-applies `STATE_DELTA` patches to `state`
   - Auto-groups `TOOL_CALL_START/ARGS/END/RESULT` into tool call objects
   - ~150 lines

3. **Refactor `DepartmentChat.svelte`** to use AG-UI store
   - Replace manual `parseSSE` + callback wiring with AG-UI store subscription
   - Tool calls now have IDs (proper tracking)
   - Messages have IDs (dedup, reference)
   - Steps visible in UI (STEP_STARTED/FINISHED)
   - ~80 lines changed

4. **Refactor `chat/+page.svelte`** (God Agent) similarly
   - ~60 lines changed

5. **Update `ToolCallCard.svelte`** for streamed args
   - Args now stream in via `TOOL_CALL_ARGS` (not all-at-once)
   - Show streaming JSON as it arrives
   - ~30 lines changed

**Validation:** `cd frontend && pnpm check && pnpm build`

---

### Phase 3: A2UI Component Registry (Frontend) — 4 days

**Goal:** Build the A2UI component system — agents emit JSON, frontend renders Svelte components.

**Tasks:**

1. **Create A2UI component registry** — `frontend/src/lib/a2ui/registry.ts`
   - `registerComponent(name: string, component: SvelteComponent)`
   - `resolveComponent(name: string): SvelteComponent | null`
   - Default registry with 12 built-in components
   - ~40 lines

2. **Create `A2uiRenderer.svelte`** — dynamic component renderer
   - Receives `{ component: string, props: object, actions?: Action[] }` from `a2ui.render` events
   - Looks up component in registry
   - Renders with `<svelte:component>`
   - Handles action clicks → sends tool call messages
   - Handles `{{template}}` interpolation in action args
   - ~80 lines

3. **Build 6 core A2UI components** (priority order):
   - `DataTable.svelte` — sortable table with row actions (~120 lines)
   - `DraftCard.svelte` — structured content draft with sections + approve/edit (~100 lines)
   - `MetricsGrid.svelte` — KPI cards with values + trends (~80 lines)
   - `FormCard.svelte` — dynamic form from JSON Schema (~120 lines)
   - `ApprovalPanel.svelte` — approve/reject with payload preview (~60 lines)
   - `MarkdownCard.svelte` — rich markdown with collapsible sections (~50 lines)
   - Total: ~530 lines

4. **Wire into DepartmentChat** — render `a2ui.render` events inline
   - When AG-UI `Custom { name: "a2ui.render" }` arrives, render `A2uiRenderer` inline in chat flow
   - ~20 lines in DepartmentChat

5. **Wire tool-component mappings** (Mode 1: Controlled)
   - Map known tool names to components in `tool-renderers.ts`
   - When `TOOL_CALL_RESULT` arrives for a mapped tool, auto-render the component
   - ~30 lines

**Validation:** `cd frontend && pnpm check && pnpm build`

---

### Phase 4: Capability Card Catalog (Frontend + Backend) — 4 days

**Goal:** Each department shows a browsable card grid of all its capabilities.

**Tasks:**

1. **Backend: Manifest contributions endpoint** — `GET /api/dept/{dept}/manifest`
   - Return the `DepartmentManifest` including all contributions
   - For now, build manifest from existing seed data + ObjectStore entities
   - ~60 lines in `rusvel-api/src/department.rs`

2. **Backend: Unified capability CRUD** — `GET/POST/PUT/DELETE /api/dept/{dept}/capabilities/{type}/{id}`
   - Wraps existing agents/skills/rules/workflows/hooks/mcp endpoints
   - Returns unified `Capability` type with `type` discriminator
   - ~80 lines

3. **Create `CapabilityCard.svelte`** component
   - Universal card for any capability type (agent, skill, rule, workflow, playbook, persona, mcp, hook)
   - Type badge with icon + color
   - Name, description (2-line clamp), tags, status dot, version
   - Hover → tooltip with full description
   - Click → slide-over detail panel
   - Overflow menu: Edit, Duplicate, Delete, Toggle
   - ~180 lines

4. **Create `CapabilityGrid.svelte`** component
   - Category tabs (All | Agents | Skills | Rules | Workflows | ...)
   - Search/filter bar
   - Responsive card grid (CSS grid, 2-4 columns)
   - "Create with AI" button (opens scoped agent conversation)
   - "Import" button (paste JSON)
   - Empty state for categories with no entries
   - ~150 lines

5. **Create `CapabilityDetail.svelte`** slide-over
   - Full description + definition (syntax-highlighted JSON/template)
   - Schema viewer (if applicable)
   - Usage stats (times used, last used)
   - Edit mode (inline editor)
   - Version info
   - ~200 lines

6. **Integrate into DepartmentPanel**
   - Replace current separate tabs (Agents, Skills, Rules, etc.) with single "Capabilities" tab showing `CapabilityGrid`
   - Keep "Actions" tab and "Engine" tab as-is
   - Add capability count badges
   - ~40 lines changed

7. **Live updates via AG-UI**
   - When agent emits `a2ui.capability_created` / `a2ui.capability_updated` / `a2ui.capability_deleted` custom events
   - CapabilityGrid reacts: adds/updates/removes cards with animation
   - ~30 lines

**Validation:** `cd frontend && pnpm check && pnpm build`

---

### Phase 5: State Sync + Workforce Integration (Backend) — 3 days

**Goal:** Connect the self-extension loop — agents that create/modify capabilities update the catalog in real time.

**Tasks:**

1. **`!build` integration** — after `!build` creates entities, emit `a2ui.capability_created` events
   - Update `build_cmd.rs` to emit custom AG-UI events for each created entity
   - Frontend receives events → catalog updates live
   - ~30 lines

2. **Agent-as-CRUD** — expose `create_skill`, `update_rule`, `delete_workflow` as agent tools
   - Register 6 CRUD tools in `rusvel-builtin-tools`: `create_agent`, `create_skill`, `create_rule`, `create_workflow`, `create_hook`, `create_mcp_server`
   - Each tool persists to ObjectStore + emits `a2ui.capability_created`
   - ~120 lines total

3. **Self-correction → catalog** — when F11 (self-correction) generates a new rule, it appears as a card
   - Critique agent uses `create_rule` tool → catalog updates
   - ~10 lines (wiring only, uses tools from step 2)

4. **Department state persistence**
   - Save per-department state snapshots to ObjectStore
   - On page load, restore last state → `STATE_SNAPSHOT` event
   - ~40 lines backend + 20 lines frontend

**Validation:** `cargo test --workspace && cd frontend && pnpm check`

---

### Phase 6: Remaining A2UI Components + Polish — 3 days

**Goal:** Complete the component library and polish interactions.

**Tasks:**

1. **Build remaining A2UI components:**
   - `PipelineView.svelte` — Kanban columns for deal stages / content calendar (~150 lines)
   - `DiffView.svelte` — side-by-side diff for code analysis (~100 lines)
   - `DagView.svelte` — DAG visualization wrapping XYFlow (~80 lines)
   - `StatusCard.svelte` — status indicator with progress bar (~40 lines)
   - `ChartCard.svelte` — simple bar/line/pie chart (~100 lines)
   - Total: ~470 lines

2. **Per-department default A2UI mappings**
   - Content: `content.draft` → `DraftCard`, `content.calendar` → `PipelineView`
   - Harvest: `harvest.pipeline` → `PipelineView`, `harvest.score` → `DataTable`
   - Code: `code.analyze` → `DiffView` + `MetricsGrid`
   - Finance: `finance.runway` → `MetricsGrid` + `ChartCard`
   - GTM: `gtm.pipeline` → `PipelineView`
   - Flow: `flow.execute` → `DagView`
   - ~60 lines config

3. **Agent tool UI hints**
   - Add optional `ui_hint` field to `ToolDefinition` in `rusvel-tool`
   - UI hint: `{ component: "DraftCard", state_path: "/draft" }`
   - When tool result arrives, if ui_hint exists, auto-render
   - ~30 lines backend, ~20 lines frontend

4. **Animations & transitions**
   - Card grid: stagger entrance animation
   - A2UI components: fade-in when rendered in chat
   - State updates: highlight changed fields briefly
   - Capability created: fly-in animation
   - ~50 lines CSS

5. **Keyboard navigation**
   - Arrow keys to navigate capability grid
   - Enter to open detail
   - Escape to close
   - Cmd+N to create new
   - ~40 lines

**Validation:** `cd frontend && pnpm check && pnpm build`

---

## Dependency Graph

```
Phase 1: AG-UI Event Migration (Backend)
    │
    ├──→ Phase 2: AG-UI Svelte Consumer (Frontend)  ─── can start after Phase 1
    │         │
    │         ├──→ Phase 3: A2UI Component Registry ─── needs AG-UI consumer
    │         │         │
    │         │         └──→ Phase 6: Remaining Components ─── needs registry
    │         │
    │         └──→ Phase 4: Capability Card Catalog ─── needs AG-UI for live updates
    │                   │
    │                   └──→ Phase 5: Workforce Integration ─── needs catalog + AG-UI
    │
    └──→ Phase 4 (backend part): Manifest endpoint ─── can start after Phase 1
```

**Critical path:** Phase 1 → Phase 2 → Phase 3 → Phase 6
**Parallel track:** Phase 4 (backend) can start with Phase 1. Phase 4 (frontend) can start with Phase 2.
**Phase 5 requires:** Phase 2 + Phase 4 complete.

**Estimated total: ~20 days of work, compressible to ~12 days with parallelization.**

---

## What This Enables

### Immediate (After Phase 3)

- Agents render rich UI inline in chat — tables, draft cards, metrics, forms
- Tool calls are visually meaningful, not JSON blobs
- Department state syncs between agent and frontend in real time
- Multi-step agent work shows progress (STEP events)

### After Phase 4

- Every department is a browsable app with a capability catalog
- Skills, rules, workflows, agents are discoverable cards
- Browse → understand → run → edit → create — all from the catalog
- `!build` creates capabilities that appear as cards instantly

### After Phase 5

- The system is self-extending: agents create new capabilities that appear in the catalog
- Self-correction loop generates rules that show up as cards
- Workforce agents (from agent-workforce.md) can extend any department
- Department evolution is visible — you can see the catalog grow over time

### Future (Phase 5 roadmap & beyond)

- **A2A integration:** External agents publish Agent Cards, appear in the capability catalog as "Remote Agent" cards
- **MCP Apps:** MCP tools from external servers render rich UI in RUSVEL's chat (iframe-based)
- **Playbooks as cards:** Multi-step recipes show as capability cards with DAG previews and "Run" buttons
- **Department marketplace:** Share/import capability bundles between RUSVEL instances
- **AI-generated dashboards:** "Show me my content performance" → agent emits A2UI `ChartCard` + `MetricsGrid` inline

---

## Relationship to Existing Plans

| Existing Plan | How This Proposal Connects |
|---------------|---------------------------|
| **ADR-014** (Dept-as-App) | This IS the UI layer for ADR-014. `DepartmentManifest.contributions` → `CapabilityGrid`. `DepartmentApp.register()` → tools that emit A2UI events. |
| **Agent Workforce** | Workforce agents use the same A2UI protocol. Builder agents emit `a2ui.capability_created` when they create entities. The catalog is the visual representation of the workforce's output. |
| **P7** (AG-UI Protocol) | This proposal supersedes P7 with a complete design. P7 was "adopt AG-UI events." This adds A2UI, state sync, capability cards, and the full frontend architecture. |
| **P4** (Approval UI) | `ApprovalPanel` A2UI component replaces the need for a separate approval page. Approvals render inline where the agent requests them. |
| **P9** (Agent Orchestration) | `delegate_agent` tool calls show as `STEP_STARTED/FINISHED` events. Sub-agent work is visible in the parent's chat stream. The `DagView` component visualizes orchestration. |
| **P5** (Self-Correction) | Critique results render as `MarkdownCard` A2UI component. Auto-generated rules appear as new cards in the capability catalog. |
| **P1** (Deferred Tools) | Deferred tools are still searchable in the capability catalog as "Tool" cards. The `tool_search` meta-tool can be visualized via `DataTable`. |
| **P11** (Playbooks) | Playbooks are capability cards of type "playbook" with DAG preview and "Run" button. |
| **P12** (Smart Routing) | Model tier shows in `RunStarted` event metadata. Cost shows in `RunFinished`. `MetricsGrid` can display cost analytics. |
| **Inspiration** (GenAICircle) | Starter kits = pre-built capability bundles. Leveling = catalog completion tracking. Executive brief = agent-generated `MetricsGrid` + `MarkdownCard`. |

---

## File Inventory (New + Modified)

### New Files (~2,400 lines total)

**Rust (~500 lines):**
- `crates/rusvel-core/src/agui.rs` — `AgUiEvent` enum + `JsonPatchOp` (~80 lines)
- `crates/rusvel-builtin-tools/src/capability_crud.rs` — 6 CRUD tools (~120 lines)
- `crates/rusvel-api/src/manifest.rs` — manifest endpoint (~60 lines)

**Svelte (~1,900 lines):**
- `frontend/src/lib/agui/consumer.ts` — AG-UI SSE consumer (~200 lines)
- `frontend/src/lib/agui/store.svelte.ts` — reactive AG-UI state store (~150 lines)
- `frontend/src/lib/a2ui/registry.ts` — component registry (~40 lines)
- `frontend/src/lib/a2ui/A2uiRenderer.svelte` — dynamic renderer (~80 lines)
- `frontend/src/lib/a2ui/DataTable.svelte` (~120 lines)
- `frontend/src/lib/a2ui/DraftCard.svelte` (~100 lines)
- `frontend/src/lib/a2ui/MetricsGrid.svelte` (~80 lines)
- `frontend/src/lib/a2ui/FormCard.svelte` (~120 lines)
- `frontend/src/lib/a2ui/ApprovalPanel.svelte` (~60 lines)
- `frontend/src/lib/a2ui/MarkdownCard.svelte` (~50 lines)
- `frontend/src/lib/a2ui/PipelineView.svelte` (~150 lines)
- `frontend/src/lib/a2ui/DiffView.svelte` (~100 lines)
- `frontend/src/lib/a2ui/DagView.svelte` (~80 lines)
- `frontend/src/lib/a2ui/StatusCard.svelte` (~40 lines)
- `frontend/src/lib/a2ui/ChartCard.svelte` (~100 lines)
- `frontend/src/lib/components/capability/CapabilityCard.svelte` (~180 lines)
- `frontend/src/lib/components/capability/CapabilityGrid.svelte` (~150 lines)
- `frontend/src/lib/components/capability/CapabilityDetail.svelte` (~200 lines)

### Modified Files

**Rust:**
- `crates/rusvel-agent/src/lib.rs` — emit `AgUiEvent` instead of `AgentEvent` (~60 lines changed)
- `crates/rusvel-api/src/chat.rs` — map `AgUiEvent` → SSE (~40 lines changed)
- `crates/rusvel-api/src/department.rs` — add manifest endpoint (~20 lines)
- `crates/rusvel-api/src/build_cmd.rs` — emit capability events (~30 lines)
- `crates/rusvel-core/src/domain.rs` — add `ui_hint` to ToolDefinition (~10 lines)

**Svelte:**
- `frontend/src/lib/components/chat/DepartmentChat.svelte` — use AG-UI store (~80 lines changed)
- `frontend/src/routes/chat/+page.svelte` — use AG-UI store (~60 lines changed)
- `frontend/src/lib/components/chat/ToolCallCard.svelte` — streamed args (~30 lines changed)
- `frontend/src/lib/components/department/DepartmentPanel.svelte` — add Capabilities tab (~40 lines changed)
- `frontend/src/lib/api.ts` — add manifest/capability types (~30 lines)

---

## Decision Checklist

- [x] AG-UI as the event protocol (not custom SSE, not WebSocket)
- [x] A2UI for declarative agent-generated UI (not server-rendered HTML, not iframes)
- [x] AG-UI STATE_DELTA with JSON Patch for shared state (not polling, not WebSocket state)
- [x] Tool-to-component mapping for known tools (Mode 1: Controlled)
- [x] A2UI JSON for dynamic agent UI (Mode 2: Declarative)
- [x] MCP Apps for external clients (Mode 3: Open-Ended) — future, not in this plan
- [x] CapabilityGrid replaces separate entity tabs (unified catalog)
- [x] Seeds from manifests as builtin cards (static catalog)
- [x] Agent CRUD tools for self-extension (dynamic catalog)
- [x] Backward compatible — old `AgentEvent` consumers (TUI, MCP) get adapter layer

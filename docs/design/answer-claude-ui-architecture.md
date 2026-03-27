# RUSVEL UI/UX Architecture — Architecture-Aligned Design

**Date:** 2026-03-27
**In response to:** Prompt 3 (Claude — Architecture-Aligned UI Design)

---

## 1. Layout Reflecting DepartmentApp Pattern

The DepartmentApp trait is the architectural atom:

```rust
pub trait DepartmentApp: Send + Sync {
    fn manifest(&self) -> DepartmentManifest;
    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
}
```

Each department declares its identity, capabilities, tabs, quick_actions, UI contributions, events, and config schema through the manifest. The UI should treat each department as **its own app** — not as a tab inside a shared container.

### The "App within an App" Pattern

```
+--48px--+--200px---------+---flexible (main)----------+--320px--------+
| GLOBAL | DEPARTMENT      |                             | CONTEXT       |
| RAIL   | SECTIONS        |  MAIN CONTENT               | PANEL         |
|        |                 |                             |               |
| [Home] | ┌─────────────┐ | ┌─────────────────────────┐ | ┌───────────┐ |
| [Chat] | │ ≡ Forge     │ | │                         │ | │ AI Chat   │ |
| [Appv] | │   forge dept│ | │  Section-specific       │ | │           │ |
| [DB  ] | └─────────────┘ | │  content renders here   │ | │ or        │ |
| [Flow] | ○ Actions       | │                         │ | │ Properties│ |
| [Term] | ○ Engine        | │  - CRUD lists/grids     │ | │ or        │ |
| ────── | ○ Agents        | │  - Chat conversation    │ | │ Exec Log  │ |
| [⚒ ]  | ○ Skills        | │  - Workflow canvas      │ | │           │ |
| [◇ ]  | ○ Rules         | │  - Dashboard cards      │ | │           │ |
| [✎ ]  | ○ Workflows     | │  - A2UI rendered comps  │ | │           │ |
| [🔍]  | ○ MCP           | │                         │ | │           │ |
| [⑂ ]  | ○ Hooks         | │                         │ | │           │ |
| [🚀]  | ○ Terminal       | │                         │ | │           │ |
| [$ ]  | ○ Events        | │                         │ | │           │ |
| [📦]  | ──────────      | │                         │ | │           │ |
| [📈]  | ○ Chat          | │                         │ | │           │ |
| [⑂ ]  | ○ Settings      | │                         │ | │           │ |
| [⚖ ]  | ──────────      | │                         │ | │           │ |
| [🎧]  | Session: ▾      | │                         │ | │           │ |
| [⊟ ]  | [test-session]  | └─────────────────────────┘ | └───────────┘ |
| ────── |                 |                             |               |
| [⚙ ]  |                 |                             |               |
+--------+-----------------+-----------------------------+---------------+
|          BOTTOM PANEL (collapsible)                                    |
|  [Terminal] [Executions] [Events]                                      |
+-----------------------------------------------------------------------+
```

### How this mirrors the architecture:

| Architecture Concept | UI Manifestation |
|---------------------|------------------|
| `DepartmentApp` trait | Each department = icon in rail + its own section sidebar + own URL namespace |
| `DepartmentManifest.tabs` | Section sidebar items (filtered by `tabsFromDepartment()`) |
| `DepartmentManifest.quick_actions` | Actions section content |
| `DepartmentManifest.color` / `icon` | Rail icon accent + sidebar header theming |
| `DepartmentManifest.system_prompt` | Visible/editable in Settings section |
| `DepartmentManifest.capabilities` | Shown in Engine section |
| `DepartmentManifest.ui.dashboard_cards` | Dashboard cards in Actions section |
| `RegistrationContext.tools` | Tool list in Settings section (allowed/disallowed toggles) |
| `RegistrationContext.event_handlers` | Events section shows consumed events |
| `RegistrationContext.job_handlers` | Execution logs in bottom panel |
| `DepartmentRegistry` (13 departments) | 13 icons in the rail |
| `AppState` (shared infrastructure) | Global pages: Home, Chat, Approvals, DB, Flows, Terminal |

### Sparse vs Rich departments

The manifest-driven section sidebar naturally handles both:

**Rich department (Forge — 3 wired tools, 5 tool handlers):**
```
Sidebar shows: Actions, Engine, Agents, Skills, Rules, Workflows, MCP, Hooks, Terminal, Events, Chat, Settings
```

**Sparse department (Legal — 0 wired tools, chat only):**
```
Sidebar shows: Actions, Agents, Skills, Rules, Chat, Settings
(Engine, MCP, Hooks, Terminal, Events hidden because manifest.tabs doesn't include them)
```

As departments get tools wired (Priority 1 gap from Ch. 7), their manifests gain more tabs, and the sidebar automatically shows more sections. No UI code changes needed — the sidebar reads from the manifest.

---

## 2. Three User Modes

### Configure Mode
**Triggered by:** Navigating to Agents, Skills, Rules, MCP, Hooks, Settings sections.

```
+--------+-----------+----------------------------------+-----------------+
| Rail   | Sidebar   |  Agents                   + ▾   | Agent Detail    |
|        |           |                                  |                 |
|        | ● Agents  |  ┌──────────┐ ┌──────────┐     | Name: CodeWriter|
|        |           |  │CodeWriter│ │Reviewer  │     | Model: opus     |
|        |           |  │ opus     │ │ sonnet   │     | Role: Write code|
|        |           |  └──────────┘ └──────────┘     |                 |
|        |           |  ┌──────────┐                   | Instructions:   |
|        |           |  │+ New     │                   | [textarea...]   |
|        |           |  └──────────┘                   |                 |
|        |           |                                  | Tools: [toggles]|
|        |           |                                  |                 |
|        |           |                                  | [Save] [Delete] |
+--------+-----------+----------------------------------+-----------------+
```

- **Main area:** List/grid of items + "New" button
- **Context panel:** Properties of selected item (editable)
- **Bottom panel:** Collapsed

### Execute Mode
**Triggered by:** Navigating to Chat, Actions, or running a Workflow.

```
+--------+-----------+----------------------------------+-----------------+
| Rail   | Sidebar   |  Chat                            | Tool Calls      |
|        |           |                                  |                 |
|        | ● Chat    |  User: Draft a blog about RUSVEL | ▼ content_draft |
|        |           |                                  |   topic: RUSVEL |
|        |           |  Agent: I'll draft that now.     |   kind: Blog    |
|        |           |  [ToolCall: content_draft(...)]  |   → 1,247 tokens|
|        |           |  [ApprovalCard: Publish?]        |                 |
|        |           |                                  | ▼ adapt         |
|        |           |  The draft is ready. Here's      |   platform: tw  |
|        |           |  the blog post about RUSVEL...   |   → 342 tokens  |
|        |           |                                  |                 |
|        |           |  [Ask Content Dept...]     [▶]  | [Approve] [Rej] |
+--------+-----------+----------------------------------+-----------------+
|  Terminal: $ cargo build                                               |
|  Compiling rusvel-app v0.1.0                                           |
+------------------------------------------------------------------------+
```

- **Main area:** Chat conversation with streaming + inline tool calls + approval cards
- **Context panel:** Tool call detail, expanded I/O, approval actions, execution cost
- **Bottom panel:** Terminal (if agent uses bash tool) or execution logs

### Monitor Mode
**Triggered by:** Navigating to Events, Dashboard (Home), Approvals.

```
+--------+-----------+----------------------------------+-----------------+
| Rail   | Sidebar   |  Events                  ⟳      | Event Detail    |
|        |           |                                  |                 |
|        | ● Events  |  ● mission.plan.generated  2m   | Kind: mission.. |
|        |           |  ● code.analyzed          5m   | Source: forge   |
|        |           |  ○ content.draft.created  12m   | Payload:        |
|        |           |  ○ harvest.scan.completed 1h    | {               |
|        |           |  ○ content.published      2h    |   "goals": 3,   |
|        |           |                                  |   "tasks": 7    |
|        |           |                                  | }               |
|        |           |                                  |                 |
|        |           |                                  | Run: run_abc123 |
|        |           |                                  | Session: test   |
+--------+-----------+----------------------------------+-----------------+
```

- **Main area:** Timeline/list of events, dashboard cards, approval queue
- **Context panel:** Detail of selected event/approval (JSON payload, run info)
- **Bottom panel:** Live event stream (auto-scrolling)

---

## 3. A2UI Integration

### Where agent-generated UI renders

The A2UI vision declares 12 component types: DataTable, DraftCard, MetricsGrid, FormCard, ChartPanel, StatusBadge, ActionButton, ProgressBar, Timeline, CodeBlock, ImageGallery, AlertCard.

These render in **two locations**:

### Location 1: Inline in Chat (Execute Mode)

When the agent generates a `STATE_DELTA` with A2UI JSON during a chat conversation, the component renders inline in the message stream — similar to Claude.ai Artifacts but within the chat flow.

```
User: Show me this month's content metrics

Agent: Here are your content metrics:

┌─ MetricsGrid ────────────────────────────┐
│  Published: 12   │  Drafts: 3   │ Scheduled: 5  │
│  Views: 4,230    │  Shares: 89  │ Engagement: 2.1% │
└──────────────────────────────────────────┘

Agent: Your LinkedIn posts are performing 40% better than Twitter this month.
```

The chat message parser detects A2UI JSON blocks and renders the corresponding Svelte component instead of raw text. This is a natural extension of the existing ToolCallCard pattern — just a different card type.

### Location 2: Dashboard Cards (Monitor Mode)

Department dashboards (`/dept/[id]/actions` or a future `/dept/[id]/dashboard`) can render A2UI components as persistent cards. These are not ephemeral chat outputs — they're stored dashboard widgets that update via events.

```
Actions page for Content department:

┌─ DraftCard ────────────┐  ┌─ MetricsGrid ──────────┐
│ "RUSVEL Architecture"  │  │ Published: 12          │
│ Status: Draft          │  │ This week: 3           │
│ [Edit] [Publish] [Chat]│  │ Engagement: 2.1%       │
└────────────────────────┘  └────────────────────────┘

┌─ Timeline ─────────────────────────────────────────┐
│ ● 2h ago: "API Guide" published to LinkedIn        │
│ ● 5h ago: "Rust Tips" scheduled for tomorrow       │
│ ○ 1d ago: "Week 12 Review" drafted                 │
└────────────────────────────────────────────────────┘
```

### How A2UI coexists with static CRUD pages

The principle: **CRUD pages are the floor, A2UI is the ceiling.**

- **CRUD pages** (Agents, Skills, Rules lists) are always available as static Svelte routes. They work without AI, without streaming, without any runtime magic.
- **A2UI components** enhance these pages with dynamic, agent-generated content. They render alongside static content, not instead of it.
- The `DepartmentManifest.ui.dashboard_cards` field already supports this: it declares which A2UI cards a department wants on its dashboard.

### Rendering pipeline:

```
Agent generates A2UI JSON
  → AG-UI STATE_DELTA event (JSON Patch)
  → SSE stream to frontend
  → Chat parser identifies A2UI block
  → Dynamic Svelte component rendered:
      <svelte:component this={a2uiComponents[block.type]} {...block.props} />
  → Component types: DataTable, DraftCard, MetricsGrid, etc.
```

---

## 4. Capability Catalog

### What it is

All agents, skills, rules, workflows, playbooks, and MCP servers are "capabilities." The catalog shows them as browsable, searchable cards — a unified view of everything a department (or the whole system) can do.

### Where it lives: Both per-department and global

**Per-department catalog: the Actions section**

Each department's Actions page becomes a capability catalog for THAT department:

```
/dept/forge/actions

┌─ Quick Actions ──────────────────────────────────────────┐
│  [Daily Plan] [Review Progress] [Set Goal] [Hire Persona]│
└──────────────────────────────────────────────────────────┘

┌─ Capabilities ───────────────────────────────────────────┐
│  Filter: [All ▾] [Agents] [Skills] [Rules] [Workflows]  │
│  Search: [________________________]                       │
│                                                           │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │🤖 Agent  │ │⚡ Skill  │ │📏 Rule   │ │🔄 Flow   │   │
│  │CodeWriter│ │deploy-chk│ │no-force  │ │blog→tweet│   │
│  │opus model│ │Run deploy│ │push rule │ │3 steps   │   │
│  │ [Chat]   │ │ [Run]    │ │ [Toggle] │ │ [Run]    │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
└──────────────────────────────────────────────────────────┘

┌─ Build Capability ───────────────────────────────────────┐
│  Describe what you need — AI discovers & installs        │
│  [________________________________________________] [▶] │
└──────────────────────────────────────────────────────────┘
```

**Global catalog: the Dashboard**

The home dashboard (`/`) shows a cross-department capability overview:

```
/

┌─ Your Agency ────────────────────────────────────────────┐
│  13 Departments  │  24 Agents  │  18 Skills  │  7 Flows │
│                                                           │
│  Recent capabilities:                                     │
│  • thread-writer (Content, Skill) — created 2h ago       │
│  • deploy-check (Code, Workflow) — ran 5m ago            │
│  • scoring-rule (Harvest, Rule) — toggled on 1d ago      │
└──────────────────────────────────────────────────────────┘
```

### Self-extension loop

When an agent creates a new capability via CRUD tools during a chat session:

```
1. Agent calls create_skill(name: "seo-optimizer", ...) via tool
2. Backend persists the skill, emits event: skill.created
3. SSE pushes event to frontend
4. If user is on /dept/content/actions → catalog card appears live
5. If user is elsewhere → badge count increments on Content icon in rail
```

The catalog is not a separate page — it's the Actions section with dynamic content. Self-extension happens naturally because the Actions section reads from the same API that agents write to.

---

## 5. Chat Placement Resolution

### The layered approach:

| Layer | Pattern | Location | When |
|-------|---------|----------|------|
| **Full Chat** | Chat-as-Section | `/dept/[id]/chat` | Extended conversations, multi-turn tool use, complex tasks |
| **Quick Chat** | Chat-in-Context-Panel | Right panel, any section | Quick questions while working on config ("what model should I use?") |
| **God Agent** | Chat-as-Page | `/chat` | Cross-department questions, system-wide commands |
| **Quick Command** | Chat-as-Overlay | Cmd+K CommandPalette | One-shot commands, navigation |

### How they connect:

- **Quick Chat → Full Chat:** If a quick chat conversation becomes complex, a "Open in full chat" button navigates to `/dept/[id]/chat` with the conversation preserved.
- **Capability action → Chat:** Clicking "Chat" on a capability card in the catalog navigates to `/dept/[id]/chat` with the prompt pre-filled (existing `pendingCommand` store).
- **Full Chat → Monitor:** After a chat session completes a task, the user can switch to Events to see what happened, or the bottom panel shows execution results.

### Architecture alignment:

The 9-step chat handler pipeline (validate → config → interceptors → rules → capabilities → RAG → AgentConfig → stream → hooks) runs identically regardless of which chat surface initiated the request. The frontend just calls `POST /api/dept/{dept}/chat` from whichever layer.

---

## 6. Navigation Flow Example

> "Switch from Content dept to Harvest dept, set up a new scoring rule, test it in chat, then check if the harvest pipeline found new leads"

### Step-by-step:

**1. Switch to Harvest**
- Click **Harvest icon** (🔍) in the icon rail
- Sidebar updates: shows Harvest's sections (Actions, Engine, Agents, Skills, Rules, Chat, ...)
- Main area shows `/dept/harvest/actions` — Harvest quick actions + capability catalog
- Dept color shifts to Harvest's accent (cyan)
- **1 click. URL: `/dept/harvest/actions`**

**2. Navigate to Rules**
- Click **Rules** in section sidebar
- Main area shows Rules list for Harvest department — existing rules as cards
- **1 click. URL: `/dept/harvest/rules`**

**3. Create a scoring rule**
- Click **+ Create New Rule** in main area header
- Form appears: Name = "high-value-signal", Content = "Score opportunities higher if they mention AI, automation, or developer tools. Minimum score threshold: 7/10."
- Toggle: Enabled = on
- Click **Save**
- Card appears in grid
- **3 actions (fill form + save). URL stays: `/dept/harvest/rules`**

**4. Test in chat**
- Click **Chat** in section sidebar
- Main area shows Harvest chat (full width)
- Type: "Scan for new opportunities and apply the high-value-signal rule"
- Agent streams response, invoking tools: `scan_sources()`, `score_opportunity()` (once wired)
- Tool calls visible inline. Results show scored opportunities.
- **1 click + 1 message. URL: `/dept/harvest/chat`**

**5. Check pipeline**
- Click **Engine** in section sidebar (or use quick action "Refresh Pipeline")
- Main area shows Engine tab with Harvest-specific tools
- Click "Refresh pipeline" — results show new leads with scores
- Alternatively: click **Events** to see `harvest.scan.completed` and `harvest.opportunity.scored` events
- **1 click. URL: `/dept/harvest/engine`**

### Total: 7 actions, 5 URL changes, zero scrolling, zero confusion.

---

## 7. Scaling from 3 to 13 Wired Departments

### Current state (3 wired):

```
Icon Rail:
  [Forge] → full sections (5 tools: mission_today, list_goals, set_goal, review, hire_persona)
  [Code]  → full sections (2 tools: analyze, search)
  [Content] → full sections (5 tools: draft, adapt, publish, schedule, approve)
  [Harvest] → basic sections (engine has pipeline/scan/score but no agent tools yet)
  [Flow]    → basic sections
  [GTM]     → basic sections (engine has CRM/outreach but no agent tools yet)
  [Finance] → basic sections
  ...7 more → basic sections
```

### After Priority 1 gap is closed (all 13 wired):

Each department's manifest gains more `tools` → more capabilities → richer Actions catalog → more Engine features. The UI automatically adapts:

1. **Sidebar sections:** `tabsFromDepartment()` returns more tabs as manifests grow. No UI code change.
2. **Actions catalog:** More capability cards appear as tools are registered. No UI code change.
3. **Engine section:** Already conditionally renders department-specific tools (code analyze, content draft, harvest pipeline). Each new department's engine tools get similar treatment.
4. **Chat:** Already routes to per-department agents with per-department tools via `ScopedToolRegistry`. New tools automatically available in chat.

The only UI work needed per department: add department-specific panels to EngineTab (like the existing code/content/harvest conditional blocks). Everything else scales from the manifest.

---

## 8. Summary — Architecture to UI Mapping

```
Architecture                          UI
─────────────                         ──
DepartmentRegistry (13 depts)    →    Icon rail (13 icons)
DepartmentManifest.tabs          →    Section sidebar items
DepartmentManifest.quick_actions →    Actions section content
DepartmentManifest.color/icon    →    Rail accent + sidebar header
DepartmentManifest.ui            →    Dashboard cards, custom sections
RegistrationContext.tools        →    Engine section + Settings toggles
RegistrationContext.event_handlers →  Events section subscription
RegistrationContext.job_handlers →    Bottom panel execution logs
AgentRuntime.stream()            →    Chat section + context panel tool cards
AG-UI STATE_DELTA                →    A2UI components inline in chat
Capability CRUD APIs             →    Actions section capability catalog
AppState (shared infra)          →    Global rail pages (Home, Chat, DB, Flows, Terminal)
```

Every UI zone traces back to an architecture concept. No orphaned UI elements. No architecture concepts hidden from the user.

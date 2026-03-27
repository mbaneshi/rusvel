# Chapter 5 — Frontend

## Stack

- **SvelteKit 5** + Vite 6
- **Tailwind CSS 4** with typography plugin
- **TypeScript 5.7** (strict)
- **pnpm** package manager (never npm)
- **Playwright** for E2E + visual regression

## Key Dependencies

| Package | Purpose |
|---------|---------|
| @xyflow/svelte | DAG/flow diagram builder |
| xterm + @xterm/addon-fit | Browser terminal |
| svelte-streamdown | Streaming markdown renderer |
| svelte-sonner | Toast notifications |
| layerchart + D3 | Charts and data viz |
| bits-ui | Headless component library |
| driver.js | Interactive product tours |
| lucide-svelte | Icon library |

## Route Tree

```
/                           Dashboard (session stats, goals, activity, events)
/chat                       God Agent Chat (global conversations)
/approvals                  Human-in-the-loop job queue (ADR-008)
/database/schema            Table schema inspector
/database/tables            Table browser with pagination
/database/sql               SQL query executor (read-only)
/flows                      DAG workflow builder & executor
/terminal                   Standalone terminal (xterm.js)
/knowledge                  RAG knowledge base (ingest/search/browse)
/settings                   Health check, API status
/dept/[id]                  Department dashboard (dynamic route)
```

### Department Page Tabs

Each `/dept/[id]` page has a tabbed sidebar panel:

```
actions     Quick actions + Capability Builder (!capability)
engine      Config: model, effort, budget, tools, system prompt
terminal    PTY-backed terminal pane (xterm.js + WebSocket)
agents      CRUD agent profiles (@agent-name mentions)
workflows   Workflow definitions (create/delete/run)
skills      Prompt templates (/skill-name invocation)
rules       System prompt rules (auto-appended)
mcp         MCP server registrations
hooks       Event-triggered automations
dirs        Project directory browser
events      Event timeline
```

## Component Inventory

### UI Primitives (15)
Avatar, Badge, Button, Card, ConfirmDialog, Dialog, EmptyState, Input, Modal, ProgressBar, Select, Separator, Skeleton, Spinner, Tabs, Textarea, Toggle, Tooltip, SectionHeader

### Chat (7)
- **DepartmentChat.svelte** — Main chat panel with streaming, tool calls, config UI, history sidebar
- **ChatTopBar.svelte** — Title bar
- **ChatSidebar.svelte** — Conversation history
- **ToolCallCard.svelte** — Tool invocation display (status, expandable I/O, colored border)
- **ApprovalCard.svelte** — Pending approval display (approve/reject buttons, yellow border)

### Department (11)
- **DepartmentPanel.svelte** — Resizable sidebar with all tabs, color-coded header
- **ActionsTab** — Quick actions + "Build Capability" button
- **AgentsTab** — Agent CRUD (name, role, model, instructions)
- **SkillsTab** — Skills management
- **RulesTab** — Rules management
- **McpTab** — MCP server list
- **HooksTab** — Event hook management
- **WorkflowsTab** — Workflow definitions
- **DirsTab** — Project directory browser
- **EventsTab** — Event timeline
- **EngineTab** — LLM config (model dropdown, effort, budget, tools, system prompt)

### Onboarding (4)
- **OnboardingChecklist** — 5-step tracker (session, goal, plan, chat, agent)
- **ProductTour** — driver.js guided tour
- **CommandPalette** — Cmd+K action search
- **DeptHelpTooltip** — Contextual help

### Workflow (2)
- **WorkflowBuilder.svelte** — Placeholder/not fully used
- **AgentNode.svelte** — Node visualization

### Approval (1)
- **ApprovalQueue.svelte** — Job queue with approve/reject actions, badge counter

### Terminal (1)
- **DeptTerminal.svelte** — xterm.js wrapper with WebSocket binary frames

---

## Stores

```typescript
// Session management
sessions: Writable<SessionSummary[]>
activeSession: Writable<SessionSummary | null>

// UI state
sidebarOpen: Writable<boolean>           // default: true
panelOpen: Writable<boolean>             // default: true
sidebarWidth: Writable<number>           // default: 256px
panelWidth: Writable<number>             // default: 288px

// Global
departments: Writable<DepartmentDef[]>
commandPaletteOpen: Writable<boolean>
pendingCommand: Writable<{ prompt: string } | null>

// Approvals
pendingApprovalCount: Writable<number>
refreshPendingApprovalCount(): Promise<void>

// Onboarding (localStorage-backed)
onboarding: { sessionCreated, goalAdded, planGenerated, deptChatUsed, agentCreated, dismissed, tourCompleted }
```

---

## Communication Patterns

### REST
Sessions, config, CRUD operations (agents, skills, rules, hooks, MCP servers, workflows), database, approvals, analytics.

### Server-Sent Events (SSE)
All chat endpoints stream via SSE:
```
POST /api/dept/{dept}/chat    → text_delta, tool_call_start, tool_call_end, run_completed, run_failed
POST /api/chat                → same event types
POST /api/capability/build    → text deltas + cost_usd
POST /api/help                → streaming AI help
```

### WebSocket
Terminal connections:
```
GET /api/terminal/ws?pane_id={id}   → binary frames (PTY output)
POST /api/terminal/pane/{id}/resize → resize notification
```

---

## App Shell Layout

```
┌─────────────────────────────────────────────────────┐
│ Sidebar (resizable 3%-28%)  │  Main Content Area     │
│                             │                        │
│ Logo "R" (collapsible)      │  Route-specific page   │
│ Session switcher            │                        │
│ + New Session button        │  For /dept/[id]:       │
│                             │  ┌──────┬──────────┐  │
│ Static nav:                 │  │Panel │ Chat     │  │
│  Chat, Approvals (badge),   │  │(tabs)│ (stream) │  │
│  Dashboard, Database,       │  │      │          │  │
│  Flows, Terminal            │  └──────┴──────────┘  │
│                             │                        │
│ Dynamic nav:                │                        │
│  [12 departments w/ icons]  │                        │
│                             │                        │
│ Settings                    │                        │
│ Cmd+K hint                  │                        │
└─────────────────────────────────────────────────────┘
```

Overlays: Toaster (sonner), CommandPalette, OnboardingChecklist, ProductTour

---

## Feature Implementation Status

### Fully Working
- Session management (create, switch, list)
- Dashboard with goals, events, analytics, activity charts
- Department chat with SSE streaming + tool call display
- Approval queue with approve/reject + sidebar badge
- Agent/Skill/Rule/Hook/MCP/Workflow CRUD (all departments)
- Database schema inspector + table browser + SQL executor
- Knowledge ingest/search/browse with semantic scores
- Flow DAG execution + terminal pane display
- Terminal integration (xterm.js + WebSocket)
- Sidebar resize + collapse
- Command palette (Cmd+K)
- Onboarding checklist + product tour
- Department color theming from backend manifest
- Streaming markdown rendering

### Minimal / Placeholder
- WorkflowBuilder and AgentNode components exist but aren't used in production routes (JSON input only for flow creation)
- DelegationTerminal component exists but unused
- Content/Harvest/Code engine UIs use API directly, no dedicated UI panels beyond chat
- Settings page is basic (health check + approvals list)

### Not Implemented
- Visual flow graph editor (xyflow imported but not fully wired)
- Multi-user presence / real-time collaboration
- Auth/RBAC UI
- Profile editor (endpoints exist, minimal frontend)

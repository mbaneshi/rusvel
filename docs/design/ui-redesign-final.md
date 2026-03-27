# RUSVEL UI Redesign — Final Design Spec

**Date:** 2026-03-27
**Status:** Ready for implementation
**Based on:** State Report (7 chapters), 3 research answers, architecture analysis

---

## 1. Findings

### What the research revealed

We analyzed 10 production apps (GitHub, Linear, Retool, Windmill, n8n, Langflow, Cursor, Claude Code, Slack, VS Code) and found consistent patterns:

| Finding | Evidence |
|---------|----------|
| Apps with 10+ workspaces use an **icon rail**, not horizontal tabs | Slack (servers), Discord (servers), VS Code (activity bar), n8n (sections) |
| The **right panel** is always for contextual detail — properties, config, or AI chat | GitHub (metadata), Linear (task props), n8n (node config), Cursor (AI chat) |
| **Chat as a permanent column** doesn't scale — it competes with structured content | Current RUSVEL: 3 columns fight for 1440px. Cursor solves this with collapsible right panel |
| **Bottom panel** for terminal/logs is universal in builder tools | VS Code, Cursor, n8n, Retool all use collapsible bottom panels |
| **Manifest-driven nav** works when the backend already declares capabilities | RUSVEL's `DepartmentManifest.tabs` already controls which tabs appear — the UI just reads it |
| **Chat placement** should be layered: full-page when focused, quick-access when not | Claude.ai (full page), Cursor (sidebar), Cmd+K (overlay) — different tools for different depths |

### What's wrong with the current layout

```
Current: 3 columns competing for horizontal space

[Sidebar 256px] [DeptPanel 288px] [Chat ~896px]
     17.8%            20%              62.2%

Problems:
1. 20 items in one flat sidebar (6 global + 13 depts + settings)
2. Department sections squeezed to 288px (10px font tabs)
3. Chat permanently visible even when configuring agents/skills
4. No URLs for sections — can't bookmark /dept/forge/skills
5. 4 collapse states (sidebar × panel) = cognitive overhead
6. Main content width varies by collapse state — unstable layout
```

### What the backend already supports

From State Report chapters 3-4:

- `DepartmentManifest` declares tabs, quick_actions, color, icon, capabilities — the sidebar can read this directly
- `tabsFromDepartment()` already filters sections per department — sparse depts show fewer items
- All ~105 API routes are department-scoped and section-agnostic — zero backend changes needed
- `RegistrationContext.tools` gives each department its own tool set — Engine section content is already per-department
- `UiContribution` interface supports `dashboard_cards` and `custom_components` — future A2UI integration point

---

## 2. The Layout

### Five zones, each mapped to a backend concept

```
┌──────┬──────────────┬──────────────────────────┬──────────────────┐
│ ICON │  SECTION     │  MAIN CONTENT            │  CONTEXT PANEL   │
│ RAIL │  SIDEBAR     │  (scrollable)            │  (collapsible)   │
│ 48px │  200px       │  flexible                │  320px           │
│      │              │                          │                  │
│ Infra│ Dept sections│  Full-width content      │  AI chat         │
│  +   │ from manifest│  for selected section    │  or properties   │
│ Dept │              │                          │  or exec output  │
│ icons│ Only on      │                          │                  │
│      │ /dept/* pages│                          │                  │
├──────┴──────────────┴──────────────────────────┴──────────────────┤
│  BOTTOM PANEL (collapsible)                                       │
│  Terminal / Execution logs / Event stream                         │
└───────────────────────────────────────────────────────────────────┘
```

### Zone → Backend mapping

| Zone | Width | Backend Source | What it shows |
|------|-------|---------------|---------------|
| **Icon Rail** | 48px | `AppState` (top) + `DepartmentRegistry` (bottom) | Global pages: Home, Chat, Approvals, DB, Flows, Terminal. Then 13 department icons. Settings at bottom. |
| **Section Sidebar** | 200px | `DepartmentManifest.tabs` via `tabsFromDepartment()` | Sections for the active department. Dept header with icon + color. Session switcher. Only appears on `/dept/*` routes. |
| **Main Content** | Flexible | CRUD APIs (`/api/agents`, `/api/skills`, etc.) + engine routes (`/api/dept/code/analyze`) + chat SSE | Full-width content: card grids, forms, chat messages, workflow canvas, dashboards. Scrollable. |
| **Context Panel** | 320px | `AgentRuntime.stream()` (chat) + entity detail APIs (properties) + `JobPort` (exec) | Three modes: (1) Quick AI chat with department agent, (2) Selected item properties, (3) Execution output with tool calls and approvals. Collapsible via button or Cmd+J. |
| **Bottom Panel** | ~200px | `TerminalPort` (WebSocket) + `JobPort` (execution) + `EventPort` (stream) | Terminal PTY output, job execution status, live event stream. Collapsible via Cmd+`. |

### Why each zone exists

**Icon Rail** — The `DepartmentRegistry` has exactly 13 entries. An icon rail fits them in ~700px vertical space without scrolling. A horizontal tab bar needs 1300px. A flat sidebar list buries departments at position 7+. The rail also separates infrastructure (`AppState` shared services) from departments (domain-specific apps) with a visual divider — mirroring the architecture's separation.

**Section Sidebar** — `DepartmentManifest.tabs` already declares what each department supports. Forge declares `['actions', 'engine', 'agents', 'skills', 'rules', 'workflows', 'mcp', 'hooks']`. Legal might only declare `['actions', 'agents', 'skills', 'rules']`. The sidebar reads this and renders accordingly. As departments get tools wired (Priority 1 gap — 10 departments), their manifests grow, and the sidebar automatically shows more sections. No frontend code changes.

**Main Content** — Currently 288px. Needs to be full-width because:
- `AgentsTab` (130 LOC) renders agent cards — need grid layout at full width
- `WorkflowsTab` (216 LOC) uses `WorkflowBuilder` — needs canvas space for `@xyflow/svelte`
- `EngineTab` (237 LOC) has input + result — needs side-by-side layout
- `DepartmentChat` streams tool calls inline — needs room for ToolCallCard + ApprovalCard

**Context Panel** — The `AgentRuntime` emits `AgentEvent::ToolCall` and `AgentEvent::ToolResult` during chat. Currently these render inline in the chat stream. With a context panel, the chat stays clean (text only) while tool details expand in the right panel. This matches Cursor's pattern: code in main area, AI tool output in side panel.

**Bottom Panel** — `TerminalPort` provides PTY output via WebSocket. Currently it's a tab inside the 288px department panel — unusable. As a bottom panel, it's accessible from any section. Same for `JobPort` execution logs and `EventPort` streams.

---

## 3. Reasoning — Why This Over Alternatives

### Alternative A: Horizontal department tabs (our original proposal)

```
[Forge] [Code] [Content] [Harvest] [Flow] [GTM] [Finance] ...
```

**Rejected because:** 13 tabs × ~100px = 1300px. Overflows on laptops. Requires horizontal scroll or truncation. The icon rail fits the same information in 48px width.

### Alternative B: Keep current sidebar but move panel content into it

```
[Sidebar with depts + sections merged]  [Main content]
```

**Rejected because:** Mixing global nav + department list + section nav in one sidebar creates 20+ items. The current problem is exactly this — too many things in one vertical list.

### Alternative C: Top bar for departments + sidebar for sections (original two-level-nav)

```
[Top bar: static nav + dept pills]
[Section sidebar] [Main content]
```

**Viable but suboptimal.** The top bar eats 80-100px of vertical space for two rows (static nav + dept bar). The icon rail achieves the same with 48px of horizontal space and zero vertical cost.

### Alternative D: The chosen design (icon rail + sidebar + main + panels)

**Chosen because:**
1. Every zone maps to a specific backend concept — no orphaned UI, no hidden architecture
2. The icon rail handles both infrastructure and 13 departments in 48px
3. Manifest-driven sidebar automatically scales as departments gain capabilities
4. Context panel enables chat-alongside-work pattern (Cursor model) without permanent 3-column layout
5. Bottom panel brings terminal and execution into every context without separate pages
6. SvelteKit nested routes give proper URLs (`/dept/forge/skills`) with zero new API endpoints

---

## 4. Frontend Implementation

### Route structure

```
src/routes/
├── +layout.svelte                              REWRITE — icon rail + top bar (minimal)
├── dept/[id]/
│   ├── +layout.svelte                          NEW — section sidebar + context panel + bottom panel
│   ├── +page.svelte                            MODIFY — redirect to /actions
│   ├── actions/+page.svelte                    NEW — capability catalog + quick actions
│   ├── engine/+page.svelte                     NEW — wraps EngineTab
│   ├── agents/+page.svelte                     NEW — wraps AgentsTab
│   ├── skills/+page.svelte                     NEW — wraps SkillsTab
│   ├── rules/+page.svelte                      NEW — wraps RulesTab
│   ├── workflows/+page.svelte                  NEW — wraps WorkflowsTab
│   ├── mcp/+page.svelte                        NEW — wraps McpTab
│   ├── hooks/+page.svelte                      NEW — wraps HooksTab
│   ├── terminal/+page.svelte                   NEW — wraps DeptTerminal
│   ├── events/+page.svelte                     NEW — wraps EventsTab
│   ├── dirs/+page.svelte                       NEW — wraps DirsTab
│   ├── chat/+page.svelte                       NEW — wraps DepartmentChat (full width)
│   └── settings/+page.svelte                   NEW — dept config (model, effort, tools)
├── chat/+page.svelte                           UNCHANGED
├── approvals/+page.svelte                      UNCHANGED
├── database/{schema,tables,sql}/+page.svelte   UNCHANGED
├── flows/+page.svelte                          UNCHANGED
├── knowledge/+page.svelte                      UNCHANGED
├── terminal/+page.svelte                       UNCHANGED
└── settings/+page.svelte                       UNCHANGED
```

### File count

| Action | Count | Description |
|--------|-------|-------------|
| Rewrite | 1 | Root `+layout.svelte` (sidebar → icon rail) |
| New | 15 | Dept layout + 13 section pages + dept redirect |
| Modify | 1 | Dept `+page.svelte` (redirect to /actions) |
| Retire | 1 | `DepartmentPanel.svelte` (logic absorbed by routes) |
| Unchanged | 11 | All existing tab components reused as-is |
| Unchanged | 8 | All non-dept routes |
| **Total** | **17 changed** | **~500 new lines, 0 backend changes** |

### Component reuse

Every existing tab component works without modification:

```
ActionsTab.svelte   (21 LOC)  → /dept/[id]/actions
EngineTab.svelte   (237 LOC)  → /dept/[id]/engine
AgentsTab.svelte   (130 LOC)  → /dept/[id]/agents
SkillsTab.svelte   (130 LOC)  → /dept/[id]/skills
RulesTab.svelte    (132 LOC)  → /dept/[id]/rules
WorkflowsTab.svelte(216 LOC)  → /dept/[id]/workflows
McpTab.svelte      (126 LOC)  → /dept/[id]/mcp
HooksTab.svelte    (147 LOC)  → /dept/[id]/hooks
DirsTab.svelte      (54 LOC)  → /dept/[id]/dirs
EventsTab.svelte    (54 LOC)  → /dept/[id]/events
DepartmentChat.svelte (full)  → /dept/[id]/chat
```

They all use `w-full` and will naturally expand to the main content area width.

### Store changes

| Store | Now | After |
|-------|-----|-------|
| `sidebarOpen` / `sidebarWidth` | Global sidebar | Unused (rail is fixed 48px) |
| `panelOpen` / `panelWidth` | DepartmentPanel | Section sidebar + context panel collapse |
| `pendingCommand` | Quick action → chat | Same + `goto('/dept/{id}/chat')` on set |
| `departments` | Sidebar list | Icon rail |
| New: `contextPanelOpen` | — | Right panel toggle |
| New: `bottomPanelOpen` | — | Bottom panel toggle |

---

## 5. Backend Changes

**None.**

All ~105 API routes remain unchanged. The frontend restructuring is purely a layout and routing change. The API is already structured as department-scoped + section-agnostic:

```
Per-dept (existing):     POST /api/dept/{dept}/chat
                         GET  /api/dept/{dept}/config
                         GET  /api/dept/{dept}/events

CRUD (existing):         GET  /api/agents?engine={dept}
                         GET  /api/skills?engine={dept}
                         GET  /api/rules?engine={dept}

Engine (existing):       POST /api/dept/code/analyze
                         POST /api/dept/content/draft
                         GET  /api/dept/harvest/pipeline

Manifest (existing):     GET  /api/departments → DepartmentDef[]
```

---

## 6. Compliance

### ADR alignment

| ADR | Impact |
|-----|--------|
| ADR-014 (DepartmentApp) | **Strengthened** — each dept gets its own icon, URL namespace, manifest-driven sidebar |
| ADR-008 (Approval gates) | **Preserved** — ApprovalCard renders in context panel during chat |
| ADR-003 (Single job queue) | **Preserved** — bottom panel shows job execution from the single queue |
| ADR-005 (String event kinds) | **Preserved** — Events section and bottom panel event stream unchanged |
| ADR-009 (AgentPort not LlmPort) | **Unchanged** — no engine changes |

### Feature preservation

Every feature from State Report Ch. 5 "Fully Working" is preserved:

- Session management → rail + sidebar switcher
- Department chat + SSE streaming → `/dept/[id]/chat` section (full width)
- Tool call display → context panel + inline in chat
- Approval queue + badge → rail icon with badge
- All CRUD (agents, skills, rules, hooks, MCP, workflows) → section pages
- Database browser → global page (unchanged)
- Knowledge base → global page (unchanged)
- Flow DAG → global page (unchanged)
- Terminal → bottom panel + `/dept/[id]/terminal` section
- Command palette (Cmd+K) → unchanged overlay
- Onboarding → unchanged overlays
- Department color theming → rail icon accent + sidebar header

### Future readiness

| Future feature | How this layout supports it |
|---------------|-----------------------------|
| A2UI components | Render inline in chat (main area) + as dashboard cards in Actions section |
| Capability catalog | Actions section becomes the catalog — grid of agent/skill/rule/workflow cards |
| Self-extension loop | Agent creates skill → event fires → card appears in Actions catalog live |
| 10 more dept tools wired | Manifest grows → sidebar shows more sections → no UI code change |
| Visual flow editor (@xyflow) | `/dept/[id]/workflows` main area has room for full-width canvas |
| AG-UI protocol | SSE events route to same rendering pipeline, just typed differently |

---

## 7. Implementation Order

| Phase | What | Files | Risk |
|-------|------|-------|------|
| 1 | Icon rail (replace sidebar) | 1 rewrite | Medium — changes every page |
| 2 | Dept layout (section sidebar + context panel + bottom panel) | 1 new | Low — new file only |
| 3 | Section page wrappers | 13 new | Low — thin wrappers |
| 4 | Default redirect + pendingCommand fix | 1 modify | Low — targeted |
| 5 | Context panel (right) + bottom panel | In phase 2 layout | Medium — new components |
| 6 | Remove DepartmentPanel import | 1 modify | Low — cleanup |

### Verification

```bash
cd frontend && pnpm dev

# Route tests:
# /                      → rail + full-width dashboard (no sidebar)
# /dept/forge            → redirects to /dept/forge/actions
# /dept/forge/skills     → rail + sidebar (Skills active) + skills grid in main
# /dept/forge/chat       → rail + sidebar (Chat active) + full-width chat
# Click Code in rail     → /dept/code/actions, sidebar updates
# Cmd+K                  → command palette works
# Browser back           → correct section + sidebar state
# Context panel toggle   → collapses/expands right panel
# Bottom panel toggle    → collapses/expands terminal

pnpm check              # 0 TypeScript errors
pnpm build              # Production build succeeds
```

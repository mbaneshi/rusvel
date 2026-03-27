# Proposal: Two-Level Department Navigation

**Date:** 2026-03-27
**Status:** Draft
**Author:** AI-assisted design
**References:** State Report (2026-03-27), ADR-014, Modular UI Blueprint, Web UI Architecture

---

## 1. Reasoning

### Why now

The State Report (Chapter 7) reveals the single biggest leverage gap: **10 of 13 departments have fully implemented engines but zero agent tools wired**. As we wire those tools, each department gains real interactive capabilities — code analysis, content drafting, harvest scanning, financial projections, etc. The current UI cannot surface this depth. A 288px tabbed panel was adequate when departments were mostly chat wrappers; it won't hold up as departments become full workspaces.

### What the codebase already tells us

From the State Report:

| Metric | Value | Source |
|--------|-------|--------|
| Frontend components | 66 Svelte | Ch. 5 |
| Department tab components | 11 | Ch. 5: DepartmentPanel + 10 tabs |
| API routes | ~105 | Ch. 4 |
| Per-department API routes | 6 x 13 = 78 | Ch. 4: chat, conversations, history, config get/put, events |
| CRUD resource routes | 27 | Ch. 4: agents(5), skills(5), rules(5), mcp(4), workflows(6), hooks(5) |
| Engine-specific routes | 12 | Ch. 4: code(2), content(5), harvest(5) |
| Departments fully wired | 3 | Ch. 7: forge, code, content |
| Departments needing tool wiring | 10 | Ch. 7: harvest through infra |
| Stores managing layout | 4 | Ch. 5: sidebarOpen, panelOpen, sidebarWidth, panelWidth |

The backend is already organized around **department + section** (e.g., `/api/dept/{dept}/chat`, `/api/agents?engine={dept}`, `/api/dept/code/analyze`). The frontend should mirror this hierarchy.

### What doesn't work today

**Three horizontal zones competing for 1440px:**

```
[Sidebar 256px] [DepartmentPanel 288px] [Chat ~896px]
     17.8%              20%                  62.2%
```

Per the State Report Chapter 5 "App Shell Layout":

- The sidebar mixes 6 static nav items + 13 departments + Settings = 20 items in a vertical scroll. Departments start at position 7, often below the fold.
- DepartmentPanel (288px) hosts 11 tabs via `text-[10px]` font buttons. Tab content renders CRUD forms designed for this cramped width.
- The DepartmentChat component gets whatever remains — its width varies based on sidebar/panel collapse state, creating 4 possible layout configurations (`sidebarOpen` x `panelOpen`).

**No URL-level section identity.** The State Report's route tree shows `/dept/[id]` as a single flat route. Switching from Skills to Agents changes component state but not the URL. This blocks bookmarking, link-sharing, and browser history.

### Alignment with existing design docs

| Document | What it proposed | How this extends it |
|----------|-----------------|-------------------|
| Modular UI Blueprint | Top bar + nav sidebar + main content + code sidebar | Departments become primary top-level nav; sections become sidebar |
| Web UI Architecture | "Whatever page you're on gets AI sidebar" | Chat becomes a section you navigate to, not a permanent fixture |
| ADR-014 (DepartmentApp) | Each dept is a self-contained app with manifest, tools, events | UI reflects this: each dept gets its own URL namespace + section sidebar |
| Department Scaling Proposal | Extensible department contributions | `UiContribution` in manifest drives which sections appear |

---

## 2. Before & After

### Before (Current — from State Report Ch. 5)

```
┌────────────────┬──────────────────┬──────────────────────────────┐
│  SIDEBAR       │  DEPT PANEL      │  CHAT                        │
│  256px (3-28%) │  288px resizable │  remaining (~62%)             │
│                │                  │                              │
│  R  RUSVEL  <  │  ≡ Forge Dept  < │  ≡ Forge Department    ⚙ +  │
│                │  forge department│                              │
│  Session: ▾    │                  │       ≡                      │
│  + New Session │  Actions|Engine| │   Forge Department           │
│                │  Agents|Flows|Sk │   Ready to work.             │
│  □ Chat        │  ills|Rules|MCP │                              │
│  ☑ Approvals   │  |Hooks|Dirs|Ev │                              │
│  ⊞ Dashboard   │                  │                              │
│  ⊟ Database    │  ┌─────────────┐│                              │
│  ⑂ Flows       │  │Build Capab. ││                              │
│  ⊞ Terminal    │  │Daily plan   ││                              │
│  ─────────     │  │Review prog. ││                              │
│  ⚒ Forge ●     │  │Set new goal ││                              │
│  ◇ Code        │  └─────────────┘│                              │
│  ✎ Content     │                  │                              │
│  🔍 Harvest    │                  │                              │
│  ⑂ Flow        │                  │                              │
│  🚀 GTM        │                  │                              │
│  $ Finance     │                  │                              │
│  📦 Product    │                  │ ┌──────────────────────────┐ │
│  📈 Growth     │                  │ │ Ask Forge Department...  │ │
│  ⑂ Distro      │                  │ └──────────────────────────┘ │
│  ⚖ Legal       │                  │                              │
│  🎧 Support    │                  │                              │
│  ⊟ Infra       │                  │                              │
│  ─────────     │                  │                              │
│  ⚙ Settings    │                  │                              │
└────────────────┴──────────────────┴──────────────────────────────┘

Route: /dept/forge              (flat — no section in URL)
Components rendered: 3 panels   (Sidebar in +layout, DepartmentPanel, DepartmentChat)
Stores active: 4               (sidebarOpen, sidebarWidth, panelOpen, panelWidth)
Collapse states: 4             (open/open, open/closed, closed/open, closed/closed)
```

### After (Proposed)

```
┌──────────────────────────────────────────────────────────────────┐
│  R │ Chat  Dashboard  Approvals  DB  Flows  Terminal  ⚙  │ ▾ s │
├──────────────────────────────────────────────────────────────────┤
│ ⚒Forge │ ◇Code │ ✎Content │ 🔍Harvest │ ⑂Flow │ 🚀GTM │ ... │
├───────────────┬──────────────────────────────────────────────────┤
│               │                                                  │
│  ○ Actions    │  Skills                        /dept/forge/skills│
│  ○ Engine     │                                                  │
│  ○ Workflows  │  ┌─────────┐ ┌─────────┐ ┌─────────┐          │
│  ● Skills     │  │+ Create │ │Discover │ │ Import  │          │
│  ○ Rules      │  └─────────┘ └─────────┘ └─────────┘          │
│  ○ Agents     │                                                  │
│  ○ MCP        │  ┌────────────────┐ ┌────────────────┐          │
│  ○ Hooks      │  │ deploy-check   │ │ code-review    │          │
│  ○ Terminal   │  │ Run deploy...  │ │ Review PR...   │          │
│  ○ Events     │  │ ⚡ Active       │ │ ⚡ Active       │          │
│  ──────────   │  └────────────────┘ └────────────────┘          │
│  ○ Chat       │  ┌────────────────┐ ┌────────────────┐          │
│  ○ Settings   │  │ daily-standup  │ │ test-runner    │          │
│  ──────────   │  │ Generate st... │ │ Run test su... │          │
│  Session: ▾   │  │ ○ Disabled     │ │ ⚡ Active       │          │
│               │  └────────────────┘ └────────────────┘          │
└───────────────┴──────────────────────────────────────────────────┘

Route: /dept/forge/skills       (section in URL — bookmarkable)
Components rendered: 2 panels   (Section sidebar in dept layout, SkillsTab in main)
Stores active: 2               (panelOpen, panelWidth — for sidebar collapse)
Collapse states: 2             (sidebar open/closed)
```

---

## 3. UX Improvements

### 3.1 Department Access (13 departments)
| Aspect | Before | After |
|--------|--------|-------|
| Position | Items 7-19 in sidebar (below fold) | Dedicated top bar, always visible |
| Space | ~14px height per item x 13 = 182px scroll zone | Horizontal icons, single row |
| Switching | Click + potential scroll | Single click, zero scroll |
| Visual identity | Same style as utility nav | Accent-colored pills with department icons |

### 3.2 Section Navigation (11 tabs → sidebar items)
| Aspect | Before | After |
|--------|--------|-------|
| Tab font | `text-[10px]` (10px) | `text-sm` (14px) — readable |
| Tab bar | Horizontal scroll, overflow hidden | Vertical sidebar, all visible |
| URL | `/dept/forge` for all tabs | `/dept/forge/skills`, `/dept/forge/chat`, etc. |
| Bookmarkable | No | Yes |
| Browser back | No history entries | Full history stack |
| Deep links | Impossible | Share `/dept/code/agents` with team |

### 3.3 Content Area
| Section | Before (288px) | After (~1100px) |
|---------|---------------|-----------------|
| **AgentsTab** (130 LOC) | Narrow cards, 1 column | Card grid `md:grid-cols-2 lg:grid-cols-3` |
| **SkillsTab** (130 LOC) | Single column, small forms | Grid cards + spacious create form |
| **RulesTab** (132 LOC) | Content truncated to 80 chars | Full content visible |
| **WorkflowsTab** (216 LOC) | Squeezed step builder | Full-width DAG builder (xyflow ready) |
| **EngineTab** (237 LOC) | Cramped input + result below | Side-by-side: input left, results right |
| **HooksTab** (147 LOC) | Narrow list, tiny toggles | Card grid with clear event/action display |
| **McpTab** (126 LOC) | Narrow server list | Rich server cards with status |
| **EventsTab** (54 LOC) | Flat list | Timeline with expandable JSON payloads |
| **DirsTab** (54 LOC) | Basic list | File-tree style with add/remove |
| **ActionsTab** (21 LOC) | Small buttons | Action card grid, richer Build Capability |
| **DepartmentChat** (full component) | Variable width, always visible | Full-width when focused, hidden when not |

### 3.4 Cognitive Load
- **Before:** 3 panels x 2 collapse states = 4 layout permutations. User must decide: do I collapse the sidebar? The panel?
- **After:** 2 hierarchy levels. Pick department (top). Pick section (sidebar). Main pane shows that section. One optional collapse (sidebar).

---

## 4. Frontend Changes

### 4.1 Route Structure (SvelteKit Nested Layouts)

Current route tree (from State Report Ch. 5):
```
/                    Dashboard
/chat                God Agent Chat
/approvals           Approval queue
/database/{schema,tables,sql}
/flows               DAG workflow builder
/terminal            Standalone terminal
/knowledge           RAG knowledge base
/settings            Health check
/dept/[id]           Department (flat)     ← single route, all tabs internal
```

Proposed route tree:
```
/                    Dashboard             (unchanged)
/chat                God Agent Chat        (unchanged)
/approvals           Approval queue        (unchanged)
/database/{schema,tables,sql}             (unchanged)
/flows               DAG workflow builder  (unchanged)
/terminal            Standalone terminal   (unchanged)
/knowledge           RAG knowledge base    (unchanged)
/settings            Health check          (unchanged)
/dept/[id]/          Redirect → /actions
/dept/[id]/actions   Quick actions + Build Capability
/dept/[id]/engine    Model, effort, budget, tools, system prompt
/dept/[id]/agents    Agent CRUD
/dept/[id]/skills    Skill CRUD
/dept/[id]/rules     Rule CRUD
/dept/[id]/workflows Workflow CRUD + run
/dept/[id]/mcp       MCP server CRUD
/dept/[id]/hooks     Hook CRUD
/dept/[id]/terminal  Department-scoped terminal (xterm.js + WS)
/dept/[id]/events    Event timeline
/dept/[id]/dirs      Working directories
/dept/[id]/chat      Department chat (SSE streaming)
/dept/[id]/settings  Department config editor
```

Every non-dept route is **unchanged**. No existing URLs break.

### 4.2 File Changes

| File | Action | Lines | Notes |
|------|--------|-------|-------|
| `src/routes/+layout.svelte` | Rewrite | ~180→~140 | PaneGroup sidebar → top bar + dept bar |
| `src/routes/dept/[id]/+layout.svelte` | New | ~80 | Section sidebar + `{@render children()}` |
| `src/routes/dept/[id]/+page.svelte` | Modify | 44→~8 | Redirect to `/actions` via `$effect` + `goto()` |
| `src/routes/dept/[id]/actions/+page.svelte` | New | ~15 | Wraps `ActionsTab` |
| `src/routes/dept/[id]/engine/+page.svelte` | New | ~15 | Wraps `EngineTab` |
| `src/routes/dept/[id]/agents/+page.svelte` | New | ~15 | Wraps `AgentsTab` |
| `src/routes/dept/[id]/skills/+page.svelte` | New | ~15 | Wraps `SkillsTab` |
| `src/routes/dept/[id]/rules/+page.svelte` | New | ~15 | Wraps `RulesTab` |
| `src/routes/dept/[id]/workflows/+page.svelte` | New | ~15 | Wraps `WorkflowsTab` |
| `src/routes/dept/[id]/mcp/+page.svelte` | New | ~15 | Wraps `McpTab` |
| `src/routes/dept/[id]/hooks/+page.svelte` | New | ~15 | Wraps `HooksTab` |
| `src/routes/dept/[id]/terminal/+page.svelte` | New | ~20 | Wraps `DeptTerminal` + pane init |
| `src/routes/dept/[id]/events/+page.svelte` | New | ~15 | Wraps `EventsTab` |
| `src/routes/dept/[id]/dirs/+page.svelte` | New | ~15 | Wraps `DirsTab` |
| `src/routes/dept/[id]/chat/+page.svelte` | New | ~15 | Wraps `DepartmentChat` (full width) |
| `src/routes/dept/[id]/settings/+page.svelte` | New | ~30 | Config form (model, effort, tools, prompt) |

**Total: 2 modified + 15 new files. ~430 new lines of code.**

### 4.3 Component Reuse (Zero Rewrites)

All 11 existing department components are reused without modification:

| Component | LOC | Used in section page |
|-----------|-----|---------------------|
| `ActionsTab.svelte` | 21 | `/actions` |
| `EngineTab.svelte` | 237 | `/engine` |
| `AgentsTab.svelte` | 130 | `/agents` |
| `SkillsTab.svelte` | 130 | `/skills` |
| `RulesTab.svelte` | 132 | `/rules` |
| `McpTab.svelte` | 126 | `/mcp` |
| `HooksTab.svelte` | 147 | `/hooks` |
| `WorkflowsTab.svelte` | 216 | `/workflows` |
| `DirsTab.svelte` | 54 | `/dirs` |
| `EventsTab.svelte` | 54 | `/events` |
| `DepartmentChat.svelte` | full | `/chat` |

Each section page is a thin wrapper (~15 lines) that:
1. Reads `page.params.id` to find the department from the `departments` store
2. Computes `deptHsl` via `getDeptColor(dept.color)` (existing utility in `colors.ts`)
3. Renders the tab component at full width inside `<div class="h-full overflow-y-auto p-6">`

### 4.4 Root Layout Rewrite

**Current** (`+layout.svelte`, ~400 lines): PaneGroup with resizable sidebar pane containing logo, session switcher, 20-item vertical nav, status bar.

**Proposed** (~140 lines):
```
<div class="h-screen flex flex-col">
  <!-- Top bar: logo + static nav + session switcher -->
  <nav class="h-11 border-b bg-sidebar flex items-center px-3 gap-1">
    [R] [Chat] [Approvals (badge)] [Dashboard] [DB] [Flows] [Terminal] [⚙]
    <spacer />
    [Session dropdown] [⌘K]
  </nav>

  <!-- Department bar: horizontal scrollable -->
  <nav class="h-10 border-b bg-sidebar/50 flex items-center px-3 gap-1 overflow-x-auto">
    {#each departments as d}
      <a href="/dept/{d.id}" class="pill">{DeptIcon} {d.name}</a>
    {/each}
  </nav>

  <!-- Content (full height minus bars) -->
  <div class="flex-1 overflow-hidden">
    {@render children()}
  </div>
</div>
```

**Removed:** PaneGroup/Pane/PaneResizer imports, sidebar collapse logic, vertical nav scroll.
**Preserved:** `onMount` department + session loading, approval badge refresh (45s interval), `iconMap` for Lucide icons, `DeptIcon` fallback, CommandPalette/Toaster/OnboardingChecklist/ProductTour overlays.

### 4.5 Department Layout (New)

**`/dept/[id]/+layout.svelte`** (~80 lines):

```
<div class="flex h-full">
  <!-- Section sidebar -->
  <aside class="w-52 border-r bg-card flex flex-col">
    <!-- Dept header: icon + title (colored) -->
    <!-- Section links from tabsFromDepartment(dept) -->
    <!-- Separator -->
    <!-- Chat + Settings links -->
    <!-- Separator -->
    <!-- Session info -->
  </aside>

  <!-- Main content -->
  <main class="flex-1 overflow-hidden">
    {@render children()}
  </main>
</div>
```

Section links derived from the manifest's `tabsFromDepartment()` (same function used by current DepartmentPanel, line 136). Active section highlighted with department accent color via `getDeptColor()`.

### 4.6 Store Impact

| Store | Current | After | Change |
|-------|---------|-------|--------|
| `sidebarOpen` | Controls global sidebar visibility | Unused | Remove or keep for future |
| `sidebarWidth` | Controls global sidebar width (256px) | Unused | Remove or keep for future |
| `panelOpen` | Controls DepartmentPanel visibility | Controls section sidebar | Rename to `sectionSidebarOpen` (optional) |
| `panelWidth` | Controls DepartmentPanel width (288px) | Controls section sidebar width | Reuse directly |
| `departments` | Powers sidebar list | Powers top dept bar | No change |
| `pendingCommand` | Quick action → chat injection | Same + auto-navigate to `/dept/{id}/chat` | Add reactive redirect in dept layout |
| `sessions` / `activeSession` | Session switcher in sidebar | Session switcher in top bar | No change |
| `pendingApprovalCount` | Badge in sidebar nav | Badge in top bar nav | No change |
| `commandPaletteOpen` | ⌘K overlay | Same | No change |

### 4.7 Retired Components

- **`DepartmentPanel.svelte`** (260 lines) — Its responsibilities split:
  - Tab routing → SvelteKit routes
  - Section sidebar → `/dept/[id]/+layout.svelte`
  - Tab filtering → `tabsFromDepartment()` in dept layout
  - Resize logic → Not needed (fixed sidebar width or CSS resize)
  - Color theming → Moved to dept layout

The component file stays in repo (no deletion) but is no longer imported.

---

## 5. Backend Changes

### None required.

The backend API (State Report Ch. 4) is already structured for this:

**Department-scoped routes (already exist):**
```
POST /api/dept/{dept}/chat                    → SSE stream
GET  /api/dept/{dept}/chat/conversations
GET  /api/dept/{dept}/chat/conversations/{id}
GET  /api/dept/{dept}/config
PUT  /api/dept/{dept}/config
GET  /api/dept/{dept}/events
```

**CRUD resources (already department-filtered via `?engine=` param):**
```
GET /api/agents?engine={dept}       GET /api/skills?engine={dept}
GET /api/rules?engine={dept}        GET /api/hooks
GET /api/mcp-servers                GET /api/workflows
```

**Engine-specific routes (already exist for wired departments):**
```
POST /api/dept/code/analyze         GET  /api/dept/code/search
POST /api/dept/content/draft        POST /api/dept/content/from-code
GET  /api/dept/harvest/pipeline     POST /api/dept/harvest/scan
```

**Chat handler internals unchanged** (State Report Ch. 4): The 9-step handler chain (validate dept → load config → interceptors → rules → capabilities → RAG → build AgentConfig → stream → post-completion hooks) is unaffected by frontend routing changes.

The `DepartmentManifest` already declares `UiContribution` with `tabs` and `dashboard_cards`, which is exactly what the new section sidebar consumes.

---

## 6. Compliance & Consistency

### 6.1 ADR Compliance

| ADR | Impact | Details |
|-----|--------|---------|
| ADR-014 (DepartmentApp) | **Strengthened** | The UI now mirrors the backend DepartmentApp pattern: each dept gets its own URL namespace. The manifest's `tabs`, `quick_actions`, and `UiContribution` drive section visibility. |
| ADR-008 (Human approval gates) | **Preserved** | ApprovalCard + approve/reject flow works identically in `/dept/[id]/chat`. ApprovalQueue at `/approvals` unchanged. |
| ADR-003 (Single job queue) | **Preserved** | Job submission from WorkflowsTab and engine actions unchanged. |
| ADR-005 (String event kinds) | **Preserved** | EventsTab component reused, just at full width. |
| ADR-007 (metadata: serde_json::Value) | **Unchanged** | No domain type changes. |
| ADR-009 (AgentPort not LlmPort) | **Unchanged** | No engine layer touched. |

### 6.2 Feature Preservation Matrix

Every feature from State Report Ch. 5 "Feature Implementation Status — Fully Working":

| Feature | Status | How preserved |
|---------|--------|---------------|
| Session management | Preserved | Session switcher moves to top bar, same stores |
| Dashboard (goals, events, analytics) | Preserved | `/` route unchanged |
| Department chat + SSE streaming | Preserved | DepartmentChat reused in `/dept/[id]/chat` |
| Tool call display (ToolCallCard) | Preserved | Same component, wider viewport |
| Approval queue + sidebar badge | Preserved | `/approvals` route unchanged, badge in top bar |
| Agent/Skill/Rule/Hook/MCP/Workflow CRUD | Preserved | Same tab components, new route wrappers |
| Database browser (schema, tables, SQL) | Preserved | `/database/*` routes unchanged |
| Knowledge ingest/search/browse | Preserved | `/knowledge` route unchanged |
| Flow DAG execution | Preserved | `/flows` route unchanged |
| Terminal (xterm.js + WebSocket) | Preserved | Both `/terminal` (global) and `/dept/[id]/terminal` |
| Command palette (⌘K) | Preserved | Same overlay component |
| Onboarding checklist + product tour | Preserved | Same overlay components |
| Department color theming | Preserved | `getDeptColor()` + `--dept` CSS variable |
| Streaming markdown | Preserved | svelte-streamdown in chat, same component |

### 6.3 Manifest-Driven Section Visibility

The `DepartmentDef` type (State Report Ch. 5, `departmentManifest.ts`) already supports per-department tab control:

```typescript
interface UiContribution {
  tabs: string[];              // controls which sections appear
  dashboard_cards?: DashboardCard[];
  has_settings?: boolean;
  custom_components?: string[];
}

function tabsFromDepartment(d: DepartmentDef): string[]
// Returns d.ui?.tabs (manifest-first) or d.tabs (flat API response)
```

The dept layout sidebar uses this exact function to filter sections. Departments declaring `tabs: ['actions', 'agents', 'skills']` only show those 3 sections — same behavior as the current DepartmentPanel tab bar.

### 6.4 Interaction with Priority 1 Gap (Tool Wiring)

State Report Ch. 7 identifies the biggest gap: **10 departments lack agent tool wiring**. This UI restructuring complements that work:

- When harvest gets tools wired (e.g., `scan_sources`, `score_opportunity`), the `/dept/harvest/engine` section page can surface those tools with full-width input/output.
- When gtm gets tools wired (e.g., `send_outreach`, `update_pipeline`), the `/dept/gtm/chat` section gives agents room to show CRM data inline.
- Each department's expanded main pane can host richer engine-specific UIs as capabilities are wired.

The UI restructuring **does not block or depend on** tool wiring. Both can proceed in parallel.

---

## 7. Progressive Delivery

| Phase | Scope | Files | Risk | Can ship alone? |
|-------|-------|-------|------|-----------------|
| 1 | Root layout: top bar + dept bar | 1 rewrite | Medium (changes every page's chrome) | Yes — immediate improvement |
| 2 | Dept nested layout + section sidebar | 1 new | Low (new file only) | No — needs phase 3 |
| 3 | Section page wrappers | 13 new | Low (thin wrappers, existing components) | Yes — with phase 2 |
| 4 | Default redirect + `pendingCommand` fix | 1 modify | Low (small targeted change) | Yes — cleanup |
| 5 | Remove DepartmentPanel import | 1 modify | Low (cleanup) | Yes — cleanup |
| **Total** | | **2 modified + 15 new** | | |

**Estimated new code: ~430 lines.** No existing component modified. No backend changes.

---

## 8. Verification

```bash
cd frontend && pnpm dev              # Dev server on :5173

# Route tests:
# 1. /                     → top bar + dept bar + full-width dashboard
# 2. /chat                 → top bar + dept bar + full-width global chat (no sidebar)
# 3. /dept/forge           → redirects to /dept/forge/actions
# 4. /dept/forge/actions   → section sidebar (Actions active) + ActionsTab in main
# 5. /dept/forge/skills    → sidebar (Skills active) + SkillsTab in main
# 6. /dept/forge/chat      → sidebar (Chat active) + DepartmentChat full-width
# 7. Click Code in dept bar → /dept/code/actions, sidebar updates for Code dept
# 8. Quick action click    → navigates to /dept/forge/chat, message auto-sent
# 9. Browser back          → returns to previous section with correct sidebar state

pnpm check                           # TypeScript + Svelte type checking (0 errors)
pnpm build                           # Production build succeeds
```

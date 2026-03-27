# RUSVEL UI Design Principles

**Date:** 2026-03-27
**Status:** Active — all frontend work must comply
**Derived from:** SOLID, DRY, Composition, SSOT, Abstraction layers
**Applied to:** 5-zone layout (Icon Rail, Zone A, Zone B, Zone C, Bottom Panel)

---

## 1. Single Responsibility — One Zone, One Job

Each zone has exactly one responsibility. If you can't describe it in one sentence, it's doing too much.

| Zone | Responsibility | Must NOT do |
|------|---------------|-------------|
| Icon Rail (48px) | Department/page selection | Render content, config, chat, or execution output |
| Zone A — Config Sidebar | Entity browsing + CRUD forms | Render chat, execution output, or A2UI components |
| Zone B — Canvas / Main | Content display + A2UI rendering | Handle navigation or entity list selection |
| Zone C — Chat Rail | Chat conversation + tool call stream | Render CRUD forms, full entity lists, or settings |
| Bottom Panel | Terminal + execution logs + event stream | Handle navigation, chat, or CRUD |

**Test:** Remove any zone. The others still function. If removing Zone C breaks Zone A, there's a coupling violation.

**Current violation:** `DepartmentPanel.svelte` (260 lines) handles tab routing, section content rendering, resize logic, color theming, and terminal pane management. Five responsibilities in one component.

**Fix:** Split into Zone A (sidebar nav) + Zone B (content rendering) + separate resize logic. Each becomes its own component under 100 lines.

---

## 2. Open/Closed — Extend via Manifest, Not Code Changes

The frontend is **open for extension** (new tabs, new departments, new engine tools appear automatically) but **closed for modification** (no component edits required).

### The Extension Point: `DepartmentManifest`

```
Backend declares:                    Frontend renders automatically:

manifest.tabs: ["agents",            Zone A sidebar:
  "skills", "rules"]                 ○ Agents  ○ Skills  ○ Rules

manifest.tabs: ["agents",            Zone A sidebar:
  "skills", "rules",                 ○ Agents  ○ Skills  ○ Rules
  "workflows", "mcp",               ○ Workflows  ○ MCP  ○ Hooks
  "hooks", "engine"]                 ○ Engine
```

Adding a new section to a department = add a string to `manifest.tabs` in Rust. Zero frontend files touched.

### The Component Registry

```typescript
const sectionRegistry: Record<string, Component> = {
  agents:    AgentsTab,
  skills:    SkillsTab,
  rules:     RulesTab,
  workflows: WorkflowsTab,
  mcp:       McpTab,
  hooks:     HooksTab,
  engine:    EngineTab,
  events:    EventsTab,
  dirs:      DirsTab,
  actions:   ActionsTab,
};

// Zone A renders whatever the manifest declares
{#each manifest.tabs as tabId}
  <SidebarItem active={activeTab === tabId} />
{/each}

// Zone B renders the active section's component — no if/else chain
<svelte:component this={sectionRegistry[activeTab]} {dept} />
```

**Rule:** No `{:else if activeTab === 'x'}` chains. The registry resolves tab ID → component. Adding a future tab (e.g., `"analytics"`) = register one component + add to manifest.

**Current violation:** `DepartmentPanel.svelte` lines 215-249 have an 11-branch `{:else if}` chain for tab content rendering. Each new tab requires editing this chain.

---

## 3. Liskov Substitution — Any Department Fills the Same Zones

Every department renders in the same zone layout. No zone has special-case logic for a specific department. The zones don't care whether Forge, Legal, or a hypothetical 14th department is active.

**Current violation:** `EngineTab.svelte` has hardcoded blocks:

```svelte
{#if dept === 'code'}
  <!-- Code-specific UI -->
{:else if dept === 'content'}
  <!-- Content-specific UI -->
{:else if dept === 'harvest'}
  <!-- Harvest-specific UI -->
{/if}
```

**Fix:** The manifest declares engine tools. The UI renders them generically:

```typescript
// manifest.engine_tools: [{ id: "analyze", label: "Analyze", ... }]
{#each manifest.engine_tools as tool}
  <svelte:component this={engineRegistry[tool.component]} {dept} />
{/each}
```

No department identity checks in rendering logic. If a department declares engine tools, they render. If it declares none, the engine section is empty or hidden.

**Test:** Create a hypothetical 14th department with a unique manifest. Does it render correctly without any frontend code change? If yes, LSP is satisfied.

---

## 4. Interface Segregation — Zones Depend on Small Interfaces

Each zone receives only the props it needs. No god-prop objects.

```typescript
// Zone A — minimal interface
interface ZoneAProps {
  dept: string;
  tabs: string[];
  activeTab: string;
}

// Zone B — minimal interface
interface ZoneBProps {
  dept: string;
  activeTab: string;
}

// Zone C — minimal interface
interface ZoneCProps {
  dept: string;
  title: string;
}

// Bottom Panel — minimal interface
interface BottomProps {
  dept: string;
  sessionId: string;
}
```

**Current violation:** `DepartmentPanel` receives 8 props: `dept`, `title`, `color`, `quickActions`, `tabs`, `sessionId`, `helpDescription`, `helpPrompts`. Most children only need `dept` and one or two others.

**Rule:** No zone receives the full `DepartmentDef` object. Extract what you need. If a zone needs more than 4 props, it's probably doing too much (see Principle 1).

---

## 5. Dependency Inversion — UI Depends on Manifest (Abstraction), Not Department IDs (Concretions)

The UI depends on the shape of `DepartmentManifest`, never on specific department identities.

```
WRONG:  Component → checks dept === 'code' → renders CodeAnalyzePanel
RIGHT:  Component → reads manifest.engine_tools → renders whatever is declared
```

**Concretely:** Search the frontend codebase for `dept === '`. Every match is a DI violation. Replace with manifest-driven rendering.

**This mirrors the backend architecture:** Engines depend on `rusvel-core` port traits (abstractions), never on adapter crates (concretions). The frontend depends on `DepartmentManifest` (abstraction), never on specific department implementations.

### The Manifest is the Abstraction Layer

```
Rust DepartmentManifest (backend)
        │
        ▼
GET /api/departments → DepartmentDef[] (JSON over HTTP)
        │
        ▼
departments store (Svelte writable — the contract)
        │
        ├──→ Icon Rail: reads dept.icon, dept.color
        ├──→ Zone A: reads tabsFromDepartment(dept)
        ├──→ Zone B: reads manifest.engine_tools, manifest.quick_actions
        ├──→ Zone C: reads dept.system_prompt, dept.capabilities
        └──→ Command palette: searches dept.name, dept.id
```

---

## 6. DRY — One Source, One Render, One Truth

### Data sources — each has exactly one authoritative location

| Data | Single Source | Consumers |
|------|-------------|-----------|
| Department list | `departments` store (from API) | Icon rail, command palette, dashboard |
| Active department | `page.params.id` (URL) | All zones — derived, never duplicated |
| Section list per dept | `tabsFromDepartment(manifest)` | Zone A sidebar, command palette |
| Active section | `activeSection` store | Zone A highlight, Zone B content |
| Department color | CSS variable `--dept-hsl` set once by dept layout | All child components via CSS |
| Chat state | One `DepartmentChat` instance in Zone C | Never duplicated |
| Session | `activeSession` store | All zones |
| Approval count | `pendingApprovalCount` store | Icon rail badge |

### The CSS variable pattern eliminates color prop drilling

```svelte
<!-- Dept layout sets it once -->
<div style="--dept-hsl: {getDeptColor(dept.color)}">
  {@render children()}
</div>

<!-- Any descendant uses it — no prop needed -->
<style>
  .dept-accent { color: hsl(var(--dept-hsl)) }
  .dept-bg { background: hsl(var(--dept-hsl) / 0.1) }
  .dept-border { border-color: hsl(var(--dept-hsl) / 0.3) }
</style>
```

No more passing `deptHsl` as a prop to every tab component. One source, consumed via CSS inheritance.

### Current violations

1. **Department list rendered twice** — once in expanded sidebar nav, once in collapsed icon sidebar. Two separate `{#each}` blocks with duplicated icon/active logic.
2. **Tab definitions in two places** — `DepartmentPanel.svelte` line 123 (hardcoded array) AND `departmentManifest.ts` (`tabsFromDepartment()`).
3. **Icon mapping duplicated** — `+layout.svelte` has `iconMap` (lines 32-52) that shadows the manifest's `icon` field and the `DeptIcon` component.

---

## 7. Composition Over Inheritance — Zones Are Siblings, Not Nested

The 5 zones are **composed horizontally** by the department layout. No zone imports or contains another zone.

```
DeptLayout
  ├── IconRail       (reads: departments store, page URL)
  ├── ZoneA          (reads: manifest.tabs, activeSection store)
  ├── ZoneB          (reads: activeSection, sectionRegistry)
  ├── ZoneC          (reads: dept, session)
  └── BottomPanel    (reads: dept, session)
```

### Communication between zones — stores and events, not props

| Mechanism | Flow | Example |
|-----------|------|---------|
| **Stores** | Zone A → Zone B | `activeSection` store: Zone A sets it on click, Zone B reads it to render content |
| **Events** | Zone A → Zone C | `pendingCommand` store: Quick action in Zone A sets command, Zone C chat picks it up |
| **CSS variables** | Layout → all zones | `--dept-hsl` set by layout, read by all zones |
| **URL** | Browser → all zones | `page.params.id` provides department ID to every zone |

### Rules

- **No zone imports another zone.** They're siblings composed by the layout.
- **No zone passes props to another zone.** They communicate via stores.
- **No zone subscribes to another zone's internal state.** Only shared stores.

This mirrors the backend: engines don't import each other. They communicate via `EventPort` (events) and `StoragePort` (shared state). Zones communicate via stores (shared state) and custom events.

---

## 8. Single Source of Truth — The Manifest Contract

The `DepartmentManifest` is the SSOT for everything about a department. The frontend must **consume** it, never **shadow** it.

### What the manifest declares (backend)

```rust
DepartmentManifest {
    id: String,              // "forge"
    name: String,            // "Forge Department"
    description: String,
    icon: String,            // "hammer"
    color: String,           // "emerald"
    system_prompt: String,
    capabilities: Vec<String>,
    tabs: Vec<String>,       // ["actions", "engine", "agents", ...]
    quick_actions: Vec<QuickAction>,
    ui: UiContribution {
        tabs: Vec<String>,
        dashboard_cards: Vec<DashboardCard>,
        has_settings: bool,
        custom_components: Vec<String>,
    },
    events_produced: Vec<String>,
    events_consumed: Vec<String>,
    config_schema: Value,
    default_config: Value,
}
```

### What the frontend must NOT define independently

- Which tabs a department has (comes from `manifest.tabs`)
- What color a department uses (comes from `manifest.color`)
- What icon a department uses (comes from `manifest.icon`)
- What quick actions are available (comes from `manifest.quick_actions`)
- What capabilities a department declares (comes from `manifest.capabilities`)
- What dashboard cards to show (comes from `manifest.ui.dashboard_cards`)

### Current violation

`+layout.svelte` defines its own `iconMap` (lines 32-52) mapping department IDs to Lucide icon components. This shadows the manifest's `icon` field.

**Fix:** Read `dept.icon` from the manifest. The `DeptIcon` component already resolves icon strings to Lucide components — use it everywhere instead of maintaining a parallel mapping.

---

## 9. Abstraction Layers — Clean Boundaries

```
Layer 1: STORES (reactive data)
  departments, activeSession, activeSection, pendingCommand
  Rules: No side effects. No API calls. Pure reactive state.
  ↓ consumed by

Layer 2: ZONES (layout composition)
  IconRail, ZoneA, ZoneB, ZoneC, BottomPanel
  Rules: Read from stores. Compose section components. No direct API calls.
  ↓ render

Layer 3: SECTIONS (feature components)
  AgentsTab, SkillsTab, RulesTab, DepartmentChat, EngineTab, ...
  Rules: Call API layer. Manage local UI state. No knowledge of which zone renders them.
  ↓ call

Layer 4: API (communication)
  getAgents(), streamDeptChat(), getDeptConfig(), createSkill(), ...
  Rules: Pure functions. Return promises or invoke callbacks. No UI state.
  ↓ talk to

Layer 5: BACKEND (truth)
  rusvel-api → engines → ports → database
  Rules: Owned by Rust. Frontend never bypasses API layer.
```

### Dependency rules

- Layers depend **downward only**: zones use stores, sections use API. Never reversed.
- **Zones never import sections directly** — they use the component registry.
- **Sections never import zones** — they don't know where they're rendered.
- **API functions are the only way to talk to the backend** — no raw `fetch()` in components.
- **Stores never call API** — components call API and update stores. (Exception: `refreshPendingApprovalCount()` which is a convenience wrapper.)

### Test for violations

```bash
# Sections should never import zone components
grep -r "import.*IconRail\|import.*ZoneA\|import.*ZoneB\|import.*ZoneC\|import.*BottomPanel" src/lib/components/department/

# Zones should never import other zones
grep -r "import.*ZoneA\|import.*ZoneB\|import.*ZoneC" src/lib/components/layout/

# Components should never use raw fetch (use api.ts instead)
grep -rn "fetch(" src/lib/components/ --include="*.svelte" | grep -v "api.ts"
```

---

## 10. Progressive Disclosure — Manifest Gates the UI Surface

Not a classic software principle, but essential for a 13-department system where 10 are sparse.

**Rule:** The UI renders exactly what the manifest declares. Nothing more.

```
Sparse department (Legal — 0 wired tools):
  manifest.tabs: ["actions", "agents", "skills", "rules"]
  → Zone A shows 4 items
  → Zone B engine section is absent (not declared)
  → Result: clean, honest interface

Rich department (Forge — 5 wired tools):
  manifest.tabs: ["actions", "engine", "agents", "skills", "rules",
                   "workflows", "mcp", "hooks", "terminal", "events"]
  → Zone A shows 10 items
  → Zone B engine section shows tool-specific UI
  → Result: full capability surface
```

As departments get tools wired (Priority 1 gap), their manifests grow, and the UI automatically shows more sections. No frontend code changes. No false affordance for sparse departments.

---

## Compliance Checklist

Before merging any frontend PR, verify:

- [ ] **SRP:** Each new component has a one-sentence responsibility description
- [ ] **O/C:** New sections use the component registry, not `{:else if}` chains
- [ ] **LSP:** No `dept === 'x'` checks in rendering logic
- [ ] **ISP:** Components receive ≤ 4 props; no full `DepartmentDef` as prop
- [ ] **DIP:** UI reads from manifest/stores, never checks concrete department IDs
- [ ] **DRY:** No data defined in two places; color via CSS variable only
- [ ] **Composition:** Zones don't import each other; communication via stores
- [ ] **SSOT:** No hardcoded icons, colors, tabs, or quick actions — all from manifest
- [ ] **Layers:** Sections don't import zones; zones don't call API directly
- [ ] **Disclosure:** Only manifest-declared tabs rendered; sparse depts show fewer items

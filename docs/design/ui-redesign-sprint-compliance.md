# UI Redesign ↔ OpenClaw Sprint Plan: Compliance Analysis

**Date:** 2026-03-27

---

## The Gap

The sprint plan (E10, Sprint 3, stories S-018 through S-025) was written against the **earlier** `two-level-nav-proposal.md`. The UI redesign evolved after 3 rounds of research into a **5-zone layout**. Here's the drift:

| Aspect | Sprint Plan (S-018–S-025) | UI Redesign Final |
|--------|--------------------------|-------------------|
| Department nav | Horizontal dept bar (S-019) | Icon rail (48px vertical) |
| Static nav | Horizontal top bar (S-018) | Icon rail top section |
| Section nav | Section sidebar (S-020) | Section sidebar (same) |
| Right panel | Not mentioned | Context panel (320px, collapsible) |
| Bottom panel | Not mentioned | Terminal + exec logs (collapsible) |
| Layout structure | Top bar → dept bar → sidebar + main | Rail → sidebar + main + context panel + bottom panel |

**The section sidebar, section pages, and DepartmentPanel retirement are identical.** The difference is in how departments and global pages are accessed (horizontal bars vs icon rail) and the addition of context + bottom panels.

---

## Story-by-Story Compliance

### Fully compatible (no changes needed)

| Story | What | Status |
|-------|------|--------|
| **S-020** | Section sidebar within each department | **Identical.** Both plans use `dept/[id]/+layout.svelte` with manifest-driven sections. |
| **S-021** | Chat section page wrapper | **Identical.** `/dept/[id]/chat/+page.svelte` wraps DepartmentChat. |
| **S-022** | Config and Events section pages | **Identical.** Same pages, same API endpoints. |
| **S-023** | Engine-specific section pages (analyze, pipeline, drafts) | **Identical.** Same section pages. UI redesign adds more (agents, skills, rules, etc. as separate pages). |
| **S-024** | Retire DepartmentPanel.svelte | **Identical.** Both plans retire it. |
| **S-025** | Keyboard shortcuts for department switching | **Compatible.** Alt+1-9 works on icon rail the same as on dept bar. |

### Needs updating (different approach, same goal)

| Story | Sprint Plan Says | UI Redesign Says | Resolution |
|-------|-----------------|-------------------|------------|
| **S-018** | Horizontal top bar with static items (Chat, Dashboard, DB, Flows, Terminal, Settings) | Icon rail top section with same items as vertical icons | **Replace:** Create `IconRail.svelte` instead of `TopBar.svelte`. Same items, vertical layout. Acceptance criteria unchanged except "horizontal" → "vertical icon rail". |
| **S-019** | Horizontal dept bar below top bar, scrollable, with DeptIcon + labels | Icon rail bottom section with dept icons (no labels, icons only) | **Replace:** Department icons in rail body instead of `DeptBar.svelte`. Acceptance criteria: remove "horizontal scrollable", add "vertical icon rail with colored active indicator". |

### New stories needed (not in sprint plan)

| New Story | What | Why |
|-----------|------|-----|
| **S-018b** | Context panel (right, 320px, collapsible) | Quick chat + properties + exec output. Not in sprint plan because original proposal didn't have it. |
| **S-018c** | Bottom panel (collapsible terminal + exec logs + events) | Terminal and execution accessible from any context. Currently terminal is a separate page or dept tab. |
| **S-023b** | Section pages for ALL tab components (agents, skills, rules, workflows, mcp, hooks, dirs) | Sprint plan only creates pages for Chat, Config, Events, and 5 engine-specific pages. The UI redesign creates pages for all 13 sections. |

---

## Impact on Other Sprints

### Sprint 4 (Revenue Ready) — S-029, S-030, S-031

The sprint plan creates:
- `/dept/[id]/pipeline/+page.svelte` (Harvest Kanban)
- `/dept/[id]/calendar/+page.svelte` (Content calendar)
- Enhanced `/approvals` page

**Compliance:** Fully compatible. These pages render in the main content area regardless of whether navigation is a top bar or icon rail. The context panel gives approval cards a natural home (approve/reject in right panel while viewing queue in main).

### Sprint 5 (Outreach Machine) — S-036, S-037, S-038

The sprint plan creates:
- `/dept/[id]/contacts/+page.svelte` (CRM)
- `/dept/[id]/deals/+page.svelte` (Deal pipeline)
- `/dept/[id]/invoices/+page.svelte` (Invoices)

**Compliance:** Fully compatible. These are additional section pages in the dept layout. In the icon rail model, they appear as sidebar items in the GTM department. The sprint plan puts them under generic `/dept/[id]/` — same as the UI redesign.

### Sprint 7 (Intelligence Layer) — S-047, S-051

The sprint plan creates:
- Enhanced dashboard (`/`) with cross-department KPIs
- AI spend dashboard (`/settings/spend/+page.svelte`)

**Compliance:** Fully compatible. Dashboard is a global page (icon rail top section → Home). Spend page is under settings. Both render full-width in the main content area.

### Sprint 8 (Open Channels) — S-057

The sprint plan creates:
- `/settings/channels/+page.svelte` (Channel management)

**Compliance:** Fully compatible. This is a global settings page, accessible via the Settings icon in the rail.

### OpenClaw Integration Proposal — Nav Compliance Section

The integration proposal states:
> **All new frontend work in this proposal MUST follow this layout:**
> - New pages (Inbox, Channels, Voice, Canvas) go in the TOP BAR as global pages
> - New department sections go in the SECTION SIDEBAR
> - No new features in the retired DepartmentPanel pattern
> - All new pages get bookmarkable URLs

**Compliance with icon rail:**
- "TOP BAR as global pages" → **Icon rail top section.** Same intent: global pages accessible without department context. Inbox, Channels, Voice, Canvas become new icons in the rail.
- "SECTION SIDEBAR" → **Identical.** Both plans use the same section sidebar.
- "No DepartmentPanel" → **Identical.**
- "Bookmarkable URLs" → **Identical.**

The only textual change needed in the integration proposal: replace "TOP BAR" with "ICON RAIL" in the nav compliance section.

---

## Updated Sprint 3 Stories

Here are the corrected S-018 and S-019, plus 3 new stories:

### S-018 (revised): Icon Rail replacing sidebar

```
[S-018] As a user, I want a persistent icon rail on the left edge showing
global pages (top) and departments (bottom), so that I always have
one-click access to any page or department.

Acceptance Criteria:
- [ ] 48px vertical icon rail replaces current 256px sidebar
- [ ] Top section: Home, Chat, Approvals, Database, Flows, Terminal icons
- [ ] Divider line
- [ ] Bottom section: 13 department icons from getDepartments() API
- [ ] Divider line
- [ ] Settings icon at bottom
- [ ] Active icon highlighted with accent color (department color for dept icons)
- [ ] Approval badge count on Approvals icon
- [ ] Tooltip on hover showing label
- [ ] Responsive: hidden behind hamburger on mobile

Size: M
Files: frontend/src/routes/+layout.svelte, frontend/src/lib/components/IconRail.svelte (new)
Dependencies: none
```

### S-019 (revised): Removed — merged into S-018

The horizontal dept bar is eliminated. Department icons live in the icon rail (S-018). Story S-019 is removed.

### S-018b (new): Context Panel

```
[S-018b] As a user, I want a collapsible right panel for quick chat,
item properties, and execution output, so that I can see contextual
information alongside the main content.

Acceptance Criteria:
- [ ] 320px collapsible right panel in dept/[id]/+layout.svelte
- [ ] Toggle via button or Cmd+J keyboard shortcut
- [ ] Default content: compact chat input for quick questions to dept agent
- [ ] When item selected in main area: shows item properties
- [ ] When chat is active: shows tool call details + approval cards
- [ ] Panel state persisted in contextPanelOpen store
- [ ] Panel hidden on global pages (non-dept routes)

Size: M
Files: frontend/src/routes/dept/[id]/+layout.svelte,
       frontend/src/lib/components/ContextPanel.svelte (new),
       frontend/src/lib/stores.ts
Dependencies: S-020
```

### S-018c (new): Bottom Panel

```
[S-018c] As a user, I want a collapsible bottom panel for terminal,
execution logs, and event stream, so that I can monitor activity
from any department section.

Acceptance Criteria:
- [ ] Collapsible bottom panel in dept/[id]/+layout.svelte
- [ ] Toggle via Cmd+` keyboard shortcut
- [ ] Tabs: Terminal, Executions, Events
- [ ] Terminal tab reuses DeptTerminal with pane from /api/terminal/dept/{id}
- [ ] Executions tab shows running/recent jobs from JobPort
- [ ] Events tab shows live event stream
- [ ] Panel state persisted in bottomPanelOpen store

Size: L
Files: frontend/src/routes/dept/[id]/+layout.svelte,
       frontend/src/lib/components/BottomPanel.svelte (new),
       frontend/src/lib/stores.ts
Dependencies: S-020
```

### S-023b (new): Full section page set

```
[S-023b] As a user, I want section pages for all department tab
components (agents, skills, rules, workflows, mcp, hooks, dirs),
so that each section gets a bookmarkable URL and full-width main area.

Acceptance Criteria:
- [ ] dept/[id]/agents/+page.svelte wraps AgentsTab
- [ ] dept/[id]/skills/+page.svelte wraps SkillsTab
- [ ] dept/[id]/rules/+page.svelte wraps RulesTab
- [ ] dept/[id]/workflows/+page.svelte wraps WorkflowsTab
- [ ] dept/[id]/mcp/+page.svelte wraps McpTab
- [ ] dept/[id]/hooks/+page.svelte wraps HooksTab
- [ ] dept/[id]/dirs/+page.svelte wraps DirsTab
- [ ] dept/[id]/actions/+page.svelte wraps ActionsTab
- [ ] dept/[id]/engine/+page.svelte wraps EngineTab
- [ ] All pages derive dept from page.params.id + departments store
- [ ] All pages compute deptHsl from getDeptColor()

Size: M
Files: 9 new +page.svelte files under frontend/src/routes/dept/[id]/
Dependencies: S-020
```

---

## Summary

| Category | Count | Detail |
|----------|-------|--------|
| Stories fully compatible | 6 | S-020, S-021, S-022, S-023, S-024, S-025 |
| Stories needing revision | 2 | S-018 (top bar → icon rail), S-019 (removed, merged into S-018) |
| New stories needed | 3 | S-018b (context panel), S-018c (bottom panel), S-023b (full section pages) |
| Downstream sprints affected | 0 | All later sprints (4-10) are fully compatible |
| Integration proposal text change | 1 line | "TOP BAR" → "ICON RAIL" in nav compliance section |

**Net effect on Sprint 3:** +1 story (was 8, now 9 after removing S-019 and adding 3). Complexity increases from the context + bottom panels, but S-018b and S-018c can be deferred to Sprint 3b if needed — the icon rail + section sidebar + section pages are the critical path.

**No changes needed to Sprints 1-2 (backend tool wiring) or Sprints 4-10 (all downstream work).**

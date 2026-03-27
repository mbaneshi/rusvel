# OpenClaw Sprint Plan — Assessment & Required Updates

**Date:** 2026-03-27
**Assesses:** `docs/plans/openclaw-sprint-plan.md` (72 stories, 10 sprints, 20 weeks)
**In light of:** UI redesign final (5-zone layout), UI design principles (10 constraints), entity hierarchy proposal (Playbook), external research (3 AI answers)

---

## Overall Verdict: 80% Ready to Execute

Sprints 1-2 (tool wiring) can start immediately. Sprint 3 needs a rewrite. A Playbook sprint needs insertion. Chat placement decision must be locked.

---

## 1. What's Strong

### Phasing is correct

```
P0: Wire 10 departments (Sprint 1-2)     ← foundation, no dependencies
P1: Frontend restructure (Sprint 3)       ← builds on wired tools
P2: Revenue hardening (Sprint 4-5)        ← builds on UI + tools
P3: Cross-engine intelligence (Sprint 6-7) ← builds on revenue
P4: Channels (Sprint 8)                    ← builds on webhooks
P5: Ecosystem (Sprint 9-10)               ← builds on everything
```

Each phase depends on the previous. No phase can be reordered. This is correct.

### Story quality

- 72 stories with acceptance criteria, size, file targets, dependencies
- Parallel agent assignments with merge order and conflict zones per sprint
- Sprint demo scenarios give concrete verification
- Risk register covers 15 risks across all phases

### Definition of Done

Updated to include the 10 UI design principles as item #10. Every frontend PR must pass:
- No `{:else if activeTab}` chains
- No `dept === 'x'` checks
- Components ≤ 4 props
- Zones don't import each other
- Colors via CSS variable
- Manifest-driven tabs
- Layers depend downward only

### Sprint 1-2 is the strongest section

The tool wiring stories (S-003 through S-015) are mechanical:
1. Follow the `dept-forge` pattern
2. Add `ctx.tools.add()` calls in `register()`
3. Each dept crate goes from ~86 to ~200-250 lines
4. Parallelizable across 4-5 agents per sprint
5. Low risk, high leverage

---

## 2. What's Stale or Misaligned

### 2.1 Sprint 3 stories describe the old layout

The epic table correctly says E10 is "5-Zone Layout" but the stories still describe the earlier two-level-nav proposal:

| Story | Says | Should Say |
|-------|------|-----------|
| **S-018** | Horizontal top bar + `TopBar.svelte` | Icon rail (48px) + `IconRail.svelte` |
| **S-019** | Horizontal dept bar + `DeptBar.svelte` | **Remove** — depts live in the icon rail body |
| **S-020** | Section sidebar only | Section sidebar + context panel (right) + bottom panel |
| **S-021** | Chat section page wrapper | **Zone C persistent chat rail** (not a page) |
| **S-022** | Config + Events pages only | All 13 section pages needed |
| **S-023** | 5 engine-specific pages | Already covered by full section page set |

### 2.2 Chat placement contradicts external research

The sprint plan treats chat as a section page (`/dept/[id]/chat/+page.svelte`). Three independent research answers (ChatGPT, Claude external, Perplexity synthesis) converge on chat as a **persistent right rail**.

**Why it matters:** The edit-test loop (`edit rule → chat test → see result → refine rule`) requires Zone A (config sidebar) and Zone C (chat) to be co-visible. Navigating to a chat page loses the config view.

**Decision needed before Sprint 3:** Persistent chat rail (Zone C) vs chat-as-page. This changes S-020, S-021, and the entire dept layout component.

### 2.3 Playbook entity declared but has zero stories

E16 "Playbook Entity" appears in the epic table at P2 but the sprint plan contains no stories for it. The entity hierarchy proposal defines:
- `Playbook` struct with FK arrays (agent_ids, skill_ids, rule_ids, workflow_ids, hook_ids, mcp_server_ids)
- Runtime resolution chain (step → agent → playbook → department → global)
- 7 API routes (CRUD + activate + active query)
- `PlaybookSelector.svelte` in Zone A
- Chat handler modification (6 of 9 steps gain playbook awareness)
- Schema changes on Agent, Skill, Rule, Workflow, Hook (new FK fields)
- Workflow migration: `agent_name` (string) → `agent_id` (UUID)

This is 7+ stories worth of work with no sprint home.

### 2.4 A2UI integration absent

No story addresses where agent-generated UI components render. The research consensus: Zone B (canvas/main area) is the render target. Chat shows a reference link. This needs:
- A2UI component registry (map type string → Svelte component)
- Zone B renderer that accepts STATE_DELTA events
- Chat integration (reference link instead of inline rendering)

### 2.5 Component registry pattern missing

The design principles require "no `{:else if activeTab}` chains — use component registry." But no Sprint 3 story creates the `sectionRegistry` that maps tab IDs to Svelte components. Without this, Sprint 3 will rebuild the antipattern it's supposed to eliminate.

### 2.6 Cross-engine pipeline (S-042) is XL and underspecified

S-042 "cross-engine workflow orchestration" is the only XL story. It doesn't specify:
- Whether it uses FlowEngine's existing DAG executor or creates a new pipeline system
- How it relates to the Playbook entity (a cross-dept pipeline IS a playbook workflow)
- How it handles department-scoped tool access when spanning departments

**Recommendation:** S-042 should use FlowEngine internally (the DAG executor already supports code/condition/agent nodes). A cross-engine pipeline is a Flow with steps that target different department agents. No new pipeline system needed.

---

## 3. What's Missing

### 3.1 Playbook sprint (7 stories)

Proposed insertion between Sprint 4 and Sprint 5:

```
Sprint 4b: "Playbook Foundation" (1 week)

S-073: Playbook struct in rusvel-core + ObjectStore CRUD
  - Playbook { id, department, name, description, agent_ids, skill_ids,
    rule_ids, workflow_ids, hook_ids, mcp_server_ids, config overrides }
  - Stored as ObjectStore kind="playbooks"
  - Size: M

S-074: Playbook API routes
  - GET/POST/PUT/DELETE /api/playbooks
  - POST /api/playbooks/{id}/activate?session_id=X
  - GET /api/playbooks/active?dept=X&session_id=X
  - Size: M

S-075: Agent FK fields (default_skill_ids, default_rule_ids, mcp_server_ids)
  - Add optional Vec fields to AgentProfile
  - Backward compatible (serde defaults to empty vec)
  - Size: S

S-076: Rule scope enum (Global | Department | Playbook | Agent)
  - Add scope + scope_id + priority fields to RuleDefinition
  - Chat handler loads rules filtered by scope chain
  - Size: M

S-077: Workflow step agent_id migration
  - WorkflowStepDef.agent_name (string) → agent_id (UUID)
  - Add skill_id and rule_ids to WorkflowStepDef
  - Migration: resolve existing agent_name strings to IDs
  - Size: M

S-078: PlaybookSelector component in Zone A sidebar
  - Radio-select list of playbooks for current department
  - Activating filters Zone A entity counts
  - Stores active playbook in session-scoped state
  - Size: M

S-079: Playbook-aware chat handler
  - Steps 2,3,4,5,7,9 of 9-step pipeline gain playbook scope
  - Tools filtered to playbook's agent_ids + mcp_server_ids
  - Rules filtered to playbook's rule_ids + agent defaults
  - Skills filtered to playbook's skill_ids
  - No playbook active = current behavior (department-wide)
  - Size: L

S-080: Reverse reference API + UI ("Used in")
  - GET /api/agents/{id}/used-in → playbooks + workflows referencing it
  - GET /api/skills/{id}/used-in → playbooks + agents referencing it
  - Entity detail pages show "Used in" section
  - Size: M
```

### 3.2 Revised Sprint 3 stories

Replace S-018 through S-025 with:

```
Sprint 3: "New Shell" (revised)

S-018r: Icon Rail component
  - 48px vertical rail replacing 256px sidebar
  - Top section: Home, Chat, Approvals, DB, Flows, Terminal icons
  - Bottom section: 13 department icons from getDepartments()
  - Active icon highlighted with department accent color
  - Approval badge count on Approvals icon
  - Tooltip on hover
  - Size: M
  Files: +layout.svelte (rewrite), IconRail.svelte (new)

S-019r: Section component registry
  - sectionRegistry: Record<string, Component> mapping tab ID → Svelte component
  - Zone B renders: <svelte:component this={sectionRegistry[activeTab]} {dept} />
  - No {:else if} chains anywhere
  - All 11 existing tab components registered
  - Size: S
  Files: src/lib/components/department/sectionRegistry.ts (new)

S-020r: Department layout with 5 zones
  - dept/[id]/+layout.svelte with:
    - Zone A: section sidebar (200px, collapsible to icons)
    - Zone B: main content area (flexible, renders active section via registry)
    - Zone C: persistent chat rail (320px, collapsible)
    - Bottom panel: terminal + execution + events (collapsible)
  - Sections derived from tabsFromDepartment(manifest)
  - Department color via CSS --dept-hsl variable (set once, consumed by all zones)
  - Size: L
  Files: dept/[id]/+layout.svelte (new), ZoneA.svelte, ZoneC.svelte, BottomPanel.svelte

S-021r: Zone C — Persistent chat rail
  - DepartmentChat component rendered in Zone C (not a page)
  - Always visible alongside Zone A and Zone B
  - Collapsible via button or Cmd+J
  - Width resizable (250-500px)
  - Tool calls shown as expandable cards in rail
  - Approval cards in rail with approve/reject buttons
  - Size: L
  Files: ZoneC.svelte, adapted DepartmentChat usage

S-022r: Store architecture for zone communication
  - activeSection store (Zone A sets, Zone B reads)
  - contextPanelOpen / bottomPanelOpen stores
  - activePlaybook store (future: PlaybookSelector sets)
  - a2uiContent store (future: chat pushes, Zone B renders)
  - pendingCommand reactive redirect (Zone A → Zone C)
  - Size: S
  Files: src/lib/stores.ts

S-023r: Section page content for all tab components
  - Zone B renders the active section's component from the registry
  - No SvelteKit route per section — sections are Zone A state, not URL routes
  - Exception: deep links like /dept/forge?section=skills can set initial activeSection
  - All 11 existing components (ActionsTab through EventsTab) work without modification
  - Size: M

S-024r: Retire DepartmentPanel.svelte
  - Remove DepartmentPanel from dept/[id]/+page.svelte
  - Remove all imports
  - Verify no broken references
  - Update visual regression baselines
  - Size: S

S-025r: Keyboard shortcuts
  - Alt+1 through Alt+9: switch department (icon rail)
  - Cmd+J: toggle Zone C (chat rail)
  - Cmd+`: toggle bottom panel
  - Cmd+B: toggle Zone A (section sidebar)
  - CommandPalette updated with department + section commands
  - Size: S
```

### 3.3 Pre-Sprint 1 audit (1 day)

Before Sprint 1 starts, verify that each engine method targeted by tool wiring actually exists and returns meaningful data:

```
For each tool in S-003 through S-015:
  1. Open the engine crate (e.g., gtm-engine/src/lib.rs)
  2. Find the method (e.g., GtmEngine::crm().add_contact())
  3. Check: does it persist data? Or return todo!()/mock?
  4. If todo!(): size the story up (add method implementation)
  5. If mock: document and decide (is mock OK for Sprint 1?)
```

Risk R1 says "High likelihood" that stub methods exist. Better to discover this before starting than mid-sprint.

### 3.4 A2UI foundation story

Add to Sprint 7 (Intelligence Layer) or Sprint 3:

```
S-081: A2UI renderer in Zone B
  - a2uiRegistry: Record<string, Component> mapping A2UI type → Svelte component
  - 5 initial types: DataTable, MetricsGrid, DraftCard, StatusBadge, CodeBlock
  - Zone B subscribes to a2uiContent store
  - Chat (Zone C) pushes A2UI blocks via store when STATE_DELTA received
  - Chat shows reference: "Results rendered in canvas →" with anchor link
  - Zone B renders component from registry
  - Multiple A2UI outputs stack vertically with provenance headers
  - Size: L
```

---

## 4. Risk Updates

### New risks from design evolution

| # | Risk | Likelihood | Impact | Mitigation |
|---|------|-----------|--------|------------|
| R16 | Zone C persistent chat breaks SSE on dept switch | Medium | High | Key DepartmentChat by dept.id to remount. Or keep single instance and switch dept prop. |
| R17 | 5-zone layout too complex for small screens | Medium | Medium | Zones A and C collapsible. Mobile: only Zone B visible, others in drawers. |
| R18 | Playbook FK arrays create orphan references on entity delete | High | Medium | Soft delete or cascade: on agent delete, remove from all playbook.agent_ids. Add cleanup job. |
| R19 | Component registry loses TypeScript type safety | Low | Low | Use `ComponentType` generics. Registry tests verify all keys resolve to real components. |
| R20 | Section-as-state (not URL) breaks browser back button | Medium | Medium | Use URL search param `?section=skills` to sync. pushState on section change. |

### Upgraded risks

| # | Was | Now | Why |
|---|-----|-----|-----|
| R3 | "Frontend restructure breaks existing routes" — Medium/High | **High/High** | 5-zone layout is more complex than the 2-level-nav it replaces. More components, more stores, more zone interactions. |
| R4 | "PaneForge interaction issues" — Low/Medium | **Removed** | PaneForge is no longer used in the icon rail layout. Zone widths are CSS flex/grid. |

---

## 5. Revised Sprint Timeline

```
Week 0:    Pre-sprint audit (1 day) — verify engine methods exist
Week 1-2:  Sprint 1 "Lock & Load" — auth + GTM/Finance/Product wiring
Week 3-4:  Sprint 2 "Full Arsenal" — remaining 7 dept wiring + integration tests
Week 5-7:  Sprint 3 "New Shell" (3 weeks, was 2) — 5-zone layout, icon rail, zones, registry
Week 8:    Sprint 3.5 "Playbook Foundation" — entity hierarchy, FK changes, PlaybookSelector
Week 9-10: Sprint 4 "Revenue Ready" — Harvest CDP, Content calendar, approval UX
Week 11-12: Sprint 5 "Outreach Machine" — GTM sequences, email, spend tracking
Week 13-14: Sprint 6 "Connective Tissue" — webhooks, cron, cross-engine pipelines
Week 15-16: Sprint 7 "Intelligence Layer" — dashboard KPIs, A2UI foundation, TUI tabs
Week 17-18: Sprint 8 "Open Channels" — channel trait, Telegram, Discord
Week 19-20: Sprint 9 "Plugin Foundation" — SDK, voice, LLM adapters
Week 21-22: Sprint 10 "Polish & Ship" — A2A, CI, benchmarks, docs, wizard

Total: 22 weeks (was 20) — added 1 week for Sprint 3 expansion + 1 week for Sprint 3.5
```

---

## 6. Action Items

| Priority | Action | When |
|----------|--------|------|
| **Now** | Start Sprint 1 (tool wiring). Backend work has zero dependency on frontend decisions. | Immediately |
| **This week** | Run pre-sprint audit: verify engine methods for S-003 through S-015 exist. | Before Sprint 1 starts |
| **This week** | Lock chat placement decision: persistent rail (Zone C) vs section page. | Before Sprint 3 planning |
| **Sprint 2** | Rewrite Sprint 3 stories per section 3.2 above. | During Sprint 2 |
| **Sprint 2** | Write Playbook stories (S-073 through S-080) for Sprint 3.5. | During Sprint 2 |
| **Sprint 3** | Build component registry FIRST (S-019r) — everything else depends on it. | Sprint 3, day 1 |

---

## 7. Document Cross-References

| Document | Status | Relationship |
|----------|--------|-------------|
| `docs/plans/openclaw-sprint-plan.md` | Needs updates per this assessment | The execution plan |
| `docs/plans/openclaw-integration-proposal.md` | Needs "ICON RAIL" text replacement | The strategy document |
| `docs/design/ui-redesign-final.md` | Current | 5-zone layout spec |
| `docs/design/ui-design-principles.md` | Current | 10 constraints for all frontend PRs |
| `docs/design/ui-redesign-sprint-compliance.md` | Partially stale | Needs update for Zone C chat rail |
| `docs/design/entity-hierarchy-proposal.md` | Current | Playbook entity spec (no sprint home yet) |
| `docs/design/answer-claude-ui-architecture.md` | Current | External research: architecture-aligned layout |
| `docs/design/answer-gemini-ui-design.md` | Current | External research: IA and user flows |
| `docs/design/answer-perplexity-ui-research.md` | Current | External research: competitive analysis |
| `docs/state-report/` | Current | 7-chapter codebase truth |

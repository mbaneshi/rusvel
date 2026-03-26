> **COMPLETED & ARCHIVED** — Done 2026-03-24.

# UI Enhancements Plan — 2026-03-23

> Beyond the foundation (shadcn-svelte, Streamdown, toasts, command palette, Svelte Flow, charts).
> Prioritized by impact and effort. All Svelte 5 + Tailwind 4 compatible.

---

## Tier 1: Quick Wins (LOW effort)

### 1. Resizable Panels — `paneforge`
**Impact: HIGH | Effort: LOW**

Replace the manual `mousedown` resize handler in `+layout.svelte` and `DepartmentPanel.svelte` with PaneForge (powers shadcn-svelte's Resizable component).

**Why:** Current handler has no keyboard support, no snap points, no min/max constraints, no localStorage persistence, and no ARIA roles. PaneForge gives all of that for free.

**Library:** `paneforge` (already in shadcn-svelte ecosystem)

**What changes:**
- Layout becomes `<PaneGroup>` with `<Pane>` + `<PaneResizer>` for sidebar | main | panel
- Sizes persist automatically
- Keyboard accessible (arrow keys to resize)
- Collapse/expand with double-click on handle

---

### 2. Keyboard Shortcuts — `@svelte-put/shortcut`
**Impact: HIGH | Effort: LOW**

Go beyond Cmd+K. Add contextual shortcuts for every high-frequency action.

**Library:** `@svelte-put/shortcut` (Svelte 5 runes-compatible, action-based)

**Shortcuts to implement:**

| Shortcut | Action |
|----------|--------|
| `G D` | Go to Dashboard |
| `G C` | Go to Chat |
| `G F` | Go to Forge |
| `G 1-9` | Go to department by index |
| `N` | New chat / new conversation |
| `S` | Focus search / chat input |
| `/` | Focus command palette search |
| `?` | Show shortcuts overlay |
| `Cmd+Shift+F` | Toggle focus mode |
| `Cmd+B` | Toggle sidebar |
| `Cmd+.` | Toggle config panel |
| `Escape` | Close modal / panel / palette |

**Pattern:** Shortcuts scoped to context (different in chat vs. settings). Show `?` overlay with all active shortcuts. Display shortcut hints in command palette items.

---

### 3. Context Menus — shadcn-svelte Context Menu
**Impact: MEDIUM | Effort: LOW**

Right-click on entities for quick actions without opening forms.

**Library:** Already in shadcn-svelte (`add context-menu`)

**Where to add:**
- **Agent card:** Edit, Duplicate, Disable, View Logs, Delete
- **Chat message:** Copy, Re-run, Pin, View Tokens
- **Workflow:** Edit, Run, Duplicate, Delete
- **Skill/Rule:** Edit, Toggle Enable, Duplicate, Delete
- **Conversation:** Rename, Export, Delete

---

### 4. Copy to Clipboard — `svelte-copy`
**Impact: LOW | Effort: LOW**

One-click copy for messages, configs, code blocks, agent JSON.

**Library:** `svelte-copy` (Svelte 5, action-based: `use:copy={'text'}`)

**Where to add:**
- Copy button on every assistant message (hover to reveal)
- Copy button on agent/skill/rule configs
- Copy button on API responses in workflow results
- Copy button on code blocks (Streamdown already has this, verify it works)
- Brief "Copied!" toast on click

---

### 5. Skeleton Loading States — shadcn-svelte Skeleton
**Impact: MEDIUM | Effort: LOW**

Replace spinners with shape-matching skeletons that feel faster.

**Library:** Already in shadcn-svelte (`add skeleton`)

**Skeletons to create:**
- **Chat skeleton:** 3 message bubbles (alternating left/right)
- **Dashboard skeleton:** 4 stat cards + 2 list cards
- **Agent list skeleton:** 3-4 card shapes
- **Department panel skeleton:** Tab bar + 3 items

**Pattern:** Use Svelte `{#await}` blocks. Show skeleton during initial load, real content after.

---

### 6. Spotlight / Focus Mode — CSS only
**Impact: MEDIUM | Effort: LOW**

Dim everything except the active panel for deep work.

**No library needed.** Pure CSS:
- Toggle `data-focus-mode` on `<html>`
- All panels except active get `opacity: 0.15; pointer-events: none; filter: blur(1px)`
- Shortcut: `Cmd+Shift+F`
- ESC or same shortcut to exit
- Subtle background color shift for visual calm

---

### 7. Confetti on Achievements — `svelte-confetti`
**Impact: LOW | Effort: LOW**

Micro-delight for a solo founder who has no team to celebrate with.

**Library:** `svelte-confetti` (zero deps, SSR-safe, pure HTML/CSS animation)

**Trigger on:**
- Workflow completed successfully
- Goal marked as achieved
- First agent created (onboarding milestone)
- Daily mission generated
- All onboarding steps completed

**Style:** Subtle cone burst from the success button, not full-screen.

---

## Tier 2: Core Feature Gaps (MEDIUM effort)

### 8. Modal / Dialog Component — shadcn-svelte Dialog
**Impact: HIGH | Effort: LOW**

Current `Modal.svelte` is 0 bytes. This blocks confirmation dialogs, edit forms, detail views.

**Library:** Already in shadcn-svelte (`add dialog`)

**Use cases:**
- Delete confirmation ("Are you sure you want to delete agent X?")
- Agent/skill/rule edit forms (instead of inline forms)
- Workflow run results detail view
- Conversation export dialog
- Settings save confirmation
- Keyboard shortcut reference overlay

---

### 9. Data Tables — shadcn-svelte Data Table (TanStack Table v8)
**Impact: HIGH | Effort: MEDIUM**

Agent/skill/rule lists have no sorting, filtering, pagination, or column controls.

**Library:** shadcn-svelte Data Table (wraps TanStack Table v8)

**Tables to create:**
- **Agents table:** Name, Role, Model, Status, Created — sortable, filterable
- **Skills table:** Name, Description, Engine — searchable
- **Rules table:** Name, Enabled toggle, Engine — filterable
- **Events table:** Source, Kind, Timestamp — sortable, paginated
- **Conversations table:** Title, Messages, Last Updated — sortable

**Features:** Column visibility toggles, row selection for bulk actions, inline status toggle.

---

### 10. Full-Text Search — `FlexSearch` + `Fuse.js`
**Impact: HIGH | Effort: MEDIUM**

Can't find conversations, agents, events, or rules. Only the command palette has fuzzy search on action names.

**Libraries:**
- `FlexSearch` — Index conversations, events, agent configs (large text corpora)
- `Fuse.js` — Fuzzy match on entity names in command palette

**Implementation:**
- Build search index on page load from stores (agents, skills, rules, conversations, events)
- Display results grouped by type: "Agents (3)", "Conversations (12)", "Events (45)"
- Integrate into command palette as a "Search" section
- Dedicated `/search?q=` route for full results

---

### 11. Drag-and-Drop — `svelte-dnd-action`
**Impact: HIGH | Effort: MEDIUM**

Workflow steps, agent priority, rule ordering — all need reordering.

**Library:** `svelte-dnd-action` (Svelte 5 compatible since v0.9.29)

**Where to add:**
- Workflow step reordering in WorkflowBuilder
- Agent list reordering (priority)
- Rule list reordering (evaluation order matters)
- Quick action reordering in department pages
- Sidebar navigation item reordering (custom department order)

---

### 12. Real-Time SSE Event Feed — `sveltekit-sse`
**Impact: HIGH | Effort: MEDIUM**

No real-time visibility into what departments are doing. Events only load on page visit.

**Library:** `sveltekit-sse` (purpose-built for SvelteKit, auto-reconnect)

**Implementation:**
- Persistent notification bell in layout header
- Click to open slide-out activity panel
- Events streamed live from all departments
- Badge count on department sidebar icons (unread events)
- Filter by: department, severity (info/warning/error), time range
- Group by time: "Just now", "5 min ago", "Today"
- Agent presence indicators: pulsing dot on sidebar when agent is active

---

### 13. Settings Page — Custom Build
**Impact: HIGH | Effort: MEDIUM**

Current settings page is read-only (67 lines). Can't configure anything.

**Sections to add:**
- **Appearance:** Theme (dark/light/system), accent color picker (6 presets), density (compact/default/comfortable)
- **LLM Providers:** Model configuration per provider, API key management, default model selection
- **Workspace:** Name, description, timezone
- **Notifications:** Which events trigger toasts, email notifications
- **Keyboard Shortcuts:** Reference list with customization
- **Data Management:** Export all data, clear cache, backup/restore
- **Danger Zone:** Reset workspace, delete all sessions

---

### 14. Markdown Editor for Skills/Rules — `carta`
**Impact: MEDIUM | Effort: MEDIUM**

Skills and rules use plain `<textarea>` for prompt templates. No preview, no syntax help.

**Library:** `carta` (lightweight Svelte markdown editor, unified/remark/rehype powered)

**Features:**
- Syntax highlighting in the editor
- Live preview pane
- Toolbar: bold, italic, code, link, heading, list
- Variable insertion (template variables like `{{agent_name}}`)
- Character/word count
- Keyboard shortcuts (Cmd+B for bold, etc.)

---

## Tier 3: AI-Specific (HIGH effort, HIGH differentiation)

### 15. Token/Cost Meters
**Impact: HIGH | Effort: HIGH**

No visibility into LLM spend. Critical for budget management.

**Implementation:**
- Per-conversation token count + cost display (in chat header)
- Per-department monthly spend (in department page header)
- Dashboard cost chart: daily/weekly/monthly spend by department
- Budget alerts: toast when department exceeds threshold
- Model cost comparison: "This message cost $0.02 with Opus, would cost $0.004 with Haiku"

**Requires backend:** SSE stream already sends `cost_usd` on completion. Aggregate and expose via `/api/analytics/costs`.

---

### 16. Approval Queue UI (ADR-008)
**Impact: HIGH | Effort: MEDIUM**

API has `/api/approvals`, `approveJob()`, `rejectJob()` — but zero UI. Human gates are blind.

**Implementation:**
- Approval badge count in sidebar (like unread notifications)
- `/approvals` route with pending jobs list
- Each job card: source department, action type, payload preview, approve/reject buttons
- Inline approval in notification panel (don't force route change)
- Keyboard shortcuts: `A` to approve, `R` to reject focused job

---

### 17. Prompt Playground
**Impact: HIGH | Effort: HIGH**

Test prompts against models before deploying to agents.

**Implementation:**
- Split-screen: prompt editor (left) | response viewer (right)
- Model selector (Ollama, Claude, OpenAI)
- Temperature/max_tokens sliders
- Variable substitution from skill templates
- Side-by-side comparison: run same prompt on 2 models
- Save as skill when satisfied
- Cost estimate before running

---

### 18. Agent Trace Visualization
**Impact: HIGH | Effort: HIGH**

See what tools an agent called, in what order, with what results.

**Implementation:**
- Collapsible step cards in chat messages
- Each step: tool name, input params (truncated), execution time, output preview, cost
- Status badges: queued (gray) → executing (amber pulse) → completed (green) → failed (red)
- Timeline/waterfall view for multi-step traces
- Filter: show/hide tool calls, show/hide thinking

**Requires backend:** Agent runtime needs to emit tool-use events to SSE stream.

---

## Gap Analysis — Critical Fixes

### Missing Components
| Component | Status | Fix |
|-----------|--------|-----|
| `Modal.svelte` | Empty (0 bytes) | Replace with shadcn Dialog |
| DepartmentPanel tabs | Defined in code, not rendered | Wire up tab UI component |
| Breadcrumbs | Missing entirely | Add shadcn Breadcrumb |
| Error boundary | No global error handling | Add SvelteKit error page + recovery UI |

### Missing Pages
| Page | Purpose |
|------|---------|
| `/approvals` | Human-in-the-loop approval queue |
| `/search` | Full-text search results |
| Error pages (404, 500) | User-friendly error states |

### Accessibility Gaps
- No ARIA labels on icon-only buttons
- No skip-to-content link
- No focus trap in command palette
- No `prefers-reduced-motion` respect
- Chat textarea has no associated label
- Sidebar collapse not announced to screen readers

### Mobile Gaps
- Sidebar takes full width on small screens
- No bottom navigation for mobile
- Department panel overflows on narrow screens
- No hamburger menu toggle
- Command palette not touch-friendly

---

## Recommended Library Stack (additions)

| Library | Purpose | Effort |
|---------|---------|--------|
| `paneforge` | Resizable panels | LOW |
| `@svelte-put/shortcut` | Keyboard shortcuts | LOW |
| `svelte-copy` | Copy to clipboard | LOW |
| `svelte-confetti` | Achievement celebrations | LOW |
| `svelte-dnd-action` | Drag-and-drop reordering | MEDIUM |
| `flexsearch` | Full-text search indexing | MEDIUM |
| `fuse.js` | Fuzzy search matching | LOW |
| `sveltekit-sse` | Real-time event streaming | MEDIUM |
| `carta` | Markdown editor for prompts | MEDIUM |
| `@git-diff-view/svelte` | Config change diff viewer | LOW |

---

## Implementation Order

### Sprint 1: Quick Wins (Tier 1)
1. PaneForge resizable panels
2. Keyboard shortcuts (`@svelte-put/shortcut`)
3. Context menus (shadcn)
4. Copy to clipboard (`svelte-copy`)
5. Skeleton loading (shadcn)
6. Focus mode (CSS)
7. Confetti (`svelte-confetti`)

### Sprint 2: Core Gaps (Tier 2a)
8. Dialog/Modal (shadcn)
9. Data tables (TanStack via shadcn)
10. Full-text search (FlexSearch)
11. Drag-and-drop (svelte-dnd-action)

### Sprint 3: Live & Settings (Tier 2b)
12. SSE event feed (sveltekit-sse)
13. Settings page overhaul
14. Markdown editor (carta)

### Sprint 4: AI-Specific (Tier 3)
15. Token/cost meters
16. Approval queue UI
17. Prompt playground
18. Agent trace visualization

> **COMPLETED & ARCHIVED** — Done 2026-03-24.

# Refactor & Enhance — Definitive Plan

> Fixes structural debt first, then adds features on a solid foundation.
> Every change has a reason. Nothing is cosmetic.

---

## Current State (honest assessment)

| Problem | Severity | Files affected |
|---------|----------|---------------|
| Mixed color tokens (`--r-*` vs shadcn) | HIGH | 8 files, 51 occurrences |
| DepartmentPanel.svelte = 788 lines | HIGH | 1 file, 10+ concerns |
| Custom event bus for panel↔chat | MEDIUM | 2 files |
| ChatTopBar still uses `--r-*` tokens | MEDIUM | 1 file, 17 occurrences |
| No error boundaries | HIGH | All routes |
| No form validation | MEDIUM | 5 CRUD forms |
| No API caching | MEDIUM | Every mount refetches |
| Modal.svelte is empty | HIGH | Blocks all dialogs |
| colorClasses map = 120 lines of static classes | LOW | 1 file |
| No tests | HIGH | All code |

**What's already solid:**
- Route consolidation done (`/dept/[id]` handles all departments)
- SSE streaming centralized via `parseSSE()`
- shadcn design system installed and working
- Streamdown replaces marked in all chat components
- Toast system wired into all CRUD operations
- Department registry is dynamic (loaded from API)

---

## Architecture: Target State

```
src/
├── app.css                          # Single token system (shadcn only)
├── lib/
│   ├── api.ts                       # API client (unchanged, already clean)
│   ├── stores.ts                    # Global stores + derived command queue
│   ├── cache.ts                     # NEW: Simple fetch cache (stale-while-revalidate)
│   ├── utils/
│   │   └── cn.ts                    # Class merge (already done)
│   ├── components/
│   │   ├── ui/                      # Design system primitives (shadcn-based)
│   │   │   ├── Button.svelte
│   │   │   ├── Card.svelte
│   │   │   ├── Dialog.svelte        # NEW: replaces empty Modal.svelte
│   │   │   ├── Skeleton.svelte      # NEW: loading placeholders
│   │   │   ├── ... (existing)
│   │   │   └── index.ts
│   │   ├── chat/
│   │   │   ├── DepartmentChat.svelte  # Chat only (unchanged)
│   │   │   ├── ChatTopBar.svelte      # Migrated to shadcn tokens
│   │   │   └── ChatSidebar.svelte     # God agent sidebar
│   │   ├── department/              # NEW: extracted from DepartmentPanel
│   │   │   ├── DepartmentPanel.svelte # Shell: tabs + resize + header only
│   │   │   ├── ActionsTab.svelte      # Quick actions + capability
│   │   │   ├── AgentsTab.svelte       # Agent CRUD
│   │   │   ├── SkillsTab.svelte       # Skill CRUD
│   │   │   ├── RulesTab.svelte        # Rule CRUD
│   │   │   ├── McpTab.svelte          # MCP server CRUD
│   │   │   ├── HooksTab.svelte        # Hook CRUD
│   │   │   ├── WorkflowsTab.svelte    # Workflow builder + runner
│   │   │   ├── DirsTab.svelte         # Directory management
│   │   │   ├── EventsTab.svelte       # Event log
│   │   │   └── CrudForm.svelte        # NEW: generic create form pattern
│   │   ├── workflow/                # Visual builder (existing)
│   │   ├── onboarding/              # Existing (migrated tokens)
│   │   ├── layout/                  # Existing
│   │   ├── typography/              # Existing (migrated tokens)
│   │   └── icons/                   # Existing
│   └── design/
│       └── theme.svelte.ts          # Theme toggle (existing)
├── routes/
│   ├── +layout.svelte               # PaneForge layout
│   ├── +error.svelte                # NEW: error boundary
│   ├── +page.svelte                 # Dashboard
│   ├── chat/+page.svelte            # God agent
│   ├── dept/[id]/+page.svelte       # All departments (existing)
│   └── settings/+page.svelte        # Enhanced settings
```

---

## Phase 1: Fix the Foundation (no features, just correctness)

### 1.1 Eliminate `--r-*` tokens — single color system

**Why:** Two competing color systems cause confusion. Every new component requires guessing which tokens to use. Shadcn wins because it's the standard, and our components already use it.

**Action:**
- In `app.css`: Remove all `--r-*` variable definitions
- In 8 files (51 occurrences): Replace every `var(--r-*)` reference:

| Old token | New token |
|-----------|-----------|
| `--r-bg-base` | `--background` |
| `--r-bg-surface` | `--card` |
| `--r-bg-raised` | `--secondary` |
| `--r-bg-overlay` | `--popover` |
| `--r-fg-default` | `--foreground` |
| `--r-fg-muted` | `--muted-foreground` |
| `--r-fg-subtle` | `--muted-foreground` |
| `--r-fg-on-brand` | `--primary-foreground` |
| `--r-border-default` | `--border` |
| `--r-border-strong` | `--border` |
| `--r-border-brand` | `--ring` |
| `--r-brand-default` | `--primary` |
| `--r-brand-hover` | `--primary` (with /90) |
| `--r-ring` | `--ring` |
| `--r-ring-offset` | `--background` |

**Files:** `ChatTopBar.svelte`, `DepartmentPanel.svelte`, `DeptHelpTooltip.svelte`, `Heading.svelte`, `Text.svelte`, `Divider.svelte`, `dept/[id]/+page.svelte`, `app.css`

**Result:** One token system. Every developer (human or AI) knows exactly which tokens to use.

---

### 1.2 Replace custom event bus with store

**Why:** `document.dispatchEvent(new CustomEvent('dept-quick-action'))` is invisible to the type system, impossible to trace in DevTools, and couples DepartmentPanel to DepartmentChat via a global side channel.

**Action:**
- Add to `stores.ts`:
```typescript
export const pendingCommand = writable<{ prompt: string } | null>(null);
```
- DepartmentPanel: `pendingCommand.set({ prompt })` instead of `dispatchEvent`
- DepartmentChat: `pendingCommand.subscribe(cmd => { if (cmd) { inputText = cmd.prompt; send(); pendingCommand.set(null); } })`

**Result:** Typed, traceable, testable.

---

### 1.3 Dialog component (replace empty Modal.svelte)

**Why:** 0-byte Modal.svelte blocks every confirmation dialog, edit form, and detail view.

**Action:**
- Create `Dialog.svelte` using bits-ui Dialog primitive (already installed)
- Props: `open` (bindable), `title`, `description`, `children` snippet
- Features: Focus trap, ESC to close, click-outside to close, ARIA roles
- Remove empty `Modal.svelte`, update `index.ts` export

**Result:** Delete confirmations, edit forms, and workflow results can use proper dialogs.

---

### 1.4 Error boundary

**Why:** No `+error.svelte` means SvelteKit shows a raw error page on any unhandled exception.

**Action:**
- Create `src/routes/+error.svelte` with:
  - Error message display
  - "Go to Dashboard" button
  - "Try Again" button (reload)
- Add try/catch wrappers in `+layout.svelte` `onMount`

**Result:** Graceful failure instead of blank screen.

---

## Phase 2: Split DepartmentPanel (the big one)

### 2.1 Extract tab components

**Why:** 788-line component with 10 tabs, 20+ functions, 5 identical CRUD patterns. Impossible to maintain, test, or reason about.

**Design:**

**DepartmentPanel.svelte (shell, ~120 lines):**
- Header with icon, title, help tooltip, collapse button
- Tab bar with dynamic tabs from `dept.tabs`
- Resize handle
- `{#if activeTab === 'agents'}<AgentsTab {dept} />{/if}` for each tab
- Pass only `dept` ID — each tab fetches its own data

**CrudForm.svelte (generic, ~80 lines):**
```svelte
<CrudForm
  title="Create Agent"
  show={showCreate}
  onsubmit={handleCreate}
  oncancel={() => showCreate = false}
>
  <Input bind:value={name} label="Name" required />
  <Input bind:value={role} label="Role" />
  <Select bind:value={model} options={modelOptions} label="Model" />
</CrudForm>
```
Wraps: validation check, submit button, cancel button, loading state, error display.

**AgentsTab.svelte (~100 lines):**
- Load agents for dept on mount
- List with delete/edit actions
- CrudForm for creation
- Context menu for quick actions

**Same pattern for:** SkillsTab, RulesTab, McpTab, HooksTab, WorkflowsTab, DirsTab, EventsTab

**Result:** Each tab is independent, testable, <120 lines. DepartmentPanel becomes a thin shell.

---

### 2.2 Move colorClasses to CSS custom properties

**Why:** 120 lines of static Tailwind class maps is fragile and bloats the component. CSS variables are the right tool for per-department theming.

**Design:**

In `DepartmentPanel.svelte` (or the dept/[id] page):
```svelte
<div style="--dept-color: {deptColorHsl}; --dept-color-light: {deptColorLightHsl};">
```

In `app.css`:
```css
@theme inline {
  --color-dept: var(--dept-color);
  --color-dept-light: var(--dept-color-light);
}
```

Then all tab components use: `bg-dept/20 text-dept border-dept/30`

**Mapping (12 colors → 12 HSL values):**
| Color | HSL |
|-------|-----|
| emerald | 160 84% 39% |
| purple | 271 91% 65% |
| amber | 38 92% 50% |
| cyan | 192 91% 36% |
| indigo | 239 84% 67% |
| rose | 347 77% 50% |
| sky | 199 89% 48% |
| orange | 25 95% 53% |
| lime | 84 85% 43% |
| pink | 330 81% 60% |
| teal | 173 80% 36% |
| violet | 263 70% 50% |

**Result:** 120 lines of colorClasses → 12 lines of HSL values. All Tailwind classes use a single `dept` token. No purge issues.

---

## Phase 3: API & Data Layer

### 3.1 Simple fetch cache

**Why:** Every department mount re-fetches models, tools, config — data that changes once per session at most.

**Design:** `src/lib/cache.ts`
```typescript
const cache = new Map<string, { data: unknown; timestamp: number }>();
const TTL = 60_000; // 1 minute

export async function cached<T>(key: string, fetcher: () => Promise<T>): Promise<T> {
  const entry = cache.get(key);
  if (entry && Date.now() - entry.timestamp < TTL) return entry.data as T;
  const data = await fetcher();
  cache.set(key, { data, timestamp: Date.now() });
  return data;
}

export function invalidate(key: string) { cache.delete(key); }
export function invalidateAll() { cache.clear(); }
```

**Usage:**
```typescript
// Before: every mount
const models = await getModels();
// After: cached for 60s
const models = await cached('models', getModels);
```

**Apply to:** `getModels()`, `getTools()`, `getDeptConfig()`, `getHookEvents()`, `getDepartments()`

**Result:** Faster navigation between departments. No unnecessary API calls.

---

### 3.2 Consistent error handling

**Why:** Some components use toast, some silently swallow, some show inline errors. User can't tell if an action succeeded or failed.

**Design rule:** Every user-initiated API call follows:
```typescript
try {
  const result = await apiCall();
  toast.success('Action completed');
  return result;
} catch (e) {
  toast.error(e instanceof Error ? e.message : 'Unknown error');
  throw e; // re-throw so caller can handle
}
```

**Apply to:** All CRUD operations (already done in DepartmentPanel), chat send (add toast on error), settings save, session create.

---

## Phase 4: New Features (on solid foundation)

### 4.1 PaneForge layout

**Why:** Current mousedown handler has no keyboard support, no snap points, no min/max constraints beyond manual clamping, no persistence.

**Library:** `paneforge` (already in shadcn ecosystem, powers shadcn Resizable)

**Design:**
```svelte
<PaneGroup direction="horizontal">
  <Pane defaultSize={15} minSize={3} collapsible collapsedSize={3}>
    <!-- Sidebar -->
  </Pane>
  <PaneResizer />
  <Pane>
    <PaneGroup direction="horizontal">
      <Pane defaultSize={25} minSize={15} collapsible>
        <!-- DepartmentPanel -->
      </Pane>
      <PaneResizer />
      <Pane>
        <!-- Chat / Main content -->
      </Pane>
    </PaneGroup>
  </Pane>
</PaneGroup>
```

**Features gained:**
- Keyboard resize (arrow keys)
- Double-click to collapse
- Persisted sizes (localStorage via `autoSaveId`)
- Accessible (ARIA roles, focus management)
- Removes ~50 lines of manual resize logic from layout + panel

---

### 4.2 Skeleton loading

**Why:** Spinners feel broken. Skeletons signal "content is coming."

**Design:** One `Skeleton.svelte` component:
```svelte
<div class={cn('animate-pulse rounded-md bg-muted', className)} {...rest}></div>
```

Create composite skeletons:
- `ChatSkeleton.svelte` — 4 alternating message bubbles
- `ListSkeleton.svelte` — 3-5 card shapes with text lines
- `DashboardSkeleton.svelte` — 4 stat cards + 2 list areas

**Apply to:** Dashboard load, department tab switching, conversation history load.

---

### 4.3 Keyboard shortcuts

**Library:** `@svelte-put/shortcut` (Svelte 5 action-based)

**Design:** Central shortcut registry in `src/lib/shortcuts.ts`:
```typescript
export const shortcuts = {
  'g d': () => goto('/'),
  'g c': () => goto('/chat'),
  'g f': () => goto('/dept/forge'),
  'n': () => { /* new chat */ },
  '?': () => { /* show overlay */ },
  'Escape': () => { /* close active modal/panel */ },
} as const;
```

Wire in `+layout.svelte` with a single `use:shortcut` action.

---

### 4.4 Context menus

**Library:** bits-ui ContextMenu (already installed)

**Design:** Right-click on:
- Agent card → Edit, Duplicate, Delete
- Chat message → Copy, Re-run
- Skill → Edit, Test, Delete
- Workflow → Run, Edit, Delete

---

### 4.5 Copy to clipboard

**Library:** `svelte-copy`

**Design:**
- Hover on assistant message → show copy icon in top-right
- Click → copies message content → toast "Copied"
- Also on: agent config JSON, workflow results, code blocks

---

### 4.6 Settings page overhaul

**Current:** Read-only health check + hardcoded engine list (194 lines)

**Target sections:**
1. **Appearance** — Theme toggle (dark/light), accent color (6 presets via CSS variable swap)
2. **Approvals** — Pending jobs with approve/reject (already partially done)
3. **System** — Health check, versions, DB location
4. **Keyboard Shortcuts** — Reference table
5. **Data** — Export all data (JSON download), clear cache

---

## Phase 5: Quality

### 5.1 Tests

**Priority tests (highest value per effort):**

1. **`api.ts` unit tests** — Mock fetch, verify request shapes, error handling
2. **`cn.ts` unit test** — Verify class merging behavior
3. **`cache.ts` unit tests** — TTL, invalidation, concurrent access
4. **Component smoke tests** — Button, Card, Input render without errors

**Runner:** `vitest` (already works with SvelteKit)

### 5.2 Accessibility audit

**Checklist:**
- [ ] All icon-only buttons have `aria-label`
- [ ] Dialog components trap focus
- [ ] Chat messages region has `role="log"` + `aria-live="polite"`
- [ ] Skip-to-content link in layout
- [ ] `prefers-reduced-motion` disables animations
- [ ] Form inputs have associated `<label>` elements
- [ ] Color is not the only indicator (add icons/text alongside colored badges)

---

## Implementation Order

```
Phase 1 (foundation) ─── must be first, everything depends on it
  1.1 Eliminate --r-* tokens           ~30 min
  1.2 Store-based command queue         ~15 min
  1.3 Dialog component                  ~30 min
  1.4 Error boundary                    ~15 min

Phase 2 (split DepartmentPanel) ─── biggest structural improvement
  2.1 Extract 9 tab components          ~2 hours
  2.2 CSS variable dept colors          ~30 min

Phase 3 (data layer) ─── prevents regressions
  3.1 Fetch cache                       ~30 min
  3.2 Consistent error handling         ~30 min

Phase 4 (features) ─── each independent, can be done in any order
  4.1 PaneForge layout                  ~1 hour
  4.2 Skeleton loading                  ~30 min
  4.3 Keyboard shortcuts                ~30 min
  4.4 Context menus                     ~30 min
  4.5 Copy to clipboard                 ~15 min
  4.6 Settings page                     ~1 hour

Phase 5 (quality) ─── ongoing
  5.1 Tests                             ~2 hours
  5.2 Accessibility                     ~1 hour
```

**Total estimated effort:** ~10-12 hours across 5 phases.

**Critical path:** Phase 1 → Phase 2 → Phase 4.1 (PaneForge). Everything else is parallelizable.

---

## What NOT to do

1. **Don't add LayerChart/D3 charts yet** — The dashboard bar chart is CSS-only and sufficient. Real charts need real data (token costs, time series) that the backend doesn't expose yet.
2. **Don't add FlexSearch yet** — Full-text search needs a backend endpoint. Client-side indexing of all conversations is too expensive.
3. **Don't add sveltekit-sse yet** — The SSE pattern works fine for chat. A live event feed needs a backend `/api/events/stream` endpoint that doesn't exist.
4. **Don't add drag-and-drop yet** — Workflow reordering is nice-to-have. Fix the workflow builder UX first (it's barely wired in).
5. **Don't add Carta markdown editor yet** — Plain textarea works for rules/skills. Editor adds complexity that isn't justified by current usage patterns.

These are all valid enhancements, but they depend on backend work or usage patterns that don't exist yet. Ship the foundation first.

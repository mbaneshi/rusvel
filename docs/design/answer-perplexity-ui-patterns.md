# UI/UX Pattern Research for AI Workspace Tools

**Date:** 2026-03-27
**Purpose:** Inform RUSVEL's 13-department AI workspace UI decisions
**Scope:** Chat placement, multi-workspace navigation, mode switching, context switching, information density

---

## 1. Chat Placement Patterns in AI Tools

### Pattern 1A: Chat-as-Page (Full Canvas)

**What it is:** Chat occupies the entire main content area. The conversation is the workspace.

**Examples:** ChatGPT, Claude.ai, Google Gemini, Perplexity

**How it works:**
- Sidebar holds conversation history (list of past chats)
- Main area is 100% chat — input at bottom, messages scrolling up
- Artifacts/outputs (code, documents, images) render inline or in a side panel
- Claude.ai introduced "Artifacts" as a split pane: chat left, rendered output right

**Why it works:**
- Maximum context for conversation — users can see long message chains
- Simple mental model: one conversation, one focus
- Works perfectly when the task IS the conversation (brainstorming, Q&A, generation)

**When to use:** When the primary interaction IS chat. When structured content is secondary or generated from chat. When users spend 80%+ time in conversation.

**When NOT to use:** When users need structured data (tables, forms, dashboards) alongside chat. When chat is one of many activities, not the primary one.

**Relevance to RUSVEL:** The God Agent chat at `/chat` should use this pattern. Department-specific chat should NOT — it competes with structured department content.

---

### Pattern 1B: Chat-as-Sidebar (Persistent Assistant)

**What it is:** Chat lives in a collapsible side panel while structured content occupies the main area.

**Examples:** GitHub Copilot in VS Code, Cursor's Composer sidebar, Google Docs "Help me write", Notion AI sidebar

**How it works:**
- Main area shows the "real work" — code, document, spreadsheet, dashboard
- Right sidebar (typically 300-400px) provides a chat interface
- Chat can reference and modify the main content
- Sidebar collapses to give full width when not needed

**Why it works:**
- Users keep their primary context (code, document) visible
- AI assists without displacing the work product
- Toggling chat is lightweight — Cmd+L in Cursor, a button click in Copilot
- The chat has context about what's in the main area

**When to use:** When users are primarily working on structured content and need AI assistance on the side. When the output of chat modifies the main content.

**When NOT to use:** When chat IS the primary activity. When the sidebar width is too narrow for meaningful conversation.

**Relevance to RUSVEL:** This is the pattern the current UI attempts (DepartmentChat as the main right area). But the current layout has THREE columns competing (sidebar + panel + chat), which undermines the pattern. The two-level-nav proposal correctly moves chat to a section, which is a better fit.

---

### Pattern 1C: Chat-as-Overlay (Transient Assistant)

**What it is:** Chat appears as a floating overlay, modal, or popover that doesn't rearrange the layout.

**Examples:** Intercom/Drift chat widgets, macOS Spotlight-style AI (Raycast AI), Cursor's inline Cmd+K

**How it works:**
- User triggers chat with a keyboard shortcut or button
- A floating panel appears over the current content
- Chat completes a focused task, then dismisses
- Underlying layout is completely undisturbed

**Why it works:**
- Zero layout shift — users don't lose their place
- Fast for quick questions ("what does this error mean?")
- Works well for command-style interactions (do X, generate Y)

**When to use:** For quick, transient interactions. When the user wants an answer, not a conversation. When layout stability is critical.

**When NOT to use:** For extended conversations. When the user needs to reference both chat history and structured content simultaneously.

**Relevance to RUSVEL:** The CommandPalette (Cmd+K) already serves this role. A "quick ask" overlay per department could complement the full chat section.

---

### Pattern 1D: Chat-as-Section (Navigable Page)

**What it is:** Chat is one section among many in a navigation hierarchy, not a persistent panel.

**Examples:** Slack (channels are chat sections), Linear (project chat tab), Basecamp (message board as a section)

**How it works:**
- Navigation lists sections: Overview, Tasks, Chat, Files, Settings
- Clicking "Chat" navigates to a full-width chat view
- Clicking "Tasks" navigates away from chat to a full-width task view
- Chat and structured content never compete for space simultaneously

**Why it works:**
- Each section gets 100% of available width
- No cramped panels — forms and chat both get proper space
- URL-addressable — bookmarkable, shareable, navigable with browser back/forward
- Clear mental model: I'm in chat mode OR I'm in config mode

**When to use:** When chat is ONE OF MANY activities in a workspace. When structured content (CRUD forms, dashboards, tables) needs full width. When the app has 5+ distinct sections per context.

**When NOT to use:** When users constantly need chat visible while doing other work. When the chat directly manipulates what's shown in another panel.

**Relevance to RUSVEL:** This is what the two-level-nav proposal adopts — `/dept/forge/chat` as a section alongside `/dept/forge/skills`, `/dept/forge/agents`, etc. This is the RIGHT pattern for a 13-department workspace where each department has 11+ sections.

---

### Pattern 1E: Hybrid — Artifacts/Canvas Pattern

**What it is:** Chat generates structured outputs that render in a parallel canvas. The user can switch between conversational mode and direct-editing mode on the canvas.

**Examples:** Claude.ai Artifacts, ChatGPT Canvas, v0.dev (chat generates UI), Bolt.new (chat generates full apps)

**How it works:**
- Left panel: chat conversation
- Right panel: live artifact (code, UI preview, document, diagram)
- User can edit the artifact directly OR ask the AI to modify it via chat
- Multiple artifacts can be tabbed or stacked

**Why it works:**
- Bridges conversation and creation — the chat produces something tangible
- Users can verify AI output immediately
- Direct editing gives users control without re-prompting
- Iteration is fast: "make the button blue" updates the artifact in real-time

**When to use:** When AI generates discrete outputs (code, documents, configs, plans). When users need to review and refine AI-generated content.

**When NOT to use:** When the output is purely informational (Q&A). When the "artifact" doesn't make sense as a standalone editable object.

**Relevance to RUSVEL:** Department chat could adopt this for specific workflows — content drafts in the Content department, code analysis results in Code, financial projections in Finance. The chat section could optionally split into chat + artifact pane when the AI generates structured output.

---

### Recommendation for RUSVEL

Use a **layered approach**:

| Layer | Pattern | Where |
|-------|---------|-------|
| God Agent | Chat-as-Page (1A) | `/chat` — full-width, artifact support |
| Department Chat | Chat-as-Section (1D) | `/dept/[id]/chat` — full-width when focused |
| Quick Ask | Chat-as-Overlay (1C) | Cmd+K CommandPalette for quick queries |
| Engine Results | Hybrid Artifacts (1E) | Within dept chat, structured outputs rendered as artifacts |

This avoids the three-column problem and gives each mode full breathing room.

---

## 2. Multi-Workspace Navigation (10+ Workspaces)

### Pattern 2A: Vertical Icon Rail + Sidebar (Slack/Discord Model)

**What it is:** A narrow (~48-64px) vertical strip of icons on the far left, each representing a workspace/server. Clicking one loads its navigation tree in an adjacent sidebar.

**Examples:**
- Slack: workspace icons (left rail) -> channel list (sidebar) -> conversation (main)
- Discord: server icons (left rail) -> channel list (sidebar) -> chat (main)
- Microsoft Teams: app icons (left rail) -> team/channel list -> content
- VS Code: activity bar (left rail) -> sidebar panel -> editor

**How it works:**
- Level 1 (Rail): 48px icons, vertically stacked. Each icon = one workspace/context.
- Level 2 (Sidebar): ~240px panel showing sections within the selected workspace.
- Level 3 (Main): Content area consuming remaining width.
- Rail items show unread badges, status indicators, notification dots.
- Rail can scroll if items exceed viewport height.

**Why it works:**
- Fits 15-20 workspaces in a single viewport without scrolling
- Icons are faster to scan than text labels
- Two-click navigation: workspace -> section -> done
- Persistent visibility — all workspaces always one click away
- Badge indicators give ambient awareness across workspaces

**When to use:** When there are 5-20 distinct top-level contexts. When users frequently switch between contexts. When each context has its own sub-navigation.

**When NOT to use:** When there are fewer than 5 contexts (overkill). When contexts don't have meaningful icons/logos. When horizontal space is at a premium on small screens.

**Relevance to RUSVEL:** With 13 departments, this is a strong candidate. Each department gets an icon in the rail. The sidebar shows that department's sections. However, the two-level-nav proposal chose horizontal tabs instead, which has tradeoffs (see 2B).

---

### Pattern 2B: Horizontal Tab Bar (Browser/Figma Model)

**What it is:** A horizontal strip of tabs or pills across the top of the viewport. Each tab represents a workspace or project.

**Examples:**
- Figma: team tabs across the top, project list below, files in main area
- Browser tabs: each tab = one context
- Vercel: project tabs/dropdown in the top nav
- Linear: workspace/team switcher as horizontal pills

**How it works:**
- Level 1 (Top bar): Horizontal scrollable row of workspace identifiers
- Level 2 (Sidebar or sub-nav): Sections within the selected workspace
- Level 3 (Main): Content area
- Tabs can scroll horizontally when they overflow
- Active tab is highlighted, others are muted

**Why it works:**
- Familiar from browser tabs — deeply ingrained muscle memory
- Labels + icons give more information than icons alone
- Natural reading direction (left to right) for scanning options
- Top placement doesn't compete with sidebar content

**Tradeoffs vs vertical rail:**
- Horizontal space is more limited — 13 departments with labels may overflow on 1280px screens
- Scrolling horizontally is less natural than scrolling vertically
- Can't show as many items without scrolling (typically 7-10 visible vs 15-20 in a rail)

**When to use:** When there are 4-10 contexts with meaningful labels. When the hierarchy is shallow (2 levels). When top-of-page real estate is available.

**When NOT to use:** When there are 15+ contexts (overflow becomes problematic). When labels are long. When vertical space is more precious than horizontal.

**Relevance to RUSVEL:** The two-level-nav proposal uses this for the department bar. With 13 departments, this will overflow on smaller screens. Consider: icon-only mode for compact viewports, or group departments into categories (Core: Forge/Code/Content/Harvest, Business: GTM/Finance/Product/Growth, Operations: Distro/Legal/Support/Infra) with expandable groups.

---

### Pattern 2C: Activity Bar + Sidebar (VS Code Model)

**What it is:** A hybrid where the left rail contains both workspace icons AND mode icons (explorer, search, source control, extensions). Clicking a rail icon changes the sidebar content.

**Examples:**
- VS Code: Activity Bar (5-8 icons) -> Sidebar (tree view) -> Editor (tabs) -> Panel (terminal/output)
- JetBrains IDEs: Tool Windows on left/right/bottom rails
- Figma (editor mode): left panel (layers), right panel (properties), top bar (tools)

**How it works:**
- Activity Bar: Fixed icons representing modes/views, not just workspaces
- Each icon swaps the sidebar content (file explorer, search results, git changes)
- The main editor area is independent — multiple tabs/splits
- Bottom panel for output/terminal/logs

**Why it works:**
- Clear mode separation: "I'm exploring files" vs "I'm searching" vs "I'm reviewing changes"
- Sidebar content is contextual to the selected mode
- Editor content persists across mode switches (critical for code)
- Four-zone layout (rail + sidebar + main + panel) handles complex workflows

**When to use:** When the app has distinct activity modes, not just contexts. When users need a persistent work area (editor, dashboard) that survives mode switches. When the tool is used all day with frequent mode shifts.

**Relevance to RUSVEL:** This maps well to a variant where the rail contains both utility modes (Dashboard, Chat, Approvals, DB, Flows) and department modes (Forge, Code, Content...). The sidebar changes based on selection. The main content area shows the active section. This combines the best of 2A and 2C.

---

### Pattern 2D: Dropdown/Switcher (Linear/Notion Model)

**What it is:** A compact dropdown in the top-left corner that lets users switch between workspaces. Only the current workspace name is visible; others are hidden behind the dropdown.

**Examples:**
- Linear: workspace name in top-left, click to switch teams
- Notion: workspace name in sidebar header, click to switch workspaces
- GitHub: repository name in header, dropdown to switch repos
- Vercel: project name in nav, dropdown to switch projects

**How it works:**
- Current context shown as text/chip in the header
- Click reveals a dropdown with all available contexts
- Dropdown may include search for many items
- Selecting a context reloads the sidebar/main content

**Why it works:**
- Minimal space — only 1 item visible at any time
- Scales to 100+ workspaces (with search)
- Clean UI — no visual clutter from unused workspaces
- Works well when users spend extended time in one context

**When to use:** When users typically work in one context for extended periods. When there are too many contexts for tabs/rail (20+). When the header needs to stay minimal.

**When NOT to use:** When users frequently switch between contexts (too many clicks). When ambient awareness of other contexts (badges, status) matters. When there are few enough contexts to show them all.

**Relevance to RUSVEL:** Not ideal as the primary pattern for departments since a solo founder likely switches frequently between departments. But useful as a SECONDARY switcher (e.g., within a department, switch sub-contexts) or for session switching.

---

### Recommendation for RUSVEL

A **hybrid of 2A (icon rail) and 2B (horizontal tabs)**:

**Option A — Icon rail (recommended for 13 departments):**
```
[48px rail]  [200px sidebar]  [remaining: main content]
 R logo       Actions          Full-width section content
 ----         Engine
 Dashboard    Agents
 Chat         Skills
 Approvals    Rules
 DB           Workflows
 Flows        MCP
 Terminal     Hooks
 ----         Events
 Forge  *     Dirs
 Code         ----
 Content      Chat
 Harvest      Settings
 Flow
 GTM
 Finance
 Product
 Growth
 Distro
 Legal
 Support
 Infra
 ----
 Settings
```

**Why this over the horizontal proposal:**
- 13 departments + 7 utility items = 20 items. Vertical rail handles this without overflow.
- Each department icon is always visible, with badge indicators for pending items.
- The sidebar changes per-department (section list) or per-utility (page-specific).
- No horizontal scrolling needed.

**Option B — Keep horizontal tabs but with overflow handling:**
- Show 7 most-used departments as pills
- "More" dropdown for remaining 6
- Pin/unpin departments based on usage

Either option is defensible. The two-level-nav proposal's horizontal approach works if overflow is handled gracefully. The rail approach is more future-proof as departments grow.

---

## 3. Configure vs Execute vs Monitor Mode Switching

### Pattern 3A: Tab-Based Mode Switching (Vercel Model)

**What it is:** Top-level horizontal tabs within a project context separate Configure, Execute, and Monitor.

**Examples:**
- Vercel: Project -> [Overview | Deployments | Analytics | Logs | Settings]
- Railway: Service -> [Deployments | Metrics | Settings | Variables]
- Netlify: Site -> [Overview | Deploys | Plugins | Build & deploy | Domain | Analytics]
- Cloudflare: Zone -> [Overview | Analytics | DNS | SSL | Firewall | ...]

**How it works:**
- After selecting a project/service, top tabs separate concerns:
  - Overview = current state (monitor)
  - Deployments = history of executions (execute + monitor)
  - Settings = configuration (configure)
  - Logs/Analytics = observability (monitor)
- Each tab is a full page with its own layout
- No real-time mode switching — just navigation

**Why it works:**
- Clear separation of concerns
- Users develop muscle memory for tab positions
- Each mode gets optimized layout (tables for logs, forms for config, charts for analytics)
- URL-addressable: `/project/foo/settings` vs `/project/foo/deployments`

**When to use:** When configure/execute/monitor are distinct activities done at different times. When each mode has enough content for a full page.

**Relevance to RUSVEL:** This maps directly to department sections. For each department:
- **Configure:** Agents, Skills, Rules, MCP, Hooks, Settings sections
- **Execute:** Chat, Actions, Engine sections
- **Monitor:** Events, Dashboard cards

The two-level-nav proposal already achieves this by making each concern a navigable section.

---

### Pattern 3B: Dashboard + Drill-Down (Supabase Model)

**What it is:** A dashboard shows aggregate status across all services, with drill-down into specific service details.

**Examples:**
- Supabase: Left sidebar lists services (Database, Auth, Storage, Functions, Logs). Each has its own sub-navigation.
- AWS Console: Service dashboard -> service-specific pages
- Datadog: Overview dashboard -> detailed views per metric/service
- Grafana: Dashboard grid -> individual panel drill-down

**How it works:**
- Top level: Dashboard with cards/widgets for each service area
- Cards show key metrics, status indicators, recent activity
- Click a card or sidebar item to drill into that service's full UI
- Breadcrumbs maintain context: Dashboard > Database > Tables > users

**Why it works:**
- Ambient awareness — see health of everything at a glance
- Progressive disclosure — summary first, details on demand
- Reduces context-switching by showing enough on the dashboard to decide where to focus
- Good for "morning check" workflow: scan dashboard, identify issues, drill in

**When to use:** When the system has 5+ distinct subsystems. When a daily check-in pattern exists. When aggregate status is valuable.

**Relevance to RUSVEL:** The existing Dashboard at `/` should serve this role. Department cards on the dashboard should show:
- Last chat message
- Pending approvals count
- Active agent count
- Recent events summary
- Health/error status

This gives the solo founder a "mission control" view before diving into any department.

---

### Pattern 3C: Unified Timeline (Railway/Render Model)

**What it is:** A chronological timeline shows all events (deploys, config changes, errors, user actions) in a single feed, regardless of source.

**Examples:**
- Railway: Activity feed showing deploys, config changes, scaling events
- Render: Events tab with chronological log
- GitHub: Activity feed per repository
- Linear: Activity feed per project

**How it works:**
- Single chronological feed mixing event types
- Each event shows: timestamp, type (deploy, config change, error), source, details
- Filterable by event type, source, time range
- Expandable for full details

**Why it works:**
- Causality is clear: "I changed the config, then the deploy failed"
- Cross-cutting visibility: see how actions in one area affect another
- Single place to understand "what happened"

**When to use:** When understanding cause-and-effect across subsystems matters. When debugging requires correlating events from different sources.

**Relevance to RUSVEL:** The Events section per department (`/dept/[id]/events`) already captures this. A cross-department timeline on the Dashboard would add significant value — seeing that a Content draft was triggered by a Harvest opportunity, or that a Code analysis led to a Product roadmap update.

---

### Pattern 3D: Split View — Config Left, Live Preview Right

**What it is:** Configuration controls on one side, live preview/execution result on the other.

**Examples:**
- Vercel's environment variable editor: form left, deployment preview right
- Storybook: controls panel left, component preview right
- Swagger/Postman: request config left, response right
- CodeSandbox: file editor left, live preview right

**How it works:**
- Left panel: forms, toggles, text inputs for configuration
- Right panel: live-updating preview, output, or result
- Changes on the left immediately reflect on the right
- Saves require explicit action (button) or auto-save

**Why it works:**
- Immediate feedback loop: see the effect of every change
- Reduces "configure then hope" anxiety
- Users learn faster by seeing cause and effect simultaneously

**When to use:** When configuration has visible output. When trial-and-error is part of the workflow.

**Relevance to RUSVEL:** Useful for specific department sections:
- **Skills:** Edit skill prompt left, test execution right
- **Rules:** Edit rule text left, see how it modifies system prompt right
- **Agents:** Configure agent left, test conversation right
- **Workflows:** Visual DAG builder left, execution log right

---

### Recommendation for RUSVEL

Combine patterns:

1. **Dashboard (3B)** at `/` — cards per department with key metrics
2. **Tab-based sections (3A)** within each department — configure/execute/monitor as sidebar sections
3. **Timeline (3C)** both per-department (`/dept/[id]/events`) and global (dashboard feed)
4. **Split view (3D)** for specific interactive sections (skill testing, agent testing, workflow execution)

---

## 4. Department/Context Switching Patterns

### Pattern 4A: Pill Tabs (Horizontal Toggle Group)

**What it is:** A row of pill-shaped buttons, one per context, with the active context highlighted.

**Examples:**
- Linear: team pills in sidebar header
- Stripe Dashboard: test/live mode toggle
- Tailwind UI component examples
- iOS segmented controls

**How it works:**
- Horizontal row of rounded buttons/chips
- Click to switch context
- Active pill has distinct background color/weight
- Optionally includes badge counts or status dots

**Why it works:**
- All options visible at once
- Single click to switch
- Visual weight on active option provides clear orientation
- Compact — works in tight spaces

**When to use:** For 2-7 options. When all options fit in available horizontal space. When switching is frequent.

**When NOT to use:** For 10+ options (overflow). When labels are long. When options have very different importance levels.

**Relevance to RUSVEL:** Works for sub-grouping within the department bar. For example, department categories:
- [Core] [Business] [Operations] — 3 pills that filter/group the 13 departments
- Or as view mode toggles within a section: [Cards] [Table] [Timeline]

---

### Pattern 4B: Sidebar Sections with Separators

**What it is:** The sidebar contains grouped items separated by visual dividers or section headers.

**Examples:**
- VS Code Explorer: folders, open editors, outline — separated by collapsible sections
- Slack: Channels, Direct Messages, Apps — separated by headers
- macOS Finder sidebar: Favorites, iCloud, Locations — separated by section headers

**How it works:**
- Section headers (often collapsible) group related items
- Visual separators (lines, spacing) create clear boundaries
- Items within sections are sorted alphabetically or by usage
- Sections can collapse to save vertical space

**Why it works:**
- Categorization reduces cognitive load when scanning many items
- Collapsible sections let users hide irrelevant groups
- Familiar from file systems and email clients

**When to use:** When items can be logically grouped (3-5 groups of 3-7 items each). When some groups are used more than others.

**Relevance to RUSVEL:** In a vertical rail or sidebar approach, departments could be grouped:
```
--- CORE ---
Forge, Code, Content, Harvest
--- BUSINESS ---
GTM, Finance, Product, Growth
--- OPERATIONS ---
Distro, Legal, Support, Infra
--- ORCHESTRATION ---
Flow
```

This adds scannability to the 13-item department list.

---

### Pattern 4C: Favorites/Pinned + Everything Else

**What it is:** Users pin their most-used contexts to a priority position. Remaining contexts are accessible but de-prioritized.

**Examples:**
- Slack: starred channels at top, all channels below
- Chrome: pinned tabs (favicon only) at left, regular tabs after
- macOS Dock: user-arranged favorite apps
- Notion: favorites section in sidebar

**How it works:**
- Top section: 3-5 pinned/favorited items (always visible)
- Below or in dropdown: all remaining items
- Users customize which items are pinned
- Pinned items may use compact representation (icon only)

**Why it works:**
- Adapts to individual usage patterns
- Reduces visual noise from rarely-used contexts
- Power users curate their own optimal layout
- Scales to any number of total items

**When to use:** When users have strong preferences for 3-5 primary contexts. When the total list is too long to show all items equally.

**Relevance to RUSVEL:** The solo founder likely uses 4-5 departments daily (Forge, Code, Content, Harvest, Flow) and others weekly/monthly. Allowing pinned departments in the rail or top bar would reduce cognitive load significantly.

---

### Pattern 4D: Keyboard-First Context Switch

**What it is:** Keyboard shortcuts provide instant context switching without mouse interaction.

**Examples:**
- Slack: Cmd+K (quick switcher), Cmd+1-9 (switch to workspace/channel by position)
- VS Code: Cmd+P (file switcher), Cmd+B (toggle sidebar), Ctrl+1-9 (switch editor tabs)
- Raycast/Alfred: always-on launcher for switching to any app/context
- iTerm2: Cmd+1-9 (switch tabs), Cmd+Shift+[ / ] (cycle tabs)

**How it works:**
- Number shortcuts (Cmd+1 through Cmd+9) for positional switching
- Fuzzy search (Cmd+K) for named switching
- Arrow keys for cycling through contexts
- Customizable keybindings

**Why it works:**
- Fastest possible switching speed — no mouse movement, no visual scanning
- Muscle memory develops for frequent contexts
- Fuzzy search handles edge cases (rarely-used contexts)
- Power users strongly prefer this

**When to use:** Always, as a complement to visual navigation. Power users expect it.

**Relevance to RUSVEL:** The CommandPalette (Cmd+K) already exists. Add:
- Cmd+1-9 for pinned departments
- Type department name in Cmd+K to navigate: "forge skills" -> `/dept/forge/skills`
- Keyboard shortcuts for section navigation within a department

---

### Recommendation for RUSVEL

Layer all four patterns:

1. **Visual navigation:** Sidebar/rail with grouped sections (4B) showing all 13 departments
2. **Quick access:** Pin/favorite top 5 departments (4C) for prominence
3. **Keyboard:** Cmd+1-5 for pinned departments, Cmd+K for fuzzy search (4D)
4. **Category pills:** Optional grouping toggle [Core | Business | Ops] (4A) if rail gets crowded

---

## 5. Information Density for Power Users

### Pattern 5A: Progressive Density (Compact/Comfortable/Spacious)

**What it is:** User-selectable density levels that control spacing, font size, and information per screen.

**Examples:**
- Gmail: Compact, Comfortable, Default density settings
- Google Sheets/Docs: zoom level
- VS Code: editor font size + UI scale
- Jira: list view (compact) vs board view (spacious)
- Linear: compact mode vs comfortable mode

**How it works:**
- A setting (often in view menu) toggles between density levels
- Compact: smaller fonts, tighter spacing, more items per screen
- Comfortable: default spacing, balanced readability
- Spacious: larger fonts, more whitespace, fewer items per screen
- Applied globally or per-section

**Why it works:**
- Respects different user preferences and screen sizes
- Power users choose compact; casual users choose comfortable
- Same data, different presentation — no feature loss
- Users feel in control of their workspace

**When to use:** When the app serves users with different expertise levels. When content is primarily lists/tables/cards that benefit from density control.

**Relevance to RUSVEL:** Offer a global density setting. The solo founder will likely use compact mode most of the time but may want comfortable mode for focused reading.

---

### Pattern 5B: Data Tables with Inline Editing

**What it is:** Tabular data display with the ability to edit cells directly in the table, without navigating to a separate detail page.

**Examples:**
- Notion databases: click a cell to edit it in-place
- Airtable: spreadsheet-like editing in table view
- Linear: click issue fields to edit inline
- Retool/internal tools: data tables with inline editing

**How it works:**
- Default state: read-only table with compact rows
- Click a cell: transforms into an input/select/textarea
- Tab/Enter: moves to next cell
- Escape: cancels edit
- Changes auto-save or save on blur

**Why it works:**
- Eliminates navigation: no open-detail -> edit -> save -> back cycle
- High information density: many rows visible at once
- Familiar from spreadsheets — universal mental model
- Bulk operations feel fast

**When to use:** When managing lists of structured objects (agents, skills, rules). When most edits are small (rename, toggle, change a value).

**When NOT to use:** When objects have complex nested structure. When editing requires rich formatting (markdown, code).

**Relevance to RUSVEL:** Agents, Skills, Rules, Hooks, MCP Servers, and Workflows would all benefit from table view with inline editing as an alternative to the current card-based CRUD. Let users switch between card view (for overview) and table view (for bulk management).

---

### Pattern 5C: Keyboard-Driven Command Bar

**What it is:** A searchable command palette that lets users execute any action by typing.

**Examples:**
- VS Code: Cmd+Shift+P command palette
- Slack: Cmd+K
- Raycast/Alfred: global launcher
- Linear: Cmd+K for both navigation and actions
- Figma: Cmd+/ for quick actions
- Notion: / (slash commands in content)

**How it works:**
- Triggered by keyboard shortcut (Cmd+K or Cmd+Shift+P)
- Modal overlay with search input
- Results show: navigation targets, actions, recent items
- Arrow keys to select, Enter to execute
- Results update as user types (fuzzy matching)
- Categories group results: "Navigate", "Actions", "Recent"

**Why it works:**
- Faster than any visual navigation for users who know what they want
- Scales infinitely — 1000 commands accessible without UI clutter
- Discoverability: users can browse all available actions
- Reduces mouse usage, which power users prefer

**When to use:** Always, for any app with more than 20 distinct actions/pages. This is table stakes for power-user tools.

**Relevance to RUSVEL:** CommandPalette already exists. Enhance it with:
- Department-aware actions: "forge create agent", "code analyze", "content draft"
- Recent items: last 5 visited department sections
- Quick actions: "build capability", "daily plan", "review progress"
- Status queries: "show pending approvals", "cost today"

---

### Pattern 5D: Contextual Panels (Inspector Pattern)

**What it is:** A right-side panel that shows details about the selected item, updating as selection changes.

**Examples:**
- macOS Finder: Preview panel (Cmd+Shift+P)
- Gmail: Reading pane
- Figma: Properties panel on the right
- VS Code: Peek Definition (inline) + Properties panel
- Jira: Issue detail slides in from right

**How it works:**
- Main area: list or grid of items
- Right panel: detailed view of the currently selected item
- Clicking items in the main list updates the panel
- Panel can be closed when not needed
- Some apps use a slide-over instead of a persistent panel

**Why it works:**
- No navigation — users stay on the list while viewing details
- Fast scanning: click through items rapidly to compare
- List context is preserved — user's position in the list doesn't change
- Reduces page loads and context switches

**When to use:** When users browse a list and need details about individual items. When comparing items matters.

**Relevance to RUSVEL:** For Agent, Skill, Rule, and Workflow lists — clicking an item could show its details in a slide-over panel rather than navigating to a detail page. Keeps the user's list position while showing full configuration.

---

### Pattern 5E: Status Bars and Ambient Information

**What it is:** Persistent UI elements that show system status without requiring navigation.

**Examples:**
- VS Code: bottom status bar (git branch, language, encoding, line/col, errors)
- Vercel: deploy status in project header
- Railway: resource usage in sidebar footer
- Datadog: alert count in header
- macOS: menu bar extras

**How it works:**
- Fixed-position strip (usually bottom or top)
- Shows 3-8 key metrics or status indicators
- Click to drill into details
- Updates in real-time
- Color-coded: green/yellow/red for status

**Why it works:**
- Zero-cost information — always visible without interaction
- Ambient awareness of system health
- Quick problem detection: "why is cost yellow?"
- Reduces need for monitoring dashboards

**When to use:** When there are 3-8 key metrics the user should always know. When real-time status matters.

**Relevance to RUSVEL:** A bottom status bar or top bar area showing:
- Active session name
- Current model + effort level
- Running cost (today/session)
- Pending approvals count
- Active jobs/tasks count
- Connection status (Ollama/Claude API)

This gives the solo founder ambient awareness without checking the dashboard.

---

### Pattern 5F: Keyboard Shortcuts Overlay

**What it is:** A discoverable overlay showing all available keyboard shortcuts for the current context.

**Examples:**
- Gmail: press `?` to see all shortcuts
- GitHub: press `?` to see all shortcuts
- VS Code: Cmd+K Cmd+S for keybinding editor
- Slack: Cmd+/ for shortcuts
- Figma: Ctrl+Shift+? for shortcuts

**How it works:**
- Single keypress (usually `?` or `Cmd+/`) shows a modal
- Shortcuts grouped by category
- Context-aware: shows relevant shortcuts for current page
- Some apps show hints inline (tooltip on hover)

**Why it works:**
- Self-documenting — users learn shortcuts progressively
- No memorization required — just check when needed
- Builds toward keyboard-first usage over time

**When to use:** Whenever the app has more than 10 keyboard shortcuts.

**Relevance to RUSVEL:** Add a `?` shortcut that shows context-aware keybindings. In a department: show department-specific shortcuts. In chat: show chat shortcuts.

---

### Recommendation for RUSVEL

Implement a power-user toolkit:

| Pattern | Priority | Effort |
|---------|----------|--------|
| Command palette enhancement (5C) | High | Medium — extend existing |
| Status bar (5E) | High | Low — new component |
| Density setting (5A) | Medium | Medium — CSS variable toggle |
| Table view for CRUD (5B) | Medium | Medium — new view per section |
| Inspector panel (5D) | Low | Medium — slide-over component |
| Shortcuts overlay (5F) | Low | Low — modal with shortcut map |

---

## 6. Synthesis: Practical Patterns for RUSVEL's 13-Department Workspace

### The Core Problem

RUSVEL needs to manage 13 departments, each with 11+ sections, across three modes (configure, execute, monitor), for a power user who uses it daily. The UI must handle:
- 13 top-level contexts (departments)
- ~143 section pages (13 departments x 11 sections)
- 7 cross-cutting pages (Dashboard, Chat, Approvals, DB, Flows, Terminal, Knowledge)
- Real-time information (chat streaming, events, job status)
- Both configuration (forms) and execution (chat, actions) in every department

### The Pattern Stack

**Layer 1 — Navigation Structure:**
Choose between:
- **Option A (Vertical Rail):** 48px icon rail for all 20 items (7 utilities + 13 departments), section sidebar within departments. More scalable, never overflows.
- **Option B (Horizontal Bars):** Top bar for 7 utilities, second bar for 13 department pills, section sidebar within departments. Current proposal. Works if overflow is handled.

**Layer 2 — Department Interior:**
- Section sidebar (200px) listing all 11 sections per department
- Main content area consuming remaining width
- Sections are URL-addressable: `/dept/[id]/section`
- Chat is a section, not a persistent panel

**Layer 3 — Power User Acceleration:**
- Cmd+K command palette with department-aware actions and fuzzy navigation
- Cmd+1-5 for pinned departments
- Status bar with ambient metrics
- `?` shortcut overlay
- Density toggle (compact/comfortable)

**Layer 4 — Information Architecture:**
- Dashboard at `/` with department status cards (Pattern 3B)
- Per-department event timelines (Pattern 3C)
- Cross-department activity feed on dashboard
- Badge indicators on department icons for pending items

**Layer 5 — Interaction Modes:**
- Chat-as-Section for department conversations (Pattern 1D)
- Chat-as-Page for God Agent (Pattern 1A)
- Quick Ask via Cmd+K for transient queries (Pattern 1C)
- Split view for interactive sections: skills testing, agent testing (Pattern 3D)

### Key Design Decisions

| Decision | Recommendation | Rationale |
|----------|---------------|-----------|
| Chat placement | Section, not persistent panel | 13 departments x 11 sections = too much to share space |
| Department nav | Horizontal bar (current proposal) OR vertical rail | Both work; rail scales better but proposal is already designed |
| Section nav | Vertical sidebar within department | Standard, scalable, URL-friendly |
| Mode switching | Sections serve as implicit modes | No explicit mode toggle needed — navigate to configure sections, execute sections, or monitor sections |
| Power user features | Command palette + keyboard shortcuts + status bar | Solo founder needs speed above all |
| Information density | Compact default + density toggle | Power user preference |
| CRUD lists | Card view (default) + table view toggle | Cards for overview, table for bulk ops |

### What the Two-Level Nav Proposal Gets Right

1. Chat as a section, not a persistent panel — correct for a multi-department workspace
2. URL-addressable sections — essential for navigation, bookmarking, browser history
3. Full-width content areas — 288px panels were too cramped
4. Zero backend changes — pure frontend restructuring
5. Component reuse — existing tab components become section pages
6. Progressive delivery — phased rollout reduces risk

### What to Consider Adding to the Proposal

1. **Department grouping** — Visual separators or category pills for the 13-item department bar
2. **Pinned departments** — Let the user pin 4-5 most-used departments for faster access
3. **Status bar** — Bottom strip showing session, model, cost, approvals, jobs
4. **Keyboard shortcuts** — Cmd+1-5 for pinned departments, `?` for shortcut overlay
5. **Density setting** — Compact mode for the power-user daily driver scenario
6. **Badge indicators** — Unread/pending counts on department pills/icons
7. **Dashboard department cards** — Quick status per department on the home page
8. **Inspector slide-over** — For viewing item details without leaving a list page

---

## Appendix: Reference App Pattern Summary

| App | Nav Pattern | Chat Pattern | Mode Pattern | Density |
|-----|------------|-------------|-------------|---------|
| ChatGPT | Sidebar (history) | Chat-as-Page | N/A | Single |
| Claude.ai | Sidebar (history) | Chat-as-Page + Artifacts | N/A | Single |
| Cursor | Activity bar + sidebar | Chat-as-Sidebar (Cmd+L) | File explorer, search, git | Configurable |
| VS Code | Activity bar + sidebar | N/A | Activity modes | Configurable |
| Slack | Workspace rail + channel sidebar | Chat-as-Page | Channels, threads, apps | Compact |
| Discord | Server rail + channel sidebar | Chat-as-Page | Channels, threads, forums | Compact |
| Figma | Team tabs + project list | N/A | Design, prototype, inspect | Single |
| Vercel | Project dropdown + tab bar | N/A | Deploy, monitor, configure tabs | Single |
| Supabase | Service sidebar | N/A | Service sub-nav | Single |
| Linear | Team pills + sidebar | N/A | Views (list, board, timeline) | Compact toggle |
| Notion | Workspace dropdown + page tree | Inline AI | Page types | Compact toggle |
| v0.dev | Minimal | Chat + Preview split | Chat generates, preview renders | Single |
| Bolt.new | Minimal | Chat + Full IDE split | Chat generates, IDE executes | Single |

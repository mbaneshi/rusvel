# UI Layout Pattern Analysis: 5 Reference Apps

> Research for RUSVEL's 13-department workspace UI design.
> Covers navigation hierarchy, workspace switching, config vs execution balance,
> sidebar responsibilities, and main content area patterns.

---

## 1. GitHub

### Navigation Hierarchy: 4 levels

```
Org/User -> Repo -> Tab -> Content
```

GitHub uses a **horizontal top bar** for the first two levels and a **horizontal tab bar** for the third.

### ASCII Layout

```
+------------------------------------------------------------------+
| [GH logo] [Search............] [+v] [notif] [avatar]            |  <- Global bar
+------------------------------------------------------------------+
| [avatar] user/repo-name  [Watch] [Fork] [Star]                  |  <- Repo header
+------------------------------------------------------------------+
| Code | Issues | Pull Requests | Actions | Projects | Settings    |  <- Tab bar (horiz)
+------------------------------------------------------------------+
|                    |                                              |
|  File tree         |   README.md / file content / diff view      |  <- Main split
|  (collapsible)     |                                              |
|  src/              |   --- rendered markdown ---                  |
|    lib/            |                                              |
|    routes/         |                                              |
|                    |                                              |
+--------------------+----------------------------------------------+
|                    |  [About]         |  <- Right sidebar         |
|                    |  [Releases]      |     (repo metadata)       |
|                    |  [Languages bar] |                           |
+--------------------+------------------+---------------------------+
```

### Key UI Decisions

**Workspace switching (repos):** No sidebar-based repo list. GitHub relies on:
- The search bar (Cmd+K) as the primary repo switcher
- Recent repos in a dropdown from the avatar/hamburger
- The URL itself as context (user/repo)

This is critical: GitHub does NOT try to show all repos in a sidebar. With millions of repos per user, they treat search as the primary navigation method.

**Horizontal tabs for modes:** Code, Issues, PRs, Actions, Settings are all peer-level tabs. This works because each tab is a distinct "mode" of operating on the same repo. You are always in exactly one tab.

**Left sidebar (file tree):** Only appears inside the Code tab. It is collapsible and context-specific to file browsing. It does NOT persist across tabs.

**Right sidebar:** Contextual metadata. On repo home: About, Releases, Contributors, Languages. On Issues: Labels, Milestones, Assignees. On PRs: Reviewers, Checks. Always read-only summary information -- never interactive forms.

**Main scrolling area:** The entire center column scrolls. Content varies by tab: file viewer, issue list, PR diff, action logs. GitHub keeps the tab bar and repo header pinned; everything below scrolls.

### Pick -> Configure -> Execute -> Monitor Pattern

- **Pick:** Search bar / URL / recent repos dropdown
- **Configure:** Settings tab (rarely visited), plus inline config (labels, milestones, branch rules)
- **Execute:** Creating issues, opening PRs, merging -- all inline in the main content area
- **Monitor:** Actions tab for CI, Insights tab for analytics, notification bell for updates

### Takeaway for RUSVEL

GitHub's key insight: when you have many workspaces (repos), do NOT list them all in a sidebar. Use search/command palette as the primary switcher, and dedicate the sidebar to within-workspace navigation only.

---

## 2. Linear

### Navigation Hierarchy: 3 levels

```
Team/Workspace -> View/Category -> Issue
```

Linear uses a **persistent left sidebar** for the first two levels and the **main area** for the third.

### ASCII Layout

```
+--12-14%--+-------------------------70-75%-------------------+
|           |                                                   |
| [Linear]  |  [Breadcrumb: Team > Active Issues]   [filter]  |
| [Search]  |  +---------------------------------------------+|
|           |  | Issue Title            Status   Priority  Due ||
| My Issues |  | Build auth module      In Prog  Urgent   3/28||
| Inbox     |  | Fix sidebar collapse   Todo     High     3/30||
| --------- |  | Design review flow     In Prog  Medium   4/02||
| Team A    |  | Update API docs        Backlog  Low      --- ||
|  Active   |  |                                               ||
|  Backlog  |  +---------------------------------------------+|
|  -------  |                                                   |
|  Views:   |                                                   |
|   Board   |  [View toggles: List | Board | Timeline]         |
|   All     |                                                   |
|  Cycles   |                                                   |
|  Projects |                                                   |
| --------- |                                                   |
| Team B    |                                                   |
|  Active   |                                                   |
| --------- |                                                   |
| Settings  |                                                   |
+-----------+---------------------------------------------------+
```

**Issue detail (slide-over or full page):**

```
+-----------+-----------------------------+--------------------+
|  Sidebar  |  Issue: Build auth module   | Status: In Prog   |
|  (same)   |                             | Priority: Urgent   |
|           |  Description text...        | Assignee: @mehdi   |
|           |  --- comments ---           | Labels: backend    |
|           |  --- activity ---           | Project: Auth      |
|           |                             | Cycle: Sprint 12   |
+-----------+-----------------------------+--------------------+
              ^ Main content (scrolls)      ^ Right sidebar
                                             (properties panel)
```

### Key UI Decisions

**Workspace/team switching:** A small dropdown at the top of the left sidebar switches between teams/workspaces. Teams are listed as collapsible sections in the sidebar itself. This works because Linear targets teams of 5-50 people with 2-10 teams -- a manageable number for a sidebar list.

**View types as toggles, not navigation:** List, Board, and Timeline are view-type toggles within the same data set, shown as segmented controls in the main content header. They do NOT each get their own sidebar entry. This keeps the sidebar focused on WHAT you are looking at (which issues) rather than HOW you view them.

**Left sidebar responsibilities:**
1. Personal views (My Issues, Inbox)
2. Team sections (expandable, each with Active/Backlog/custom views)
3. Cross-cutting: Cycles, Projects, Views (saved filters)
4. Settings (bottom)

**Right sidebar (issue detail only):** Properties panel -- Status, Priority, Assignee, Labels, Project, Cycle, Estimate. This is the "configure" zone for individual items.

**Main scrolling area:** Issue list (with view-type variants) or issue detail. Clean, minimal chrome. The list is dense -- Linear gets ~15-20 issues visible at once.

### Pick -> Configure -> Execute -> Monitor Pattern

- **Pick:** Sidebar nav (team > filter/view) + Cmd+K search
- **Configure:** Workspace Settings page + right sidebar on issue detail
- **Execute:** Inline status changes, drag-and-drop on board, keyboard shortcuts
- **Monitor:** Inbox (notifications), Activity feed on issues, Cycles (progress tracking)

### Takeaway for RUSVEL

Linear's key insight: collapsible team/workspace sections in the sidebar scale to ~10-15 sections. For RUSVEL's 13 departments, this model fits naturally. View-type toggles (list/board/timeline) belong in the main content header, NOT the sidebar.

---

## 3. Retool / Windmill

### Navigation Hierarchy: 3 levels

```
Workspace -> Resource Type -> Individual Resource
```

Retool and Windmill are internal tool builders. They face the "many apps in one workspace" problem directly.

### ASCII Layout -- Retool Home / App List

```
+------------------------------------------------------------------+
| [Retool logo]  [workspace-name v]  [Search]  [+Create]  [avatar]|  <- Top bar
+------------------------------------------------------------------+
| Apps | Workflows | Databases | Resources | Settings              |  <- Top tabs
+------------------------------------------------------------------+
|                                                                    |
|  [Search apps...]                [Grid | List]   [Sort: Recent]  |
|                                                                    |
|  +--Folder: Marketing--+  +--Folder: Sales--+  +--Folder: Ops--+|
|  | App 1               |  | App 4           |  | App 7          ||
|  | App 2               |  | App 5           |  | App 8          ||
|  | App 3               |  | App 6           |  |                ||
|  +---------------------+  +-----------------+  +----------------+|
|                                                                    |
+------------------------------------------------------------------+
```

### ASCII Layout -- Retool App Builder

```
+------------------------------------------------------------------+
| [<- Back to apps]  App Name  [Preview] [Share] [Deploy]          |  <- Builder bar
+------------------------------------------------------------------+
|  Component   |                                   |  Inspector     |
|  Palette     |     CANVAS                        |  (right)       |
|              |                                   |                |
|  [Table]     |  +--Table1-----------------+      |  Table1        |
|  [Button]    |  | col1 | col2 | col3     |      |  Data: {{q1}}  |
|  [Input]     |  | row  | row  | row      |      |  Columns: ...  |
|  [Chart]     |  +-------------------------+      |  Events: ...   |
|  [Form]      |                                   |  Style: ...    |
|  [Text]      |  [Button1]  [Input1]              |                |
|  ...         |                                   |                |
+--------------+-----------------------------------+----------------+
|  Queries: [query1] [query2] [+ New Query]                        |  <- Bottom panel
|  SELECT * FROM users WHERE id = {{Input1.value}}                 |
+------------------------------------------------------------------+
```

### Key UI Decisions

**Workspace switching:** Top-left dropdown. Workspaces are the outermost container -- you rarely switch between them. Most users live in one workspace.

**Top-level tab bar for resource types:** Apps, Workflows, Databases, Resources (connections), Settings. This is the key organizational pattern: different TYPES of things get top tabs, not different instances. You pick the type first, then browse/search within.

**Folder-based organization for many items:** Inside "Apps," Retool uses folders + search + sort. This scales to hundreds of apps. No sidebar listing every app -- that would be unmanageable.

**Three-panel builder layout (left palette | canvas | right inspector):** This is the standard builder pattern used by Figma, Webflow, Xcode, and Unity. The left panel provides what you CAN add, the center is WHERE you work, the right panel is HOW the selected item is configured.

**Bottom panel for data/queries:** Queries, transformers, and code live in a collapsible bottom panel. This separates "visual building" (center) from "data wiring" (bottom).

**Windmill's variation:** Windmill uses a similar structure but with a file-tree left sidebar (scripts, flows, resources, schedules, variables). Their flow editor is a vertical DAG canvas similar to n8n.

### Pick -> Configure -> Execute -> Monitor Pattern

- **Pick:** Top tabs for type, then folder browse or search for instance
- **Configure:** Right inspector panel (per-component), bottom query panel (data), Resources tab (connections)
- **Execute:** Preview mode, Deploy button, Run button on workflows
- **Monitor:** Workflow runs list, audit logs, usage analytics (separate pages)

### Takeaway for RUSVEL

Retool's key insight: separate the BROWSING experience (flat list + folders + search) from the BUILDING experience (three-panel builder). The transition from list to builder is a full context switch -- a new page layout appears. RUSVEL could use this pattern: department list -> department workspace (different layout).

---

## 4. n8n / Langflow

### Navigation Hierarchy: 2-3 levels

```
Workspace -> Workflow List -> Workflow Editor (canvas)
```

### ASCII Layout -- n8n Home

```
+--56px--+----------------------------------------------------------+
|        |  Workflows   [Search...]   [+ Create Workflow]           |
|  [n8n] |  ---------------------------------------------------------+
|  Home  |  Name              | Tags     | Status  | Last run      |
|  Work  |  Lead Enrichment   | sales    | Active  | 2 min ago     |
|  Cred  |  Daily Digest      | content  | Active  | 1 hr ago      |
|  Var   |  Slack -> Notion    | ops      | Error   | 3 hr ago      |
|  Exec  |  Email Classifier   | ai       | Inactive| yesterday     |
|  ----  |                                                           |
|  Set   |                                                           |
+--------+-----------------------------------------------------------+
```

### ASCII Layout -- n8n Workflow Editor

```
+------------------------------------------------------------------+
| [<-] Workflow: Lead Enrichment   [Save] [Toggle Active] [Execute]|
+------------------------------------------------------------------+
|  Nodes  |                                        |  Node Config  |
|  Panel   |     CANVAS (infinite pan/zoom)        |  (right)      |
|  (slide) |                                       |               |
|  [Search]|   [Webhook] --> [HTTP] --> [IF]       |  HTTP Request |
|           |                   |         |         |  Method: POST |
|  Trigger |               [Set]    [Slack]        |  URL: {{...}} |
|  Action  |                                       |  Headers: ... |
|  AI      |                                       |  Body: ...    |
|  Flow    |         [+] (add node)                |  Auth: cred1  |
|           |                                       |               |
+-----------+---------------------------------------+---------------+
|  Execution log: Run #42 - Success - 6 nodes - 1.2s              |
|  [Node outputs]  Webhook: {data...}  HTTP: {response...}        |
+------------------------------------------------------------------+
```

### Key UI Decisions

**Minimal left sidebar (icon rail):** n8n uses a narrow ~56px icon sidebar for top-level sections: Home, Workflows, Credentials, Variables, Executions, Settings. This is just 6 items -- no nesting, no expansion. It stays out of the way.

**Workflow list is the home screen:** The primary view when you open n8n is a sortable/filterable table of workflows. Tags provide cross-cutting organization. No folders -- just tags + search.

**Canvas is king:** The workflow editor takes over the entire main area. The node palette slides in from the left (triggered by clicking [+] on the canvas). The right panel appears when you click a node. Both panels overlay/push the canvas -- they are not always visible.

**Right panel for node configuration:** When you select a node, a right panel opens with its full configuration: parameters, credentials, expressions, input/output data from the last run. This is the MOST important panel -- where all the real work happens.

**Bottom panel for execution output:** After running a workflow, execution results appear in a bottom panel showing each node's input/output data. You can click through nodes to inspect their data.

**Langflow's variation:** Langflow (AI workflow builder) uses a very similar canvas pattern but with AI-specific node types (LLMs, prompts, chains, vector stores). Their left panel is a categorized component library. Right panel shows component config. They add a "Playground" button that opens a chat interface to test the flow -- this is a key difference for AI tools.

**Credentials as a separate top-level section:** Both n8n and Langflow recognize that credentials (API keys, OAuth tokens) are cross-cutting resources that multiple workflows share. They get their own page, not buried in workflow settings.

### Pick -> Configure -> Execute -> Monitor Pattern

- **Pick:** Workflow list (home screen) with search/filter/tags
- **Configure:** Canvas for structure, right panel for node config, Credentials page for auth
- **Execute:** "Execute Workflow" button (top bar), or Webhook trigger (automatic)
- **Monitor:** Executions page (run history, logs, errors), bottom panel in editor (last run data)

### Takeaway for RUSVEL

n8n's key insight: the icon rail sidebar (6-8 items) plus a canvas-dominant editor is the right pattern for workflow/flow tools. Configuration happens in a right panel that appears on selection. Execution monitoring is a bottom panel or separate section. For RUSVEL's flow builder, this is the reference pattern.

---

## 5. Cursor / Claude Code (Web)

### Navigation Hierarchy: 2-3 levels

```
Project -> File/Chat -> Content
```

### ASCII Layout -- Cursor

```
+--48px--+--200px----+---------------------------+--350px---------+
| [files]|  Explorer  |  main.ts          x      |  AI Chat       |
| [srch] |  src/      |  ----------------------- |                |
| [git]  |   lib/     |  1  import { app } from  |  You: Fix the  |
| [ext]  |    api.ts  |  2  import { db } from   |  auth bug in   |
| [AI]   |    auth.ts |  3                        |  api.ts        |
|        |   routes/  |  4  const server = app()  |                |
|        |    +page   |  5  server.listen(3000)   |  Cursor: I'll  |
|        |            |  6                        |  fix the auth  |
|        |  node_mod/ |  7  // TODO: add auth     |  check...      |
|        |            |  8                        |                |
|        |            |                           |  [Apply] [Copy]|
|        |            |                           |                |
+--------+------------+---------------------------+                |
| [AI]   |  TERMINAL                              |  Tool calls:   |
| [term] |  $ pnpm dev                            |  - read api.ts |
| [debug]|  Server running on :3000               |  - edit auth.ts|
| [ports]|  $                                     |  - run tests   |
+--------+-------------------------------------------+-------------+
```

### ASCII Layout -- Claude Code Web (claude.ai with artifacts/tools)

```
+------------------------------------------------------------------+
| [Claude]  [New chat v]   [model: opus]   [avatar]                |
+------------------------------------------------------------------+
| Sidebar   |                                                       |
| (chats)   |     CONVERSATION                                     |
|           |                                                       |
| Today     |     You: Analyze my codebase and fix the auth bug   |
|  Chat 1   |                                                       |
|  Chat 2   |     Claude: I'll investigate the auth module.        |
| Yesterday |     [Tool: read_file("src/auth.ts")]                 |
|  Chat 3   |     [Tool: grep("validateToken")]                    |
|  Chat 4   |     [Tool: edit_file("src/auth.ts", ...)]            |
| Last week |                                                       |
|  Chat 5   |     I found and fixed the issue. The token           |
|           |     validation was not checking expiry...             |
|           |                                                       |
|           |     +--Artifact: auth.ts (updated)--+                |
|           |     | export function validate(t) {  |                |
|           |     |   if (isExpired(t)) return false|               |
|           |     +--------------------------------+                |
|           |                                                       |
|           |  [Type a message...                        ] [Send]  |
+-----------+-------------------------------------------------------+
```

### Key UI Decisions

**Cursor: VS Code layout with AI panel bolted on.** Cursor inherits VS Code's iconic four-zone layout:
1. **Activity bar** (leftmost icon rail, ~48px): switches between Explorer, Search, Git, Extensions, AI
2. **Primary sidebar** (left, ~200px): file tree, search results, git changes -- context for the active activity
3. **Editor area** (center, flexible): tabs, split editors, the actual code
4. **Panel** (bottom, collapsible): terminal, problems, output, debug console

Cursor adds a **secondary sidebar** (right, ~350px) for AI chat. This is the key innovation: the AI chat is a persistent right panel that coexists with the editor. You can see code AND chat simultaneously.

**Tab-based multi-file editing:** The editor area uses tabs for multiple open files. Each tab is a file. You can split the editor vertically or horizontally for side-by-side editing.

**Inline AI vs Panel AI:** Cursor offers BOTH:
- Cmd+K for inline AI (edit a selection in place)
- Right panel for conversational AI (multi-turn, tool use)
- Cmd+I for "Composer" (multi-file agent mode, takes over more of the screen)

This duality is important: quick edits happen inline, complex tasks happen in the panel.

**Claude Code Web:** Much simpler layout. Left sidebar is chat history (chronological). Main area is the conversation with inline tool calls and artifacts. No file tree -- Claude navigates the codebase through tool calls displayed in the conversation. The conversation IS the workspace.

**Tool output inline in conversation:** Both Cursor and Claude Code show tool invocations (file reads, edits, terminal commands) as collapsible cards within the chat flow. This maintains context -- you can see what the AI did and why.

### Pick -> Configure -> Execute -> Monitor Pattern

- **Pick:** Cursor: Open folder/workspace. Claude Code: create/select a chat (with project context)
- **Configure:** Cursor: Settings, .cursorrules file, model selection. Claude Code: CLAUDE.md, model picker, MCP tools
- **Execute:** Chat input (natural language) -> AI uses tools -> applies changes
- **Monitor:** Terminal output, diff view (see changes), tool call cards in chat, git diff

### Takeaway for RUSVEL

Cursor's key insight: AI chat works best as a persistent RIGHT sidebar alongside the main working area. It does NOT replace the main content -- it augments it. The user needs to see both the work product (code, config, data) AND the AI conversation simultaneously.

---

## Cross-Cutting Analysis

### Pattern Comparison Table

| Aspect | GitHub | Linear | Retool | n8n | Cursor |
|--------|--------|--------|--------|-----|--------|
| Workspace count | 1000s (repos) | 2-10 (teams) | 1-5 | 1-3 | 1-5 (projects) |
| Workspace switcher | Cmd+K search | Sidebar dropdown | Top-left dropdown | N/A (single) | File > Open |
| Left sidebar | File tree (code tab only) | Nav + team sections | Component palette (builder) | Icon rail (6 items) | Activity bar + file tree |
| Right sidebar | Metadata (read-only) | Properties (editable) | Inspector (editable) | Node config (editable) | AI Chat |
| Main area | Content (varies by tab) | Issue list/detail | Canvas (builder) | Canvas (workflow) | Code editor |
| Tab/mode switching | Horizontal tabs (7) | View toggles (3) | Top tabs (5) | Icon sidebar (6) | Activity bar icons |
| Config location | Settings tab | Settings page + right panel | Right inspector + bottom panel | Right panel + Credentials page | Settings + config files |
| Bottom panel | N/A | N/A | Query editor | Execution log | Terminal |

### The Three Sidebar Archetypes

**1. Icon Rail (n8n, Cursor activity bar, VS Code)**
- Width: 40-56px
- Contains: 5-8 icons representing top-level modes
- Best for: apps with few, distinct modes
- Scales to: ~10 items max before it becomes confusing

**2. Navigational Sidebar (Linear, GitHub's file tree)**
- Width: 200-280px
- Contains: hierarchical nav items, collapsible sections, search
- Best for: apps with structured hierarchies (teams > views, folders > files)
- Scales to: ~20-30 visible items with collapsible groups

**3. Contextual Panel (Retool component palette, n8n node palette)**
- Width: 200-300px
- Contains: items relevant to the current editing mode
- Best for: builder/editor interfaces
- Often: slides in/out, is NOT always visible

### Right Sidebar Usage Pattern

Every app uses the right sidebar for the SAME thing: **properties/configuration of the selected item.** This is remarkably consistent:
- GitHub: issue labels, PR reviewers, repo metadata
- Linear: issue status, priority, assignee, labels
- Retool: component properties, data bindings, style
- n8n: node parameters, credentials, expressions
- Cursor: AI chat (the "selected item" is your current problem)

The right sidebar is ALWAYS:
- Contextual (changes based on what is selected)
- Editable (not just read-only)
- Collapsible (can be hidden when not needed)
- 250-400px wide

### How They Handle 10+ Workspaces/Projects

| Strategy | Used By | When It Works |
|----------|---------|---------------|
| Search/Cmd+K as primary switcher | GitHub | 100+ workspaces |
| Sidebar sections (collapsible) | Linear | 5-15 workspaces |
| Top dropdown + folder grid | Retool | 10-100 workspaces |
| Tags + flat list | n8n | 10-50 workflows |
| Recent list + search | Cursor/Claude | 5-20 projects |

For RUSVEL's 13 departments: the **Linear model** (sidebar sections) is the closest fit. Thirteen departments is within the range where a sidebar with collapsible sections works without becoming overwhelming. But you should ALSO have Cmd+K search as a power-user shortcut.

---

## Recommendations for RUSVEL's 13-Department UI

### Proposed Layout

```
+--48px--+--220px---------+--flexible (50-70%)---+--320px---------+
| ICON   | DEPT SIDEBAR    | MAIN CONTENT         | CONTEXT PANEL  |
| RAIL   |                 |                       |                |
| [Home] | Session: [...v] | [Breadcrumb + actions]| [AI Chat]     |
| [Chat] | ______________ |                       | or             |
| [Flow] |                 | Content varies:       | [Properties]   |
| [DB]   | DEPARTMENTS     | - Dashboard cards     | or             |
| [Term] | > Forge    (3)  | - Item list/table     | [Node Config]  |
|        | > Code     (1)  | - Chat conversation   | or             |
|        | > Harvest  (5)  | - Flow canvas         | [Exec Output]  |
|        |   Content       | - Database viewer     |                |
|        |   GTM           | - Settings forms      |                |
|        |   Finance       |                       |                |
|        |   Product       |                       |                |
|        |   Growth        |                       |                |
|        |   Distro        |                       |                |
|        |   Legal         |                       |                |
|        |   Support       |                       |                |
|        |   Infra         |                       |                |
|        | ______________ |                       |                |
|        | [Settings]      |                       |                |
|        | [API: :3000]    |                       |                |
+--------+-----------------+-----------------------+----------------+
|                           BOTTOM PANEL (collapsible)              |
|  [Terminal] [Executions] [Logs]                                   |
+------------------------------------------------------------------+
```

### Key Design Decisions

**1. Dual-sidebar: Icon Rail (left) + Department Nav (adjacent)**

Combine n8n's icon rail with Linear's navigational sidebar. The icon rail provides 5-6 global modes (Home/Dashboard, Chat, Flows, Database, Terminal). The department sidebar lists all 13 departments with collapsible sections. This gives you two levels of navigation without horizontal tabs.

Current RUSVEL layout mixes global pages (Chat, Dashboard, Database, Flows, Terminal) with departments in a single flat list. Separating them into icon rail (global) vs sidebar body (departments) creates clearer hierarchy.

**2. Right panel for AI Chat (Cursor model)**

The AI chat should be a persistent, collapsible right panel -- not a separate page. This lets users chat with department agents WHILE viewing department data. The current `/chat` page forces a full context switch away from department content.

When not chatting, the right panel can show:
- Item properties (Linear model) when an item is selected
- Node configuration (n8n model) when editing a flow
- Execution output when monitoring a run

**3. View-type toggles in the main content header (Linear model)**

Within a department, offer view-type toggles (List, Board, Timeline, Chat) as a segmented control in the main content header. Do NOT add these as separate sidebar entries. The sidebar says WHERE (which department), the header says HOW (which view).

**4. Bottom panel for terminal/execution (Cursor model)**

Terminal, execution logs, and event streams belong in a collapsible bottom panel, not on separate pages. This keeps them accessible from any context without losing the main view.

**5. Session switcher stays in sidebar (current design is good)**

The session concept maps well to Retool's workspace switcher. Keep it at the top of the sidebar. Consider making it a smaller dropdown rather than a full section.

**6. Cmd+K command palette for power users (already implemented)**

This is correct. Every app in this analysis uses a command palette as the fast-path for navigation. Keep it.

### The "Pick -> Configure -> Execute -> Monitor" Flow for RUSVEL

```
PICK        -> Sidebar: select department (or Cmd+K to jump)
CONFIGURE   -> Right panel: department settings, agent config, rules
EXECUTE     -> Right panel: AI chat with department agent
              Main area: view results (list, board, canvas)
MONITOR     -> Bottom panel: execution logs, event stream
              Main area: dashboard cards with status
              Sidebar: badge counts per department
```

### What to Change from Current Layout

| Current | Proposed | Rationale |
|---------|----------|-----------|
| Flat sidebar (global + depts mixed) | Icon rail (global) + dept sections (departments) | Clearer hierarchy, reduces visual clutter |
| Chat as separate page | Chat as right sidebar panel | Lets user see dept data + chat simultaneously |
| Terminal as separate page | Terminal as bottom panel | Available from any context |
| No right sidebar | Add collapsible right panel | Properties, AI chat, node config |
| No bottom panel | Add collapsible bottom panel | Terminal, execution logs, events |
| Single scrolling main area | Main area with optional bottom split | Matches IDE/builder mental model |

### Progressive Disclosure for 13 Departments

Thirteen sidebar items is borderline -- it works on a large screen but gets cramped on a laptop. Consider:

1. **Favorites/pinned departments** at the top (3-5 most used)
2. **"More departments"** collapsible section for the rest
3. **Badge counts** (unread/pending items) to draw attention to active departments
4. **Collapsed mode**: icon rail only shows global items + pinned departments
5. **Cmd+K** as the fast path for infrequently used departments

This mirrors how Slack handles many channels: starred channels at top, then organized sections, with Cmd+K for jumping.

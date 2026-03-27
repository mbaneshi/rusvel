# RUSVEL UI/UX Design — Information Architecture & Layout

**Date:** 2026-03-27
**In response to:** Prompt 2 (Gemini — Product Design & Information Architecture)

---

## 1. Navigation Hierarchy

### Three levels, each with a clear mechanism:

```
Level 1: DEPARTMENT        → Icon rail (left, persistent)
Level 2: SECTION           → Sidebar nav (left, contextual to department)
Level 3: ACTION            → Main content area (create, list, edit, run)
```

**Level 1 is persistent.** The user can always see and switch departments. This is critical — the solo founder context-switches between departments 10-20 times per day.

**Level 2 is contextual.** It changes when the department changes. Forge shows Actions, Engine, Agents, Skills, Chat, etc. Code shows the same set but with Code-specific engine tools. The section list comes from the department manifest (`tabsFromDepartment()`).

**Level 3 is content.** It fills the main area. A Skills section shows: header with title + "+ New" button, a grid/list of skill cards, create/edit forms. A Chat section shows the full department chat with streaming, tool calls, and artifacts.

### Why not horizontal tabs for departments?

13 departments in a horizontal tab bar creates a scrolling problem. At ~100px per tab, you need 1300px — barely fits on a 1440px screen, impossible on a laptop. The icon rail fits 13 items in ~700px vertical space with room to spare. Icons are also faster to scan than text labels.

### Why not a flat sidebar (current design)?

The current sidebar mixes 6 global pages + 13 departments + 1 settings = 20 items. The user scrolls past Chat, Approvals, Dashboard, Database, Flows, Terminal to reach departments. The icon rail separates global pages (top of rail) from departments (rail body), eliminating the scroll-to-find problem.

---

## 2. Layout Zones

```
+--48px--+--200px---------+--flexible (main)----------+--320px--------+
| ICON   | SECTION        |                            | CONTEXT       |
| RAIL   | SIDEBAR        |  MAIN CONTENT              | PANEL         |
|        |                |  (scrollable)              | (collapsible) |
| [Home] | ≡ Forge Dept   | ┌────────────────────────┐ |               |
| [Chat] |   forge        | │ Skills             + ▾ │ | AI Chat       |
| [Appv] | ──────────── │ │                        │ | or Properties |
| [DB  ] | ○ Actions      | │ ┌──────┐ ┌──────┐    │ | or Exec Log   |
| [Flow] | ○ Engine       | │ │deploy│ │review│    │ |               |
| [Term] | ● Skills       | │ │check │ │code  │    │ | ────────────  |
| ────── | ○ Rules        | │ └──────┘ └──────┘    │ | Quick Chat:   |
| [Forge]| ○ Agents       | │ ┌──────┐ ┌──────┐    │ | [Ask Forge..] |
| [Code] | ○ Workflows    | │ │daily │ │test  │    │ |               |
| [Cntnt]| ○ MCP          | │ │stndp │ │suite │    │ | Recent Events |
| [Harvt]| ○ Hooks        | │ └──────┘ └──────┘    │ | • goal.created|
| [Flow] | ○ Terminal      | │                        │ | • plan.gen'd  |
| [GTM]  | ○ Events       | │ + Create New Skill      │ | • review.done |
| [Fin]  | ──────────── │ └────────────────────────┘ |               |
| [Prod] | ○ Chat         |                            |               |
| [Grwth]| ○ Settings     |                            |               |
| [Dstr] | ──────────── |                            |               |
| [Legl] | Session: ▾     |                            |               |
| [Supp] | test-session   |                            |               |
| [Infra]|                |                            |               |
| ────── |                |                            |               |
| [⚙]   |                |                            |               |
+--------+----------------+----------------------------+---------------+
|          BOTTOM PANEL (collapsible)                                   |
|  [Terminal] [Executions] [Events]                                     |
|  $ pnpm dev                                                          |
|  Server running on :3000                                              |
+----------------------------------------------------------------------+
```

### Zone responsibilities:

| Zone | Width | Purpose | Persistent? |
|------|-------|---------|-------------|
| **Icon Rail** | 48px | Level 1 nav — global pages (top) + departments (body) + settings (bottom) | Always visible |
| **Section Sidebar** | 200px | Level 2 nav — sections within current department. Session switcher at bottom. | Visible on dept pages, hidden on global pages |
| **Main Content** | Flexible | Level 3 — full-width content for the selected section. Scrollable. | Always visible |
| **Context Panel** | 320px | AI chat OR item properties OR execution output. Collapsible. | Toggleable (Cmd+J or button) |
| **Bottom Panel** | ~200px | Terminal, execution logs, event stream. Collapsible. | Toggleable (Cmd+`) |

### Why a right context panel?

The research shows every major workspace app uses a right panel for the same thing: **contextual information about what you're looking at.** GitHub shows issue metadata. Linear shows task properties. Cursor shows AI chat. n8n shows node configuration.

For RUSVEL, the right panel serves three roles depending on context:

1. **AI Chat** (default) — Talk to the department agent while viewing structured content. The solo founder can ask "create a skill that does X" while looking at the skills list.
2. **Item Properties** — When editing an agent, skill, or rule, the right panel shows its properties. The main area shows the list; the right panel shows the selected item's detail.
3. **Execution Output** — When running a workflow or quick action, the right panel shows streaming results, tool calls, and approval cards.

### Where does chat live?

**Hybrid approach:**

- **Full-page chat:** `/dept/[id]/chat` — navigate to it for extended conversations. Gets the full main area width. This is the primary chat experience.
- **Quick chat in context panel:** The right panel has a compact chat input at the bottom. For quick questions without navigating away from Skills or Agents. Think "Hey Forge, what's the best model for this agent?" while editing an agent.
- **Global chat:** `/chat` — full-page God Agent chat for cross-department questions.
- **Command palette:** Cmd+K — quick commands, not conversational.

This layered approach means the user is never MORE than one action away from chatting with their department agent.

---

## 3. Mode Handling

The UI doesn't have explicit mode switches. Instead, different sections naturally support different modes:

### Configure Mode
**Where:** Agents, Skills, Rules, MCP, Hooks, Settings sections
**Layout behavior:**
- Main area shows a list/grid of items + create form
- Right panel shows selected item properties (or AI chat to help configure)
- Section sidebar is fully visible
- Bottom panel collapsed (not needed)

```
[Rail] [Sidebar] [  Agent List (grid)    ] [Selected Agent Config]
                  ┌──────┐ ┌──────┐
                  │Agent1│ │Agent2│
                  └──────┘ └──────┘
                  ┌──────┐
                  │+ New │
                  └──────┘
```

### Execute Mode
**Where:** Chat, Actions, Workflows sections
**Layout behavior:**
- Main area shows chat conversation OR workflow execution
- Right panel shows streaming tool calls and approval cards (when on chat), or workflow step output
- Section sidebar is visible but could collapse for more chat width
- Bottom panel may open for terminal output during execution

```
[Rail] [Sidebar] [ Chat Messages (streaming) ] [Tool Calls/Approvals]
                  User: Draft a blog about...
                  Agent: I'll draft that now...
                  [ToolCall: content_draft()]
                  [ToolResult: Draft created]
```

### Monitor Mode
**Where:** Events, Dashboard (home), Approvals
**Layout behavior:**
- Main area shows timeline, dashboard cards, or approval queue
- Right panel shows event detail or approval detail on selection
- Bottom panel may show live event stream
- Section sidebar visible for navigation

```
[Rail] [Sidebar] [ Event Timeline            ] [Event Detail JSON]
                  • mission.plan.generated 2m
                  • code.analyzed 5m
                  • content.draft.created 12m
```

The key insight: **mode is implicit in section choice**, not an explicit toggle. Clicking "Skills" puts you in configure mode. Clicking "Chat" puts you in execute mode. Clicking "Events" puts you in monitor mode. No mode buttons needed.

---

## 4. Department Identity

Each department has an icon, color (HSL), name, and personality. With 13 departments, this needs careful handling to avoid visual chaos.

### Design rules:

1. **Icon rail:** Each department shows its Lucide icon. The active department's icon gets a colored background pill (using the department's HSL color). Inactive icons are muted gray.

2. **Section sidebar header:** Shows the department icon + name + subtitle, with the department's accent color as a subtle left border or background tint. This is the only place the department name appears in full.

3. **Main content:** Department color is used SPARINGLY:
   - Section header titles use the department accent color
   - Active tab indicator uses the accent color
   - Card borders do NOT use department color (too noisy with 13 colors)
   - Buttons use the standard primary color, not department-specific

4. **Context panel:** Department color appears in the chat header only.

5. **Bottom panel:** No department color — it's a shared utility zone.

### The principle: **Color signals context, not decoration.**

When the user switches from Forge (emerald) to Content (purple), the icon rail highlight and sidebar header change color. This gives a clear "I'm in a different department now" signal without painting the entire UI purple.

---

## 5. User Flow Example

> "I want to set up a new content skill, test it in chat, then create a workflow that uses it."

### Step 1: Navigate to Content department
- Click the **Content icon** (✎) in the icon rail
- Section sidebar loads Content's sections: Actions, Engine, Skills, Rules, Agents, Chat, ...
- Main area shows the default section (Actions) with Content's quick actions

### Step 2: Go to Skills
- Click **Skills** in the section sidebar
- Main area shows Content's skills grid — existing skills as cards
- URL updates to `/dept/content/skills`

### Step 3: Create a new skill
- Click **+ Create New Skill** button in main area header
- A form appears (inline or modal): Name, Description, Prompt Template
- Fill in: name="thread-writer", description="Turn a blog into a Twitter thread", template="Convert this content into a Twitter thread of 5-8 tweets: {{input}}"
- Click Save — new skill card appears in the grid

### Step 4: Test in chat
- Click **Chat** in the section sidebar
- URL updates to `/dept/content/chat`
- Main area shows Content department chat (full width)
- Type: `/thread-writer Here is my latest blog post about RUSVEL architecture...`
- The skill template resolves, agent streams a response with the generated thread
- Tool calls visible inline: content_draft(), adapt() for Twitter format
- Result: a formatted Twitter thread appears in the chat

### Step 5: Create a workflow using the skill
- Click **Workflows** in the section sidebar
- URL updates to `/dept/content/workflows`
- Main area shows workflow list + create form
- Click **+ New Workflow**
- Define steps:
  1. Agent: "content-writer", Prompt: "Draft a blog from this topic: {{input}}"
  2. Agent: "content-writer", Prompt: "/thread-writer {{previous_output}}"
  3. Agent: "content-writer", Prompt: "Adapt this for LinkedIn: {{previous_output}}"
- Save workflow as "Blog → Thread → LinkedIn"
- Click **Run** — execution streams in the right context panel, showing each step's output

### Navigation trace:
```
[Content icon] → /dept/content/actions
  → Skills     → /dept/content/skills     (create skill)
  → Chat       → /dept/content/chat       (test skill)
  → Workflows  → /dept/content/workflows  (create + run workflow)
```

Total clicks: 5 (select dept, select Skills, create, select Chat, select Workflows). No scrolling, no collapsing panels, no guessing where things are.

---

## 6. Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Level 1 nav | Icon rail (48px) | Fits 13 departments + 6 global pages without scrolling |
| Level 2 nav | Section sidebar (200px) | Contextual to department, derived from manifest |
| Department switching | Icon rail click | Always visible, one click, badge indicators |
| Section switching | Sidebar click | URL-addressable, full history support |
| Chat placement | Hybrid: full-page section + quick chat in context panel | Full width when focused, accessible when not |
| Configure mode | Main area: list/grid + forms. Right panel: item properties | Natural from section choice |
| Execute mode | Main area: chat/workflow. Right panel: tool calls/approvals | Natural from section choice |
| Monitor mode | Main area: timeline/dashboard. Right panel: detail | Natural from section choice |
| Dept identity | Accent color on rail + sidebar header only | Context signal, not decoration |
| Right panel | AI chat OR properties OR exec output | Contextual, collapsible, 320px |
| Bottom panel | Terminal + execution logs | Collapsible, shared utility |

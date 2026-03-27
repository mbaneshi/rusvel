# RUSVEL UI/UX Architecture — Claude

> Architecture-aligned UI design. Copy everything below the line into Claude.

---

I'm the builder of RUSVEL, an AI-powered virtual agency for solo founders (Rust + SvelteKit). I need to design a UI that reflects the actual architecture, not fight against it. Here's the codebase truth:

## Architecture
Hexagonal (ports & adapters). 50 Rust crates, 52,560 lines of Rust, 10,623 lines of frontend.

- **rusvel-core**: 16 port traits + domain types + DepartmentApp trait
- **13 engines**: Each with real domain logic (code parsing, content generation, CRM, etc.)
- **13 dept-* crates**: Each implements DepartmentApp — contributes tools, event handlers, job handlers
- **DepartmentManifest**: Each department declares its identity, capabilities, tabs, quick_actions, UI contributions, events produced/consumed, config schema
- **API**: ~105 routes. Per-department: chat(SSE), conversations, config, events. CRUD: agents, skills, rules, workflows, hooks, mcp-servers. Engine-specific: code/analyze, content/draft, harvest/pipeline.
- **Chat handler**: 9-step pipeline — validate dept → load config → interceptors (!build, /skill, @agent) → load rules → inject capabilities → RAG search → build AgentConfig → stream via AgentRuntime → post-completion hooks
- **Agent runtime**: Tool-use loop with streaming (AgentEvent: TextDelta, ToolCall, ToolResult)
- **3 fully wired departments** (forge, code, content) — agents can invoke their engine tools
- **10 departments** with implemented engines but no agent tools wired yet

## Key Design Principles
1. Departments are self-contained apps (ADR-014). Each has its own manifest, tools, events, config.
2. Agents do the work — the founder decides what, agents figure out how.
3. Local-first, one person, not a team tool.
4. The system can extend itself — agents create capabilities via CRUD tools, capability cards appear in UI.
5. Rich observability — show what's happening (tool calls, reasoning, timing), not just final answers.

## UI Vision (not yet implemented)
- **AG-UI protocol**: 15 typed events replacing custom SSE, including STATE_DELTA with JSON Patch
- **A2UI**: Agents can generate declarative UI components (DataTable, DraftCard, MetricsGrid)
- **Capability catalog**: Unified grid showing all agents/skills/rules/workflows as browsable cards
- **Self-extension**: Agents create new capabilities → events fire → cards appear live in catalog

## Current Frontend (66 Svelte components)
- 11 department tab components (Actions, Engine, Agents, Skills, Rules, MCP, Hooks, Workflows, Dirs, Events + EngineTab)
- DepartmentChat with full streaming, tool call cards, approval cards
- DepartmentPanel with resizable tabbed sidebar
- Dashboard, Flows (xyflow imported but not wired), Database browser, Knowledge base, Terminal (xterm.js + WebSocket)
- Onboarding: CommandPalette, OnboardingChecklist, ProductTour

## The Question

Given this architecture, design a UI layout that:

1. **Reflects the DepartmentApp pattern** — each department should feel like its own app within the larger workspace, with its identity (color, icon, quick_actions) front and center.

2. **Supports the 3 user modes**:
   - Configure: set up agents, skills, rules, hooks, MCP servers
   - Execute: chat with department, run workflows, trigger quick actions
   - Monitor: view events, check approvals, review dashboards

3. **Prepares for A2UI**: When agents generate UI (DataTable, MetricsGrid, DraftCard), where does it render? How does agent-generated UI coexist with static CRUD pages?

4. **Handles the capability catalog**: If all agents, skills, rules, workflows, playbooks are "capability cards" in a unified view — where does that live? Per department? Global? Both?

5. **Solves chat placement**: Chat is the primary interaction mode, but users also need structured config pages. How should chat relate to other sections? Consider: chat-as-sidebar, chat-as-page, chat-as-overlay, chat-everywhere-inline.

6. **Scales from 3 wired departments to 13**: Right now only Forge, Code, Content have real tools. Soon all 13 will. The UI should handle both sparse departments (just chat + basic CRUD) and rich departments (engine tools + custom workflows + platform adapters).

Please propose:
- A layout diagram (ASCII is fine)
- Zone responsibilities (what goes where and why)
- Navigation flow for: "Switch from Content dept to Harvest dept, set up a new scoring rule, test it in chat, then check if the harvest pipeline found new leads"
- How agent-generated UI (A2UI components) would render in this layout
- How the capability catalog integrates

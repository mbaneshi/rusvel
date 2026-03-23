# UI Overhaul Plan — 2026-03-23

## Goal
Bring RUSVEL's frontend from functional prototype to production-grade AI agency dashboard.

## Library Stack

| Layer | Current | Target |
|---|---|---|
| UI primitives | Hand-rolled 15 components | shadcn-svelte + Bits UI |
| Markdown/streaming | marked | Streamdown (svelte-streamdown) |
| Charts | None | LayerChart (via shadcn-svelte) |
| Workflow builder | Form-based | Svelte Flow (@xyflow/svelte) |
| Command palette | None | cmdk-sv (via shadcn-svelte Command) |
| Toasts | None | Svelte French Toast |
| Animation | Minimal | svelte/motion (built-in) |
| Syntax highlighting | None | Shiki (via Streamdown) |

## Implementation Order

### Phase 1: Foundation — shadcn-svelte
- Install shadcn-svelte + bits-ui + clsx + tailwind-merge
- Initialize shadcn-svelte config
- Replace hand-rolled components with shadcn equivalents
- Remap design tokens to shadcn CSS variable conventions

### Phase 2: Chat — Streamdown
- Install svelte-streamdown
- Replace marked rendering in DepartmentChat
- Shiki syntax highlighting on code blocks
- Streaming-aware partial markdown handling

### Phase 3: Command Palette — cmdk-sv
- Install cmdk-sv (or use shadcn-svelte Command)
- Cmd+K global launcher
- Sections: departments, agents, workflows, conversations, quick actions
- Keyboard shortcut overlay (?)

### Phase 4: Toasts — Svelte French Toast
- Install svelte-french-toast
- Add toast feedback to all CRUD operations
- Notification bell + persistent feed for approvals/errors

### Phase 5: Dashboard — LayerChart
- Install layerchart
- KPI cards with sparklines per department
- LLM cost tracker (tokens/USD by department)
- Cross-department activity timeline

### Phase 6: Workflow Builder — Svelte Flow
- Install @xyflow/svelte
- Visual node editor for agent chains
- Node types: Agent, Tool, Approval Gate, Condition, Output
- Live execution overlay (status badges, edge animation)

### Phase 7: Polish
- Agent presence indicators on sidebar
- Split-screen artifacts panel
- Motion/animation pass (panel transitions, number ticking)
- Reasoning trace visualization

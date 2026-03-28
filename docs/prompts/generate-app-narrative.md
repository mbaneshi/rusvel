# Prompt: Generate RUSVEL Application Narrative

Use this prompt in Cursor (or any AI IDE) with the full codebase open.

## How to use it in Cursor

1. Open this file.
2. Select the prompt text inside the code fence below.
3. **Cmd+L** (or paste into chat) with something like: *apply this prompt to the codebase*.
4. Save the model output to `docs/design/how-it-works.md`.

## Why this prompt works

- **Dependency order** — core → adapters → surfaces so understanding builds incrementally.
- **Exact paths** — named files reduce guessing and hallucinated paths.
- **Fixed outline** — 11 sections track real control flow, not a generic template.
- **Narrative, not reference** — asking for a *story of data flow* yields clearer prose than “document the API.”
- **Concrete anchors** — requiring real type and function names avoids vague architecture hand-waving.

---

## The Prompt

```
Read the following files in order, then write a comprehensive narrative document
explaining how this application works — from boot to user interaction to shutdown.
Write it for a senior developer joining the project, not as API docs but as a
story of how data and control flow through the system.

## Files to read (in this order)

### 1. Entry point and composition
- crates/rusvel-app/src/main.rs        (composition root: what gets built, wired, started)
- crates/rusvel-app/src/boot.rs        (department registration lifecycle)

### 2. Core contracts
- crates/rusvel-core/src/ports.rs      (all port traits — the system's boundaries)
- crates/rusvel-core/src/domain.rs     (every domain type — the shared vocabulary)
- crates/rusvel-core/src/department/app.rs     (DepartmentApp trait)
- crates/rusvel-core/src/department/context.rs (registration context + artifacts)

### 3. How requests flow
- crates/rusvel-api/src/lib.rs         (router assembly, AppState, middleware stack, start_server)
- crates/rusvel-api/src/department.rs  (department chat SSE — the main user interaction)
- crates/rusvel-api/src/chat.rs        (god agent chat)
- crates/rusvel-api/src/sse_helpers.rs (shared SSE event mapping)

### 4. Agent runtime
- crates/rusvel-agent/src/lib.rs       (AgentRuntime, run_streaming, AgentEvent)
- crates/rusvel-tool/src/lib.rs        (ToolRegistry, ScopedToolRegistry)
- crates/rusvel-builtin-tools/src/file_ops.rs (example tool implementation)

### 5. One real engine end-to-end
- crates/forge-engine/src/lib.rs       (ForgeEngine — orchestration, mission, goals)
- crates/dept-forge/src/lib.rs         (how forge registers as a DepartmentApp)
- crates/dept-forge/src/manifest.rs    (manifest declaration)

### 6. Data layer
- crates/rusvel-db/src/store.rs        (SQLite implementation of all 5 stores)
- crates/rusvel-db/src/migrations.rs   (schema evolution)

### 7. Background work
- crates/rusvel-jobs/src/lib.rs        (job queue contract)
- search main.rs for "job_worker"      (how jobs are dispatched to engines)

### 8. Surfaces
- crates/rusvel-cli/src/lib.rs         (3-tier CLI: one-shot, REPL, TUI)
- crates/rusvel-mcp/src/lib.rs         (MCP server for AI tool use)
- frontend/src/routes/+page.svelte     (dashboard entry)
- frontend/src/routes/dept/[id]/chat/+page.svelte  (department chat UI)

## Document structure to produce

Write a single markdown file with these sections:

1. **What RUSVEL Is** — one paragraph, what problem it solves, for whom
2. **Boot Sequence** — what happens from `cargo run` to "ready to serve requests"
   (SQLite init → LLM providers → agent runtime → tool registry → department boot → job worker → HTTP server)
3. **The Port System** — how hexagonal architecture works here, key ports, why engines never touch adapters
4. **Department Lifecycle** — DepartmentApp trait, manifest, registration, tools/events/jobs, the registry
5. **Request Flow: Department Chat** — step by step from HTTP POST to SSE stream back to client
   (routing → context loading → AgentRuntime::run_streaming → tool calls → LLM → SSE events → persist → hooks)
6. **The Agent Runtime** — how run_streaming works, the tool-use loop, streaming events, model tier routing
7. **Background Jobs** — job queue, approval gates, how engines get async work done
8. **The Frontend** — SvelteKit structure, how it connects to the API, SSE consumption, key pages
9. **Data Model** — sessions, events, objects, metrics — the 5-store split
10. **Adding a New Department** — the 3-file recipe (engine crate, dept-* wrapper, manifest)
11. **Key Design Decisions** — summarize the ADRs that most affect daily development (ADR-007 metadata, ADR-009 AgentPort, ADR-014 DepartmentApp)

## Style guidelines
- Write as narrative prose, not bullet lists
- Include actual type names and function names when explaining flow
- Show the data path, not just the architecture boxes
- Mention file paths so readers can jump to the code
- No fluff — every sentence should teach something
- Target length: 3000-5000 words
```

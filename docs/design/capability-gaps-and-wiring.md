# Capability Gaps: Skill Execution, Hook Dispatch, Online Discovery

> Current state: CRUD exists for all entities. Execution and discovery are missing.
> This doc covers what's broken, why it matters, and how to fix it properly.

---

## Gap 1: Skills Are Not Executable in Chat

### What exists
- `SkillDefinition` stored in ObjectStore["skills"] with `prompt_template` containing `{{input}}` placeholders
- Full CRUD API (`GET/POST/PUT/DELETE /api/skills`)
- Frontend shows skills in DepartmentPanel, can create/delete
- Skills can be clicked as "quick actions" — but they just paste raw template text into chat

### What's missing
- No `/skill-name` detection in chat messages
- No template interpolation (`{{input}}` → user's actual input)
- No skill-aware prompt building in `department_chat_handler`

### Why it matters
Skills are the reusable unit of work. Without execution, they're just text blobs. The user has to manually copy-paste prompt templates. This defeats the purpose of having skills — they should be one-click or one-command.

### Implementation

**In `department.rs` → `department_chat_handler`**, add skill detection AFTER `!build` check, BEFORE normal chat:

```
if message starts with "/skill-name" or "!skill skill-name":
  1. Extract skill name + remaining text as input
  2. Look up skill by name in ObjectStore["skills"] (filtered by engine)
  3. Interpolate {{input}} → remaining text
  4. Use interpolated prompt as the chat message
  5. Continue normal chat flow (rules injection, MCP config, etc.)
```

**New function in `skills.rs`:**
```rust
pub async fn resolve_skill(engine: &str, message: &str, storage: &dyn StoragePort) -> Option<String>
```

Returns `Some(interpolated_prompt)` if message matches a skill, `None` otherwise.

**Frontend:** Skills in DepartmentPanel dispatch the interpolated template, not the raw template.

---

## Gap 2: Hook Dispatch (Events → Actions)

### What exists
- `HookDefinition` stored in ObjectStore["hooks"] with event, matcher, hook_type, action, enabled
- Full CRUD API (`GET/POST/PUT/DELETE /api/hooks`)
- 16 supported event types listed in `GET /api/hook-events`
- Events are emitted via `EventPort` on chat.completed, workflow.completed, etc.

### What's missing
- No code that listens for events and fires matching hooks
- Hooks are stored but never triggered
- No hook executor (command, http, prompt types)

### Why it matters
Hooks are the automation backbone. Without dispatch, there's no way to trigger workflows on events, no post-chat actions, no scheduled tasks. The user defines hooks expecting them to fire — they silently don't.

### Implementation

**New module: `rusvel-api/src/hook_dispatch.rs`**

```rust
pub async fn dispatch_hooks(
    event_kind: &str,
    payload: &serde_json::Value,
    engine: &str,
    storage: &Arc<dyn StoragePort>,
)
```

Logic:
1. Load enabled hooks from ObjectStore["hooks"] filtered by engine
2. Match `hook.event` against `event_kind` (exact match or glob pattern via `hook.matcher`)
3. For each matching hook, execute based on `hook.hook_type`:
   - `"command"` → `tokio::process::Command::new("sh").arg("-c").arg(&hook.action)`
   - `"http"` → `reqwest::Client::new().post(&hook.action).json(&payload)`
   - `"prompt"` → feed `hook.action` as prompt to `ClaudeCliStreamer` (fire-and-forget)

**Wire into existing event emission points:**
- `department.rs` line where `event_port.append(...)` is called after chat.completed
- `workflows.rs` after workflow execution completes
- Any future event emission

**Safety:** Hooks run async (tokio::spawn), don't block chat response. Errors logged, not propagated.

---

## Gap 3: Rules Injection — Verify It's Actually Working

### What exists
- `load_rules_for_engine()` in `rules.rs` returns enabled rules
- `department_chat_handler` calls it and appends `--- Rules ---` block to system prompt
- Rules CRUD works

### Current status
Rules ARE injected into the system prompt. This gap is **smaller than originally assessed**. The rules block is appended to `config.system_prompt` before building CLI args.

### What could be improved
- Rules should also apply to `!build` generated prompts (currently `!build` skips rules)
- Rules should apply to workflow step execution (currently workflows build their own prompts)
- No rule priority/ordering

### Implementation
- In `build_cmd.rs`: load rules and append to the generation prompt
- In `workflows.rs` `run_workflow`: load rules for the workflow's engine and inject into each step's system block
- Add optional `priority: i32` field to RuleDefinition for ordering

---

## Gap 4: MCP Discovery from Online Registries

### What exists
- MCP server configs stored and wired to `claude -p --mcp-config`
- Manual creation via CRUD or `!build mcp: description`

### What's missing
- No search against mcp.so, smithery.ai, npm registry
- No validation that MCP server packages exist
- No auto-install of npm packages
- User must know exact package names and commands

### Why it matters
There are 3000+ MCP servers available. The user shouldn't need to know package names. "I need database access" should find and install the right MCP server.

### Implementation — Part of Capability Engine (Gap 5)

MCP discovery is NOT a standalone feature. It's the core use case of the Capability Engine. The engine uses Claude with WebSearch/WebFetch to:
1. Search mcp.so / npm for relevant servers
2. Verify packages exist (`npm view @package/name`)
3. Generate McpServerConfig JSON
4. Install into ObjectStore

**No separate registry client needed.** Claude does the searching. This keeps the codebase simple and leverages Claude's ability to evaluate relevance.

---

## Gap 5: Capability Engine — Online Discovery + On-Demand Creation

### What exists
- `!build <type>: desc` creates ONE entity at a time (agent, skill, rule, mcp, hook)
- Plan doc at `docs/plans/capability-engine.md` with full vision
- No implementation

### What's missing
- Multi-entity bundle generation (describe a need → get agent + skill + mcp + workflow + rules)
- Online registry search (mcp.so, npm, GitHub)
- Verification step (test MCP connections, validate packages)
- `POST /api/capability/build` endpoint

### Why it matters
This is the "self-building" promise of RUSVEL. Without it, every capability requires manual CRUD or individual `!build` commands. The Capability Engine turns "I need X" into a fully wired capability in one shot.

### Implementation

**New file: `rusvel-api/src/capability.rs`** (~200 lines)

```
POST /api/capability/build
  Body: { description: String, engine: Option<String> }
  Response: SSE stream (delta → progress, done → installed entities)
```

**System prompt includes:**
- All entity JSON schemas (from build_cmd.rs, already written)
- Instructions to search online (WebSearch, WebFetch)
- Instructions to verify packages exist before recommending
- Output format: `CapabilityBundle` JSON with agents[], skills[], rules[], mcp_servers[], hooks[], workflows[], explanation

**Execution:**
1. Build prompt with schema knowledge + user description + current department context
2. Call `claude -p --model opus --effort max --allowedTools "WebSearch,WebFetch,Bash"`
3. Stream progress to user via SSE
4. On Done: extract JSON bundle, persist each entity to ObjectStore
5. Return summary of what was installed

**Chat integration:** `!capability <description>` prefix in department chat (alongside existing `!build`)

**Frontend:** "Build Capability" button in DepartmentPanel Actions tab, or natural language in chat.

---

## Implementation Order

| # | Gap | Depends on | Effort | Reason for order |
|---|-----|-----------|--------|-----------------|
| 1 | Skill execution | Nothing | Small | Simplest, immediate user value |
| 2 | Rules in workflows/build | Nothing | Small | Quick fix, correctness |
| 3 | Hook dispatch | Nothing | Medium | Enables automation |
| 4 | Capability Engine | Gaps 1-3 working | Medium | Core feature, needs stable base |
| 5 | Frontend wiring | Gap 4 | Small | UI for capability engine |

---

## Architecture Alignment

All changes follow hexagonal architecture:
- **No new ports needed** — ObjectStore, EventPort, and ClaudeCliStreamer cover all cases
- **No new crates needed** — all code lives in `rusvel-api` (surface layer)
- **No engine changes** — engines remain port-dependent, all new logic is in the API surface
- **ObjectStore stays generic** — new entity types use existing `put/get/list/delete`

The Capability Engine is a **surface concern** (API endpoint that composes adapters), not a domain concern. It belongs in `rusvel-api`, not in an engine crate.

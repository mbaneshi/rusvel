# Sprint Plan: From Current State to Capability Engine

> Aligned to actual codebase as of 2026-03-23 (commit 3ae7020).

---

## What's Built & Working

| Feature | Backend | Frontend | Chat Wiring |
|---------|---------|----------|-------------|
| 5 Departments (Code/Content/Harvest/GTM/Forge) | 30 routes | DepartmentPanel + DepartmentChat | Streaming SSE |
| Agents CRUD | `agents.rs` → ObjectStore("agents") | Live UI: create form, delete, list | Not wired (no @mention) |
| Skills CRUD | `skills.rs` → ObjectStore("skills") | Live UI: create, delete, click-to-fill chat | Works (fills chat input) |
| Rules CRUD | `rules.rs` → ObjectStore("rules") | Live UI: create, toggle on/off, delete | **DONE** — injected into system prompt |
| Department Config | per-dept model/effort/tools/budget | Config panel in DepartmentChat | **DONE** — passed as `claude -p` flags |
| God Agent Chat | `/api/chat` streaming | Chat page with history sidebar | **DONE** |
| Config API | models list, tools list, get/put | ChatTopBar component | **DONE** |
| Session Management | CRUD + mission + goals + events | Dashboard + sidebar switcher | **DONE** |
| Design System | — | 14 components + tokens + icons | — |

---

## What's Left (5 Steps)

### Step 1: Wire Agent @mention into Chat

**What:** When user types `@agent-name`, load agent from ObjectStore, override department config for that call.

**Where:** `department.rs` → `department_chat_handler()`, after rules injection (line ~240).

**Code — follows the exact pattern of `load_rules_for_engine`:**
```rust
// After rules injection, before building prompt:
if let Some(agent_name) = extract_agent_mention(&body.message) {
    let agents: Vec<crate::agents::AgentProfile> = state.storage.objects()
        .list("agents", ObjectFilter::default()).await
        .unwrap_or_default().into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    if let Some(agent) = agents.iter().find(|a| a.name == agent_name) {
        system_prompt = agent.instructions.clone();
        config.model = agent.default_model.model.clone();
        if !agent.allowed_tools.is_empty() {
            config.allowed_tools = agent.allowed_tools.clone();
        }
        if let Some(budget) = agent.budget_limit {
            config.max_budget_usd = Some(budget);
        }
    }
}

fn extract_agent_mention(msg: &str) -> Option<&str> {
    msg.split_whitespace()
        .find(|w| w.starts_with('@'))
        .map(|w| w.trim_start_matches('@'))
}
```

**Frontend hint:** Show available agents in DepartmentChat input placeholder or as autocomplete.

**Effort:** ~1 hour.

---

### Step 2: MCP Servers + Hooks (CRUD + Wire)

**What:** Two new entity types, same CRUD pattern as agents/skills/rules. Wire into chat handler.

**Backend — 2 new files following `agents.rs` pattern exactly:**

`mcp_servers.rs` (ObjectStore kind: `"mcp_servers"`):
- Types: `McpServerDef { id, name, description, transport, command, url, args, env, enabled, metadata }`
- 5 CRUD handlers + `load_mcp_for_engine(state, engine) -> Vec<McpServerDef>`

`hooks.rs` (ObjectStore kind: `"hooks"`):
- Types: `HookDefinition { id, name, event, hook_type, command, url, enabled, metadata }`
- 5 CRUD handlers + `load_hooks_for_event(state, event) -> Vec<HookDefinition>`

**Wire MCP into chat** — in `department_chat_handler()`, after building `cli_args`:
```rust
let servers = crate::mcp_servers::load_mcp_for_engine(&state, engine).await;
if !servers.is_empty() {
    let mcp_json = serde_json::json!({"mcpServers": servers.iter().map(|s| {
        (s.name.clone(), serde_json::json!({"command": s.command, "args": s.args, "env": s.env}))
    }).collect::<serde_json::Map<_,_>>()}).to_string();
    cli_args.extend(["--mcp-config".into(), mcp_json]);
}
```

**Wire hooks into chat** — after `StreamEvent::Done` stores assistant message:
```rust
let hooks = crate::hooks::load_hooks_for_event(&state_clone, "chat.completed").await;
for hook in hooks {
    if hook.hook_type == "command" { if let Some(cmd) = &hook.command {
        let cmd = cmd.clone();
        tokio::spawn(async move { let _ = tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await; });
    }}
}
```

**Frontend — extend `DepartmentPanel.svelte`:**
- Add `api.ts` types + CRUD functions (same pattern as agents/skills/rules)
- Add "MCP" and "Hooks" tabs to the tab list
- MCP tab: server list with enable/disable, "+ Add Server" form (name, transport, command/url, env)
- Hooks tab: hook list with enable/disable, "+ Add Hook" form (name, event dropdown, type, command/url)

**Routes:** 10 new routes (5 per entity) in `lib.rs`.

**Effort:** ~4 hours.

---

### Step 3: Capability Engine

**What:** `POST /api/capability/build` — natural language in, structured entities installed into ObjectStore.

**New file: `capability.rs`** — follows `department_chat_handler()` pattern:

```rust
pub async fn build_capability(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CapabilityRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    // Same streaming pattern as department_chat_handler
    let streamer = ClaudeCliStreamer::new();
    let prompt = format!("{}\n\nUser request: {}", CAPABILITY_SYSTEM_PROMPT, body.description);
    let args = vec![
        "--model".into(), "opus".into(),
        "--effort".into(), "max".into(),
        "--allowedTools".into(), "WebSearch WebFetch Bash".into(),
        "--no-session-persistence".into(),
    ];
    let rx = streamer.stream_with_args(&prompt, &args);

    // On Done: parse JSON bundle, insert entities into ObjectStore
    let stream = ReceiverStream::new(rx).map(move |event| {
        match &event {
            StreamEvent::Done { full_text, .. } => {
                tokio::spawn(install_bundle(full_text, engine, storage));
                // ...
            }
            // Delta and Error: same as department handler
        }
    });
    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}

async fn install_bundle(text: &str, engine: &Option<String>, storage: &Arc<dyn StoragePort>) {
    // Parse JSON with agents[], skills[], rules[], mcp_servers[], hooks[]
    // For each: storage.objects().put("agents", uuid, value)
    // Same ObjectStore operations used everywhere
}
```

**System prompt** tells Claude the exact JSON schema matching our types (Agent, Skill, Rule, McpServerDef, HookDefinition) and instructs it to search mcp.so, npm, GitHub for real packages.

**Integration into department chat** — detect `!build` prefix:
```rust
if body.message.trim_start().starts_with("!build") {
    let desc = body.message.trim_start_matches("!build").trim();
    return capability::build_capability(state, CapabilityRequest {
        description: desc.into(), engine: Some(engine.into())
    }).await;
}
```

**Frontend:** No new UI needed — user types `!build I need to scrape LinkedIn for opportunities` in any department chat. The response streams like normal, and installed entities appear in the Agents/Skills/Rules/MCP tabs.

Optionally: add a "Build Capability" button in the Actions tab that prefills `!build ` in the chat input.

**Effort:** ~5 hours.

---

### Step 4: Agent Teams + Workflows

**What:** Compose agents into teams, execute multi-step workflows.

**2 new files following same CRUD pattern:**

`teams.rs` (ObjectStore: `"agent_teams"`):
```rust
struct AgentTeam { id, name, description, agent_ids: Vec<String>, strategy: String, metadata }

// CRUD + POST /api/agent-teams/{id}/run
async fn run_team(state, id, body) -> Sse<...> {
    // Load team → load agents by IDs → spawn claude -p per agent
    // "parallel": join_all(handles)
    // "sequential": chain outputs
    // Stream progress to client
}
```

`workflows.rs` (ObjectStore: `"workflows"`):
```rust
struct WorkflowTemplate { id, name, description, steps: Vec<WorkflowStep>, metadata }
struct WorkflowStep { name, prompt, agent_id: Option<String>, depends_on: Vec<String>, approval_required: bool }

// CRUD + POST /api/workflows/{id}/run
// Resolve dependency graph → execute steps → pause at approval gates
```

**Frontend:** Add "Teams" and "Workflows" tabs to DepartmentPanel or Settings page.

**Effort:** ~6 hours.

---

### Step 5: Analytics + MCP Flag + Seed Data

**Analytics** — record usage after every chat completion (in `department_chat_handler` Done branch):
```rust
let _ = storage.objects().put("usage", &uuid::Uuid::now_v7().to_string(), serde_json::json!({
    "engine": engine, "model": config.model, "cost_usd": cost,
    "response_length": msg.content.len(), "timestamp": Utc::now().to_rfc3339(),
})).await;
```

API: `GET /api/analytics/usage` — aggregates from ObjectStore("usage").

**MCP flag** — in `main.rs`:
```rust
#[arg(long)] mcp: bool,
// if cli.mcp { return rusvel_mcp::run_stdio(mcp).await; }
```

**Seed data** — in `main.rs` after DB init, check if ObjectStore("agents") is empty → insert default agents, skills, rules so UI isn't empty on first boot.

**Effort:** ~3 hours.

---

## Dependency Graph

```
Step 1: Agent @mention (1h) ─────────────────────────┐
Step 2: MCP servers + Hooks (4h) ───┐                 │
                                    ├──→ Step 3: Capability Engine (5h)
                                    │         └──→ Step 4: Teams + Workflows (6h)
Step 5: Analytics + MCP flag + Seed (3h) ─── independent
```

Step 1 and 2 can be done in parallel. Step 3 needs 2 (so it can install MCP servers and hooks). Step 4 needs 3 (Capability Engine can generate teams/workflows). Step 5 is independent.

---

## Effort Summary

| Step | Description | Effort |
|------|-------------|--------|
| 1 | Wire agent @mention into chat | 1h |
| 2 | MCP servers + Hooks CRUD + wire | 4h |
| 3 | Capability Engine (`!build` prefix) | 5h |
| 4 | Agent Teams + Workflows | 6h |
| 5 | Analytics + MCP flag + Seed data | 3h |
| **Total** | | **~19 hours** |

At 3-4 hours/day → **5-6 days**.

---

## Definition of Done

- [ ] `@agent-name` in chat overrides department config with agent's config
- [ ] MCP servers: CRUD + injected as `--mcp-config` into claude calls
- [ ] Hooks: CRUD + fire on chat.completed events
- [ ] `!build` in any department chat triggers Capability Engine
- [ ] Capability Engine searches online, generates entities, installs to ObjectStore
- [ ] Installed items appear immediately in Agents/Skills/Rules/MCP/Hooks tabs
- [ ] Agent Teams: create + parallel/sequential execution
- [ ] Workflows: create + dependency-ordered steps + approval gates
- [ ] Usage tracked per chat completion, aggregates via API
- [ ] `--mcp` flag dispatches MCP server
- [ ] Seed data populates defaults on first boot
- [ ] All tests pass, `cargo build` + `npm run check` clean

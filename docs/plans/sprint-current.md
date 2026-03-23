# Sprint Plan: From Storage Layer to Capability Engine

> Aligned to actual codebase state as of 2026-03-23.
> Every step follows existing patterns exactly.

---

## What's Already Built

### Backend (rusvel-api) — 1,573 lines across 8 modules

| Module | Lines | What It Does |
|--------|-------|-------------|
| `department.rs` | 472 | Generic handlers + macro-generated wrappers for 5 departments (30 endpoints) |
| `chat.rs` | 275 | God agent streaming chat + conversation history |
| `routes.rs` | 150 | Sessions, mission, goals, events |
| `config.rs` | 162 | ChatConfig with model/effort/tools + model list + tool list |
| `agents.rs` | 134 | CRUD for AgentProfile via ObjectStore("agents") |
| `skills.rs` | 97 | CRUD for SkillDefinition via ObjectStore("skills") |
| `rules.rs` | 117 | CRUD for RuleDefinition via ObjectStore("rules") + `load_rules_for_engine()` |
| `lib.rs` | 166 | Router with 48 routes + AppState + frontend serving |

### Frontend (api.ts) — All CRUD functions exist

```typescript
// Already implemented:
getAgents(engine?), createAgent(body), deleteAgent(id)
getSkills(engine?), createSkill(body), deleteSkill(id)
getRules(engine?), createRule(body), updateRule(id, body), deleteRule(id)
getDeptConfig(dept), updateDeptConfig(dept, config)
streamDeptChat(dept, message, conversationId, onDelta, onDone, onError)
```

### What's Wired vs. Not Wired

| Feature | Backend | Frontend API | Frontend UI | Chat Integration |
|---------|---------|-------------|-------------|-----------------|
| 5 Departments | DONE | DONE | DONE (DepartmentChat) | DONE |
| Agents CRUD | DONE | DONE | **HARDCODED** arrays | **NOT WIRED** |
| Skills CRUD | DONE | DONE | **HARDCODED** arrays | N/A (click-to-fill) |
| Rules CRUD | DONE | DONE | **NO UI** | **DONE** (load_rules_for_engine injects into prompt) |
| MCP Servers | — | — | — | — |
| Hooks | — | — | — | — |
| Agent @mention | — | — | — | — |
| Teams | — | — | — | — |
| Workflows | — | — | — | — |
| Capability Engine | — | — | — | — |

---

## The Sprint (7 Steps)

### Step 1: Connect UI to Live CRUD (Agents, Skills, Rules)

**What:** Replace hardcoded `prebuiltAgents`/`prebuiltSkills` arrays in department pages with real API calls. Add Rules tab.

**Why first:** Backend + API client are done. The only gap is the UI reading/writing real data. Everything else builds on this.

**Changes per department page (`code/+page.svelte` and 4 others):**

```svelte
<!-- Replace hardcoded arrays with API-backed state -->
<script lang="ts">
    import { getAgents, createAgent, deleteAgent, getSkills, createSkill, deleteSkill, getRules, createRule, updateRule, deleteRule } from '$lib/api';
    import type { Agent, Skill, Rule } from '$lib/api';

    let agents: Agent[] = $state([]);
    let skills: Skill[] = $state([]);
    let rules: Rule[] = $state([]);

    // Load on mount + when session changes
    async function loadResources() {
        [agents, skills, rules] = await Promise.all([
            getAgents(dept), getSkills(dept), getRules(dept)
        ]);
    }
</script>
```

**Agents tab:** Replace `{#each prebuiltAgents}` with `{#each agents}`. Add:
- "New Agent" button → Modal form (name, role, instructions, model, tools)
- Delete button per agent card
- Each agent card shows: name, model badge, role, instruction preview

**Skills tab:** Same — replace hardcoded, add create/delete.

**Rules tab:** New tab (add to tab list). Shows:
- List of rules with Toggle for enabled/disabled → calls `updateRule()`
- "Add Rule" button → Modal (name, content textarea)
- Delete per rule

**Shared component:** Create `$lib/components/crud/ResourceModal.svelte` — generic create/edit modal reused across agents, skills, rules. Takes schema as prop, renders form fields.

**Seed data:** In `main.rs` on first boot — check if ObjectStore("agents") is empty, if so insert 5 default agents + 5 skills + 3 rules. This replaces the hardcoded arrays as initial data.

**Effort:** ~4 hours.

---

### Step 2: Wire Agent @mention into Department Chat

**What:** When user types `@agent-name` in a message, load that agent from ObjectStore and override the department config for that call.

**Where:** `department.rs` → `department_chat_handler()`, before building the prompt (around line 230).

**Pattern:** Follows existing code exactly — same `state.storage.objects().list()` + `serde_json::from_value()` pattern used in `load_rules_for_engine()`.

```rust
// After loading rules (line 232), before building prompt:
let config = if let Some(agent_name) = extract_agent_mention(&body.message) {
    let agents: Vec<AgentProfile> = state.storage.objects()
        .list("agents", ObjectFilter::default()).await
        .unwrap_or_default().into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();
    if let Some(agent) = agents.iter().find(|a| a.name == agent_name) {
        let mut c = config;
        c.system_prompt = agent.instructions.clone();
        c.model = agent.default_model.model.clone();
        if !agent.allowed_tools.is_empty() {
            c.allowed_tools = agent.allowed_tools.clone();
        }
        if let Some(budget) = agent.budget_limit {
            c.max_budget_usd = Some(budget);
        }
        c
    } else { config }
} else { config };

fn extract_agent_mention(msg: &str) -> Option<&str> {
    msg.split_whitespace()
        .find(|w| w.starts_with('@'))
        .map(|w| w.trim_start_matches('@'))
}
```

**Frontend hint:** In DepartmentChat input placeholder, show available agents: "Ask Code Department... (@rust-engine for specialized help)"

**Effort:** ~1 hour.

---

### Step 3: MCP Servers + Hooks CRUD

**What:** Two new entity types following the exact same CRUD pattern as agents/skills/rules.

**Backend — new files:**

`crates/rusvel-api/src/mcp_servers.rs` — follows `agents.rs` pattern exactly:
```rust
const STORE_KIND: &str = "mcp_servers";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub transport: String,       // "stdio" | "http"
    pub command: Option<String>, // for stdio
    pub url: Option<String>,     // for http
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub metadata: serde_json::Value,
}

// list_mcp_servers, create_mcp_server, get_mcp_server, update_mcp_server, delete_mcp_server
// Same handler pattern as agents.rs — State<Arc<AppState>>, ObjectStore operations
```

`crates/rusvel-api/src/hooks.rs` — same pattern:
```rust
const STORE_KIND: &str = "hooks";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub id: String,
    pub name: String,
    pub event: String,           // "chat.completed" | "chat.started"
    pub hook_type: String,       // "command" | "http"
    pub command: Option<String>,
    pub url: Option<String>,
    pub enabled: bool,
    pub metadata: serde_json::Value,
}

// Same CRUD handlers
pub async fn load_hooks_for_event(state: &Arc<AppState>, event: &str) -> Vec<HookDefinition>
```

**Wire into chat — `department.rs`:**

After `StreamEvent::Done` stores the assistant message (around line 274):
```rust
// Fire hooks for chat.completed
let hooks = crate::hooks::load_hooks_for_event(&state_clone, "chat.completed").await;
for hook in hooks {
    match hook.hook_type.as_str() {
        "command" => if let Some(cmd) = &hook.command {
            let cmd = cmd.clone();
            tokio::spawn(async move {
                let _ = tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await;
            });
        },
        "http" => if let Some(url) = &hook.url {
            let url = url.clone();
            let payload = serde_json::json!({"engine": engine_str, "event": "chat.completed"});
            tokio::spawn(async move {
                let _ = reqwest::Client::new().post(&url).json(&payload).send().await;
            });
        },
        _ => {}
    }
}
```

**Wire MCP servers into chat — `department.rs`:**

In `department_chat_handler()`, after building `cli_args`:
```rust
let mcp_servers = crate::mcp_servers::load_mcp_for_engine(&state, engine).await;
if !mcp_servers.is_empty() {
    let mcp_json = build_mcp_config_json(&mcp_servers);
    cli_args.extend(["--mcp-config".into(), mcp_json]);
}
```

**Routes in `lib.rs`:**
```rust
.route("/api/mcp-servers", get(mcp_servers::list).post(mcp_servers::create))
.route("/api/mcp-servers/{id}", get(mcp_servers::get_one).put(mcp_servers::update).delete(mcp_servers::delete))
.route("/api/hooks", get(hooks::list).post(hooks::create))
.route("/api/hooks/{id}", get(hooks::get_one).put(hooks::update).delete(hooks::delete))
```

**Frontend api.ts additions:**
```typescript
export interface McpServer { id: string; name: string; description: string; transport: string; command?: string; url?: string; args: string[]; env: Record<string,string>; enabled: boolean; metadata: Record<string,unknown>; }
export interface Hook { id: string; name: string; event: string; hook_type: string; command?: string; url?: string; enabled: boolean; metadata: Record<string,unknown>; }
// Same CRUD pattern: get*, create*, update*, delete*
```

**Frontend UI:** Add "MCP" and "Hooks" tabs to Settings page. Same card + Toggle + Modal pattern.

**Effort:** ~4 hours.

---

### Step 4: Capability Engine

**What:** `POST /api/capability/build` — takes natural language, outputs structured JSON that gets installed into ObjectStore.

**New file: `crates/rusvel-api/src/capability.rs`**

**Pattern:** Follows `department_chat_handler()` exactly — same streaming SSE, same `ClaudeCliStreamer`, same `StreamEvent` handling. The difference is:
1. System prompt knows RUSVEL's entity schemas
2. Claude gets `--allowedTools WebSearch,WebFetch,Bash` to search online registries
3. On `StreamEvent::Done`, parse the JSON output and insert entities into ObjectStore

```rust
pub async fn build_capability(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CapabilityRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let prompt = format!("{}\n\nUser request: {}", CAPABILITY_SYSTEM_PROMPT, body.description);

    let streamer = ClaudeCliStreamer::new();
    let args = vec![
        "--model".into(), "opus".into(),
        "--effort".into(), "max".into(),
        "--allowedTools".into(), "WebSearch WebFetch Bash".into(),
        "--no-session-persistence".into(),
    ];
    let rx = streamer.stream_with_args(&prompt, &args);

    // Same ReceiverStream pattern as department_chat_handler
    let storage = state.storage.clone();
    let engine = body.engine.clone();

    let stream = ReceiverStream::new(rx).map(move |event| {
        match &event {
            StreamEvent::Delta { text } => /* same delta forwarding */,
            StreamEvent::Done { full_text, .. } => {
                // Parse JSON bundle, install into ObjectStore
                let storage = storage.clone();
                let engine = engine.clone();
                let text = full_text.clone();
                tokio::spawn(async move {
                    install_capability_bundle(&text, &engine, &storage).await;
                });
                /* same done event */
            },
            StreamEvent::Error { message } => /* same error forwarding */,
        }
    });

    Ok(Sse::new(stream).keep_alive(KeepAlive::default()))
}
```

**`install_capability_bundle`** — parses JSON, calls `storage.objects().put("agents", ...)` for each entity type. Same ObjectStore pattern used everywhere.

**The system prompt** defines the JSON schema matching our exact types:
- `Agent` matches `AgentProfile` fields
- `Skill` matches `SkillDefinition` fields
- `Rule` matches `RuleDefinition` fields
- `McpServer` matches `McpServerDef` fields
- `Hook` matches `HookDefinition` fields

**Route:**
```rust
.route("/api/capability/build", post(capability::build_capability))
```

**Frontend integration — two options:**

Option A: Dedicated "Build" button in each department's Actions tab:
```svelte
<button onclick={() => sendQuickAction('!capability: ' + capInput)}>
    Build Capability
</button>
```

Option B: Detect `!capability:` prefix in `department_chat_handler`:
```rust
if body.message.trim_start().starts_with("!capability:") {
    let desc = body.message.trim_start_matches("!capability:").trim();
    return capability::build_capability(state, CapabilityRequest { description: desc.into(), engine: Some(engine.into()) }).await;
}
```

Both options work — Option B is simpler (no new UI), Option A is more discoverable.

**After installation:** Emit an event so the department's Events tab shows what was installed. The frontend reloads agents/skills/rules tabs to reflect new data.

**Effort:** ~5 hours.

---

### Step 5: Agent Teams + Workflow Execution

**What:** Compose multiple agents into teams. Execute workflows with dependency-ordered steps.

**New files:**

`crates/rusvel-api/src/teams.rs`:
```rust
const STORE_KIND: &str = "agent_teams";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTeam {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agent_ids: Vec<String>,
    pub strategy: String,  // "parallel" | "sequential"
    pub metadata: serde_json::Value,
}

// CRUD + run endpoint
pub async fn run_team(State(state), Path(id), Json(body): Json<RunRequest>)
    -> Result<Sse<...>, (StatusCode, String)> {
    let team = load_from_store("agent_teams", &id, &state.storage).await?;
    let agents = load_agents_by_ids(&team.agent_ids, &state.storage).await;

    match team.strategy.as_str() {
        "parallel" => {
            // Spawn N claude -p processes, each with agent's config
            // Collect results via channels, stream progress to client
        }
        "sequential" => {
            // Run one at a time, chain outputs
        }
    }
}
```

`crates/rusvel-api/src/workflows.rs`:
```rust
const STORE_KIND: &str = "workflows";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub name: String,
    pub prompt: String,
    pub agent_id: Option<String>,
    pub depends_on: Vec<String>,
    pub approval_required: bool,
}

// CRUD + run/status endpoints
```

**Frontend:** Teams and Workflows tabs in Settings or dedicated page.

**Effort:** ~6 hours.

---

### Step 6: Analytics + Cost Tracking

**What:** Record usage after every chat completion. Expose aggregates via API.

**Collection point in `department.rs`** — after `StreamEvent::Done`:
```rust
let usage = serde_json::json!({
    "engine": engine_str,
    "model": config.model,
    "cost_usd": cost,
    "response_length": msg.content.len(),
    "timestamp": Utc::now().to_rfc3339(),
});
let _ = storage.objects().put("usage", &uuid::Uuid::now_v7().to_string(), usage).await;
```

**New file: `crates/rusvel-api/src/analytics.rs`:**
```rust
pub async fn get_usage(State(state), Query(params)) -> Result<Json<Vec<UsageSummary>>> {
    let all = state.storage.objects().list("usage", ObjectFilter::default()).await?;
    // Aggregate by day/engine/model
}

pub async fn get_cost_breakdown(State(state)) -> Result<Json<CostBreakdown>> {
    // Sum cost_usd grouped by engine
}
```

**Frontend:** Dashboard widgets showing spend per department + total.

**Effort:** ~3 hours.

---

### Step 7: Wire MCP `--mcp` Flag

**In `main.rs`** — add Clap arg + dispatch:
```rust
#[arg(long)]
mcp: bool,

// In main():
if cli.mcp {
    let mcp = Arc::new(RusvelMcp::new(forge.clone(), sessions.clone()));
    return rusvel_mcp::run_stdio(mcp).await.map_err(|e| e.into());
}
```

**Effort:** ~15 minutes.

---

## Dependency Graph

```
Step 1: Connect UI to live CRUD ──→ Step 2: Agent @mention
                                ──→ Step 3: MCP servers + Hooks
                                         └──→ Step 4: Capability Engine
                                                   └──→ Step 5: Teams + Workflows
Step 6: Analytics (independent, after Step 1)
Step 7: MCP flag (independent)
```

---

## Effort Summary

| Step | Description | Effort |
|------|-------------|--------|
| 1 | Connect UI to live CRUD (agents, skills, rules) | 4h |
| 2 | Wire agent @mention into chat | 1h |
| 3 | MCP servers + hooks CRUD + wire into chat | 4h |
| 4 | Capability Engine (AI builds capabilities on demand) | 5h |
| 5 | Agent teams + workflow execution | 6h |
| 6 | Analytics + cost tracking | 3h |
| 7 | Wire MCP `--mcp` flag | 15m |
| **Total** | | **~23 hours** |

At 3-4 hours/day → **6-8 days**.

---

## Definition of Done

- [ ] Agents tab shows real data from API, supports create/delete
- [ ] Skills tab shows real data from API, click-to-fill works
- [ ] Rules tab exists with enable/disable toggle, create/delete
- [ ] @agent-name in chat overrides department config
- [ ] MCP servers: CRUD + injected as `--mcp-config` into claude calls
- [ ] Hooks: CRUD + fire on chat.completed
- [ ] `!capability:` prefix triggers AI-driven capability building
- [ ] Installed capabilities appear immediately in relevant tabs
- [ ] Agent teams: create + parallel/sequential execution
- [ ] Workflows: create + dependency-ordered execution
- [ ] Usage recorded after every chat, aggregates available via API
- [ ] `--mcp` flag dispatches MCP server
- [ ] All tests pass, `cargo build` + `npm run check` clean

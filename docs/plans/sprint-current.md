# Sprint Plan: Departments + CRUD + Integrations

> Aligned to RUSVEL's actual codebase patterns and SOLID principles.
> Written: 2026-03-23

---

## Architecture Principles

Everything in this sprint follows what already works:

1. **ObjectStore is the universal CRUD layer** — `put(kind, id, json)`, `get(kind, id)`, `delete(kind, id)`, `list(kind, filter)`. Already in `rusvel-core/ports.rs:162`, implemented in `rusvel-db/store.rs:255`. All new entities use this — no new tables, no new traits.

2. **Domain types live in `rusvel-core/domain.rs`** — `AgentProfile` already exists at line 128 with `id`, `name`, `role`, `instructions`, `default_model`, `allowed_tools`, `capabilities`, `budget_limit`, `metadata`. We extend this pattern for Skills, Rules, etc.

3. **Department pattern is the template** — `department.rs` has generic handlers that take `engine: &str` and dispatch. Code department has 6 wrappers (chat, config_get, config_update, conversations, history, events). All new departments copy this exactly.

4. **DepartmentChat.svelte is the reusable frontend** — takes `{dept, title, icon}` props. Quick actions dispatch via `CustomEvent('dept-quick-action')`. Config panel persists via `updateDeptConfig()`.

5. **API client pattern** — `api.ts` has typed `request<T>(path, options)` wrapper. SSE streaming via `streamDeptChat()` with `onDelta/onDone/onError` callbacks.

6. **No new crates** — all new features go in existing crates. `rusvel-api` for handlers, `rusvel-core/domain.rs` for types, frontend components in `$lib/`.

---

## What Exists Today

| Component | Status | Location |
|-----------|--------|----------|
| ObjectStore trait | Done | `ports.rs:162` — `put/get/delete/list` |
| ObjectStore impl (SQLite) | Done | `store.rs:255` — upsert with JSON data column |
| AgentProfile domain type | Done | `domain.rs:128` — name, role, instructions, model, tools, capabilities |
| DepartmentConfig | Done | `department.rs:34` — model, effort, tools, budget, system_prompt, add_dirs |
| Department generic handlers | Done | `department.rs` — chat, config, conversations, history, events |
| Code department wrappers | Done | `department.rs:413-450` — 6 thin wrappers |
| Code department routes | Done | `lib.rs:77-82` — 6 routes |
| DepartmentChat.svelte | Done | Reusable component with config panel, streaming |
| Code page with tabs | Done | Actions, Agents, Skills, Projects, Events tabs |
| API client dept functions | Done | `api.ts` — getDeptConfig, streamDeptChat, etc. |
| Design system | Done | 14 components, semantic tokens, icons |

---

## Step 1: Wire 4 Department Backends

**Add to `department.rs`:** 24 thin wrappers (6 per department x 4 departments).

Each wrapper is 3 lines — delegates to generic handler with engine string:

```rust
// Content
pub async fn content_chat(State(state): State<Arc<AppState>>, Json(body): Json<ChatRequest>)
    -> Result<Sse<impl Stream<...>>, (StatusCode, String)> {
    department_chat_handler("content", state, body).await
}
pub async fn content_config_get(State(state): State<Arc<AppState>>)
    -> Result<Json<DepartmentConfig>, (StatusCode, String)> {
    get_dept_config("content", &state).await.map(Json)
}
// ... same for config_update, conversations, history, events
// Repeat for harvest, gtm, forge
```

**Add to `lib.rs`:** 24 routes following the existing Code pattern:
```rust
.route("/api/dept/content/chat", post(department::content_chat))
.route("/api/dept/content/config", get(department::content_config_get))
// ... etc for all 4 departments
```

**Why mechanical:** The generic handlers already exist. `DepartmentConfig::default_for()` already has `match` arms for "content", "harvest", "gtm". This is pure wiring.

**Effort:** 1 hour.

---

## Step 2: Build 4 Department Pages

**Clone Code page pattern** for each department. Only these things change:

| Department | `dept=` | Icon | Color | Quick Actions |
|------------|---------|------|-------|---------------|
| Content | `"content"` | `*` | purple (purple-600/400/900) | "Draft blog post", "Adapt for Twitter", "Content calendar", "Engagement report" |
| Harvest | `"harvest"` | `$` | amber (amber-600/400/900) | "Scan for opportunities", "Score opportunity", "Draft proposal", "Pipeline status" |
| GTM | `"gtm"` | `^` | cyan (cyan-600/400/900) | "List contacts", "Draft outreach", "Generate invoice", "Deal pipeline" |
| Forge | `"forge"` | `=` | indigo (brand-600/400/900) | "Daily plan", "Review goals", "Hire persona", "Weekly review", "System health" |

Each page: `<DepartmentChat dept="content" title="Content Department" icon="*" />`

Left sidebar has same tab structure (Actions, Agents, Skills, Projects, Events) with domain-specific quick actions.

**Effort:** 2 hours.

---

## Step 3: Agents CRUD

### Backend

**Domain type** — `AgentProfile` already exists in `domain.rs:128`:
```rust
pub struct AgentProfile {
    pub id: AgentProfileId,
    pub name: String,
    pub role: String,
    pub instructions: String,     // = system prompt
    pub default_model: ModelRef,
    pub allowed_tools: Vec<String>,
    pub capabilities: Vec<Capability>,
    pub budget_limit: Option<f64>,
    pub metadata: serde_json::Value,
}
```

We use this type directly. The `metadata` field carries department scoping (`{"engine": "code"}` or absent for global).

**Storage:** `ObjectStore("agents", agent.id, serde_json::to_value(agent))`

**New file: `crates/rusvel-api/src/agents.rs`:**
```rust
pub async fn list_agents(State(state), Query(params)) -> Result<Json<Vec<AgentProfile>>> {
    let all = state.storage.objects().list("agents", ObjectFilter::default()).await?;
    let mut agents: Vec<AgentProfile> = all.into_iter().filter_map(|v| serde_json::from_value(v).ok()).collect();
    if let Some(engine) = params.engine {
        agents.retain(|a| a.metadata.get("engine").and_then(|e| e.as_str()) == Some(&engine)
            || a.metadata.get("engine").is_none());  // include global agents
    }
    Ok(Json(agents))
}

pub async fn create_agent(State(state), Json(mut agent): Json<AgentProfile>) -> Result<Json<AgentProfile>> {
    if agent.id.is_empty() { agent.id = AgentProfileId::new(); }
    state.storage.objects().put("agents", &agent.id.to_string(), serde_json::to_value(&agent)?).await?;
    Ok(Json(agent))
}

pub async fn get_agent(State(state), Path(id)) -> Result<Json<AgentProfile>> { ... }
pub async fn update_agent(State(state), Path(id), Json(agent)) -> Result<Json<AgentProfile>> { ... }
pub async fn delete_agent(State(state), Path(id)) -> StatusCode { ... }
```

**Routes in `lib.rs`:**
```rust
.route("/api/agents", get(agents::list_agents).post(agents::create_agent))
.route("/api/agents/{id}", get(agents::get_agent).put(agents::update_agent).delete(agents::delete_agent))
```

**Register module in `lib.rs`:** `pub mod agents;`

### Frontend

**API additions to `api.ts`:**
```typescript
export interface Agent {
    id: string;
    name: string;
    role: string;
    instructions: string;
    default_model: { provider: string; name: string };
    allowed_tools: string[];
    capabilities: string[];
    budget_limit: number | null;
    metadata: Record<string, unknown>;
}

export async function getAgents(engine?: string): Promise<Agent[]> {
    const q = engine ? `?engine=${engine}` : '';
    return request(`/api/agents${q}`);
}
export async function createAgent(agent: Partial<Agent>): Promise<Agent> {
    return request('/api/agents', { method: 'POST', body: JSON.stringify(agent) });
}
export async function updateAgent(id: string, agent: Partial<Agent>): Promise<Agent> {
    return request(`/api/agents/${id}`, { method: 'PUT', body: JSON.stringify(agent) });
}
export async function deleteAgent(id: string): Promise<void> {
    return request(`/api/agents/${id}`, { method: 'DELETE' });
}
```

**UI in department pages (Agents tab):** Replace hardcoded array with:
- `onMount` → `getAgents(dept)` → populate list
- "New Agent" button → opens Modal with form (name, role, instructions, model select, tool toggles)
- Each agent card: edit button → Modal, delete button → confirm + `deleteAgent()`
- Cards show: name, model badge, role description, tool count

### Wire into Chat

In `department_chat_handler()`, before building prompt:
```rust
// If message contains @agent-name, load that agent and override config
if let Some(agent_name) = extract_agent_mention(&body.message) {
    let agents = state.storage.objects().list("agents", ObjectFilter::default()).await?;
    if let Some(agent_val) = agents.into_iter()
        .filter_map(|v| serde_json::from_value::<AgentProfile>(v).ok())
        .find(|a| a.name == agent_name) {
        config.system_prompt = agent_val.instructions.clone();
        config.model = agent_val.default_model.name.clone();
        if !agent_val.allowed_tools.is_empty() {
            config.allowed_tools = agent_val.allowed_tools.clone();
        }
    }
}
```

**Seed data:** Insert 5 defaults on first boot via `rusvel-app/src/main.rs` (check if `list("agents", ...)` is empty, if so insert seed agents).

**Effort:** 3 hours.

---

## Step 4: Skills CRUD

### Backend

**Domain type — new in `domain.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: SkillId,
    pub name: String,
    pub description: String,
    pub prompt_template: String,
    pub metadata: serde_json::Value,  // {"engine": "code"} for scoping
}
```

**Storage:** `ObjectStore("skills", skill.id, json)`

**New file: `crates/rusvel-api/src/skills.rs`** — same CRUD pattern as agents (list, create, get, update, delete). Filter by `metadata.engine`.

**Routes:**
```rust
.route("/api/skills", get(skills::list_skills).post(skills::create_skills))
.route("/api/skills/{id}", get(skills::get_skill).put(skills::update_skill).delete(skills::delete_skill))
```

### Frontend

**API additions to `api.ts`:**
```typescript
export interface Skill { id: string; name: string; description: string; prompt_template: string; metadata: Record<string, unknown>; }
export async function getSkills(engine?: string): Promise<Skill[]> { ... }
export async function createSkill(skill: Partial<Skill>): Promise<Skill> { ... }
export async function updateSkill(id: string, skill: Partial<Skill>): Promise<Skill> { ... }
export async function deleteSkill(id: string): Promise<void> { ... }
```

**UI in Skills tab:** Replace hardcoded array. Click skill → fills chat input with `prompt_template`. CRUD via Modal forms.

**Effort:** 2 hours.

---

## Step 5: Rules CRUD

### Backend

**Domain type — new in `domain.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub id: RuleId,
    pub name: String,
    pub content: String,          // the rule text
    pub enabled: bool,
    pub metadata: serde_json::Value, // {"engine": "code"} or absent for global
}
```

**Storage:** `ObjectStore("rules", rule.id, json)`

**New file: `crates/rusvel-api/src/rules.rs`** — same CRUD pattern.

**Routes:**
```rust
.route("/api/rules", get(rules::list_rules).post(rules::create_rule))
.route("/api/rules/{id}", get(rules::get_rule).put(rules::update_rule).delete(rules::delete_rule))
```

### Wire into Chat

In `department_chat_handler()`, before building prompt — load enabled rules and append to system prompt:

```rust
let rules: Vec<RuleDefinition> = state.storage.objects()
    .list("rules", ObjectFilter::default()).await?
    .into_iter().filter_map(|v| serde_json::from_value(v).ok())
    .filter(|r: &RuleDefinition| r.enabled)
    .filter(|r| {
        let engine_match = r.metadata.get("engine").and_then(|e| e.as_str());
        engine_match == Some(engine) || engine_match.is_none() // match department or global
    })
    .collect();

if !rules.is_empty() {
    let rules_text = rules.iter()
        .map(|r| format!("[{}]: {}", r.name, r.content))
        .collect::<Vec<_>>().join("\n");
    config.system_prompt = format!("{}\n\nActive rules:\n{}", config.system_prompt, rules_text);
}
```

### Frontend

**UI in new "Rules" tab** (add to department page tab list): List rules with enable/disable Toggle. CRUD via Modal.

**Seed data:** 3 default rules:
- "Engine isolation" (engine: code) — "Engines depend only on rusvel-core traits. Never import adapter crates."
- "Design system" (engine: code) — "Use semantic tokens (--r-*). Never use raw Tailwind color classes."
- "Event emission" (global) — "Emit events via EventPort after every state change."

**Effort:** 2 hours.

---

## Step 6: MCP Server Registry

### Backend

**Domain type — new in `domain.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerDef {
    pub id: McpServerId,
    pub name: String,
    pub description: String,
    pub transport: String,       // "stdio" | "http" | "sse"
    pub command: Option<String>,
    pub url: Option<String>,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub enabled: bool,
    pub metadata: serde_json::Value, // {"engine": "code"} for scoping
}
```

**Storage:** `ObjectStore("mcp_servers", id, json)`

**New file: `crates/rusvel-api/src/mcp_servers.rs`** — same CRUD pattern.

### Wire into Chat

In `DepartmentConfig::to_claude_args()` or in `department_chat_handler()`:
```rust
let servers = load_mcp_servers_for_engine(engine, &state.storage).await;
if !servers.is_empty() {
    let mcp_config = build_mcp_json(&servers); // {"mcpServers": {...}}
    args.extend(["--mcp-config".into(), mcp_config]);
}
```

### Frontend

**UI in Settings page** (new "MCP Servers" section) or per-department:
- List servers with status badge + enable/disable Toggle
- "Add Server" button → Modal (name, transport type, command/url, env key-value pairs)
- Presets: one-click buttons for popular servers (Context7, Playwright, GitHub)

**Effort:** 3 hours.

---

## Step 7: Hooks Registry

### Backend

**Domain type — new in `domain.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub id: HookId,
    pub name: String,
    pub event: String,           // "chat.completed", "chat.started", "error"
    pub hook_type: String,       // "command" | "http"
    pub command: Option<String>,
    pub url: Option<String>,
    pub enabled: bool,
    pub metadata: serde_json::Value,
}
```

**Storage:** `ObjectStore("hooks", id, json)`

**New file: `crates/rusvel-api/src/hooks.rs`** — same CRUD pattern.

### Wire into Chat

In `department_chat_handler()`, after the `StreamEvent::Done` branch stores the assistant message:
```rust
// Fire hooks
let hooks: Vec<HookDefinition> = state.storage.objects()
    .list("hooks", ObjectFilter::default()).await
    .unwrap_or_default().into_iter()
    .filter_map(|v| serde_json::from_value(v).ok())
    .filter(|h: &HookDefinition| h.enabled && h.event == "chat.completed")
    .collect();

for hook in hooks {
    match hook.hook_type.as_str() {
        "command" => if let Some(cmd) = &hook.command {
            tokio::spawn(async move { let _ = tokio::process::Command::new("sh").arg("-c").arg(cmd).output().await; });
        },
        "http" => if let Some(url) = &hook.url {
            let client = reqwest::Client::new();
            tokio::spawn(async move { let _ = client.post(url).json(&payload).send().await; });
        },
        _ => {}
    }
}
```

### Frontend

**UI in Settings page** (new "Hooks" section):
- List hooks grouped by event type
- "Add Hook" → Modal (name, event dropdown, type, command/url)
- Enable/disable Toggle
- Last fired timestamp (from metadata)

**Effort:** 3 hours.

---

## Step 8: Wire MCP `--mcp` Flag

**In `crates/rusvel-app/src/main.rs`:**

Add to Clap struct:
```rust
#[arg(long, help = "Run as MCP server (stdio JSON-RPC)")]
mcp: bool,
```

Add dispatch before CLI/API branching:
```rust
if cli.mcp {
    return rusvel_mcp::run_stdio(/* pass needed ports */).await;
}
```

**Effort:** 15 minutes.

---

## Step 9: Multi-Agent Teams

### Backend

**Domain type — new in `domain.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTeam {
    pub id: AgentTeamId,
    pub name: String,
    pub description: String,
    pub agent_ids: Vec<String>,
    pub strategy: String,         // "parallel" | "sequential" | "best-of-n"
    pub metadata: serde_json::Value,
}
```

**Storage:** `ObjectStore("agent_teams", id, json)`

**New file: `crates/rusvel-api/src/teams.rs`:**
- CRUD endpoints (same pattern)
- `POST /api/agent-teams/{id}/run` — executes the team:
  - Loads agent definitions by `agent_ids`
  - Spawns `claude -p` per agent with its config
  - Collects results
  - Returns combined output

```rust
pub async fn run_team(State(state), Path(id), Json(body): Json<TeamRunRequest>) -> Result<Json<Vec<AgentResult>>> {
    let team: AgentTeam = load_from_store("agent_teams", &id, &state.storage).await?;
    let agents = load_agents_by_ids(&team.agent_ids, &state.storage).await;

    match team.strategy.as_str() {
        "parallel" => {
            let handles: Vec<_> = agents.iter().map(|agent| {
                let streamer = ClaudeCliStreamer::new();
                let prompt = format!("{}\n\n{}", agent.instructions, body.task);
                let args = build_agent_cli_args(agent);
                tokio::spawn(async move { collect_full_response(streamer, &prompt, &args).await })
            }).collect();
            let results = futures::future::join_all(handles).await;
            Ok(Json(results.into_iter().filter_map(|r| r.ok()).collect()))
        }
        "sequential" => { /* chain outputs */ }
        _ => Err(...)
    }
}
```

### Frontend

- Team CRUD in a "Teams" tab or dedicated page
- "Run Team" button → shows parallel progress (spinner per agent)
- Results view: cards per agent with output

**Effort:** 5 hours.

---

## Step 10: Workflow Templates

### Backend

**Domain type — new in `domain.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: WorkflowId,
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
```

**Storage:** `ObjectStore("workflows", id, json)`

**Execution:** `POST /api/workflows/{id}/run` → resolves dependency graph → runs steps (using agent config if specified) → stores results as events → pauses at approval gates.

### Frontend

- Workflow template CRUD
- Pipeline visualization (steps as cards, dependencies as arrows)
- Run button → live progress
- Approval gates show as pending items

**Effort:** 4 hours.

---

## Step 11: Cost & Analytics

### Backend

**Collection point:** `department_chat_handler()` already receives `StreamEvent::Done { cost_usd }`. Extend to store in ObjectStore:

```rust
// After storing assistant message, also store usage record:
let usage = serde_json::json!({
    "engine": engine,
    "model": config.model,
    "cost_usd": cost,
    "response_length": msg.content.len(),
    "conversation_id": conv_id,
    "timestamp": Utc::now().to_rfc3339(),
});
let _ = storage.objects().put("usage", &uuid::Uuid::now_v7().to_string(), usage).await;
```

**API:**
```rust
GET /api/analytics/usage?range=7d    → aggregate by day
GET /api/analytics/cost?group=engine → breakdown by department
```

### Frontend

Dashboard widgets or Settings section:
- Cost by department (simple table or bar)
- Usage over time (list of daily totals)
- Total spend

**Effort:** 3 hours.

---

## Step 12: Preset Library

### Backend

**Ship preset definitions in code** (a `const` in `rusvel-api/src/presets.rs`):
```rust
pub fn get_presets() -> Vec<PresetBundle> { ... }

pub struct PresetBundle {
    pub id: String,
    pub name: String,
    pub description: String,
    pub agents: Vec<AgentProfile>,
    pub skills: Vec<SkillDefinition>,
    pub rules: Vec<RuleDefinition>,
}
```

**API:**
```rust
GET  /api/presets           → list available presets
POST /api/presets/{id}/install   → insert all items into ObjectStore
POST /api/presets/{id}/uninstall → remove items by metadata.preset_id
```

### Frontend

Settings page "Presets" section:
- Grid of preset cards (name, description, what's included)
- Install/Uninstall button per preset
- Installed badge

**Effort:** 2 hours.

---

## Step 13: Job Queue Worker + Approval Flow

### Backend

**In `main.rs`:** spawn worker loop:
```rust
let job_port = job_port.clone();
let engines = engines.clone();
tokio::spawn(async move {
    loop {
        if let Ok(Some(job)) = job_port.dequeue().await {
            match job.kind {
                JobKind::ContentPublish => {
                    if job.status == JobStatus::PendingApproval { continue; }
                    // execute via content-engine
                }
                // ... other kinds
            }
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
});
```

**Approval API:**
```rust
POST /api/jobs/{id}/approve → set status to Ready, worker picks it up
POST /api/jobs/{id}/reject  → set status to Cancelled
GET  /api/jobs?status=pending_approval → list items needing approval
```

### Frontend

- Approval badge in sidebar (count of pending items)
- Approval page or modal: list pending jobs with context, approve/reject buttons

**Effort:** 4 hours.

---

## Dependency Graph

```
Step 1: Department backends ──→ Step 2: Department pages
                                   ├──→ Step 3: Agents CRUD ──→ Step 9: Agent Teams
                                   ├──→ Step 4: Skills CRUD  ──→ Step 10: Workflows
                                   ├──→ Step 5: Rules CRUD
                                   └──→ Step 6: MCP servers
Step 7: Hooks (independent after Step 1)
Step 8: MCP flag (independent)
Step 11: Analytics (after Step 1 — needs chat usage data)
Step 12: Presets (after Steps 3, 4, 5)
Step 13: Job queue + approval (after Step 10)
```

---

## Module Structure (what gets created)

### Backend — new files in `crates/rusvel-api/src/`:
```
agents.rs       → CRUD for AgentProfile (already in domain.rs)
skills.rs       → CRUD for SkillDefinition (new domain type)
rules.rs        → CRUD for RuleDefinition (new domain type)
mcp_servers.rs  → CRUD for McpServerDef (new domain type)
hooks.rs        → CRUD for HookDefinition (new domain type)
teams.rs        → CRUD + run for AgentTeam (new domain type)
workflows.rs    → CRUD + run for WorkflowTemplate (new domain type)
analytics.rs    → Aggregation queries on ObjectStore("usage")
presets.rs      → Preset definitions + install/uninstall
```

Each file follows the same pattern:
1. Deserialize from ObjectStore JSON
2. Filter by `metadata.engine` for department scoping
3. Axum handlers with `State<Arc<AppState>>` extraction
4. Return `Result<Json<T>, (StatusCode, String)>`

### Domain — additions to `crates/rusvel-core/src/domain.rs`:
```rust
// Already exists:
AgentProfile, AgentProfileId, Capability

// New types:
SkillDefinition, SkillId
RuleDefinition, RuleId
McpServerDef, McpServerId
HookDefinition, HookId
AgentTeam, AgentTeamId
WorkflowTemplate, WorkflowStep, WorkflowId
```

All follow the existing pattern: derive `Debug, Clone, Serialize, Deserialize`, include `metadata: serde_json::Value`.

### Frontend — additions to `frontend/src/lib/api.ts`:
```typescript
// Types: Agent, Skill, Rule, McpServer, Hook, AgentTeam, Workflow
// CRUD functions: getX, createX, updateX, deleteX for each
// Run functions: runTeam, runWorkflow
// Analytics: getUsage, getCostBreakdown
```

### Frontend — new or modified pages:
```
routes/content/+page.svelte    → clone Code page pattern
routes/harvest/+page.svelte    → clone Code page pattern
routes/gtm/+page.svelte        → clone Code page pattern
routes/forge/+page.svelte      → extend with department chat
routes/settings/+page.svelte   → add MCP, Hooks, Presets, Analytics sections
```

---

## Effort Summary

| Step | Description | Effort | Depends On |
|------|-------------|--------|------------|
| 1 | Wire 4 department backends | 1h | — |
| 2 | Build 4 department pages | 2h | 1 |
| 3 | Agents CRUD + wire into chat | 3h | 2 |
| 4 | Skills CRUD | 2h | 2 |
| 5 | Rules CRUD + wire into chat | 2h | 2 |
| 6 | MCP server registry + wire into chat | 3h | 2 |
| 7 | Hooks registry + wire into chat | 3h | 1 |
| 8 | Wire MCP `--mcp` flag | 15m | — |
| 9 | Multi-agent teams | 5h | 3 |
| 10 | Workflow templates | 4h | 4 |
| 11 | Cost & analytics | 3h | 1 |
| 12 | Preset library | 2h | 3, 4, 5 |
| 13 | Job queue + approval flow | 4h | 10 |
| **Total** | | **~34 hours** |

At 3-4 hours/day → **9-11 days**.

---

## Definition of Done

- [ ] All 5 departments: working chat + config + events
- [ ] Agents: CRUD in DB, @mention overrides chat config
- [ ] Skills: CRUD in DB, click-to-fill chat input
- [ ] Rules: CRUD in DB, auto-inject into system prompt
- [ ] MCP servers: CRUD in DB, passed to `claude -p --mcp-config`
- [ ] Hooks: CRUD in DB, fire on chat.completed events
- [ ] Agent teams: CRUD + parallel/sequential execution
- [ ] Workflows: CRUD + dependency-ordered execution
- [ ] Analytics: cost tracking per department/model
- [ ] Presets: one-click install of curated bundles
- [ ] MCP `--mcp` flag dispatches server
- [ ] Job queue worker processes jobs with approval gates
- [ ] All tests pass, clean build

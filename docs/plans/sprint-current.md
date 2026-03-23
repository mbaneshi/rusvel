# Sprint Plan: Departments + CRUD + Integrations

> Where we are, what's next, and how to get there.
> Written: 2026-03-23 | Updated: 2026-03-23

---

## Where We Are

### Built & Working

**Department Pattern (Code department = proof of concept):**
- Generic `department_chat_handler()` — works for any engine via string param
- Per-department config (model, effort, tools, budget, permissions, add-dirs, system prompt) stored in ObjectStore
- Per-department conversation namespace with history
- Per-department event emission
- Streaming SSE via `claude -p --output-format stream-json` with all flags
- Full frontend: chat UI, config panel (model picker, effort toggle, tools toggle), tabs (Actions, Agents, Skills, Projects, Events)
- Reusable `DepartmentChat.svelte` component

**God Agent (global chat):**
- Streaming chat at `/api/chat`
- Conversation history + sidebar
- Config controls (model, effort, tools)

**Design System:**
- 14 UI components, semantic tokens, icon registry, typography, layout primitives

**Infrastructure:**
- 20 crates, 149 tests, clean build
- SQLite WAL with 5 canonical stores
- Session management (create, list, switch)
- Forge engine wired (mission_today, goals)

### Not Built

| Item | What exists | What's missing |
|------|------------|----------------|
| Content/Harvest/GTM/Forge departments | Generic handler ready, placeholder pages | Backend wrappers, routes, frontend pages |
| Agents | Hardcoded array in Code page UI | DB storage, CRUD API, live UI |
| Skills | Hardcoded array in Code page UI | DB storage, CRUD API, live UI |
| Rules | Nothing | DB storage, CRUD API, UI |
| MCP dispatch | `rusvel-mcp` crate built with 5 tools | `--mcp` flag not dispatched in main.rs |
| Job queue worker | `JobPort` with enqueue/dequeue/approve | No worker loop processing jobs |
| Approval flow | Domain types exist in engines | No API endpoints, no UI |

---

## The Sprint (10 Steps)

### Step 1: Wire Remaining 4 Departments (Backend)

**What:** Add wrapper functions + routes for Content, Harvest, GTM, Forge — same pattern as Code.

**Where:**
- `crates/rusvel-api/src/department.rs` — add `content_chat`, `content_config_get`, etc. (6 functions x 4 departments = 24 wrappers, each is 3 lines)
- `crates/rusvel-api/src/lib.rs` — add 24 routes (4 departments x 6 endpoints)

**Pattern (already proven with Code):**
```rust
// Each department gets 6 endpoints:
POST /api/dept/{engine}/chat              → streaming SSE
GET  /api/dept/{engine}/chat/conversations → list conversations
GET  /api/dept/{engine}/chat/conversations/{id} → message history
GET  /api/dept/{engine}/config            → get config
PUT  /api/dept/{engine}/config            → update config
GET  /api/dept/{engine}/events            → filtered events
```

**Why first:** Everything downstream (agents, skills, frontend pages) depends on these endpoints existing.

**Effort:** ~1 hour. Mechanical copy of Code pattern.

---

### Step 2: Build Department Frontend Pages

**What:** Create real pages for Content, Harvest, GTM, Forge — each with the department chat UI and department-specific quick actions.

**Where:** `frontend/src/routes/{content,harvest,gtm,forge}/+page.svelte`

**Pattern:** Clone Code page structure, change:
- Engine name (`dept="content"`)
- Color theme (Content=purple, Harvest=amber, GTM=cyan, Forge=indigo)
- Icon character (Content=`*`, Harvest=`$`, GTM=`^`, Forge=`=`)
- Quick actions (domain-specific per engine)
- Pre-built agents (relevant subset per engine)

**Department-specific quick actions:**

| Department | Quick Actions |
|------------|--------------|
| **Content** | "Draft blog post about...", "Adapt for Twitter thread", "Generate content calendar", "Review unpublished drafts", "Engagement report" |
| **Harvest** | "Scan for opportunities", "Score this opportunity", "Draft proposal for...", "Pipeline status", "Competitor analysis" |
| **GTM** | "List contacts", "Draft outreach sequence", "Generate invoice", "Deal pipeline status", "Revenue report" |
| **Forge** | "Generate daily plan", "Review goals progress", "Hire persona for task", "System health check", "Weekly review" |

**Effort:** ~2 hours. Four pages, same skeleton, different content.

---

### Step 3: Agents CRUD (DB → API → UI)

**What:** Replace hardcoded agent arrays with real CRUD backed by ObjectStore.

**Domain model:**
```rust
// In department.rs or new agents.rs
struct AgentDefinition {
    id: String,
    name: String,
    description: String,
    model: String,              // opus, sonnet, haiku
    engine: Option<String>,     // which department this agent belongs to, or null for global
    system_prompt: String,
    allowed_tools: Vec<String>,
    disallowed_tools: Vec<String>,
    max_turns: Option<u32>,
    created_at: String,
}
```

**Storage:** `ObjectStore("agents", agent_id, json)`

**API:**
```
GET    /api/agents                → list all agents
GET    /api/agents?engine=code    → list agents for a department
POST   /api/agents                → create agent
GET    /api/agents/{id}           → get agent
PUT    /api/agents/{id}           → update agent
DELETE /api/agents/{id}           → delete agent
```

**Frontend:** Replace hardcoded `prebuiltAgents` array in Code page's Agents tab with:
- Fetch from API on mount
- "Create Agent" button → modal form (name, model, description, system prompt, tools)
- Edit/delete on each agent card
- Agents scoped by department (engine filter) or global

**How agents are used:** When the user sends a chat message, they can @mention an agent. The department chat handler looks up the agent definition from DB, injects its system prompt and tool config into the `claude -p` call. This replaces the generic department system prompt with the agent's specialized prompt.

**Seed data:** On first boot (or via migration), insert the 5 strategy-report agents as defaults so the UI isn't empty.

**Effort:** ~3 hours (backend + frontend + seed data).

---

### Step 4: Skills CRUD (DB → API → UI)

**What:** Replace hardcoded skills arrays with real CRUD.

**Domain model:**
```rust
struct SkillDefinition {
    id: String,
    name: String,               // e.g. "wire-engine"
    description: String,
    engine: Option<String>,     // scoped to department or global
    prompt_template: String,    // the actual prompt with {placeholders}
    created_at: String,
}
```

**Storage:** `ObjectStore("skills", skill_id, json)`

**API:**
```
GET    /api/skills               → list all
GET    /api/skills?engine=code   → filter by department
POST   /api/skills               → create
PUT    /api/skills/{id}          → update
DELETE /api/skills/{id}          → delete
```

**Frontend:** Replace hardcoded `prebuiltSkills` in Skills tab with:
- Fetch from API
- "Create Skill" button → form (name, description, prompt template)
- Click skill → fills chat input with skill's prompt template
- Edit/delete per skill

**How skills are used:** Clicking a skill in the sidebar fills the chat input with the skill's `prompt_template`. User can edit the prompt before sending. The department chat handler processes it like any other message.

**Seed data:** Insert the 5 strategy-report skills as defaults.

**Effort:** ~2 hours. Simpler than agents (no tool config).

---

### Step 5: Rules CRUD (DB → API → UI)

**What:** Rules are instructions that get injected into the department's system prompt based on conditions.

**Domain model:**
```rust
struct RuleDefinition {
    id: String,
    name: String,
    description: String,
    engine: Option<String>,     // scoped to department or global
    condition: String,          // when to apply: "always", or a keyword/glob
    content: String,            // the rule text injected into system prompt
    enabled: bool,
    created_at: String,
}
```

**Storage:** `ObjectStore("rules", rule_id, json)`

**API:**
```
GET    /api/rules                → list all
GET    /api/rules?engine=code   → filter by department
POST   /api/rules                → create
PUT    /api/rules/{id}          → update
DELETE /api/rules/{id}          → delete
```

**How rules are used:** Before building the department chat prompt, the handler:
1. Loads all enabled rules for this engine (+ global rules)
2. Appends rule content to the system prompt
3. This way, rules like "never import adapter crates in engines" automatically appear in every Code department chat

**Frontend:** New "Rules" tab in department pages (replaces or extends existing tabs):
- List rules with enable/disable toggle
- "Add Rule" button → form (name, content, condition)
- Edit/delete per rule

**Seed data:** Insert architecture rules as defaults:
- "Engines depend only on rusvel-core" (engine: code)
- "Use design system tokens, not raw Tailwind" (engine: code)
- "Always emit events after state changes" (global)

**Effort:** ~2 hours.

---

### Step 6: Wire Rules into Department Chat

**What:** Make rules actually work — inject them into the system prompt before each chat call.

**Where:** `department.rs` → `department_chat_handler()`

**Change:** Before calling `build_dept_prompt()`:
1. Load enabled rules from ObjectStore for this engine + global
2. Append rule content to `config.system_prompt`
3. Pass enriched prompt to `build_dept_prompt()`

```rust
// In department_chat_handler, before building prompt:
let rules = load_rules_for_engine(engine, &state.storage).await;
let enriched_prompt = format!(
    "{}\n\n{}",
    config.system_prompt,
    rules.iter().map(|r| format!("RULE — {}: {}", r.name, r.content)).collect::<Vec<_>>().join("\n")
);
let prompt = build_dept_prompt(&enriched_prompt, &history, &body.message);
```

**Effort:** ~30 minutes.

---

### Step 7: Wire Agents into Department Chat

**What:** Allow users to @mention agents in chat, which overrides the department's default config with the agent's specialized config.

**Where:** `department.rs` → `department_chat_handler()`

**Change:** Before building the prompt:
1. Parse the user message for `@agent-name` mentions
2. If found, load agent from ObjectStore
3. Override system prompt, model, effort, tools with agent's config
4. Build `claude -p` args from agent config instead of department config

```rust
// Detect @agent-name in message
if let Some(agent_name) = extract_agent_mention(&body.message) {
    if let Some(agent) = load_agent_by_name(agent_name, &state.storage).await {
        config.system_prompt = agent.system_prompt;
        config.model = agent.model;
        // ... override other fields
    }
}
```

**Effort:** ~1 hour.

---

### Step 8: Wire MCP `--mcp` Flag

**What:** One if-branch in main.rs to dispatch MCP server.

**Where:** `crates/rusvel-app/src/main.rs`

**Change:**
1. Add `--mcp` flag to Clap CLI args
2. If flag is set, call `rusvel_mcp::run_stdio(state).await`
3. Return early (MCP server takes over stdin/stdout)

**Why:** With MCP wired, Claude Code can call `rusvel.session_list`, `rusvel.mission_today` etc. natively. The self-building loop closes.

**Effort:** ~15 minutes.

---

### Step 9: Settings Page

**What:** Build the `/settings` page with global configuration, not just per-department config.

**Sections:**
- **Profile:** User identity (loaded from `~/.rusvel/profile.toml`)
- **LLM Providers:** Which providers are available, default model, fallback chain
- **Global Rules:** Rules that apply across all departments
- **Global Agents:** Agents available in all departments
- **System Status:** Health check, DB stats, test count, uptime
- **Danger Zone:** Reset data, clear conversations

**Effort:** ~3 hours.

---

### Step 10: Job Queue Worker + Approval Flow

**What:** The job queue has enqueue/dequeue/approve but no worker loop. Wire it.

**Backend:**
1. `tokio::spawn` a loop in main.rs that calls `job_port.dequeue()` every 5 seconds
2. Match `JobKind` to engine method
3. For approval-required jobs: set status to `PendingApproval`, don't execute
4. Add `POST /api/jobs/{id}/approve` and `POST /api/jobs/{id}/reject` endpoints

**Frontend:**
- Approval queue in Settings or as a global notification badge
- List of pending approvals with context (what, why, which engine)
- Approve/reject buttons

**Why last:** This is the most complex step and requires everything else to be working. Content publishing and outreach need approval gates.

**Effort:** ~4 hours.

---

## Dependency Graph

```
Step 1: Department wrappers (backend)
  └→ Step 2: Department pages (frontend)
       └→ Step 3: Agents CRUD
       └→ Step 4: Skills CRUD
       └→ Step 5: Rules CRUD
            └→ Step 6: Wire rules into chat
            └→ Step 7: Wire agents into chat
Step 8: MCP flag (independent, do anytime)
Step 9: Settings page (after 3,4,5)
Step 10: Job queue + approval (last)
```

Steps 1-2 unblock everything. Steps 3-5 can be done in parallel. Steps 6-7 depend on 3 and 5. Step 8 is independent. Steps 9-10 are last.

---

## Effort Summary

| Step | Description | Effort |
|------|-------------|--------|
| 1 | Wire 4 department backends | 1 hour |
| 2 | Build 4 department frontend pages | 2 hours |
| 3 | Agents CRUD (DB → API → UI) | 3 hours |
| 4 | Skills CRUD (DB → API → UI) | 2 hours |
| 5 | Rules CRUD (DB → API → UI) | 2 hours |
| 6 | Wire rules into department chat | 30 min |
| 7 | Wire agents into department chat | 1 hour |
| 8 | Wire MCP `--mcp` flag | 15 min |
| 9 | Settings page | 3 hours |
| 10 | Job queue + approval flow | 4 hours |
| **Total** | | **~19 hours** |

At 3-4 hours/day → **5-6 days** to complete this sprint.

---

---

## Steps 11-16: Integrations Layer

> These wire external tools from `enhance-by-community.md` into RUSVEL as product features.
> All stored in DB, managed via API, controlled from UI — not files on disk.

---

### Step 11: MCP Server Registry (DB → API → UI)

**What:** Users can add/remove/configure MCP servers from the UI. Each department can have its own MCP servers.

**Why:** The community doc lists Context7, Firecrawl, Playwright, GitHub, Supabase, Brave Search, Memory MCP — all useful. Instead of editing `.mcp.json` on disk, RUSVEL manages them as a product feature.

**Domain model:**
```rust
struct McpServerConfig {
    id: String,
    name: String,               // "context7", "playwright", "github"
    description: String,
    transport: String,          // "stdio" | "http" | "sse" | "websocket"
    command: Option<String>,    // for stdio: "npx -y @anthropic/context7-mcp"
    url: Option<String>,        // for http/sse/ws
    args: Vec<String>,
    env: HashMap<String, String>,
    engine: Option<String>,     // scoped to department or null for global
    enabled: bool,
    created_at: String,
}
```

**Storage:** `ObjectStore("mcp_servers", id, json)`

**API:**
```
GET    /api/mcp-servers              → list all
POST   /api/mcp-servers              → add server
PUT    /api/mcp-servers/{id}         → update
DELETE /api/mcp-servers/{id}         → remove
POST   /api/mcp-servers/{id}/test    → test connection
```

**How it works:** When `department_chat_handler` builds the `claude -p` command, it also loads enabled MCP servers for that department and passes them via `--mcp-config` flag as inline JSON.

```rust
// In to_claude_args(), after existing flags:
let mcp_servers = load_mcp_servers_for_engine(engine, storage).await;
if !mcp_servers.is_empty() {
    let mcp_json = serde_json::to_string(&mcp_config_from(mcp_servers))?;
    args.extend(["--mcp-config".into(), mcp_json]);
}
```

**Frontend:** New "MCP" tab in Settings page or per-department:
- List connected servers with status indicator (green/red)
- "Add Server" button → form (name, transport, command/url, env vars)
- Enable/disable toggle per server
- "Test Connection" button
- Preset library: one-click add for popular servers (Context7, GitHub, Playwright)

**Seed data:** Pre-populate with recommended servers from community doc:
- Context7 (live docs)
- Playwright (browser automation — for harvest-engine scraping)
- GitHub (repo management — for code-engine)

**Effort:** ~3 hours.

---

### Step 12: Hooks Registry (DB → API → UI)

**What:** Lifecycle hooks managed from UI, stored in DB. Users create hooks that fire on events like PostToolUse, SessionStart, Stop, etc.

**Why:** Community doc shows auto-lint, block dangerous commands, Slack notifications, auto-commit — all via hooks. Instead of editing `settings.json`, RUSVEL treats hooks as a manageable resource.

**Domain model:**
```rust
struct HookDefinition {
    id: String,
    name: String,
    description: String,
    event: String,              // "PostToolUse", "Stop", "SessionStart", etc.
    hook_type: String,          // "command" | "http" | "prompt"
    matcher: Option<String>,    // e.g. "Write|Edit" for PostToolUse
    command: Option<String>,    // shell command to run
    url: Option<String>,        // webhook URL
    prompt: Option<String>,     // for prompt-type hooks
    engine: Option<String>,     // scoped to department or global
    enabled: bool,
    created_at: String,
}
```

**Storage:** `ObjectStore("hooks", id, json)`

**API:**
```
GET    /api/hooks                    → list all
POST   /api/hooks                    → create
PUT    /api/hooks/{id}               → update
DELETE /api/hooks/{id}               → remove
```

**How it works:** After a department chat completes (or at other lifecycle points), the handler:
1. Loads enabled hooks for the matching event
2. For "command" hooks: spawns the shell command
3. For "http" hooks: sends a POST to the URL
4. For "prompt" hooks: adds the prompt output to the next system prompt

```rust
// In department_chat_handler, after storing assistant message:
let hooks = load_hooks_for_event("PostToolUse", engine, &state.storage).await;
for hook in hooks {
    if hook.matches_tool("Write|Edit") {
        match hook.hook_type.as_str() {
            "command" => tokio::spawn(run_hook_command(hook.command)),
            "http" => tokio::spawn(call_webhook(hook.url, payload)),
            _ => {}
        }
    }
}
```

**Frontend:** "Hooks" tab in Settings:
- List hooks grouped by event type
- "Add Hook" button → form (event, type, command/url, matcher)
- Enable/disable toggle
- Last execution status + timestamp

**Seed data:**
- Auto-format Rust files after edit (PostToolUse, command: `rustfmt`)
- Desktop notification on completion (Stop, command: `osascript -e '...'`)

**Effort:** ~3 hours.

---

### Step 13: Multi-Agent Orchestration

**What:** Run multiple agents in parallel on a task, each with isolated context. Similar to claude-squad and Agent Teams.

**Why:** Community doc shows this is "the biggest frontier." RUSVEL already has 10 personas in forge-engine. This makes them work together.

**Domain model:**
```rust
struct AgentTeam {
    id: String,
    name: String,
    description: String,
    agents: Vec<String>,        // agent IDs from the agents table
    strategy: String,           // "parallel" | "sequential" | "best-of-n"
    merge_strategy: String,     // "concatenate" | "vote" | "review"
    engine: Option<String>,
    created_at: String,
}
```

**Storage:** `ObjectStore("agent_teams", id, json)`

**API:**
```
GET    /api/agent-teams              → list teams
POST   /api/agent-teams              → create team
POST   /api/agent-teams/{id}/run     → execute team on a task
GET    /api/agent-teams/{id}/results → get results from last run
DELETE /api/agent-teams/{id}         → remove
```

**How it works:** When a team runs:
1. Load all agent definitions in the team
2. For "parallel": spawn N `claude -p` processes simultaneously, each with its agent's config
3. For "sequential": run agents one after another, passing previous output as context
4. For "best-of-n": run N agents, then a reviewer agent picks the best output
5. Store results, show in UI

```rust
async fn run_agent_team(team: &AgentTeam, task: &str, state: &AppState) -> Vec<AgentResult> {
    let agents = load_agents_by_ids(&team.agents, &state.storage).await;
    match team.strategy.as_str() {
        "parallel" => {
            let handles: Vec<_> = agents.iter()
                .map(|a| tokio::spawn(run_single_agent(a, task)))
                .collect();
            join_all(handles).await
        }
        "sequential" => {
            let mut context = task.to_string();
            let mut results = vec![];
            for agent in &agents {
                let result = run_single_agent(agent, &context).await;
                context = format!("{context}\n\nPrevious agent ({}) said:\n{}", agent.name, result.output);
                results.push(result);
            }
            results
        }
        _ => vec![]
    }
}
```

**Frontend:** "Teams" section in department pages or dedicated `/teams` page:
- Create team: select agents, pick strategy
- Run team: input task, see parallel execution with live progress
- Results view: side-by-side agent outputs

**Seed data:**
- "Code Review Team": [rust-engine, security-auditor, test-writer] → parallel → reviewer picks best
- "Full Stack Team": [rust-engine, api-builder, svelte-ui] → sequential → each builds on previous

**Effort:** ~5 hours. This is the most complex integration.

---

### Step 14: Workflow Templates (GSD-style)

**What:** Pre-built workflow templates that break complex tasks into context-window-sized chunks with specs.

**Why:** GSD framework (23k stars) proves this pattern works. RUSVEL should offer the same spec-driven development internally.

**Domain model:**
```rust
struct WorkflowTemplate {
    id: String,
    name: String,               // "New Engine", "New Feature", "Bug Fix"
    description: String,
    steps: Vec<WorkflowStep>,
    engine: Option<String>,
    created_at: String,
}

struct WorkflowStep {
    name: String,
    prompt: String,             // what the agent should do
    agent: Option<String>,      // which agent to use
    depends_on: Vec<String>,    // step names this depends on
    approval_required: bool,
}
```

**Storage:** `ObjectStore("workflows", id, json)`

**API:**
```
GET    /api/workflows                → list templates
POST   /api/workflows                → create template
POST   /api/workflows/{id}/run       → execute workflow
GET    /api/workflows/{id}/status    → check progress
```

**How it works:**
1. User picks a workflow template (e.g., "Add New Engine")
2. System resolves dependency graph between steps
3. Independent steps run in parallel (using agent teams if available)
4. Steps with `approval_required` pause and wait
5. Each step's output is stored and passed as context to dependent steps
6. UI shows pipeline progress (step 1 ✓, step 2 running, step 3 waiting)

**Seed data:**
```yaml
- name: "Wire New Engine"
  steps:
    - name: "Create engine crate"
      prompt: "Create crates/{name}-engine with Cargo.toml, lib.rs, tests"
      agent: "rust-engine"
    - name: "Add CLI commands"
      prompt: "Add rusvel {name} subcommands to rusvel-cli"
      agent: "api-builder"
      depends_on: ["Create engine crate"]
    - name: "Add API endpoints"
      prompt: "Add /api/dept/{name}/* endpoints to rusvel-api"
      agent: "api-builder"
      depends_on: ["Create engine crate"]
    - name: "Build frontend page"
      prompt: "Create /routes/{name}/+page.svelte with DepartmentChat"
      agent: "svelte-ui"
      depends_on: ["Add API endpoints"]
    - name: "Write tests"
      prompt: "Write tests for the new engine"
      agent: "test-writer"
      depends_on: ["Create engine crate"]
    - name: "Security review"
      prompt: "Review new endpoints for vulnerabilities"
      agent: "security-auditor"
      depends_on: ["Add API endpoints"]
      approval_required: true
```

**Frontend:** "/workflows" page or within department pages:
- Template library (browse, create)
- Run workflow: fill parameters, watch progress
- Pipeline visualization (steps as nodes, dependencies as edges)

**Effort:** ~4 hours.

---

### Step 15: Cost & Analytics Dashboard

**What:** Track token usage, costs, and performance across departments and agents.

**Why:** Community doc shows ccusage, Usage Monitor, SigNoz dashboards. RUSVEL should track its own LLM spend.

**Domain model:**
```rust
struct UsageRecord {
    id: String,
    engine: String,
    agent: Option<String>,
    model: String,
    input_tokens: u64,
    output_tokens: u64,
    cost_usd: f64,
    duration_ms: u64,
    conversation_id: String,
    created_at: String,
}
```

**Storage:** `MetricStore` (already exists in rusvel-core for time-series data)

**Collection:** The `StreamEvent::Done { cost_usd }` already captures cost. Extend to also record tokens and duration. Store in MetricStore after each chat completion.

**API:**
```
GET /api/analytics/usage          → usage by day/week/month
GET /api/analytics/cost           → cost breakdown by engine/agent/model
GET /api/analytics/performance    → response times, error rates
```

**Frontend:** Dashboard widgets or dedicated "/analytics" page:
- Cost by department (bar chart)
- Usage over time (line chart)
- Top agents by cost
- Average response time by model
- Budget remaining (if limits set)

**Effort:** ~3 hours.

---

### Step 16: Preset Library (One-Click Setup)

**What:** A curated library of agents, skills, rules, MCP servers, hooks, and workflows that users can install with one click.

**Why:** Community doc lists 135 agents, 35 skills, 150+ plugins, 1,367 agent skills. RUSVEL should make the best ones instantly available.

**How it works:**
- Ship a JSON/TOML file with preset definitions bundled into the binary
- Settings page has a "Preset Library" section
- Each preset shows: name, description, what it installs (agents, skills, rules, etc.)
- "Install" button inserts records into ObjectStore
- "Uninstall" removes them

**Presets to include:**

| Preset | Installs |
|--------|----------|
| **RUSVEL Starter** | 5 agents + 5 skills + 3 rules (what we've been calling seed data) |
| **Security Pack** | security-auditor agent + OWASP rules + security-review skill |
| **Content Creator** | content writing agents + SEO rules + platform adapter skills |
| **Full Stack Dev** | rust-engine + svelte-ui + test-writer + api-builder agents |
| **DevOps** | CI/CD hooks + deploy-check skill + GitHub MCP server |
| **Web Research** | Firecrawl MCP + Brave Search MCP + web scraping agents |

**Frontend:** Grid of preset cards in Settings:
- Installed vs available status
- One-click install/uninstall
- Preview what's included before installing

**Effort:** ~2 hours (presets are just JSON, UI is a simple grid).

---

## Updated Dependency Graph

```
Step 1: Department wrappers (backend)
  └→ Step 2: Department pages (frontend)
       └→ Step 3: Agents CRUD
       └→ Step 4: Skills CRUD
       └→ Step 5: Rules CRUD
            └→ Step 6: Wire rules into chat
            └→ Step 7: Wire agents into chat
                 └→ Step 13: Multi-agent orchestration
                 └→ Step 14: Workflow templates
Step 8: MCP flag (independent)
Step 9: Settings page (after 3,4,5)
  └→ Step 11: MCP server registry
  └→ Step 12: Hooks registry
  └→ Step 15: Cost & analytics
  └→ Step 16: Preset library
Step 10: Job queue + approval (after 14)
```

---

## Updated Effort Summary

| Step | Description | Effort |
|------|-------------|--------|
| 1 | Wire 4 department backends | 1 hour |
| 2 | Build 4 department frontend pages | 2 hours |
| 3 | Agents CRUD (DB → API → UI) | 3 hours |
| 4 | Skills CRUD (DB → API → UI) | 2 hours |
| 5 | Rules CRUD (DB → API → UI) | 2 hours |
| 6 | Wire rules into department chat | 30 min |
| 7 | Wire agents into department chat | 1 hour |
| 8 | Wire MCP `--mcp` flag | 15 min |
| 9 | Settings page | 3 hours |
| 10 | Job queue + approval flow | 4 hours |
| 11 | MCP server registry (DB → API → UI) | 3 hours |
| 12 | Hooks registry (DB → API → UI) | 3 hours |
| 13 | Multi-agent orchestration | 5 hours |
| 14 | Workflow templates (GSD-style) | 4 hours |
| 15 | Cost & analytics dashboard | 3 hours |
| 16 | Preset library | 2 hours |
| **Total** | | **~39 hours** |

At 3-4 hours/day → **10-13 days** for the full sprint.

---

## Definition of Done

### Core (Steps 1-10)
- [ ] All 5 departments have working chat + config + events
- [ ] Agents: create, edit, delete, @mention in chat overrides config
- [ ] Skills: create, edit, delete, click-to-fill chat input
- [ ] Rules: create, edit, delete, auto-inject into system prompt
- [ ] MCP `--mcp` flag dispatches MCP server
- [ ] Settings page shows profile, providers, global config
- [ ] Job queue worker processes at least one job type
- [ ] Approval endpoints exist and pending items show in UI

### Integrations (Steps 11-16)
- [ ] MCP servers: add, remove, configure from UI, passed to `claude -p` calls
- [ ] Hooks: create, enable/disable, fire on lifecycle events
- [ ] Agent teams: create team, run parallel/sequential, view results
- [ ] Workflows: create template, run with progress tracking, approval gates
- [ ] Analytics: cost per department, usage over time, budget tracking
- [ ] Presets: one-click install of curated agent/skill/rule bundles

### Quality
- [ ] All 149+ tests still pass
- [ ] `cargo build` and `npm run check` clean
- [ ] No architecture violations (engines don't import adapters)

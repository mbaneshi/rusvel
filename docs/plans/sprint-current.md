# Sprint Plan: Departments + CRUD Infrastructure

> Where we are, what's next, and how to get there.
> Written: 2026-03-23

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

## Definition of Done

- [ ] All 5 departments have working chat + config + events
- [ ] Agents: create, edit, delete, @mention in chat overrides config
- [ ] Skills: create, edit, delete, click-to-fill chat input
- [ ] Rules: create, edit, delete, auto-inject into system prompt
- [ ] MCP `--mcp` flag dispatches MCP server
- [ ] Settings page shows profile, providers, global config
- [ ] Job queue worker processes at least one job type
- [ ] Approval endpoints exist and pending items show in UI
- [ ] All 149+ tests still pass
- [ ] `cargo build` and `npm run check` clean

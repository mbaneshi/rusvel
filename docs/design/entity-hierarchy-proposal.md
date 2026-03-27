# Entity Hierarchy & Capability Composition

**Date:** 2026-03-27
**Status:** Proposal
**Problem:** All department entities (Agent, Skill, Rule, Workflow, Hook, MCP) are flat and independent. No foreign keys between them. No composition. No way to say "this agent uses these skills under these rules."

---

## 1. Current State — Flat & Disconnected

```
Agent ──── (nothing) ──── Skill
  │                         │
  │                         │
(nothing)              (nothing)
  │                         │
Rule ──── (nothing) ──── Workflow
  │                         │
  │                     references Agent by NAME (string, fragile)
  │
Hook ──── (nothing) ──── MCP Server
```

Every entity is stored independently in ObjectStore with `kind` + `id`. Department scoping is a loose `metadata.engine` string. No enforced relationships.

**What breaks:**
- Delete an Agent → Workflow step still references it by name → silent failure at runtime
- Create a Skill → No way to say "only Agent X should use this skill"
- Enable a Rule → It applies to ALL agents in the department, even ones where it's irrelevant
- Add an MCP Server → Every agent in the department gets access, no scoping

---

## 2. The Missing Entity: Playbook

The founder doesn't think in agents, skills, and rules independently. They think in **use cases**:

> "I want a cold outreach routine that finds leads, scores them, drafts personalized emails, and follows up."

That's not one agent or one workflow. It's a **bundle of capabilities** working together. We call this a **Playbook**.

### What a Playbook is

A Playbook is a named, reusable configuration that binds agents, skills, rules, workflows, hooks, and MCP servers into a coherent unit for a specific use case.

```
Playbook: "Cold Outreach"
  ├── Agents: [outreach-writer, lead-scorer]
  ├── Skills: [cold-email, follow-up-template, personalization]
  ├── Rules: [no-competitors, professional-tone, max-3-sentences]
  ├── Workflows: [discover-score-draft-send]
  ├── Hooks: [on-reply → update-crm, on-bounce → remove-contact]
  └── MCP Servers: [gmail-api, linkedin-api]
```

### Why Playbook is the right root entity

| Alternative | Problem |
|-------------|---------|
| Agent as root | An agent can participate in multiple use cases. Binding skills/rules to agents makes them inflexible. |
| Workflow as root | Workflows are execution plans, not capability bundles. A use case might have multiple workflows or zero workflows (just chat + skills). |
| Department as root | Too coarse. A department has many use cases. GTM has "cold outreach", "deal closing", "invoicing" — each needs different capabilities. |
| Skill as root | Too fine-grained. A skill is a single prompt template, not a composition unit. |

**Playbook is the right granularity** because it maps to how the founder thinks about their work: "I'm doing cold outreach today" → activate the cold outreach playbook → the right agents, skills, rules, and workflows are all in scope.

---

## 3. The Full Entity Hierarchy

```
Department (the team — from DepartmentManifest)
  │
  ├── Playbook (the use case / routine — NEW)
  │     │
  │     ├── agent_ids: [AgentId]         ← which agents participate
  │     ├── skill_ids: [SkillId]         ← which skills are available
  │     ├── rule_ids: [RuleId]           ← which rules are enforced
  │     ├── workflow_ids: [WorkflowId]   ← which workflows can run
  │     ├── hook_ids: [HookId]           ← which automations are active
  │     ├── mcp_server_ids: [McpId]      ← which external tools are accessible
  │     └── config overrides             ← model, effort, budget for this playbook
  │
  ├── Agent (the worker)
  │     ├── default_skills: [SkillId]    ← skills this agent prefers (NEW)
  │     ├── default_rules: [RuleId]      ← rules this agent follows (NEW)
  │     ├── allowed_tools: [String]      ← tool names (existing)
  │     └── mcp_servers: [McpId]         ← MCP access (NEW)
  │
  ├── Skill (the prompt template)
  │     ├── requires_agent: Option<AgentId>  ← optional: only usable by this agent (NEW)
  │     └── tags: [String]                   ← for discovery (NEW)
  │
  ├── Rule (the constraint)
  │     ├── applies_to: RuleScope            ← Department | Playbook | Agent | Global (NEW)
  │     └── priority: u32                    ← resolution order (NEW)
  │
  ├── Workflow (the execution plan)
  │     ├── steps: [WorkflowStep]
  │     │     ├── agent_id: AgentId          ← FK, not string name (CHANGED)
  │     │     ├── skill_id: Option<SkillId>  ← which skill template to use (NEW)
  │     │     └── rule_ids: [RuleId]         ← step-specific rules (NEW)
  │     └── trigger: Option<TriggerDef>      ← auto-start conditions (NEW)
  │
  ├── Hook (the automation trigger)
  │     └── playbook_id: Option<PlaybookId>  ← scoped to playbook (NEW)
  │
  └── MCP Server (the external tool)
        └── playbook_ids: [PlaybookId]       ← which playbooks can use it (NEW)
```

### Foreign key summary

```
Playbook ──references──→ Agent, Skill, Rule, Workflow, Hook, MCP Server
Agent ────references──→ Skill (defaults), Rule (defaults), MCP Server
Skill ────references──→ Agent (optional owner)
Rule ─────references──→ scope (Department | Playbook | Agent)
Workflow ─references──→ Agent (by ID), Skill (per step), Rule (per step)
Hook ─────references──→ Playbook (optional scope)
MCP Server references──→ Playbook (access list)
```

---

## 4. Runtime Resolution — How Capabilities Compose at Execution Time

When the user chats with a department or runs a workflow, the system needs to resolve: **which agents, skills, rules, and tools are active?**

### Resolution chain (most specific wins)

```
Level 4: Step overrides     (workflow step has specific skill + rules)
Level 3: Agent defaults     (agent has default skills + rules)
Level 2: Playbook bundle    (playbook binds capabilities together)
Level 1: Department scope   (all capabilities with metadata.engine = dept)
Level 0: Global             (capabilities with no department scope)
```

### Example: User sends a chat message in GTM department with "Cold Outreach" playbook active

```
1. Department scope: GTM
   → All agents where metadata.engine = "gtm"
   → All skills where metadata.engine = "gtm"
   → All rules where metadata.engine = "gtm" AND enabled = true

2. Playbook filter: "Cold Outreach"
   → Only agents in playbook.agent_ids
   → Only skills in playbook.skill_ids
   → Only rules in playbook.rule_ids
   → Only MCP servers in playbook.mcp_server_ids

3. Agent context: outreach-writer
   → Agent's default_skills added
   → Agent's default_rules added
   → Agent's allowed_tools filter applied

4. Chat handler builds AgentConfig:
   system_prompt = dept.system_prompt
                 + playbook rules (joined)
                 + agent-specific rules (joined)
   tools = ScopedToolRegistry filtered to playbook + agent scope
   skills = playbook skills + agent default skills (for /skill interpolation)
```

### What changes in the 9-step chat handler

```
Current:                              With playbooks:

1. Validate dept                      1. Validate dept
2. Load dept config                   2. Load dept config + active playbook
3. Interceptors (!build, /skill, @a)  3. Interceptors (skill resolved from playbook scope)
4. Load rules (ALL enabled for dept)  4. Load rules (playbook-scoped + agent-scoped)
5. Inject capabilities                5. Inject capabilities (playbook-filtered)
6. RAG search                         6. RAG search
7. Build AgentConfig                  7. Build AgentConfig (playbook overrides: model, effort)
8. Stream via AgentRuntime            8. Stream via AgentRuntime
9. Post-completion hooks              9. Post-completion hooks (playbook-scoped hooks)
```

Steps 2, 3, 4, 5, 7, 9 gain playbook awareness. The change is additive — if no playbook is active, behavior is identical to current (department-wide scope).

---

## 5. Playbook Data Structure

```rust
pub struct Playbook {
    pub id: PlaybookId,
    pub department: String,           // FK to department (required)
    pub name: String,
    pub description: String,
    pub icon: Option<String>,         // for UI display

    // Capability bindings (foreign keys)
    pub agent_ids: Vec<AgentProfileId>,
    pub skill_ids: Vec<String>,       // skill IDs
    pub rule_ids: Vec<String>,        // rule IDs
    pub workflow_ids: Vec<String>,    // workflow IDs
    pub hook_ids: Vec<String>,        // hook IDs
    pub mcp_server_ids: Vec<String>,  // MCP server IDs

    // Config overrides (optional — falls back to dept config)
    pub model: Option<String>,
    pub effort: Option<String>,
    pub max_budget_usd: Option<f64>,
    pub system_prompt_additions: Option<String>,

    // Lifecycle
    pub is_default: bool,             // auto-activate when entering dept
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
}
```

**Storage:** `ObjectStore`, kind = `"playbooks"`
**Scoping:** Required `department` field (not loose metadata)
**API routes:**
```
GET    /api/playbooks?dept={dept}
POST   /api/playbooks
GET    /api/playbooks/{id}
PUT    /api/playbooks/{id}
DELETE /api/playbooks/{id}
POST   /api/playbooks/{id}/activate   ← set as active playbook for session
GET    /api/playbooks/active?dept={dept}&session_id={id}
```

---

## 6. Daily Routine Mapping

### How a solo founder's day maps to the hierarchy

**Morning: Planning (Forge department)**
```
Playbook: "Daily Planning"
  Agents: [mission-planner, goal-tracker]
  Skills: [daily-brief, priority-sort, blocker-check]
  Rules: [focus-on-revenue, max-3-goals]
  Workflow: [scan-goals → check-blockers → generate-plan]
  Hooks: [on-plan-generated → notify-dashboard]
```

**Mid-morning: Business development (Harvest + GTM)**
```
Playbook: "Deal Pipeline"
  Agents: [lead-scorer, proposal-writer, outreach-sender]
  Skills: [score-rubric, cold-email, proposal-template]
  Rules: [min-score-7, professional-tone, no-spam]
  Workflow: [scan-sources → score → draft-proposal → send-outreach]
  Hooks: [on-reply → escalate, on-deal-won → create-invoice]
  MCP: [gmail-api, linkedin-api]
```

**Afternoon: Content creation (Content department)**
```
Playbook: "Blog Pipeline"
  Agents: [researcher, writer, editor]
  Skills: [blog-outline, seo-optimize, thread-adapter]
  Rules: [brand-voice, technical-accuracy, include-code-samples]
  Workflow: [research → draft → edit → adapt-for-twitter → schedule]
  Hooks: [on-published → share-linkedin, on-scheduled → add-to-calendar]
  MCP: [dev-to-api, twitter-api]
```

**Evening: Review (Forge department)**
```
Playbook: "Daily Review"
  Agents: [reviewer, metrics-analyzer]
  Skills: [progress-summary, blocker-analysis]
  Rules: [honest-assessment, data-driven]
  Workflow: [collect-metrics → summarize → update-goals]
```

### How playbook activation works in the UI

```
Zone A sidebar when GTM department is active:

┌──────────────┐
│ ≡ GTM Dept   │
│──────────────│
│ ▾ Playbooks  │ ← NEW section
│   ● Deal Pipeline (active)
│   ○ Cold Outreach
│   ○ Invoice Mgmt
│   ○ + New...
│──────────────│
│ ○ Agents (3) │ ← filtered to active playbook
│ ○ Skills (5) │ ← filtered to active playbook
│ ○ Rules  (4) │ ← filtered to active playbook
│ ○ Workflows  │
│ ○ Hooks      │
│ ○ Events     │
│──────────────│
│ ○ Chat       │ ← chat uses active playbook's scope
│ ○ Settings   │
└──────────────┘
```

When the user activates "Deal Pipeline" playbook:
- Zone A shows only the agents/skills/rules bound to that playbook
- Zone C chat scope narrows to playbook's agents + skills + rules
- Zone B shows playbook-specific quick actions and workflows
- Switching playbooks = switching context within the department

---

## 7. Discovery Model — How Users Find & Understand Capabilities

### Three discovery paths

**Path 1: Playbook-first (top-down)**
> "What can this department do?"

```
Department → Playbooks list → Select playbook → See bound capabilities
```

The playbook name and description tell the user what it's for. Clicking in shows which agents, skills, rules, and workflows are involved. This is the primary discovery path for new users.

**Path 2: Entity-first (bottom-up)**
> "Where is this skill used?"

```
Skill → "Used in" section → Shows playbooks + agents that reference it
```

Every entity page shows reverse references: which playbooks include it, which agents use it, which workflows reference it. This is the dependency graph made visible.

**Path 3: Capability catalog (cross-cutting)**
> "Show me everything in this department"

```
Department → Actions/Catalog → All capabilities as cards, filterable by type
```

The existing capability catalog (from A2UI vision) shows all entities as cards. With playbooks, cards can be grouped by playbook ("Deal Pipeline: 3 agents, 5 skills, 4 rules") or shown flat with playbook tags.

### Reverse reference display

Every entity gets a "Used in" section showing what references it:

```
Agent: "outreach-writer"
  Used in:
  ├── Playbook: "Cold Outreach" (active)
  ├── Playbook: "Deal Pipeline"
  ├── Workflow: "discover-score-draft-send" (step 3)
  └── Workflow: "follow-up-sequence" (step 1)

Skill: "cold-email"
  Used in:
  ├── Playbook: "Cold Outreach"
  ├── Agent: "outreach-writer" (default skill)
  └── Workflow: "discover-score-draft-send" (step 3, skill override)

Rule: "professional-tone"
  Used in:
  ├── Playbook: "Cold Outreach"
  ├── Playbook: "Deal Pipeline"
  └── Agent: "outreach-writer" (default rule)
```

---

## 8. Extensibility — How New Capabilities Compose

### Adding a new capability to an existing playbook

```
1. Create a Skill: "negotiation-template"
2. Edit Playbook "Deal Pipeline": add skill_id to skill_ids array
3. Done — next chat in Deal Pipeline scope can use /negotiation-template
```

### Agent self-extension (from A2UI vision)

```
1. User chats: "I need a skill for writing LinkedIn connection requests"
2. Agent calls create_skill() tool → Skill created
3. Agent calls update_playbook() tool → Skill added to active playbook
4. Skill immediately available in current playbook scope
5. Event: "skill.created" → Zone A catalog updates live
```

### Cross-department composition

```
Playbook: "Code to Content Pipeline" (in Content dept)
  Agents:
    - code-analyzer (from Code dept, cross-referenced)
    - content-writer (from Content dept)
  Skills:
    - code-summary (from Code dept)
    - blog-from-analysis (from Content dept)
  Workflow:
    Step 1: code-analyzer + code-summary → analyze codebase
    Step 2: content-writer + blog-from-analysis → draft blog from analysis
```

Cross-department references work because agent_ids and skill_ids are global UUIDs. The playbook just binds them — it doesn't care which department originally created the entity. The `metadata.engine` field tells you the origin; the playbook binding tells you the usage.

---

## 9. Schema Changes

### New fields on existing entities

```rust
// Agent — add default bindings
pub struct AgentProfile {
    // ... existing fields ...
    pub default_skill_ids: Vec<String>,     // NEW
    pub default_rule_ids: Vec<String>,      // NEW
    pub mcp_server_ids: Vec<String>,        // NEW
}

// Skill — add optional owner + tags
pub struct SkillDefinition {
    // ... existing fields ...
    pub owner_agent_id: Option<String>,     // NEW: only this agent can use it
    pub tags: Vec<String>,                  // NEW: for discovery
}

// Rule — add scope + priority
pub struct RuleDefinition {
    // ... existing fields ...
    pub scope: RuleScope,                   // NEW: Global | Department | Playbook | Agent
    pub scope_id: Option<String>,           // NEW: the specific playbook/agent ID
    pub priority: u32,                      // NEW: higher = applied later (overrides)
}

// WorkflowStepDef — use IDs not names
pub struct WorkflowStepDef {
    pub agent_id: String,                   // CHANGED: was agent_name (string)
    pub skill_id: Option<String>,           // NEW: which skill template for this step
    pub rule_ids: Vec<String>,              // NEW: step-specific rule overrides
    pub prompt_template: String,
    pub step_type: String,
}

// Hook — add playbook scope
pub struct HookDefinition {
    // ... existing fields ...
    pub playbook_id: Option<String>,        // NEW: scoped to playbook
}
```

### New entity

```rust
pub struct Playbook {
    pub id: String,
    pub department: String,
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub agent_ids: Vec<String>,
    pub skill_ids: Vec<String>,
    pub rule_ids: Vec<String>,
    pub workflow_ids: Vec<String>,
    pub hook_ids: Vec<String>,
    pub mcp_server_ids: Vec<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub max_budget_usd: Option<f64>,
    pub system_prompt_additions: Option<String>,
    pub is_default: bool,
    pub tags: Vec<String>,
    pub metadata: serde_json::Value,
}
```

### Storage

```
ObjectStore, kind = "playbooks"
```

No new database tables needed — ObjectStore handles it. The only migration needed is adding the new fields to existing entity schemas (all backward-compatible via serde defaults).

---

## 10. Relationship Diagram (After)

```
                    ┌─────────────┐
                    │ Department  │
                    │ (manifest)  │
                    └──────┬──────┘
                           │ has many
                    ┌──────▼──────┐
                    │  Playbook   │──────────────────────────────┐
                    │ (use case)  │                              │
                    └──┬──┬──┬──┬─┘                              │
            ┌──────────┘  │  │  └──────────┐                    │
            │             │  │             │                    │
     ┌──────▼──────┐ ┌───▼──▼───┐ ┌───────▼──────┐  ┌─────────▼────────┐
     │   Agent     │ │  Skill   │ │    Rule      │  │   Workflow       │
     │ (worker)    │ │ (prompt) │ │ (constraint) │  │ (execution plan) │
     └──┬───┬──────┘ └──────────┘ └──────────────┘  └──┬───────────────┘
        │   │                                           │
        │   └── default_skills ──→ Skill                │ steps[].agent_id ──→ Agent
        │   └── default_rules ──→ Rule                  │ steps[].skill_id ──→ Skill
        │   └── mcp_servers ──→ MCP Server              │ steps[].rule_ids ──→ Rule
        │
     ┌──▼──────────┐  ┌───────────────┐
     │    Hook     │  │  MCP Server   │
     │ (trigger)   │  │ (ext. tool)   │
     └─────────────┘  └───────────────┘

Foreign key direction: Playbook → {Agent, Skill, Rule, Workflow, Hook, MCP}
                       Agent → {Skill, Rule, MCP} (defaults)
                       Workflow.Step → {Agent, Skill, Rule} (per step)
                       Hook → Playbook (optional scope)
```

---

## 11. API Changes

### New endpoints

```
GET    /api/playbooks?dept={dept}
POST   /api/playbooks
GET    /api/playbooks/{id}
PUT    /api/playbooks/{id}
DELETE /api/playbooks/{id}
POST   /api/playbooks/{id}/activate?session_id={id}
GET    /api/playbooks/active?dept={dept}&session_id={id}

GET    /api/agents/{id}/used-in          ← reverse references
GET    /api/skills/{id}/used-in          ← reverse references
GET    /api/rules/{id}/used-in           ← reverse references
```

### Modified endpoints

```
PUT /api/agents/{id}                     ← now accepts default_skill_ids, default_rule_ids
PUT /api/workflows/{id}                  ← steps now use agent_id (UUID) not agent_name
POST /api/dept/{dept}/chat               ← accepts optional playbook_id query param
```

---

## 12. Backend Impact

### Crate changes

| Crate | Change | Size |
|-------|--------|------|
| `rusvel-core` | Add `Playbook` struct, `RuleScope` enum, new fields on Agent/Skill/Rule/Workflow | ~100 lines |
| `rusvel-api` | Add playbook CRUD routes, reverse-reference endpoints, modify chat handler | ~200 lines |
| `rusvel-agent` | Modify `AgentConfig` builder to accept playbook scope for tool/rule filtering | ~50 lines |
| Each `dept-*` | Add playbook tool (`playbook.activate`, `playbook.create`) to RegistrationContext | ~30 lines each |

### Migration

All changes are additive — new optional fields with serde defaults. No breaking changes to existing stored data. Old entities without the new fields continue to work (playbook_id = None means "department-wide scope" — current behavior).

---

## 13. Frontend Impact

### Zone A sidebar gains "Playbooks" section

```
│ ▾ Playbooks          │ ← NEW top-level section
│   ● Deal Pipeline    │ ← active playbook (radio selection)
│   ○ Cold Outreach    │
│   ○ + New Playbook   │
│──────────────────────│
│ ○ Agents (3/8)       │ ← count shows "in playbook / total"
│ ○ Skills (5/12)      │ ← filtered to active playbook
│ ○ Rules  (4/9)       │
```

### Zone B main content — entity pages gain "Used in" section

```
Agent: outreach-writer
┌──────────────────────────────────────┐
│ Used in:                              │
│ ├── Playbook: Cold Outreach (active) │
│ ├── Playbook: Deal Pipeline          │
│ └── Workflow: outreach-sequence (s3) │
└──────────────────────────────────────┘
```

### New component: `PlaybookSelector.svelte`

Renders in Zone A as a radio-select list of playbooks for the current department. Activating a playbook filters all other sections. Stored as session-scoped state (which playbook is active per department per session).

---

## 14. Compliance

### With design principles

| Principle | How playbooks comply |
|-----------|---------------------|
| **SRP** | Playbook does one thing: bind capabilities into a use case. It doesn't execute, schedule, or render. |
| **O/C** | New playbooks created at runtime without code changes. New entity types can be added to playbook bindings via metadata. |
| **DIP** | Chat handler depends on "active scope" abstraction, not concrete playbook struct. No playbook → works like today. |
| **DRY** | Capabilities defined once (Agent, Skill, Rule). Playbooks reference them — no duplication. |
| **SSOT** | The playbook is the single source of "what's active for this use case." No parallel config. |

### With OpenClaw sprint plan

| Sprint | Impact |
|--------|--------|
| Sprint 1-2 (tool wiring) | None — tools are department-scoped, playbooks are additive |
| Sprint 3 (frontend) | Add PlaybookSelector to Zone A sidebar |
| Sprint 4 (revenue) | Harvest + Content playbooks make the pipeline UX concrete |
| Sprint 5 (outreach) | GTM playbooks bind outreach agents + email skills + CRM hooks |
| Sprint 6 (webhook/cron) | Playbooks can include cron-triggered workflows |

### With UI redesign

| Zone | Playbook impact |
|------|----------------|
| Icon Rail | None — departments unchanged |
| Zone A | Gains "Playbooks" section at top, entity counts filtered by active playbook |
| Zone B | Entity pages gain "Used in" reverse references |
| Zone C | Chat scope filtered to active playbook's agents/skills/rules |
| Bottom | Execution logs can show playbook context |

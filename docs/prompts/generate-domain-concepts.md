# Prompt: How RUSVEL's Domain Concepts Work Together

Feed this to Cursor (or any AI IDE) with the full codebase open.

## How to use

1. Open this file in Cursor
2. Select the prompt text inside the code fence below
3. **Cmd+L** → paste with: *apply this prompt to the codebase*
4. Save output to `docs/design/domain-concepts.md`

---

## The Prompt

````
You are documenting how RUSVEL's domain concepts work together for someone who
will use and extend the system. Read the files listed below, then write a guide
that explains each concept, how it's stored, and most importantly how they
compose at runtime.

## Files to read (grouped by concept)

### The orchestration hub — read this FIRST
- crates/rusvel-api/src/department.rs       (dept_chat handler: where ALL concepts meet)

### Department system
- crates/rusvel-core/src/department/app.rs      (DepartmentApp trait)
- crates/rusvel-core/src/department/manifest.rs (DepartmentManifest — what a dept declares)
- crates/rusvel-core/src/department/context.rs  (RegistrationContext — how depts wire in)
- crates/rusvel-core/src/registry.rs            (DepartmentRegistry — 14 departments)
- crates/dept-forge/src/lib.rs                  (example: how Forge registers)

### Entities (the building blocks users create)
- crates/rusvel-api/src/skills.rs       (SkillDefinition + resolve_skill)
- crates/rusvel-api/src/agents.rs       (AgentProfile storage + @mention)
- crates/rusvel-api/src/rules.rs        (RuleDefinition + load_rules_for_engine)
- crates/rusvel-api/src/hooks.rs        (HookDefinition — event triggers)
- crates/rusvel-api/src/hook_dispatch.rs (how hooks fire: command/http/prompt)
- crates/rusvel-api/src/mcp_servers.rs  (McpServerConfig — external tool servers)

### Orchestration (multi-step execution)
- crates/rusvel-api/src/workflows.rs    (simple sequential workflows)
- crates/rusvel-api/src/playbooks.rs    (playbooks: Agent/Flow/Approval steps)
- crates/flow-engine/src/lib.rs         (DAG flow engine with petgraph)
- crates/flow-engine/src/executor.rs    (how DAG nodes execute)

### Session & runtime
- crates/rusvel-core/src/domain.rs      (Session, Run, Thread, AgentConfig, all domain types)
- crates/rusvel-agent/src/lib.rs        (AgentRuntime — run_streaming, tool-use loop)
- crates/rusvel-api/src/chat.rs         (God Agent chat — cross-department)

### Storage
- crates/rusvel-db/src/store.rs         (ObjectStore — where entities live)

## Document structure

Write a single markdown document with these sections:

### 1. The Mental Model (1 paragraph)
Everything in RUSVEL is scoped to a **Department**. Each department owns its
agents, skills, rules, hooks, and MCP servers. When a user chats with a
department, all these pieces compose into a single agent execution. Explain
this in one clear paragraph.

### 2. Department — The Container
- What DepartmentApp declares (manifest: tools, events, jobs, personas, skills, rules)
- How 14 departments register at boot
- How departments scope everything via `metadata.engine`
- The department registry and config cascade

### 3. Session — The Work Context
- What a Session is (project, lead, campaign, general)
- The hierarchy: Session → Run → Thread → Message
- How sessions scope events, memory, goals, and budget
- SessionConfig: model override, budget limit, approval policies

### 4. Skill — Template-Driven Commands
- What a skill is (prompt template with {{input}} interpolation)
- How `/skill-name some input` is intercepted in chat
- How resolve_skill() matches and expands
- Scoped by department via metadata.engine

### 5. Agent — Persona Override
- What an AgentProfile is (name, role, instructions, model, tools)
- How `@agent-name` in chat overrides the system prompt
- How agents are stored per department
- Relationship to AgentRuntime and AgentConfig

### 6. Rule — System Prompt Injection
- What a rule is (named content block, enabled/disabled)
- How load_rules_for_engine() filters by department
- How rules append to the system prompt as `--- Rules ---`
- When to use rules vs skills vs agent instructions

### 7. Hook — Event-Driven Automation
- What a hook is (event + matcher + action)
- Three types: command (shell), http (webhook), prompt (AI)
- When hooks fire (chat completion, tool use, session events)
- dispatch_hooks(): fire-and-forget async execution

### 8. MCP Server — External Tools
- What an MCP server config is (stdio/http/sse/ws)
- How build_mcp_config_for_engine() generates the config
- How external tools become available to agents

### 9. Workflow — Sequential Steps
- Simple WorkflowDefinition with ordered steps
- Each step: agent_name + prompt_template
- POST /api/workflows/{id}/run triggers execution

### 10. Playbook — Multi-Strategy Orchestration
- PlaybookStep with three action types: Agent, Flow, Approval
- How {{last_output}} chains step results forward
- Built-in playbooks: Content from Code, Opportunity Pipeline, Daily Brief
- Relationship to workflows (simpler) and flows (more powerful)

### 11. Flow — DAG Execution
- FlowDef with nodes and connections (petgraph DAG)
- Node types: code, condition, agent, browser
- How executor.rs traverses the DAG
- Parallel branch support and checkpointing

### 12. How They All Compose — The Chat Request
Walk through exactly what happens when a user sends a message to a department:

```
User sends POST /api/dept/{id}/chat with message body
  │
  ├─ 1. Load department config (registry defaults + stored overrides)
  ├─ 2. Load conversation history (ObjectStore[dept_msg_{dept}])
  ├─ 3. Check for !build command (capability engine)
  ├─ 4. Resolve /skill-name → expand template with {{input}}
  ├─ 5. Check @agent-name → override system prompt + model
  ├─ 6. Load rules → append enabled rules to system prompt
  ├─ 7. Build context pack (goals, events, metrics from session)
  ├─ 8. Inject department capabilities (code: analyze/search, content: draft/publish, etc.)
  ├─ 9. RAG: embed query → vector search → inject knowledge snippets
  ├─ 10. Build AgentConfig (model, tools, final system prompt, session_id)
  ├─ 11. AgentRuntime.run_streaming() → LLM + tool-use loop → SSE events
  ├─ 12. On completion: store message, emit event, dispatch hooks
  │
  └─ Response: SSE stream of AgentEvents (text, tool calls, completion)
```

### 13. Storage Map
Show where each entity lives:

| Entity | ObjectStore Kind | Scope |
|--------|-----------------|-------|
| Agent | `agents` | per-dept via metadata.engine |
| Skill | `skills` | per-dept via metadata.engine |
| Rule | `rules` | per-dept via metadata.engine |
| Hook | `hooks` | per-dept via metadata.engine |
| MCP Server | `mcp_servers` | per-dept via metadata.engine |
| Workflow | `workflows` | global |
| Chat History | `dept_msg_{dept}` | per-dept |
| Config | `dept_config` | per-dept |
| Flow | flows engine storage | flow-engine |
| Playbook | in-memory + module store | global |

### 14. When to Use What
Give practical guidance:

| Need | Use |
|------|-----|
| Reusable prompt shortcut | **Skill** — `/research {{input}}` |
| Persistent behavior modification | **Rule** — "Always respond in bullet points" |
| Specialized persona | **Agent** — `@security-reviewer` with custom instructions |
| React to events automatically | **Hook** — on chat.completed → POST to Slack |
| Multi-step with AI decisions | **Playbook** — Agent → Approval → Flow |
| Complex conditional logic | **Flow** — DAG with branches and parallel execution |
| Simple ordered steps | **Workflow** — step1 → step2 → step3 |
| Connect external AI tools | **MCP Server** — stdio/http tool provider |

## Style guidelines
- Write as a guide for users and developers, not internal docs
- Show the data path, not just definitions
- Use actual type names, function names, and file paths
- Include the chat request walkthrough as a numbered sequence
- The storage map table is mandatory
- Target length: 4000-6000 words
````

# RUSVEL Domain Concepts — How Everything Fits Together

## The Mental Model

Everything in RUSVEL is scoped to a **Department**. Each department owns its agents, skills, rules, hooks, and MCP servers. When a user chats with a department, all these pieces compose into a single agent execution. **Session** is the work boundary that tracks cost, memory, goals, and events across departments. **Events** are the glue — hooks react to events, enabling cross-department automation without coupling.

---

## The Hierarchy

```
Session                          ← your work context (project, lead, campaign)
 ├── has Goals                   ← what you're trying to achieve
 ├── has Budget                  ← spending limit across all departments
 ├── scopes Events              ← everything that happens is tagged to session
 ├── scopes Memory              ← what the system remembers, per session
 │
 └── talks to Departments ──────← the 14 organizational units
      │
      ├── owns Agents            ← personas (@security-reviewer)
      ├── owns Skills            ← prompt shortcuts (/research topic)
      ├── owns Rules             ← "always do X" injected into every chat
      ├── owns Hooks             ← "when X happens, do Y"
      ├── owns MCP Servers       ← external tool providers
      ├── owns Config            ← model, temperature, per-dept settings
      ├── owns Chat History      ← conversation per department
      │
      └── can trigger:
           ├── Workflow          ← simple: step1 → step2 → step3
           ├── Playbook          ← rich: Agent → Approval → Flow
           └── Flow              ← DAG: parallel branches, conditions
```

---

## JSON Manifest Shape

```json
{
  "session": {
    "id": "uuid",
    "kind": "project",
    "goals": ["Launch MVP by April"],
    "budget_limit_usd": 50.0,
    "departments": {
      "forge": {
        "agents": [
          { "name": "strategist", "role": "Plan daily missions", "model": "opus" }
        ],
        "skills": [
          { "name": "daily-brief", "template": "Generate executive brief for {{input}}" }
        ],
        "rules": [
          { "name": "concise", "content": "Keep responses under 200 words", "enabled": true }
        ],
        "hooks": [
          { "event": "forge.chat.completed", "type": "http", "action": "https://slack.com/webhook" }
        ],
        "mcp_servers": [
          { "name": "github", "type": "stdio", "command": "mcp-github" }
        ],
        "chat_history": "dept_msg_forge",
        "config": { "default_model": "sonnet", "temperature": 0.7 }
      },
      "content": { "...same shape..." },
      "harvest": { "...same shape..." }
    },
    "workflows": [
      { "name": "publish-pipeline", "steps": ["draft", "review", "publish"] }
    ],
    "playbooks": [
      {
        "name": "content-from-code",
        "steps": [
          { "action": "Agent", "persona": "code-analyst", "prompt": "Analyze {{input}}" },
          { "action": "Approval", "message": "Proceed with draft?" },
          { "action": "Agent", "persona": "writer", "prompt": "Write article from {{last_output}}" }
        ]
      }
    ],
    "flows": [
      {
        "name": "opportunity-pipeline",
        "nodes": ["scan", "score", "decide", "propose"],
        "connections": [
          { "scan": "score" },
          { "score": "decide" },
          { "decide[true]": "propose" },
          { "decide[false]": "archive" }
        ]
      }
    ]
  }
}
```

---

## Visual Diagrams

### System Overview

```mermaid
graph TB
    subgraph Session["SESSION (work context)"]
        Goals["Goals"]
        Budget["Budget"]
        Memory["Memory (FTS5)"]
        EventBus["Event Bus"]
    end

    subgraph Departments["DEPARTMENTS (14 units)"]
        subgraph Forge["Forge"]
            FA["Agents"]
            FS["Skills"]
            FR["Rules"]
            FH["Hooks"]
            FM["MCP Servers"]
            FC["Chat History"]
        end
        subgraph Content["Content"]
            CA["Agents"]
            CS["Skills"]
            CR["Rules"]
            CH["Hooks"]
        end
        subgraph Harvest["Harvest"]
            HA["Agents"]
            HS["Skills"]
            HR["Rules"]
            HH["Hooks"]
        end
        Other["+ 11 more departments"]
    end

    subgraph Orchestration["ORCHESTRATION (cross-department)"]
        Workflow["Workflow\n(sequential)"]
        Playbook["Playbook\n(Agent + Approval + Flow)"]
        Flow["Flow\n(DAG with branches)"]
    end

    Session --> Departments
    Departments --> Orchestration
    EventBus --> FH & CH & HH
    Forge & Content & Harvest --> EventBus
    Orchestration --> EventBus
```

### Chat Request Flow

```mermaid
sequenceDiagram
    actor User
    participant API as Department API
    participant Config as Config Cascade
    participant Skill as Skill Resolver
    participant Agent as Agent Override
    participant Rules as Rule Loader
    participant RAG as Knowledge/RAG
    participant Runtime as AgentRuntime
    participant LLM as LLM Provider
    participant Hooks as Hook Dispatch
    participant Events as Event Bus

    User->>API: POST /api/dept/{id}/chat
    API->>Config: Load dept config (registry + stored + user)
    API->>API: Load conversation history

    alt message starts with /skill-name
        API->>Skill: resolve_skill(message)
        Skill-->>API: expanded template with {{input}}
    end

    alt message mentions @agent-name
        API->>Agent: lookup AgentProfile
        Agent-->>API: override system prompt + model
    end

    API->>Rules: load_rules_for_engine(dept_id)
    Rules-->>API: enabled rules appended to system prompt

    API->>RAG: embed query + vector search
    RAG-->>API: knowledge snippets injected

    API->>Runtime: create(AgentConfig) + run_streaming()

    loop Tool-use loop
        Runtime->>LLM: prompt + tools
        LLM-->>Runtime: text or tool_call
        Runtime-->>User: SSE: text_delta / tool_call
    end

    Runtime-->>API: Done (final text)
    API->>API: Store assistant message
    API->>Events: emit("{dept}.chat.completed")
    API->>Hooks: dispatch_hooks(event)

    par Hook execution (fire-and-forget)
        Hooks->>Hooks: command: sh -c "..."
        Hooks->>Hooks: http: POST webhook
        Hooks->>Hooks: prompt: claude -p "..."
    end

    API-->>User: SSE: run_completed
```

### Entity Scoping

```mermaid
graph LR
    subgraph Global
        Sessions["Sessions"]
        Workflows["Workflows"]
        Playbooks["Playbooks"]
        Flows["Flows"]
    end

    subgraph "Scoped by metadata.engine"
        Agents["Agents"]
        Skills["Skills"]
        Rules["Rules"]
        Hooks["Hooks"]
        MCP["MCP Servers"]
        Chat["Chat History\n(dept_msg_{id})"]
        DeptConfig["Dept Config"]
    end

    subgraph "Scoped by session_id"
        Events["Events"]
        Memory["Memory"]
        Goals["Goals"]
        Runs["Runs"]
        Threads["Threads"]
    end

    Sessions --> Events & Memory & Goals & Runs
    Runs --> Threads
```

### Three Orchestration Levels

```mermaid
graph TB
    subgraph Workflow["WORKFLOW (simple)"]
        direction LR
        W1["Step 1:\nagent + prompt"] --> W2["Step 2:\nagent + prompt"] --> W3["Step 3:\nagent + prompt"]
    end

    subgraph Playbook["PLAYBOOK (rich)"]
        direction LR
        P1["Agent Step:\ncode-analyst\nanalyzes repo"] --> P2["Approval Step:\nhuman reviews"] --> P3["Agent Step:\nwriter creates\narticle"] --> P4["Flow Step:\npublish pipeline"]
    end

    subgraph Flow["FLOW (DAG)"]
        direction TB
        F1["Scan\n(code node)"] --> F2["Score\n(agent node)"]
        F2 --> F3{"Score > 80?\n(condition)"}
        F3 -->|true| F4["Generate Proposal\n(agent node)"]
        F3 -->|false| F5["Archive\n(code node)"]
        F4 --> F6["Notify\n(code node)"]
        F5 --> F6
    end

    style Workflow fill:#e8f5e9
    style Playbook fill:#e3f2fd
    style Flow fill:#fce4ec
```

### Event-Driven Automation

```mermaid
graph LR
    subgraph Triggers
        ChatDone["chat.completed"]
        Published["content.published"]
        Scanned["harvest.scan.completed"]
        JobDone["job.completed"]
    end

    subgraph Hooks
        H1["Hook: command\nsh -c 'notify.sh'"]
        H2["Hook: http\nPOST slack webhook"]
        H3["Hook: prompt\nclaude -p 'summarize'"]
    end

    subgraph Downstream
        Slack["Slack notification"]
        Flow2["Trigger a Flow"]
        Job["Enqueue a Job"]
    end

    ChatDone --> H1 & H2
    Published --> H2 & H3
    Scanned --> H1
    H1 --> Job
    H2 --> Slack
    H3 --> Flow2
```

---

## Storage Map

| Entity | ObjectStore Kind | Scope | Filtered By |
|--------|-----------------|-------|-------------|
| Agent | `agents` | per-dept | `metadata.engine` |
| Skill | `skills` | per-dept | `metadata.engine` |
| Rule | `rules` | per-dept | `metadata.engine` |
| Hook | `hooks` | per-dept | `metadata.engine` |
| MCP Server | `mcp_servers` | per-dept | `metadata.engine` |
| Workflow | `workflows` | global | — |
| Playbook | in-memory + store | global | — |
| Flow | flow-engine storage | global | — |
| Chat History | `dept_msg_{dept}` | per-dept | key prefix |
| Dept Config | `dept_config` | per-dept | key |
| Session | `SessionStore` | global | — |
| Run | `SessionStore` | per-session | `session_id` |
| Thread | `SessionStore` | per-run | `run_id` |
| Event | `EventStore` | per-session | `session_id` |
| Memory | `MemoryPort` (FTS5) | per-session | `session_id` |
| Goal | `ObjectStore` | per-session | `session_id` |
| Metric | `MetricStore` | per-dept | `department` tag |

---

## When To Use What

| Need | Use | Example |
|------|-----|---------|
| Reusable prompt shortcut | **Skill** | `/research {{input}}` expands to full research prompt |
| Persistent behavior rule | **Rule** | "Always cite sources" injected into every chat |
| Specialized persona | **Agent** | `@security-reviewer` with infosec instructions |
| React to events | **Hook** | on `content.published` → POST to Slack |
| Simple ordered steps | **Workflow** | draft → review → publish |
| Mixed actions with approval | **Playbook** | AI analyzes → human approves → AI executes |
| Complex branching logic | **Flow** | DAG with conditions, parallel branches |
| External tool provider | **MCP Server** | GitHub, Jira, or custom tools via stdio/http |
| Track work context | **Session** | Project with goals, budget, memory |
| Organize capabilities | **Department** | 14 units, each with own agents/skills/rules |

---

## The Key Insights

1. **Department is the scoping boundary.** Agents, skills, rules, hooks, MCP servers, and chat are all scoped by `metadata.engine` matching the department ID. Creating an agent in Content means it only appears in Content chat.

2. **Session is the work boundary.** Events, memory, goals, and budget are scoped to a session. You talk to any department within a session, but the session tracks total cost and context.

3. **Events are the glue.** Hooks react to events. Departments emit events. Jobs emit events. This is how automation chains together without departments knowing about each other.

4. **Three orchestration levels exist for a reason:**
   - **Workflow** — when you know the exact steps and just need sequential execution
   - **Playbook** — when you need human-in-the-loop approval or mixed AI/Flow steps
   - **Flow** — when you need conditional logic, parallel branches, or complex DAGs

5. **Everything composes in the chat handler.** A single chat request resolves skills, loads agents, injects rules, searches knowledge, runs the LLM, and dispatches hooks — all in one request/response cycle.

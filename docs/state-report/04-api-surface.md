# Chapter 4 — API Surface

## AppState

```rust
pub struct AppState {
    pub forge: Arc<ForgeEngine>,
    pub code_engine: Option<Arc<CodeEngine>>,
    pub content_engine: Option<Arc<ContentEngine>>,
    pub harvest_engine: Option<Arc<HarvestEngine>>,
    pub flow_engine: Option<Arc<FlowEngine>>,
    pub sessions: Arc<dyn SessionPort>,
    pub events: Arc<dyn EventPort>,
    pub jobs: Arc<dyn JobPort>,
    pub database: Arc<Database>,
    pub storage: Arc<dyn StoragePort>,
    pub profile: Option<UserProfile>,
    pub registry: DepartmentRegistry,
    pub embedding: Option<Arc<dyn EmbeddingPort>>,
    pub vector_store: Option<Arc<dyn VectorStorePort>>,
    pub memory: Arc<dyn MemoryPort>,
    pub deploy: Option<Arc<dyn DeployPort>>,
    pub agent_runtime: Arc<AgentRuntime>,
    pub tools: Arc<dyn ToolPort>,
    pub terminal: Option<Arc<dyn TerminalPort>>,
    pub cdp: Option<Arc<rusvel_cdp::CdpClient>>,
}
```

All heavy components use `Arc` for safe async sharing across Axum handlers.

## Complete Route Map (~105 routes)

### Core & Health
```
GET  /api/health
```

### Sessions & Mission
```
GET  /api/sessions
POST /api/sessions
GET  /api/sessions/{id}
GET  /api/sessions/{id}/mission/today
GET  /api/sessions/{id}/mission/goals
POST /api/sessions/{id}/mission/goals
GET  /api/sessions/{id}/events
```

### Chat (God Agent — SSE streaming)
```
POST /api/chat                              → SSE stream
GET  /api/chat/conversations
GET  /api/chat/conversations/{id}
```

### Config
```
GET  /api/config
PUT  /api/config
GET  /api/config/models
GET  /api/config/tools
```

### Departments (registry + parameterized routes)
```
GET  /api/departments
GET  /api/profile
PUT  /api/profile
```

#### Per-Department (6 routes x 12 departments)
```
POST /api/dept/{dept}/chat                  → SSE stream
GET  /api/dept/{dept}/chat/conversations
GET  /api/dept/{dept}/chat/conversations/{id}
GET  /api/dept/{dept}/config
PUT  /api/dept/{dept}/config
GET  /api/dept/{dept}/events
```

### CRUD Resources
```
# Agents
GET    /api/agents?engine={dept}
POST   /api/agents
GET    /api/agents/{id}
PUT    /api/agents/{id}
DELETE /api/agents/{id}

# Skills (prompt templates)
GET    /api/skills?engine={dept}
POST   /api/skills
GET    /api/skills/{id}
PUT    /api/skills/{id}
DELETE /api/skills/{id}

# Rules (system prompt injections)
GET    /api/rules?engine={dept}
POST   /api/rules
GET    /api/rules/{id}
PUT    /api/rules/{id}
DELETE /api/rules/{id}

# MCP Servers
GET    /api/mcp-servers
POST   /api/mcp-servers
PUT    /api/mcp-servers/{id}
DELETE /api/mcp-servers/{id}

# Workflows
GET    /api/workflows
POST   /api/workflows
GET    /api/workflows/{id}
PUT    /api/workflows/{id}
DELETE /api/workflows/{id}
POST   /api/workflows/{id}/run

# Hooks
GET    /api/hooks
POST   /api/hooks
PUT    /api/hooks/{id}
DELETE /api/hooks/{id}
GET    /api/hooks/events
```

### Engine-Specific Routes

#### Code
```
POST /api/dept/code/analyze
GET  /api/dept/code/search?q={query}&limit={n}
```

#### Content
```
POST  /api/dept/content/draft
POST  /api/dept/content/from-code
PATCH /api/dept/content/{id}/approve
POST  /api/dept/content/publish
GET   /api/dept/content/list
```

#### Harvest
```
POST /api/dept/harvest/score
POST /api/dept/harvest/scan
POST /api/dept/harvest/proposal
GET  /api/dept/harvest/pipeline
GET  /api/dept/harvest/list
```

### Flow Engine (DAG)
```
GET    /api/flows
POST   /api/flows
GET    /api/flows/{id}
PUT    /api/flows/{id}
DELETE /api/flows/{id}
POST   /api/flows/{id}/run
GET    /api/flows/{id}/executions
GET    /api/flows/{id}/executions/{exec_id}/panes
GET    /api/flows/executions/{id}
POST   /api/flows/executions/{id}/resume
POST   /api/flows/executions/{id}/retry/{node_id}
GET    /api/flows/executions/{id}/checkpoint
GET    /api/flows/node-types
```

### Playbooks
```
GET  /api/playbooks/runs
GET  /api/playbooks/runs/{run_id}
GET  /api/playbooks
POST /api/playbooks
GET  /api/playbooks/{id}
POST /api/playbooks/{id}/run
```

### Starter Kits
```
GET  /api/kits
GET  /api/kits/{id}
POST /api/kits/{id}/install
```

### Approvals (Human-in-the-Loop)
```
GET  /api/approvals
POST /api/approvals/{id}/approve
POST /api/approvals/{id}/reject
```

### Knowledge / RAG
```
GET    /api/knowledge
POST   /api/knowledge/ingest
POST   /api/knowledge/search
POST   /api/knowledge/hybrid-search
GET    /api/knowledge/stats
GET    /api/knowledge/related
DELETE /api/knowledge/{id}
```

### Database Browser (RusvelBase)
```
GET  /api/db/tables
GET  /api/db/tables/{table}/schema
GET  /api/db/tables/{table}/rows
POST /api/db/sql
```

### Terminal
```
GET /api/terminal/dept/{dept_id}
GET /api/terminal/runs/{run_id}/panes
GET /api/terminal/ws                        → WebSocket
```

### Browser (CDP)
```
GET  /api/browser/status
POST /api/browser/connect
GET  /api/browser/tabs
POST /api/browser/observe/{tab}
GET  /api/browser/captures
GET  /api/browser/captures/stream
POST /api/browser/act
```

### System & Self-Improvement
```
POST /api/system/test
POST /api/system/build
GET  /api/system/status
POST /api/system/fix
POST /api/system/ingest-docs
```

### Visual Regression
```
GET  /api/system/visual-report
POST /api/system/visual-report
POST /api/system/visual-report/self-correct
POST /api/system/visual-test
```

### Misc
```
POST /api/capability/build                  → SSE stream
GET  /api/analytics
POST /api/help                              → SSE stream
GET  /api/brief
POST /api/brief/generate
```

---

## Chat Handler Internals

The department chat handler (`POST /api/dept/{dept}/chat`) is the most complex handler:

```
1. Validate dept against registry
2. Load dept config (three-layer cascade: registry → stored → session)
3. Interceptors:
   a. !build <entity_type> → capability builder (synchronous)
   b. /skill-name → resolve_skill() → expand prompt template
   c. @agent-name → override system prompt + model
4. Load enabled rules → append to system prompt
5. Inject engine capabilities (Code, Content, Harvest) into context
6. RAG: embed query → VectorStorePort search → inject relevant knowledge
7. Build AgentConfig with merged system prompt
8. Stream via AgentRuntime → SSE events
9. Post-completion: store message, emit event, dispatch hooks
```

### SSE Event Types
```
text_delta      → incremental text chunk
tool_call_start → tool name + args
tool_call_end   → tool result
run_completed   → final message + cost
run_failed      → error details
```

### Config Cascade
```
Layer 1: Registry defaults (from DepartmentManifest)
Layer 2: Stored overrides (ObjectStore["dept_config"][dept])
Layer 3: User profile context (injected into system prompt)
```

---

## CLI Command Tree

```
rusvel [--mcp] [--mcp-http] [--tui] <COMMAND>

session create <name> | list | switch <id>

forge mission today | goals | goal add | review [--period]
brief

shell                                   # REPL (reedline)

# 12 department commands (all share: status, list, events)
finance | growth | distro | legal | support | infra | product | code | harvest | content | gtm | browser

# Engine-specific:
code analyze [path] | search <query>
content draft <topic>
harvest pipeline
browser connect | status | captures
```

---

## MCP Server (6 tools)

Transport: stdio JSON-RPC (default) or HTTP+SSE (`--mcp-http`)

| Tool | Input | Output |
|------|-------|--------|
| session_list | -- | Vec<Session> |
| session_create | name, kind | { session_id } |
| mission_today | session_id | DailyPlan |
| mission_goals | session_id | Vec<Goal> |
| mission_add_goal | session_id, title, description, timeframe | Goal |
| visual_inspect | routes[], update_baselines | { success, stdout, stderr } |

HTTP MCP adds optional Bearer token auth and SSE push channel.

---

## Handler Module Index (26 modules in rusvel-api)

```
agents          analytics       approvals       browser
build_cmd       capability      chat            config
db_routes       department      engine_routes   flow_routes
help            hook_dispatch   hooks           kits
knowledge       mcp_servers     playbooks       routes
rules           skills          system          terminal
visual_report   workflows
```

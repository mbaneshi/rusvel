# RUSVEL Modular UI Blueprint

> Every Claude Code feature → API endpoint → UI module → User control

## Layout Architecture

```
┌──────────────────────────────────────────────────────────────────────┐
│  Top Bar: Model picker | Effort slider | Cost | Session name        │
├────────┬──────────────────────────────────┬──────────────────────────┤
│ Nav    │  Main Content                    │  Code Sidebar            │
│        │                                  │  ┌──────────────────────┐│
│ Chat   │  (Dashboard / Forge / Code /     │  │ Chat + Streaming     ││
│ Dash   │   Harvest / Content / GTM /      │  │ Tool Calls           ││
│ Forge  │   Settings / Agents / Skills /   │  │ File Preview         ││
│ Code   │   Knowledge / Projects)          │  │ Terminal Output      ││
│ Harvest│                                  │  └──────────────────────┘│
│ Content│                                  │  Tabs:                   │
│ GTM    │                                  │  [Chat] [Tools] [Files]  │
│ ────── │                                  │  [Terminal] [Agents]     │
│ Agents │                                  │                          │
│ Skills │                                  │                          │
│ Plugins│                                  │                          │
│ MCP    │                                  │                          │
│ ────── │                                  │                          │
│ Settings                                  │                          │
└────────┴──────────────────────────────────┴──────────────────────────┘
```

## Module Map

Every module is: **1 SvelteKit route + 1 API endpoint group + 1 Rust handler + 1 DB collection**

### M01: Chat Engine (code sidebar)
```
Route:     Persistent sidebar (always visible)
API:       POST /api/chat/stream
           GET  /api/chat/conversations
           GET  /api/chat/conversations/{id}
           DEL  /api/chat/conversations/{id}
CLI flags: --output-format stream-json --verbose --no-session-persistence
DB:        objects("chat_message")
Status:    ✅ Built
```

### M02: Model & Effort Control
```
Route:     Top bar component
API:       GET  /api/config/models          (available models)
           PUT  /api/config/model           (set active model)
           PUT  /api/config/effort          (set effort level)
CLI flags: --model {model} --effort {level}
DB:        objects("user_config")
Status:    🔲 Not built
```
UI: Dropdown (sonnet/opus/haiku) + slider (low/medium/high/max) + token counter

### M03: Cost & Budget Tracker
```
Route:     Top bar widget + /settings/cost
API:       GET  /api/analytics/cost         (total, per-session, per-day)
           PUT  /api/config/budget          (set budget limit)
CLI flags: --max-budget-usd {amount}
DB:        metrics("token_usage"), metrics("cost")
Status:    🔲 Not built
```
UI: Running cost display, budget cap input, daily/weekly/monthly charts

### M04: Tools & Permissions Manager
```
Route:     /settings/tools + sidebar toggle panel
API:       GET  /api/config/tools           (all available tools + status)
           PUT  /api/config/tools           (enable/disable tools)
           GET  /api/config/permissions     (current permission rules)
           PUT  /api/config/permissions     (update rules)
CLI flags: --allowedTools "..." --disallowedTools "..." --permission-mode {mode}
DB:        objects("tool_config"), objects("permission_rule")
Status:    🔲 Not built
```
UI: Checkbox grid of tools (Read, Write, Edit, Bash, Grep, Glob, WebSearch, WebFetch, Agent)
    Permission mode radio (default/plan/acceptEdits/dontAsk)
    Custom rules editor (allow/deny patterns)

### M05: Agent Gallery & Manager
```
Route:     /agents
API:       GET  /api/agents                 (list all agents)
           POST /api/agents                 (create agent)
           PUT  /api/agents/{id}            (update agent)
           DEL  /api/agents/{id}            (delete agent)
           POST /api/agents/{id}/run        (run agent)
CLI flags: --agents '{json}' --agent {name}
DB:        objects("agent_definition")
Files:     ~/.rusvel/agents/*.md (frontmatter + prompt)
Status:    🔲 Not built
```
UI: Card grid of agents (name, description, model, tools, status)
    Create/edit form (name, prompt, model, tools, effort, memory setting)
    "Hire from Persona" button → browse 112 personas
    Run agent → opens in sidebar chat

Agent definition format (matches Claude Code):
```yaml
---
name: code-reviewer
description: Reviews code for quality and security issues
model: sonnet
tools: Read, Grep, Glob
effort: high
memory: project
---
You are a code reviewer. Review the code for...
```

### M06: Subagent Orchestration
```
Route:     /agents/{id}/subagents
API:       GET  /api/agents/{id}/subagents  (list subagents for an agent)
           POST /api/agents/{id}/dispatch   (dispatch to subagent)
CLI flags: Part of --agents json, agent's sub_agents field
DB:        objects("subagent_run")
Status:    🔲 Not built
```
UI: Visual agent hierarchy (parent → children)
    Dispatch panel (select agent, set input, run)
    Live status of running subagents

### M07: Skill Browser & Editor
```
Route:     /skills
API:       GET  /api/skills                 (list all skills)
           POST /api/skills                 (create skill)
           PUT  /api/skills/{id}            (update skill)
           DEL  /api/skills/{id}            (delete skill)
           POST /api/skills/{id}/run        (execute skill)
Files:     ~/.rusvel/skills/{name}/SKILL.md
DB:        objects("skill_definition")
Status:    🔲 Not built
```
UI: Skill cards (name, description, trigger, scope)
    SKILL.md editor (frontmatter + markdown body)
    Quick-run button
    Import from marketplace

Skill format:
```yaml
---
name: draft-blog-post
description: Draft a blog post about a topic
user-invocable: true
allowed-tools: Read, WebSearch, Edit
model: opus
effort: high
---
Draft a blog post about $ARGUMENTS for Mehdi Baneshi.
Use his profile context. Write in his style: direct, technical, no fluff.
Target platforms: DEV.to, LinkedIn.
```

### M08: Plugin Manager
```
Route:     /plugins
API:       GET  /api/plugins                (installed plugins)
           POST /api/plugins/install        (install from source)
           DEL  /api/plugins/{id}           (uninstall)
           PUT  /api/plugins/{id}/toggle    (enable/disable)
           GET  /api/plugins/marketplace    (browse marketplace)
Files:     ~/.rusvel/plugins/{name}/manifest.json
DB:        objects("plugin")
Status:    🔲 Not built
```
UI: Installed plugins list (name, version, status, toggle)
    Marketplace browser (search, install, update)
    Plugin details (skills, agents, hooks, MCP servers bundled)

Plugin structure:
```
my-plugin/
├── manifest.json    (name, version, description, author)
├── skills/          (SKILL.md files)
├── agents/          (agent definition .md files)
├── hooks/           (hooks.json)
└── mcp/             (MCP server configs)
```

### M09: MCP Server Manager
```
Route:     /settings/mcp
API:       GET  /api/mcp/servers            (configured servers)
           POST /api/mcp/servers            (add server)
           PUT  /api/mcp/servers/{id}       (update config)
           DEL  /api/mcp/servers/{id}       (remove server)
           POST /api/mcp/servers/{id}/test  (health check)
           GET  /api/mcp/tools              (tools from all servers)
CLI flags: --mcp-config {json}
DB:        objects("mcp_server")
Status:    🔲 Not built (rusvel-mcp crate exists for serving, not managing)
```
UI: Server list (name, type, status indicator, tool count)
    Add server form (stdio/http/sse/ws config)
    Tool browser per server
    Health check button

### M10: Hooks & Automation
```
Route:     /settings/hooks
API:       GET  /api/hooks                  (all hooks by event)
           POST /api/hooks                  (create hook)
           PUT  /api/hooks/{id}             (update hook)
           DEL  /api/hooks/{id}             (delete hook)
           GET  /api/hooks/events           (list all hook events)
DB:        objects("hook")
Status:    🔲 Not built
```
UI: Hook list grouped by event (23 events)
    Hook editor (event, matcher, type: command/http/prompt/agent, action)
    Enable/disable toggle per hook
    Execution log

Events: SessionStart, SessionEnd, PreToolUse, PostToolUse, PostToolUseFailure,
        PermissionRequest, Notification, SubagentStart, SubagentStop,
        TeammateIdle, TaskCompleted, InstructionsLoaded, ConfigChange,
        PreCompact, PostCompact, Elicitation, ElicitationResult,
        WorktreeCreate, WorktreeRemove, UserPromptSubmit, Stop, StopFailure

### M11: Project Manager
```
Route:     /projects
API:       GET  /api/projects               (list projects)
           POST /api/projects               (create project)
           PUT  /api/projects/{id}          (update project)
           DEL  /api/projects/{id}          (delete project)
           POST /api/projects/{id}/activate (set as active)
CLI flags: --add-dir {path}
DB:        objects("project")
Status:    🔲 Not built
```
UI: Project cards (name, path, description, last accessed)
    Project settings (default model, tools, agents, CLAUDE.md)
    Multi-project context (combine directories)

### M12: Knowledge Base / RAG
```
Route:     /knowledge
API:       GET  /api/knowledge              (list documents)
           POST /api/knowledge              (ingest document)
           DEL  /api/knowledge/{id}         (remove document)
           POST /api/knowledge/search       (semantic search)
           GET  /api/knowledge/stats        (index stats)
DB:        memory entries + objects("knowledge_doc")
Status:    🔲 Not built (MemoryPort exists for storage)
```
UI: Document list (name, type, size, indexed date)
    Upload / paste / URL import
    Search interface (query → relevant chunks)
    Auto-inject toggle (always include top-K in prompts)

### M13: Context Inspector
```
Route:     Sidebar panel / /settings/context
API:       GET  /api/context/current        (what Claude sees right now)
           GET  /api/context/usage          (token count / window %)
           POST /api/context/compact        (trigger compaction)
DB:        Computed from current session state
Status:    🔲 Not built
```
UI: Token usage bar (used/total with % indicator)
    Context breakdown (system prompt, history, knowledge, tools)
    Compact button
    CLAUDE.md preview
    Memory preview (first 200 lines)

### M14: Session & History Manager
```
Route:     /sessions (or sidebar panel)
API:       GET  /api/sessions               (existing, keep)
           POST /api/sessions               (existing, keep)
           GET  /api/sessions/{id}/history  (full conversation)
           POST /api/sessions/{id}/fork     (fork session)
           DEL  /api/sessions/{id}          (archive)
CLI flags: --resume --continue --fork-session --name
DB:        sessions + objects("chat_message")
Status:    ✅ Partially built (sessions exist, chat history exists)
```
UI: Session list with search
    Resume / fork buttons
    Session tags and bookmarks
    Export conversation

### M15: Terminal / Bash Panel
```
Route:     Sidebar tab
API:       POST /api/terminal/exec          (run command)
           WS   /api/terminal/stream        (live terminal via WebSocket)
DB:        objects("terminal_history")
Status:    🔲 Not built
```
UI: Terminal emulator in sidebar
    Command history
    Output streaming
    Background task indicator

### M16: File Browser & Editor
```
Route:     Sidebar tab
API:       GET  /api/files/tree             (directory listing)
           GET  /api/files/read             (file content)
           PUT  /api/files/write            (write file)
           POST /api/files/search           (grep/glob)
DB:        None (filesystem is the source of truth)
Status:    🔲 Not built
```
UI: File tree (collapsible)
    File viewer (syntax highlighted, read-only by default)
    Edit mode (with save)
    Search panel (grep results)

### M17: Cron & Scheduled Tasks
```
Route:     /settings/automation or sidebar
API:       GET  /api/cron                   (list scheduled)
           POST /api/cron                   (create schedule)
           DEL  /api/cron/{id}              (cancel)
CLI flags: CronCreate/CronDelete/CronList tools
DB:        objects("cron_job")
Status:    🔲 Not built
```
UI: Scheduled task list (name, interval, next run, status)
    Create form (prompt, interval/cron expression)
    Execution log

### M18: Approval Queue
```
Route:     /approvals (or notification badge)
API:       GET  /api/approvals              (pending approvals)
           POST /api/approvals/{id}/approve
           POST /api/approvals/{id}/reject
DB:        objects("approval")
Status:    🔲 Not built (domain types exist)
```
UI: Approval cards (type, requester, data preview)
    Approve/Reject buttons
    History log
    Notification badge in nav

### M19: Self-Awareness Panel
```
Route:     /settings/system
API:       GET  /api/system/status          (engine health, test count, crate info)
           GET  /api/system/gaps            (parsed from gap-analysis.md)
           POST /api/system/self-improve    (run claude -p on own codebase)
           POST /api/system/test            (cargo test)
           POST /api/system/build           (cargo build + npm run build)
DB:        objects("self_improvement_log")
Status:    🔲 Not built
```
UI: System health dashboard
    Engine status (Forge ✅, Code 🔲, Harvest 🔲, etc.)
    Gap list with "Fix this" buttons
    Build/test trigger
    Self-improvement log

## Data Model Summary

All stored in SQLite via ObjectStore:

| Collection | What | Module |
|---|---|---|
| `chat_message` | Conversation history | M01 |
| `user_config` | Model, effort, budget, preferences | M02, M03 |
| `tool_config` | Tool enable/disable, permission rules | M04 |
| `agent_definition` | Agent name, prompt, tools, model | M05, M06 |
| `skill_definition` | Skill content, frontmatter, scope | M07 |
| `plugin` | Installed plugins, status | M08 |
| `mcp_server` | MCP server configs | M09 |
| `hook` | Hook definitions by event | M10 |
| `project` | Project registry | M11 |
| `knowledge_doc` | RAG documents + embeddings | M12 |
| `cron_job` | Scheduled tasks | M17 |
| `approval` | Pending/completed approvals | M18 |
| `self_improvement_log` | Self-build history | M19 |
| `token_usage` (metrics) | Cost tracking | M03 |
| `terminal_history` | Command history | M15 |

## Command Builder (Backend)

The Rust backend constructs claude -p from module state:

```rust
pub struct ChatConfig {
    pub model: String,           // from M02
    pub effort: String,          // from M02
    pub max_budget: Option<f64>, // from M03
    pub allowed_tools: Vec<String>,    // from M04
    pub disallowed_tools: Vec<String>, // from M04
    pub permission_mode: String,       // from M04
    pub system_prompt: String,   // from profile + M12 knowledge
    pub add_dirs: Vec<PathBuf>,  // from M11 projects
    pub mcp_config: Option<String>,    // from M09
    pub agents: Option<String>,        // from M05
    pub max_turns: Option<u32>,
}

impl ChatConfig {
    pub fn to_claude_args(&self) -> Vec<String> {
        let mut args = vec![
            "-p".into(),
            "--output-format".into(), "stream-json".into(),
            "--verbose".into(),
            "--no-session-persistence".into(),
            "--model".into(), self.model.clone(),
            "--effort".into(), self.effort.clone(),
        ];
        if let Some(budget) = self.max_budget {
            args.extend(["--max-budget-usd".into(), budget.to_string()]);
        }
        if !self.allowed_tools.is_empty() {
            args.extend(["--allowedTools".into(), self.allowed_tools.join(" ")]);
        }
        if !self.disallowed_tools.is_empty() {
            args.extend(["--disallowedTools".into(), self.disallowed_tools.join(" ")]);
        }
        args.extend(["--permission-mode".into(), self.permission_mode.clone()]);
        args.extend(["--system-prompt".into(), self.system_prompt.clone()]);
        for dir in &self.add_dirs {
            args.extend(["--add-dir".into(), dir.display().to_string()]);
        }
        if let Some(ref mcp) = self.mcp_config {
            args.extend(["--mcp-config".into(), mcp.clone()]);
        }
        if let Some(ref agents) = self.agents {
            args.extend(["--agents".into(), agents.clone()]);
        }
        if let Some(turns) = self.max_turns {
            args.extend(["--max-turns".into(), turns.to_string()]);
        }
        args
    }
}
```

## Implementation Priority

| Phase | Modules | What you get |
|---|---|---|
| **1** | M02, M03, M04 | Model picker, cost display, tool toggles in chat |
| **2** | M05, M07 | Agent gallery, skill browser — create/run from UI |
| **3** | M11, M14 | Projects, session management |
| **4** | M12, M13 | Knowledge base, context inspector |
| **5** | M08, M09 | Plugins, MCP server manager |
| **6** | M10, M17 | Hooks, cron scheduler |
| **7** | M15, M16 | Terminal, file browser in sidebar |
| **8** | M06, M18 | Subagent orchestration, approval queue |
| **9** | M19 | Self-awareness, self-improvement |

## Modular Component Architecture (Frontend)

```
frontend/src/
├── lib/
│   ├── api.ts                    (API client)
│   ├── stores.ts                 (global stores)
│   ├── types.ts                  (shared TypeScript interfaces)
│   └── components/
│       ├── chat/
│       │   ├── ChatSidebar.svelte    (persistent code sidebar)
│       │   ├── MessageBubble.svelte  (markdown-rendered message)
│       │   ├── StreamingText.svelte  (streaming + cursor)
│       │   └── QuickActions.svelte   (suggestion chips)
│       ├── controls/
│       │   ├── ModelPicker.svelte    (dropdown)
│       │   ├── EffortSlider.svelte   (slider)
│       │   ├── CostBadge.svelte      (running total)
│       │   └── ToolToggles.svelte    (checkbox panel)
│       ├── agents/
│       │   ├── AgentCard.svelte      (agent display)
│       │   ├── AgentEditor.svelte    (create/edit form)
│       │   └── PersonaGallery.svelte (browse + hire)
│       ├── skills/
│       │   ├── SkillCard.svelte
│       │   ├── SkillEditor.svelte    (SKILL.md editor)
│       │   └── SkillRunner.svelte
│       ├── plugins/
│       │   ├── PluginCard.svelte
│       │   └── Marketplace.svelte
│       ├── knowledge/
│       │   ├── DocList.svelte
│       │   ├── DocUpload.svelte
│       │   └── SearchPanel.svelte
│       └── system/
│           ├── ContextBar.svelte     (token usage)
│           ├── HealthPanel.svelte    (engine status)
│           └── SelfImprove.svelte    (build/test triggers)
├── routes/
│   ├── +layout.svelte              (nav + sidebar + topbar)
│   ├── +page.svelte                (dashboard)
│   ├── chat/+page.svelte           (full-page chat, existing)
│   ├── forge/+page.svelte          (existing)
│   ├── code/+page.svelte           (existing)
│   ├── harvest/+page.svelte        (existing)
│   ├── content/+page.svelte        (existing)
│   ├── gtm/+page.svelte            (existing)
│   ├── agents/+page.svelte         (agent gallery)
│   ├── skills/+page.svelte         (skill browser)
│   ├── plugins/+page.svelte        (plugin manager)
│   ├── knowledge/+page.svelte      (knowledge base)
│   ├── projects/+page.svelte       (project manager)
│   ├── approvals/+page.svelte      (approval queue)
│   └── settings/
│       ├── +page.svelte            (general settings)
│       ├── tools/+page.svelte      (tools & permissions)
│       ├── mcp/+page.svelte        (MCP servers)
│       ├── hooks/+page.svelte      (automation)
│       ├── cost/+page.svelte       (analytics)
│       └── system/+page.svelte     (self-awareness)
```

## Backend Module Architecture (Rust)

```
crates/rusvel-api/src/
├── lib.rs              (router + AppState)
├── routes.rs           (existing: sessions, goals, events)
├── chat.rs             (existing: streaming chat)
├── config.rs           (M02: model/effort, M03: budget, M04: tools/perms)
├── agents.rs           (M05: agent CRUD, M06: subagent dispatch)
├── skills.rs           (M07: skill CRUD + run)
├── plugins.rs          (M08: plugin install/manage)
├── mcp.rs              (M09: MCP server management)
├── hooks.rs            (M10: hook CRUD)
├── projects.rs         (M11: project registry)
├── knowledge.rs        (M12: RAG ingestion + search)
├── context.rs          (M13: context inspection)
├── terminal.rs         (M15: command execution)
├── files.rs            (M16: file tree + read/write)
├── cron.rs             (M17: scheduled tasks)
├── approvals.rs        (M18: approval queue)
└── system.rs           (M19: self-awareness + self-build)
```

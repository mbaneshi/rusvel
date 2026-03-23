# RUSVEL Web UI Architecture — Claude Code as a Service

> The browser wraps Claude Code. Claude Code is one tool inside RUSVEL, not the other way around.

## Core Concept

```
┌─────────────────────────────────────────────────────────────────┐
│  Browser (localhost:3000)                                       │
│ ┌──────────┬────────────────────────────────────────────────────┐│
│ │ Sidebar  │  Main Area                                        ││
│ │          │  ┌──────────────────────────────────────────────┐  ││
│ │ Chat     │  │ Chat / Dashboard / Forge / Code / etc.      │  ││
│ │ History  │  │                                              │  ││
│ │ Projects │  │ Whatever page you're on gets AI sidebar      │  ││
│ │ Agents   │  │                                              │  ││
│ │ Settings │  └──────────────────────────────────────────────┘  ││
│ └──────────┴────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
         │
    Axum API (Rust backend)
         │
    ┌────┴────┐
    │claude -p│  ← spawned per request with flags
    └─────────┘
```

## How `claude -p` Becomes Manageable

Every Claude Code feature maps to a `claude -p` flag. The web UI manages these flags through a settings panel, and the Rust backend constructs the correct command.

### The Command Builder

The backend builds a `claude -p` command from user-configurable settings:

```rust
// What the backend constructs per chat request:
claude -p "<prompt>"
  --output-format stream-json
  --verbose
  --model {model}                          // from UI model picker
  --effort {effort}                        // from UI effort slider
  --max-turns {max_turns}                  // from UI settings
  --max-budget-usd {budget}                // from UI budget control
  --permission-mode {mode}                 // from UI permission selector
  --allowedTools "{tools}"                 // from UI tool toggles
  --system-prompt "{profile + context}"    // from profile.toml + knowledge
  --add-dir {project_dirs}                 // from UI project selector
  --mcp-config {mcp_json}                  // from UI MCP manager
  --agents '{agents_json}'                 // from UI agent definitions
```

## Feature → UI Panel Mapping

### Panel 1: Chat (already built)
**What it does:** Primary conversation interface
**Claude flags used:** `--output-format stream-json`, `--verbose`, `--model`, `--effort`
**RUSVEL adds:**
- Conversation history (persisted in DB, loaded as context)
- Profile injection (system prompt from profile.toml)
- Conversation search (FTS5)
- Quick actions ("Plan my day", "Draft a post", etc.)

### Panel 2: Model & Effort
**UI:** Dropdown + slider in chat header
**Claude flags:**
- `--model sonnet|opus|haiku` → model picker dropdown
- `--effort low|medium|high|max` → effort slider
- `--max-budget-usd` → budget input with running total
**RUSVEL adds:**
- Cost tracking per conversation (stored in DB)
- Budget alerts (from safety module)
- Model usage analytics

### Panel 3: Tools & Permissions
**UI:** Settings page or collapsible panel
**Claude flags:**
- `--allowedTools "Bash(git:*) Edit Read"` → checkbox list of tools
- `--disallowedTools "Bash(rm *)"` → deny list
- `--permission-mode default|plan|acceptEdits|dontAsk` → radio buttons
**RUSVEL adds:**
- Per-project tool presets (saved in DB)
- Tool usage analytics
- Permission audit log

### Panel 4: Context & Knowledge
**UI:** Expandable panel showing what Claude knows
**Claude flags:**
- `--system-prompt` → profile.toml + custom instructions
- `--append-system-prompt` → per-conversation context
- `--add-dir` → project directory selector
**RUSVEL adds:**
- Profile editor (edit profile.toml from UI)
- Knowledge base (RAG: store documents, search semantically)
- CLAUDE.md viewer/editor
- Context window usage indicator
- Memory viewer (what auto-memory has stored)

### Panel 5: Projects & Sessions
**UI:** Project switcher in sidebar
**Claude flags:**
- `--add-dir` → multiple project directories
- `--name` → session naming
- `--resume` / `--continue` → session resumption
**RUSVEL adds:**
- Project registry (name, path, description, linked repos)
- Session history with search
- Session bookmarks and tags
- Cross-project context ("when working on Codeilus, also know about ContentForge")

### Panel 6: Agents & Skills
**UI:** Agent gallery + skill browser
**Claude flags:**
- `--agents '{"reviewer":{"prompt":"..."}}'` → agent definitions
- `--agent reviewer` → run as specific agent
**RUSVEL adds:**
- Agent library (CRUD from UI)
- Skill editor (create/edit SKILL.md files)
- Agent templates (from 112 personas in old/)
- Agent usage stats
- One-click "hire" from persona catalog

### Panel 7: MCP Servers
**UI:** Server manager panel
**Claude flags:**
- `--mcp-config` → MCP server definitions
- `--strict-mcp-config` → only use configured servers
**RUSVEL adds:**
- Visual MCP server manager (add/remove/test)
- Server health monitoring
- Tool discovery browser
- Resource browser

### Panel 8: Automation & Hooks
**UI:** Automation builder
**Claude flags:**
- Hooks are configured in settings.json, not CLI flags
- `--bare` skips hooks
**RUSVEL adds:**
- Hook builder UI (event → action)
- Cron job scheduler (visual)
- Workflow templates
- Hook execution log

### Panel 9: Analytics & Cost
**UI:** Dashboard with charts
**No direct Claude flags** — RUSVEL tracks this:
- Token usage per conversation, per day, per project
- Cost breakdown (model × tokens)
- Rate limit status
- Response time tracking
- Most-used tools
- Agent delegation patterns

## Data Model (what RUSVEL persists)

```
projects/          → name, path, description, default_model, default_tools
sessions/          → conversation history, linked project, cost
messages/          → role, content, tokens, cost, tools_used
agents/            → name, prompt, tools, model, persona_id
skills/            → name, content (SKILL.md), project scope
knowledge/         → documents, embeddings, tags (RAG)
hooks/             → event, matcher, action, enabled
mcp_servers/       → name, type, config, health_status
analytics/         → token_usage, cost, tool_calls, response_times
approvals/         → type, status, requester, data
```

All stored in SQLite via existing StoragePort (ObjectStore + MetricStore).

## Implementation Phases

### Phase 1: Chat Enhancement (current → next)
- [x] Basic streaming chat with profile
- [ ] Model picker in chat header
- [ ] Effort slider
- [ ] Cost display per message
- [ ] Tool toggle panel

### Phase 2: Project & Session Management
- [ ] Project registry (CRUD)
- [ ] Session list with search
- [ ] Session resume/fork
- [ ] Cross-session search

### Phase 3: Agent & Skill Management
- [ ] Agent CRUD from UI
- [ ] Persona gallery (browse + hire)
- [ ] Skill editor
- [ ] Agent templates

### Phase 4: Knowledge & RAG
- [ ] Document ingestion
- [ ] Semantic search
- [ ] Auto-inject relevant knowledge into prompts
- [ ] CLAUDE.md editor

### Phase 5: Automation & Analytics
- [ ] Hook builder
- [ ] Cron scheduler
- [ ] Cost analytics dashboard
- [ ] Tool usage charts

## Key Architectural Decisions

1. **claude -p per request** — Each chat message spawns a new `claude -p` process. No persistent Claude session (stateless). History is managed by RUSVEL, injected as context.

2. **Settings in DB, not files** — User preferences (model, effort, tools, permissions) stored in RUSVEL's DB and passed as flags. The web UI is the config editor, not JSON files.

3. **Profile is the identity** — `~/.rusvel/profile.toml` is the single source of "who you are". Everything else (projects, agents, knowledge) is in the DB.

4. **Backend builds the command** — The frontend never constructs claude commands. It sends structured requests (model, effort, tools, message) and the Rust backend builds the exact `claude -p` invocation.

5. **RUSVEL owns the history** — Claude Code's session persistence is disabled (`--no-session-persistence`). RUSVEL manages all conversation history, search, and context injection. This gives us full control over what context Claude sees.

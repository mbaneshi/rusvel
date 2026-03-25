# RUSVEL — Complete Manual Testing Guide

## Prerequisites

- Rust toolchain installed (`cargo build` succeeds)
- Ollama running with at least one model (e.g. `qwen3:14b`, `llama3.1:8b`)
- Frontend built (`cd frontend && pnpm build`) — already embedded in binary

---

## 1. Start the Server

```bash
cargo run
```

Open **http://localhost:3000** in your browser.

---

## 2. Web UI — Pages & Features

### 2.1 Dashboard (`/`)

- 12 department cards with icons and color coding
- Goals display with progress bars
- Analytics overview: agents, skills, rules, conversations counts
- Recent events activity feed grouped by engine
- Active goals list with progress tracking

### 2.2 God Agent Chat (`/chat`)

- Sidebar with conversation history (create new, switch between)
- Type a message → expect SSE-streamed AI response
- Suggested prompts: "Plan my day", "Draft a blog post", "Review my goals", "What should I focus on?"
- Streamdown markdown rendering in responses
- Copy-to-clipboard button on assistant messages

### 2.3 Department Pages (`/dept/{id}`)

Each of the 12 departments has its own page with a **chat panel** + **department panel** side-by-side.

**Chat panel (right side):**
- SSE streaming chat with department-specific AI agent
- Top bar: model selector, effort level (Low/Med/High/Max), tools toggle
- `/help <question>` command interception for contextual help
- `!build` command to generate agents/skills/rules from natural language
- Conversation history switching

**Department panel (left side) — 9 tabs:**

| Tab | What to test | Expected |
|-----|-------------|----------|
| **Actions** | Click quick action buttons, try "Build Capability" | Sends prompt to chat or triggers AI capability discovery |
| **Agents** | Create agent (name, role, model, system prompt), delete | Agent appears in list, persists across page loads |
| **Workflows** | Create workflow with visual WorkflowBuilder, add steps, run | Graph visualization with SvelteFlow, execution shows cost + confetti |
| **Skills** | Create skill (name, description, prompt template), click "use skill" | Skill sends pending command to chat |
| **Rules** | Create rule (name, content), toggle enable/disable | Rules injected into department system prompt |
| **MCP** | Add MCP server (name, type, command), delete | Server appears in list |
| **Hooks** | Create hook (name, event type, action), toggle | Hook fires on matching events |
| **Dirs** | Add/remove working directories | Directory list persists |
| **Events** | View department event log | Shows recent events with timestamps |

**Test each department** (all 12):

| Department | URL | Quick Actions to Try |
|-----------|-----|---------------------|
| Forge | `/dept/forge` | Daily plan, Review progress, Set new goal |
| Code | `/dept/code` | Analyze codebase, Run tests, Find TODOs |
| Harvest | `/dept/harvest` | Scan opportunities, Score pipeline, Draft proposal |
| Content | `/dept/content` | Draft blog post, Adapt for Twitter, Content calendar |
| GTM | `/dept/gtm` | List contacts, Draft outreach, Deal pipeline, Generate invoice |
| Finance | `/dept/finance` | Record income, Log expense, Calculate runway, P&L report |
| Product | `/dept/product` | View roadmap, Add feature, Pricing analysis |
| Growth | `/dept/growth` | Funnel analysis, KPI dashboard, Cohort report |
| Distro | `/dept/distro` | SEO audit, Marketplace listings, Distribution strategy |
| Legal | `/dept/legal` | Draft contract, Compliance check, IP review |
| Support | `/dept/support` | Open tickets, Write KB article, NPS survey |
| Infra | `/dept/infra` | Deploy status, Health check, Incident report |

### 2.4 Knowledge Base (`/knowledge`)

- **Ingest Knowledge**: Enter text + source name → click ingest
- **Semantic Search**: Search query → results with similarity scores
- **Browse All Entries**: Searchable list with delete buttons
- **Stats bar**: Total entries, embedding model, dimensions

### 2.5 Settings (`/settings`)

- **Pending Approvals**: View jobs awaiting approval, approve/reject with JSON payload preview
- **System Health**: Version, API status, LLM provider, database status
- **Engine Status**: Forge, Code, Harvest, Content, GoToMarket engine health indicators

### 2.6 Command Palette (`Cmd+K` / `Ctrl+K`)

- Opens overlay with search input
- 15+ navigation commands (all pages + 12 departments)
- 3 action commands: new session, generate plan, new chat
- Arrow keys to navigate, Enter to select, Escape to close

### 2.7 Onboarding

- **Getting Started Checklist** (bottom-left widget):
  - Steps: create session, add goal, generate plan, dept chat, create agent
  - Progress bar, auto-detects completion, dismissible
- **Product Tour**: Step-by-step driver.js guided tour highlighting sidebar, session switcher, Forge, Chat, Settings
- **DeptHelpTooltip**: Per-department contextual help

### 2.8 Layout Features

- Resizable sidebar (drag handle, collapsible to icon-only mode)
- Session switcher dropdown in sidebar
- New session creation form (name + kind selector)
- Navigation to all 14 items (Dashboard, Chat, Settings, Knowledge + 12 departments)
- Toast notifications (svelte-sonner)

---

## 3. CLI — Tier 1: One-shot Commands

```bash
# Help
cargo run -- --help

# Session management
cargo run -- session create "test-session"
cargo run -- session list
cargo run -- session switch <ID>

# Forge mission planning (needs Ollama)
cargo run -- forge mission today
cargo run -- forge mission goals
cargo run -- forge mission goal add "Ship MVP" --description "Launch v1" --timeframe week
cargo run -- forge mission review
cargo run -- forge mission review --period month

# Department commands (all 11 departments)
# Each supports: status, list [--kind X] [--limit N], events [--limit N]

cargo run -- finance status
cargo run -- finance list
cargo run -- finance events
cargo run -- finance events --limit 20

cargo run -- code status
cargo run -- code list
cargo run -- code events

cargo run -- harvest status
cargo run -- harvest list
cargo run -- harvest events

cargo run -- content status
cargo run -- content list
cargo run -- content events

cargo run -- gtm status
cargo run -- gtm list
cargo run -- gtm events

cargo run -- growth status
cargo run -- growth list
cargo run -- growth list --kind funnel
cargo run -- growth events

cargo run -- product status
cargo run -- product list
cargo run -- product events

cargo run -- distro status
cargo run -- distro list
cargo run -- distro events

cargo run -- legal status
cargo run -- legal list
cargo run -- legal events

cargo run -- support status
cargo run -- support list
cargo run -- support events

cargo run -- infra status
cargo run -- infra list
cargo run -- infra events
```

---

## 4. CLI — Tier 2: Interactive REPL

```bash
cargo run -- shell
```

### Commands to test

| Command | Expected |
|---------|----------|
| `help` or `?` | Show available commands |
| `status` | Show all departments summary |
| `use finance` | Switch into Finance department context |
| `status` (inside dept) | Show department-specific summary |
| `list` (inside dept) | List department items |
| `list funnel` (inside dept) | List items filtered by kind |
| `events` (inside dept) | Show department events |
| `back` or `..` | Leave department context |
| `session list` | List all sessions |
| `session create "my-session"` | Create a new session |
| `session switch <ID>` | Switch active session |
| `dashboard` | Info about TUI dashboard |
| `exit` / `quit` / `q` / Ctrl+D | Exit the shell |
| Tab | Autocomplete commands |
| Ctrl+R | History search |

---

## 5. CLI — Tier 3: TUI Dashboard

```bash
cargo run -- --tui
```

- Full-screen terminal UI (ratatui) with 4 panels: Tasks, Goals, Pipeline, Events
- Navigate between panels
- Press `q` to exit

---

## 6. MCP Server

```bash
cargo run -- --mcp
```

Starts a stdio JSON-RPC server for MCP client integration (e.g. Claude Code). Waits for JSON-RPC input on stdin.

---

## 7. API Routes (~79 routes)

While the server is running (`cargo run`), test with curl:

### Health

```bash
curl http://localhost:3000/api/health
```

### Sessions & Mission Planning

```bash
curl http://localhost:3000/api/sessions
curl -X POST http://localhost:3000/api/sessions \
  -H "Content-Type: application/json" \
  -d '{"name":"test-session","kind":"default"}'
curl http://localhost:3000/api/sessions/<ID>
curl http://localhost:3000/api/sessions/<ID>/mission/today
curl http://localhost:3000/api/sessions/<ID>/mission/goals
curl -X POST http://localhost:3000/api/sessions/<ID>/mission/goals \
  -H "Content-Type: application/json" \
  -d '{"title":"Ship MVP","description":"Launch v1","timeframe":"week"}'
curl http://localhost:3000/api/sessions/<ID>/events
```

### God Agent Chat

```bash
# SSE stream
curl -N -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"hello"}'

curl http://localhost:3000/api/chat/conversations
curl http://localhost:3000/api/chat/conversations/<ID>
```

### Department Chat & Config

```bash
# Department SSE chat
curl -N -X POST http://localhost:3000/api/dept/finance/chat \
  -H "Content-Type: application/json" \
  -d '{"message":"show me my runway"}'

curl http://localhost:3000/api/dept/finance/chat/conversations
curl http://localhost:3000/api/dept/finance/chat/conversations/<ID>
curl http://localhost:3000/api/dept/finance/config
curl http://localhost:3000/api/dept/finance/events
```

### Department Registry

```bash
curl http://localhost:3000/api/departments
```

### Profile

```bash
curl http://localhost:3000/api/profile
curl -X PUT http://localhost:3000/api/profile \
  -H "Content-Type: application/json" \
  -d '{"name":"Mehdi","email":"mehdi@example.com"}'
```

### Config

```bash
curl http://localhost:3000/api/config
curl -X PUT http://localhost:3000/api/config \
  -H "Content-Type: application/json" \
  -d '{"model":"qwen3:14b"}'
curl http://localhost:3000/api/config/models
curl http://localhost:3000/api/config/tools
```

### Agents CRUD

```bash
curl http://localhost:3000/api/agents
curl http://localhost:3000/api/agents?engine=finance
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{"name":"Tax Agent","role":"tax advisor","engine":"finance","system_prompt":"You help with taxes"}'
curl http://localhost:3000/api/agents/<ID>
curl -X PUT http://localhost:3000/api/agents/<ID> \
  -H "Content-Type: application/json" \
  -d '{"name":"Tax Agent v2","role":"senior tax advisor"}'
curl -X DELETE http://localhost:3000/api/agents/<ID>
```

### Skills CRUD

```bash
curl http://localhost:3000/api/skills
curl http://localhost:3000/api/skills?engine=code
curl -X POST http://localhost:3000/api/skills \
  -H "Content-Type: application/json" \
  -d '{"name":"Analyze PR","description":"Review a pull request","engine":"code","prompt":"Analyze this PR: {{input}}"}'
curl http://localhost:3000/api/skills/<ID>
curl -X PUT http://localhost:3000/api/skills/<ID> \
  -H "Content-Type: application/json" \
  -d '{"name":"Analyze PR v2"}'
curl -X DELETE http://localhost:3000/api/skills/<ID>
```

### Rules CRUD

```bash
curl http://localhost:3000/api/rules
curl http://localhost:3000/api/rules?engine=content
curl -X POST http://localhost:3000/api/rules \
  -H "Content-Type: application/json" \
  -d '{"name":"Tone Guide","content":"Always write in professional tone","engine":"content"}'
curl http://localhost:3000/api/rules/<ID>
curl -X PUT http://localhost:3000/api/rules/<ID> \
  -H "Content-Type: application/json" \
  -d '{"content":"Write in a friendly professional tone"}'
curl -X DELETE http://localhost:3000/api/rules/<ID>
```

### MCP Servers CRUD

```bash
curl http://localhost:3000/api/mcp-servers
curl http://localhost:3000/api/mcp-servers?engine=code
curl -X POST http://localhost:3000/api/mcp-servers \
  -H "Content-Type: application/json" \
  -d '{"name":"GitHub","type":"stdio","command":"npx @modelcontextprotocol/server-github","engine":"code"}'
curl -X PUT http://localhost:3000/api/mcp-servers/<ID> \
  -H "Content-Type: application/json" \
  -d '{"name":"GitHub v2"}'
curl -X DELETE http://localhost:3000/api/mcp-servers/<ID>
```

### Hooks CRUD

```bash
curl http://localhost:3000/api/hooks
curl http://localhost:3000/api/hooks?engine=forge&event=PostToolUse
curl http://localhost:3000/api/hooks/events
curl -X POST http://localhost:3000/api/hooks \
  -H "Content-Type: application/json" \
  -d '{"name":"Log chat","event":"PostToolUse","action":"echo done","engine":"forge"}'
curl -X PUT http://localhost:3000/api/hooks/<ID> \
  -H "Content-Type: application/json" \
  -d '{"name":"Log chat v2"}'
curl -X DELETE http://localhost:3000/api/hooks/<ID>
```

### Workflows CRUD + Execution

```bash
curl http://localhost:3000/api/workflows
curl -X POST http://localhost:3000/api/workflows \
  -H "Content-Type: application/json" \
  -d '{"name":"Content Pipeline","steps":[{"agent":"writer","prompt":"Draft a blog post about {{input}}"},{"agent":"editor","prompt":"Edit this draft"}]}'
curl http://localhost:3000/api/workflows/<ID>
curl -X PUT http://localhost:3000/api/workflows/<ID> \
  -H "Content-Type: application/json" \
  -d '{"name":"Content Pipeline v2"}'
curl -X POST http://localhost:3000/api/workflows/<ID>/run \
  -H "Content-Type: application/json" \
  -d '{"input":"Rust async patterns"}'
curl -X DELETE http://localhost:3000/api/workflows/<ID>
```

### Approvals

```bash
curl http://localhost:3000/api/approvals
curl -X POST http://localhost:3000/api/approvals/<ID>/approve
curl -X POST http://localhost:3000/api/approvals/<ID>/reject
```

### Knowledge Base

```bash
curl http://localhost:3000/api/knowledge
curl http://localhost:3000/api/knowledge/stats
curl -X POST http://localhost:3000/api/knowledge/ingest \
  -H "Content-Type: application/json" \
  -d '{"text":"RUSVEL is an AI-powered virtual agency","source":"docs"}'
curl -X POST http://localhost:3000/api/knowledge/search \
  -H "Content-Type: application/json" \
  -d '{"query":"virtual agency"}'
curl -X DELETE http://localhost:3000/api/knowledge/<ID>
```

### Capability Discovery

```bash
curl -X POST http://localhost:3000/api/capability/build \
  -H "Content-Type: application/json" \
  -d '{"prompt":"I need an agent that tracks expenses"}'
```

### Analytics

```bash
curl http://localhost:3000/api/analytics
```

### Help

```bash
curl -X POST http://localhost:3000/api/help \
  -H "Content-Type: application/json" \
  -d '{"question":"How do I create a workflow?"}'
```

---

## 8. Automated Tests

```bash
cargo test                     # All 98 test suites
cargo test -p rusvel-core      # Core types and ports
cargo test -p rusvel-db        # Database store
cargo test -p rusvel-api       # API routes
cargo test -p forge-engine     # Agent orchestration
cargo test -p content-engine   # Content engine
cargo test -p harvest-engine   # Harvest engine
```

---

## 9. End-to-End Flows

### Flow 1: Session → Goal → Daily Plan

1. Create a session: sidebar → new session form (or `cargo run -- session create "my-project"`)
2. Navigate to `/dept/forge`
3. Click "Set new goal" quick action → chat creates a goal
4. Click "Daily plan" → AI generates prioritized daily plan
5. Verify goal appears on dashboard (`/`) with progress bar

### Flow 2: Department Chat → Agent → Skill → Rule

1. Go to `/dept/finance`
2. In Agents tab: create an agent (name: "Tax Agent", role: "tax advisor")
3. In Skills tab: create a skill (name: "Tax Estimate", prompt: "Estimate taxes for {{input}}")
4. In Rules tab: create a rule (name: "Currency", content: "Always use USD")
5. Click "use skill" on Tax Estimate → verify it sends prompt to chat
6. Chat with the department → verify agent responds with rule applied

### Flow 3: Workflow Creation & Execution

1. Go to any department (e.g. `/dept/content`)
2. Open Workflows tab
3. Create a workflow: add multiple steps in the visual WorkflowBuilder
4. Verify graph renders with SvelteFlow (nodes + edges)
5. Run the workflow → observe step-by-step execution, cost display, confetti

### Flow 4: Knowledge Base Round-trip

1. Go to `/knowledge`
2. Ingest some text with a source name
3. Search for it → verify it appears with similarity score
4. Browse all entries → verify it's listed
5. Delete it → verify it's removed

### Flow 5: Capability Discovery (!build)

1. Go to any department chat
2. Type `!build create an agent that reviews pull requests`
3. Expect: AI discovers and installs agent/skills/rules as a JSON bundle

### Flow 6: Approval Workflow

1. Trigger an action that requires approval (content publishing, outreach)
2. Go to `/settings` → Pending Approvals section
3. Review JSON payload → approve or reject

### Flow 7: Multi-Department Navigation

1. Open Command Palette (Cmd+K)
2. Search "finance" → navigate to Finance
3. Search "code" → navigate to Code
4. Verify each department has its own color theme, icon, and quick actions

---

## Quick Start

Run `cargo run` and open **http://localhost:3000**. Follow Flow 1 (session → goal → plan), then explore department pages. Use Cmd+K to jump between departments. Try a few CLI commands in another terminal. That covers the core surface area.

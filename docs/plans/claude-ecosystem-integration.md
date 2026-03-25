# Claude Ecosystem Integration Plan

> Integrating patterns from `claude-hud` and `claude-peers-mcp` into Rusvel
> Date: 2026-03-25

---

## Source Repos

| Repo | What it does | Stars |
|------|-------------|-------|
| [jarrodwatts/claude-hud](https://github.com/jarrodwatts/claude-hud) | Claude Code statusline plugin — context health, tool tracking, agent tracking, todo progress | ~12.8k |
| [louislva/claude-peers-mcp](https://github.com/louislva/claude-peers-mcp) | MCP server enabling peer-to-peer messaging between Claude Code instances via broker daemon | ~1.1k |

---

## 1. Claude HUD — Patterns for Rusvel

### 1A. TUI Dashboard Observability (rusvel-tui)

**Problem:** Rusvel's TUI (`--tui`) has 4 panels (Tasks, Goals, Pipeline, Events) but no real-time agent/tool activity tracking.

**Pattern to adopt:** HUD's transcript parsing + live activity rendering.

**Implementation:**
- Add an **Activity Panel** to `rusvel-tui` showing:
  - Running agents (which department, which persona, elapsed time)
  - Active tools (which built-in tool is executing, target)
  - Job queue progress (pending/running/completed counts)
  - Event stream (last N events with department color coding)
- Parse `rusvel-event` bus in real-time (Rusvel already has event persistence)
- Color-code by health: green (normal), yellow (slow), red (error/timeout)

**Files to modify:**
- `crates/rusvel-tui/` — Add activity panel widget
- `crates/rusvel-event/` — Add subscription API for live event streaming

**Effort:** Medium. Rusvel already has event persistence; need to wire it into TUI rendering.

---

### 1B. Context Health for Agent Sessions

**Problem:** Rusvel agents (via `rusvel-agent`) have no visibility into LLM context consumption during multi-turn conversations.

**Pattern to adopt:** HUD's context percentage calculation + threshold warnings.

**Implementation:**
- Track token usage per agent session in `rusvel-agent`
- Store `input_tokens`, `output_tokens`, `cache_tokens` per turn
- Calculate context % relative to model's window (Ollama, Claude, OpenAI each differ)
- Expose via `/api/chat/sessions/:id/health` endpoint
- Frontend: show context health bar in chat UI per department
- Auto-summarize/compress when approaching 80% (like HUD's autocompact buffer)

**Files to modify:**
- `crates/rusvel-agent/` — Token tracking per session
- `crates/rusvel-llm/` — Return token counts from all providers
- `crates/rusvel-api/src/chat.rs` — Health endpoint
- `frontend/src/routes/dept/[id]/` — Context health bar component

**Effort:** Medium-High. Requires LLM provider changes to return token metadata.

---

### 1C. Git-Aware Code Engine

**Problem:** `code-engine` analyzes code but doesn't track git state.

**Pattern to adopt:** HUD's git integration (branch, dirty, ahead/behind, file stats).

**Implementation:**
- Add `GitStatus` to `code-engine`'s analysis output
- Track: branch, dirty files, ahead/behind remote, file change stats
- Show in department UI for Code department
- Use for smarter code analysis: prioritize dirty files, show diff context

**Files to modify:**
- `crates/code-engine/` — Add git status module
- `crates/rusvel-api/src/engine_routes.rs` — Include git data in code analysis response

**Effort:** Low. Git commands are straightforward; code-engine already shells out for analysis.

---

## 2. Claude Peers MCP — Patterns for Rusvel

### 2A. Inter-Department Agent Messaging (High Priority)

**Problem:** Department engines communicate only through the event bus (fire-and-forget). No request/response pattern between departments.

**Pattern to adopt:** Peers' broker + message queue + peer discovery.

**Implementation:**

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ Harvest Eng.  │     │  Agent Broker │     │ Content Eng.  │
│              │────>│  (in-process)  │────>│              │
│ "Found opp,  │     │  SQLite queue  │     │ "Draft blog   │
│  need content"│     │  + routing     │     │  from opp"    │
└──────────────┘     └──────────────┘     └──────────────┘
                            │
                     ┌──────┴──────┐
                     │  Code Eng.   │
                     │ "Analyze     │
                     │  repo first" │
                     └─────────────┘
```

**Design:**
- **AgentBroker** in `rusvel-agent` — in-process message router (no separate daemon needed; Rusvel is a single binary)
- **DepartmentPeer** trait in `rusvel-core`:
  ```rust
  #[async_trait]
  pub trait DepartmentPeer: Send + Sync {
      fn department_id(&self) -> &str;
      fn summary(&self) -> String;  // What this dept is working on
      async fn receive_message(&self, from: &str, msg: DeptMessage) -> Result<DeptResponse>;
  }
  ```
- **Message types:**
  - `RequestAnalysis` — Ask another dept to analyze something
  - `ShareResult` — Push a result to another dept
  - `Coordinate` — Multi-dept workflow step
- **SQLite message queue** (reuse `rusvel-db`) for persistence + delivery tracking
- **Schema** (adapted from peers-mcp):
  ```sql
  CREATE TABLE dept_messages (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      from_dept TEXT NOT NULL,
      to_dept TEXT NOT NULL,
      msg_type TEXT NOT NULL,
      payload TEXT NOT NULL,        -- JSON
      session_id TEXT NOT NULL,
      sent_at TEXT NOT NULL,
      delivered INTEGER DEFAULT 0,
      response TEXT                 -- JSON response when fulfilled
  );
  ```

**Use cases unlocked:**
- Harvest discovers opportunity → sends `RequestAnalysis` to Content → Content drafts blog post
- Code analyzes repo → sends `ShareResult` to Growth → Growth updates metrics
- Forge orchestrates: asks Finance for runway, Product for roadmap, then plans sprint

**Files to modify:**
- `crates/rusvel-core/src/ports/` — Add `DepartmentPeer` trait
- `crates/rusvel-agent/` — Add `AgentBroker` module
- `crates/rusvel-db/` — Add `dept_messages` table + migration
- `crates/forge-engine/` — Wire broker for God Agent orchestration
- Each engine that participates — implement `DepartmentPeer`

**Effort:** High. Core architectural addition, but unlocks the key vision of departments collaborating autonomously.

---

### 2B. Peer Discovery for Multi-Instance Rusvel

**Problem:** Running multiple Rusvel instances (e.g., one per project) can't communicate.

**Pattern to adopt:** Peers' broker daemon + registration + scope-based discovery.

**Implementation:**
- Optional `--broker` flag on `rusvel` binary to start broker mode
- Broker listens on configurable port (default: 7899)
- Other Rusvel instances register on startup, heartbeat every 15s
- Scope-based discovery: by machine, by project, by git repo
- Use case: multiple Rusvel instances managing different repos can share learnings

**Schema** (adapted from peers-mcp):
```sql
CREATE TABLE rusvel_peers (
    id TEXT PRIMARY KEY,
    pid INTEGER NOT NULL,
    project_root TEXT NOT NULL,
    git_root TEXT,
    summary TEXT NOT NULL,
    registered_at TEXT NOT NULL,
    last_seen TEXT NOT NULL
);
```

**Files to modify:**
- `crates/rusvel-app/` — Add `--broker` flag
- `crates/rusvel-api/` — Add peer registration routes
- New module in `rusvel-agent` — Peer client

**Effort:** Medium. Nice-to-have after 2A is done. Only valuable when running multiple instances.

---

### 2C. Auto-Summary for Department Context

**Problem:** God Agent (forge-engine) routes requests to departments but doesn't know their current state/focus.

**Pattern to adopt:** Peers' `set_summary` + auto-summary generation.

**Implementation:**
- Each engine auto-generates a status summary after significant actions:
  ```
  "Content: 3 drafts in review, 1 published today. Focus: blog series on AI tooling."
  "Harvest: 12 opportunities in pipeline, 4 scored >0.8. Scanning ProductHunt."
  "Code: Last analyzed rusvel-api (432 functions, 12 issues). BM25 index current."
  ```
- Store in `DepartmentRegistry` (already exists in `rusvel-core`)
- God Agent reads summaries before routing → smarter delegation
- Expose via `/api/departments` with live summary field
- Frontend: show department cards with live status

**Files to modify:**
- `crates/rusvel-core/src/registry.rs` — Add `summary` field to department metadata
- Each engine — implement `fn summary(&self) -> String`
- `crates/rusvel-api/src/department.rs` — Return summaries
- `frontend/src/routes/+page.svelte` — Show live dept status

**Effort:** Low-Medium. Each engine just needs a `summary()` method. High value for UX.

---

### 2D. Channel-Push Pattern for Real-Time Frontend

**Problem:** Frontend uses polling for chat and department updates.

**Pattern to adopt:** Peers' channel notification protocol (instant message push).

**Implementation:**
- Rusvel already has SSE for chat streaming
- Extend SSE to push department events, job completions, inter-dept messages
- Frontend subscribes to `/api/events/stream` (global) or `/api/dept/:id/events/stream`
- When AgentBroker routes a message, push notification to all connected frontends
- Show toast/notification: "Harvest sent analysis request to Content"

**Files to modify:**
- `crates/rusvel-api/` — Add global SSE event stream
- `crates/rusvel-event/` — Add subscriber notification
- `frontend/src/lib/` — EventSource client for live updates
- `frontend/src/routes/+layout.svelte` — Global notification toast

**Effort:** Medium. SSE infrastructure exists; need to generalize beyond chat.

---

## 3. Combined: Orchestration Dashboard

Combining both repos' patterns into a unified experience:

```
┌─────────────────────────────────────────────────────┐
│ Rusvel Dashboard                                     │
├─────────────┬───────────────────────────────────────┤
│ Departments │  Activity Feed (from HUD patterns)     │
│             │  ┌─────────────────────────────────┐   │
│ ● Forge     │  │ ◐ Content drafting blog post     │   │
│   Planning  │  │ ◐ Harvest scanning ProductHunt   │   │
│ ● Content   │  │ ✓ Code analyzed rusvel-api        │   │
│   3 drafts  │  │ ◐ Forge coordinating sprint       │   │
│ ● Harvest   │  │                                   │   │
│   12 opps   │  │ Messages (from Peers patterns)    │   │
│ ● Code      │  │ Harvest → Content: "Draft from    │   │
│   Idle      │  │   opp #42 (AI tooling)"           │   │
│ ● Finance   │  │ Content → Forge: "Draft ready     │   │
│   Healthy   │  │   for review"                     │   │
│             │  └─────────────────────────────────┘   │
│             │                                        │
│             │  Context Health (from HUD patterns)    │
│             │  Forge: ████████░░ 78% (warn)          │
│             │  Content: ███░░░░░░░ 28%               │
│             │  Harvest: █████░░░░░ 52%               │
├─────────────┴───────────────────────────────────────┤
│ Jobs: 3 running │ Events: 47 today │ Errors: 0      │
└─────────────────────────────────────────────────────┘
```

---

## 4. Implementation Priority

| # | Feature | Source | Effort | Value | Priority |
|---|---------|--------|--------|-------|----------|
| 1 | Inter-Dept Messaging (2A) | peers-mcp | High | Critical | **P0** |
| 2 | Auto-Summary for Depts (2C) | peers-mcp | Low-Med | High | **P1** |
| 3 | Real-Time Event Push (2D) | peers-mcp | Medium | High | **P1** |
| 4 | TUI Activity Panel (1A) | claude-hud | Medium | Medium | **P2** |
| 5 | Context Health Tracking (1B) | claude-hud | Med-High | Medium | **P2** |
| 6 | Git-Aware Code Engine (1C) | claude-hud | Low | Low-Med | **P3** |
| 7 | Multi-Instance Peers (2B) | peers-mcp | Medium | Low | **P3** |

---

## 5. Phase Plan

### Phase A: Department Peer Protocol (P0 + P1)
1. Add `DepartmentPeer` trait to `rusvel-core`
2. Add `dept_messages` table to `rusvel-db`
3. Build `AgentBroker` in `rusvel-agent`
4. Wire forge-engine as orchestrator
5. Implement `summary()` on 5 wired engines
6. Add `/api/departments` with live summaries
7. Frontend department cards with status

### Phase B: Real-Time Observability (P1 + P2)
1. Generalize SSE beyond chat
2. Push dept events + inter-dept messages to frontend
3. Add activity feed component
4. Add context health tracking to `rusvel-agent`
5. TUI activity panel

### Phase C: Extended Integration (P3)
1. Git-aware code analysis
2. Multi-instance peer discovery (optional)
3. Cross-project knowledge sharing

---

## 6. Key Takeaways

| From claude-hud | From claude-peers-mcp |
|---|---|
| Transcript parsing → Activity tracking | Broker pattern → Inter-dept messaging |
| Context % calculation → Session health | Peer discovery → Department awareness |
| Modular rendering → TUI widgets | Channel push → Real-time frontend |
| Git integration → Code engine enhancement | Auto-summary → Smarter God Agent routing |
| Color thresholds → Health visualization | SQLite message queue → Async dept coordination |

Both repos solve problems that map directly to Rusvel's architecture. The highest-leverage integration is the **inter-department messaging pattern** from peers-mcp — it transforms departments from isolated silos into a collaborative network, which is the core vision of Rusvel as a "virtual agency."

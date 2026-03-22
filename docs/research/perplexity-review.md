# Perplexity Architecture Review — 2026-03-21

> Raw feedback from Perplexity on the RUSVEL design brief.

---

## Key Critiques (Action Required)

### 1. Event Storm Risk
EventPort + SchedulePort + AutomationPort + 7 engines = distributed system inside one process.
Hard to reason about, debug, and test.

**Action:** Single canonical workflow substrate. All async work goes through one job queue.

### 2. Overlapping Workflow Concerns
AutomationPort vs Forge Engine vs SchedulePort vs Ops/Mission workflows are ALL "workflow engines."
Risk: 4 overlapping workflow DSLs.

**Action:** Force everything through a single automation substrate (jobs + flows).
- SchedulePort creates jobs
- AutomationPort defines flows (sequences of jobs)
- Forge Engine is a special flow executor (agent workflows)
- Ops/Mission consume the job system, don't re-implement it

### 3. Too Many Top-Level Engines (7 is a lot)
They'll be thin or unfinished.

**Action:** Consider merging:
- Ops + Connect → **GoToMarket Engine** (CRM + outreach + pipeline)
- Mission could fold into Forge (mission as orchestrated agents)

**Decision needed:** Do we merge to 5 engines or keep 7 but accept some will be thin initially?

### 4. StoragePort Too Broad
"Persist anything" → re-inventing a typed repository layer.

**Action:** Define canonical stores:
- Events store
- Objects store (content/opportunity/contact/etc.)
- Sessions store
- Schedules/Jobs store
- Metrics store

### 5. Code Engine Scope Too Large
Parsing 12+ languages, dependency graphs, metrics, anti-patterns, AND learning paths = a whole product.

**Action:** Thin first version: one language (Rust) + symbol graph + BM25 search. Expand later.

### 6. LlmPort vs AgentPort Boundary Unclear
Where do prompting, tool selection, and retries live?

**Action:** Be strict:
- LlmPort = raw model access (generate, stream, embed). No business logic.
- AgentPort = orchestration layer (prompt construction, tool selection, retries, memory injection)
- Never call LlmPort directly from an engine; always go through AgentPort.

---

## SessionPort Design (Perplexity's Recommendation)

Three-level hierarchy:

```
Session (durable workspace/project)
└── Run (specific execution: agent run, workflow, analysis)
    └── Thread (conversational/log stream within a run)
```

Core types:
- `SessionId`, `RunId`, `ThreadId` newtypes
- `Session` { id, name, kind, tags, config_ref, created_at, updated_at }
- `SessionKind` { Project, Lead, ContentCampaign, General }
- `Run` { id, session_id, engine, input_summary, status, llm_budget_used, tool_calls_count }
- `RunStatus` { Queued, Running, Succeeded, Failed, Cancelled }
- `Thread` { id, run_id, channel }
- `ThreadChannel` { User, Agent, System, Event }

Session responsibilities:
- Attach config, auth scopes, default tools/agents
- Provide namespaced memory view: `memory_for_session(SessionId)`
- Provide filtered event stream per session
- Provide automation view: schedules, workflows, tasks per session

Key UX: "Open 'Client: ACME' session → link repo + opportunities + content + outreach → everything scoped there."

---

## Shared Domain Types (Perplexity's Additions)

### New types to add to rusvel-core:
- `UserId`, `WorkspaceId` (model even for single user)
- `CredentialRef` (opaque handle, engines never see raw tokens)
- `ModelProvider`, `ModelRef` (provider + model name)
- `AgentProfile` (name, role, instructions, default_model, allowed_tools, capabilities)
- `ContentKind` (LongForm, Tweet, Thread, LinkedInPost, Blog, VideoScript, Email, Proposal)
- `MessageChannel` (Email, LinkedInDM, TwitterDM, SMS, GenericWebhook)
- `OutboundMessage` (channel, to, from, subject, body, status)
- `ScheduleKind` (Cron, Interval, OneOff, EventTriggered)
- `AutomationTarget` (AgentRun, ContentPublish, OutreachSequence, Custom)
- `WorkflowRef` (opaque handle into automation definitions)
- `RepoRef` (local path + optional remote)
- `CodeSnapshotRef` (point-in-time analysis reference)
- `Task` (id, session_id, goal_id, title, status, due_at, priority)

### Schema evolution strategy:
Base columns + `metadata: serde_json::Value` on all domain types.
Add fields without breaking older engines.

---

## Patterns to Adopt

### From ADK-Rust:
- Application → Runner → Agent → Service layer mapping
- Agent descriptors: typed configs stored in StoragePort and versioned
- Internal "AgentCall" message type for agent-to-agent communication via EventPort

### From Windmill:
- Central job queue in SQLite (status, retries, next_run_at, payload)
- Worker pool pops jobs off and executes them
- Triggers as first-class resources (not per-engine scheduling)
- "Script" vs "Flow" maps to ToolPort vs AutomationPort

### From Codeilus:
- Separate "indexer" from "analyzer"
- Code graph as its own store with IDs
- Multi-stage pipeline, each stage independently runnable

### From Content/Business OS:
- Context packs: standardized context for agents (project brief, recent commits, top opportunities)
- Documentation as first-class artifact in MemoryPort

---

## Top 3 Risks

1. **Scope explosion / unfinished engines** — Beautiful architecture, no killer workflow
2. **Workflow duplication** — 4 overlapping workflow DSLs
3. **Observability gap** — Without tracing/run IDs/replay, multi-agent debugging is hell

---

## Blind Spots Identified

1. **Human-in-the-loop approvals** — "Agent proposes, human approves" for outreach/publishing. Model approvals explicitly in domain types.

2. **Safety policies as reusable resources** — Per-session and per-run: token/$ limits, allowed providers, red-team rules.

3. **Async-first** — Make as much as possible job-based. Only keep truly interactive stuff (chat, TUI) synchronous.

4. **Schema evolution** — Base columns + metadata JSON for field additions without migrations.

5. **"Inbox/Capture" domain** — Lightweight funnel for emails, links, docs, tasks into Sessions. The boring but critical stuff.

6. **The one thing to nail:** Session-centric UX. Every action, agent, mission, and content clearly lives in a session with a legible timeline and state.

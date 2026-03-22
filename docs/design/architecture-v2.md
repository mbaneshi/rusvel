# RUSVEL — Architecture v2 (Post-Review)

> Incorporates Perplexity feedback. This supersedes architecture.md.

---

## What Changed from v1

| v1 | v2 | Why |
|----|-----|-----|
| 7 engines | 5 engines | Ops + Connect merged → GoToMarket. Mission folded into Forge. |
| AutomationPort + SchedulePort separate | Single Job Queue substrate | Eliminate 4 overlapping workflow DSLs |
| StoragePort "persist anything" | 5 canonical stores | Clear boundaries, no re-inventing ORM |
| Code Engine: 12+ languages | Code Engine v0: Rust-only + symbol graph + BM25 | Ship thin, expand later |
| Engines call LlmPort directly | Engines go through AgentPort only | Clean boundary: LlmPort = raw, AgentPort = orchestration |
| No Session hierarchy | Session → Run → Thread | Everything keyed by session |
| No approval model | Explicit human-in-the-loop approvals | Solo founder = "propose then approve" |
| No job queue | Central SQLite job queue | All async work goes through one substrate |
| No Inbox/Capture | Inbox as cross-cutting concern | Funnel everything into sessions |

---

## Revised Architecture

```
┌─────────────────── SURFACES ───────────────────┐
│  CLI (Clap)  │  TUI (Ratatui)  │  Web (Svelte) │
│                  MCP Server                      │
└──────────────────────┬─────────────────────────┘
                       │
┌──────────────────────┴─────────────────────────┐
│              DOMAIN ENGINES (5)                  │
│                                                 │
│  Forge         │ Code       │ Harvest           │
│  (+ Mission)   │            │                   │
│                │            │                   │
│  Content       │ GoToMarket                     │
│                │ (Ops+Connect+Outreach)          │
└──────────────────────┬─────────────────────────┘
                       │ uses (traits only)
┌──────────────────────┴─────────────────────────┐
│              FOUNDATION                          │
│                                                 │
│  ┌──────────── rusvel-core ──────────────┐      │
│  │  Port Traits + Shared Domain Types    │      │
│  └───────────────────────────────────────┘      │
│                                                 │
│  ┌──────────── Adapters ─────────────────┐      │
│  │  rusvel-llm     (model providers)     │      │
│  │  rusvel-agent   (agent runtime)       │      │
│  │  rusvel-db      (SQLite + 5 stores)   │      │
│  │  rusvel-event   (event bus + persist)  │      │
│  │  rusvel-memory  (context + search)    │      │
│  │  rusvel-tool    (tool registry)       │      │
│  │  rusvel-jobs    (central job queue)   │      │
│  │  rusvel-auth    (credentials)         │      │
│  │  rusvel-config  (settings)            │      │
│  └───────────────────────────────────────┘      │
└─────────────────────────────────────────────────┘
```

## The 5 Engines (was 7)

### 1. Forge Engine (Agent Orchestration + Mission)
The meta-engine. Orchestrates agents across all other engines.
Now includes Mission (goals, daily planning, reviews) as "mission agents."

- Agent personas, workflows (Sequential/Parallel/Loop/Graph)
- Safety: circuit breaker, rate limiter, budget, loop detection
- Mission: `forge mission today` → agent reads goals + engine states → daily plan
- Goals, reviews, decisions all managed by mission agents
- Git worktree isolation for code-modifying agents

### 2. Code Engine (Code Intelligence)
**v0 scope:** Rust only + symbol graph + BM25 search.
Expand to 12+ languages later.

- tree-sitter parsing (Rust first)
- Symbol graph + dependency detection
- BM25 full-text search
- Complexity metrics
- LLM-powered narration (explain code)

### 3. Harvest Engine (Opportunity Discovery)
Find gigs, jobs, opportunities.

- CDP-based passive scraping
- Source adapters (Upwork first, then expand)
- AI scoring + proposal generation
- Pipeline: discover → score → qualify → propose → track

### 4. Content Engine (Creation & Publishing)
Write once, publish everywhere.

- Markdown-first authoring
- AI pipeline: generate → adapt → review → approve → publish
- Platform adapters (DEV.to first, then Twitter/LinkedIn)
- Scheduling, analytics
- **Human approval gate** before publishing

### 5. GoToMarket Engine (Ops + Connect + Outreach)
Everything about running the business and building relationships.

- CRM: contacts, leads, deals, pipeline
- Outreach: email sequences, follow-ups
- Invoicing and payment tracking
- SOPs and knowledge base
- AI spend tracking
- **Human approval gate** before sending outreach

---

## The 10 Core Ports (was 13)

Consolidated from 13 to 10 by removing redundancy:

| Port | Responsibility | Notes |
|------|---------------|-------|
| `LlmPort` | Raw model access: generate, stream, embed | Never called directly by engines |
| `AgentPort` | Agent orchestration: create, run, stop, status | Wraps LlmPort + ToolPort + MemoryPort |
| `ToolPort` | Tool registry + execution | JSON Schema declarations |
| `EventPort` | System-wide typed event bus | Immutable, append-only |
| `StoragePort` | 5 canonical stores (see below) | Not "persist anything" |
| `MemoryPort` | Context, knowledge, semantic search | Session-namespaced |
| `JobPort` | Central job queue (replaces AutomationPort + SchedulePort) | All async work |
| `SessionPort` | Session → Run → Thread hierarchy | Everything keyed by session |
| `AuthPort` | Credentials (opaque handles) | Engines never see raw tokens |
| `ConfigPort` | Settings, preferences | Per-session overrides |

**Removed:** `HarvestPort` and `PublishPort` are now **engine-internal traits**, not core ports. They're domain-specific, not cross-cutting.

---

## StoragePort: 5 Canonical Stores

Instead of "persist anything":

```rust
pub trait StoragePort: Send + Sync {
    fn events(&self) -> &dyn EventStore;      // Append-only event log
    fn objects(&self) -> &dyn ObjectStore;     // Domain objects (Content, Opportunity, Contact, etc.)
    fn sessions(&self) -> &dyn SessionStore;   // Session/Run/Thread hierarchy
    fn jobs(&self) -> &dyn JobStore;           // Job queue (pending, running, completed)
    fn metrics(&self) -> &dyn MetricStore;     // Time-series metrics (engagement, spend, velocity)
}
```

Each store has a focused API. No generic key-value "put anything."

---

## Central Job Queue (replaces AutomationPort + SchedulePort)

All async work goes through one substrate (inspired by Windmill):

```rust
pub struct Job {
    pub id: JobId,
    pub session_id: SessionId,
    pub kind: JobKind,
    pub payload: serde_json::Value,
    pub status: JobStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retries: u32,
    pub max_retries: u32,
    pub error: Option<String>,
}

pub enum JobKind {
    AgentRun,          // Forge engine handles
    ContentPublish,    // Content engine handles
    OutreachSend,      // GoToMarket engine handles
    HarvestScan,       // Harvest engine handles
    CodeAnalyze,       // Code engine handles
    ScheduledCron,     // Recurring job
    Custom(String),    // Extensible
}

pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    AwaitingApproval,  // Human-in-the-loop gate
}

#[async_trait]
pub trait JobPort: Send + Sync {
    async fn enqueue(&self, job: NewJob) -> Result<JobId>;
    async fn dequeue(&self, kinds: &[JobKind]) -> Result<Option<Job>>;
    async fn complete(&self, id: &JobId, result: JobResult) -> Result<()>;
    async fn fail(&self, id: &JobId, error: String) -> Result<()>;
    async fn schedule(&self, job: NewJob, cron: &str) -> Result<JobId>;
    async fn cancel(&self, id: &JobId) -> Result<()>;
    async fn approve(&self, id: &JobId) -> Result<()>;  // Human approval
    async fn list(&self, filter: JobFilter) -> Result<Vec<Job>>;
}
```

One queue. One worker pool. All engines submit jobs. The worker routes to the right engine based on `JobKind`.

---

## Session Hierarchy (new)

```rust
pub struct Session {
    pub id: SessionId,
    pub name: String,
    pub kind: SessionKind,
    pub tags: Vec<String>,
    pub config: SessionConfig,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum SessionKind {
    Project,           // A codebase / product
    Lead,              // A potential client
    ContentCampaign,   // A content series
    General,           // Catch-all
}

pub struct Run {
    pub id: RunId,
    pub session_id: SessionId,
    pub engine: EngineKind,
    pub input_summary: String,
    pub status: RunStatus,
    pub llm_budget_used: f64,
    pub tool_calls_count: u32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

pub struct Thread {
    pub id: ThreadId,
    pub run_id: RunId,
    pub channel: ThreadChannel,
    pub messages: Vec<ThreadMessage>,
}

pub enum ThreadChannel { User, Agent, System, Event }
pub enum EngineKind { Forge, Code, Harvest, Content, GoToMarket }
```

---

## Human-in-the-Loop Approval Model

Critical for a solo founder — agents propose, human approves:

```rust
pub enum ApprovalStatus {
    NotRequired,       // Auto-execute (e.g., code analysis)
    Pending,           // Waiting for human
    Approved,          // Human said yes
    Rejected,          // Human said no
    AutoApproved,      // Policy allowed auto-approval
}

pub struct ApprovalPolicy {
    pub engine: EngineKind,
    pub action: String,          // "publish", "send_outreach", "spend > $1"
    pub requires_approval: bool,
    pub auto_approve_below: Option<f64>,  // Auto-approve if cost < threshold
}
```

Applied to:
- Content publishing (always requires approval by default)
- Outreach sending (always requires approval by default)
- Agent runs above cost threshold
- Invoice creation

---

## Shared Domain Types (updated)

All in rusvel-core. All have `metadata: serde_json::Value` for schema evolution.

```rust
// Identity
pub struct UserId(Uuid);
pub struct WorkspaceId(Uuid);

// LLM
pub struct ModelRef { pub provider: ModelProvider, pub model: String }
pub enum ModelProvider { Claude, OpenAI, Gemini, Ollama, Other(String) }

// Content (universal message type from adk-rust)
pub struct Content { pub parts: Vec<Part> }
pub enum Part { Text(String), Image(Bytes), Audio(Bytes), Video(Bytes), File { name: String, data: Bytes } }

// Agent
pub struct AgentProfile {
    pub id: AgentProfileId,
    pub name: String,
    pub role: String,
    pub instructions: String,
    pub default_model: ModelRef,
    pub allowed_tools: Vec<String>,
    pub capabilities: Vec<Capability>,
    pub budget_limit: Option<f64>,
}

// Opportunity
pub struct Opportunity {
    pub id: OpportunityId,
    pub session_id: SessionId,
    pub source: OpportunitySource,
    pub title: String,
    pub url: Option<String>,
    pub description: String,
    pub score: f64,
    pub stage: OpportunityStage,
    pub value_estimate: Option<f64>,
    pub metadata: serde_json::Value,
}
pub enum OpportunitySource { Upwork, Freelancer, LinkedIn, GitHub, Manual, Other(String) }
pub enum OpportunityStage { Cold, Contacted, Qualified, ProposalSent, Won, Lost }

// Content
pub struct ContentItem {
    pub id: ContentId,
    pub session_id: SessionId,
    pub kind: ContentKind,
    pub title: String,
    pub body_markdown: String,
    pub platform_targets: Vec<Platform>,
    pub status: ContentStatus,
    pub approval: ApprovalStatus,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub published_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}
pub enum ContentKind { LongForm, Tweet, Thread, LinkedInPost, Blog, VideoScript, Email, Proposal }
pub enum ContentStatus { Draft, Adapted, Scheduled, Published, Archived }

// Contact
pub struct Contact {
    pub id: ContactId,
    pub session_id: SessionId,
    pub name: String,
    pub emails: Vec<String>,
    pub links: Vec<String>,
    pub company: Option<String>,
    pub role: Option<String>,
    pub tags: Vec<String>,
    pub last_contacted_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

// Goal
pub struct Goal {
    pub id: GoalId,
    pub session_id: SessionId,
    pub title: String,
    pub description: String,
    pub timeframe: Timeframe,
    pub status: GoalStatus,
    pub progress: f64,
    pub metadata: serde_json::Value,
}
pub enum Timeframe { Day, Week, Month, Quarter }
pub enum GoalStatus { Active, Completed, Abandoned, Deferred }

// Task
pub struct Task {
    pub id: TaskId,
    pub session_id: SessionId,
    pub goal_id: Option<GoalId>,
    pub title: String,
    pub status: TaskStatus,
    pub due_at: Option<DateTime<Utc>>,
    pub priority: Priority,
    pub metadata: serde_json::Value,
}
pub enum TaskStatus { Todo, InProgress, Done, Cancelled }
pub enum Priority { Low, Medium, High, Urgent }

// Code
pub struct RepoRef { pub local_path: PathBuf, pub remote_url: Option<String> }
pub struct CodeSnapshotRef { pub id: SnapshotId, pub repo: RepoRef, pub analyzed_at: DateTime<Utc> }

// Events
pub struct Event {
    pub id: EventId,
    pub session_id: Option<SessionId>,
    pub run_id: Option<RunId>,
    pub source: EngineKind,
    pub kind: String,           // Flexible string, not giant enum
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}
```

**Key change:** `Event.kind` is now a `String` (not a giant enum). This avoids rusvel-core knowing about every possible event type. Engines define their own event kind constants.

---

## Dependency Graph (updated)

```
rusvel-app (binary, composition root)
├── rusvel-cli
├── rusvel-api (Axum)
├── rusvel-tui (Ratatui)
├── rusvel-mcp (rmcp)
│
├── forge-engine ─────┐
├── code-engine ──────┤
├── harvest-engine ───┤── depend on rusvel-core ONLY
├── content-engine ───┤
├── gtm-engine ───────┘
│
├── rusvel-llm ───────┐
├── rusvel-agent ─────┤
├── rusvel-db ────────┤
├── rusvel-event ─────┤── implement rusvel-core traits
├── rusvel-memory ────┤
├── rusvel-tool ──────┤
├── rusvel-jobs ──────┤
├── rusvel-auth ──────┤
└── rusvel-config ────┘
```

## Workspace (updated)

```
all-in-one-rusvel/
├── crates/
│   ├── rusvel-core/        ← 10 port traits + shared domain types
│   ├── rusvel-db/          ← SQLite WAL + 5 canonical stores
│   ├── rusvel-llm/         ← LlmPort adapters (Ollama first)
│   ├── rusvel-agent/       ← AgentPort runtime (wraps LLM+Tool+Memory)
│   ├── rusvel-event/       ← EventPort bus + persistence
│   ├── rusvel-memory/      ← MemoryPort + session-namespaced search
│   ├── rusvel-tool/        ← ToolPort registry + JSON Schema
│   ├── rusvel-jobs/        ← Central job queue (was AutomationPort + SchedulePort)
│   ├── rusvel-auth/        ← AuthPort (opaque credential handles)
│   ├── rusvel-config/      ← ConfigPort (TOML + per-session overrides)
│   │
│   ├── forge-engine/       ← Agent orchestration + Mission (goals/planning)
│   ├── code-engine/        ← Code intelligence (Rust-only v0)
│   ├── harvest-engine/     ← Opportunity discovery
│   ├── content-engine/     ← Content creation + publishing
│   ├── gtm-engine/         ← GoToMarket (CRM + outreach + ops)
│   │
│   ├── rusvel-api/         ← Axum HTTP + WebSocket + SSE
│   ├── rusvel-cli/         ← Clap CLI
│   ├── rusvel-tui/         ← Ratatui TUI
│   ├── rusvel-mcp/         ← MCP server (stdio + SSE)
│   └── rusvel-app/         ← Binary entry point (composition root)
│
├── frontend/               ← SvelteKit 5 + Tailwind 4
├── Cargo.toml
└── CLAUDE.md
```

20 crates (was 22). Cleaner.

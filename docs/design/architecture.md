> **SUPERSEDED** — See architecture-v2.md for current architecture.

# RUSVEL — Architecture Document

---

## Hexagonal Architecture (Ports & Adapters)

```
                    ┌──────────────────────────────┐
                    │         SURFACES              │
                    │  CLI │ TUI │ Web │ MCP        │
                    └───────────┬──────────────────┘
                                │ calls
                    ┌───────────┴──────────────────┐
                    │       DOMAIN ENGINES          │
                    │                               │
                    │  Each engine composes ports    │
                    │  to implement domain logic     │
                    └───────────┬──────────────────┘
                                │ uses
    ┌───────────────────────────┴───────────────────────────┐
    │                    FOUNDATION                          │
    │                                                        │
    │  ┌─────────────────────────────────────────────┐      │
    │  │           rusvel-core (PORTS)                │      │
    │  │                                             │      │
    │  │  Pure traits. Zero deps. The contract.      │      │
    │  └─────────────────────────────────────────────┘      │
    │           ▲                                            │
    │           │ implements                                  │
    │  ┌────────┴────────────────────────────────────┐      │
    │  │           ADAPTERS                           │      │
    │  │                                             │      │
    │  │  rusvel-llm, rusvel-db, rusvel-event, ...   │      │
    │  │  Concrete implementations of port traits    │      │
    │  └─────────────────────────────────────────────┘      │
    └───────────────────────────────────────────────────────┘
```

## Dependency Rules

1. **rusvel-core** depends on NOTHING (only std, serde, async-trait, thiserror, chrono, uuid)
2. **Adapters** depend on rusvel-core + their specific libraries (e.g., rusvel-db depends on rusqlite)
3. **Engines** depend on rusvel-core ports (traits) — NEVER on concrete adapters
4. **Surfaces** depend on engines + adapters (they wire everything together)
5. **rusvel-app** is the composition root — it creates concrete adapters and injects them into engines

```
rusvel-app (binary)
├── rusvel-cli
├── rusvel-api
├── rusvel-tui
├── rusvel-mcp
│
├── forge-engine ─────┐
├── code-engine ──────┤
├── harvest-engine ───┤
├── content-engine ───┤── all depend on rusvel-core (traits only)
├── ops-engine ───────┤
├── mission-engine ───┤
├── connect-engine ───┘
│
├── rusvel-llm ───────┐
├── rusvel-db ────────┤
├── rusvel-event ─────┤
├── rusvel-memory ────┤── all implement rusvel-core traits
├── rusvel-tool ──────┤
├── rusvel-schedule ──┤
├── rusvel-auth ──────┤
└── rusvel-config ────┘
```

## Crate Specifications

### rusvel-core (The Contract)

Zero external framework deps. Defines:

**Port traits:**
```rust
#[async_trait]
pub trait LlmPort: Send + Sync {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse>;
    async fn generate_stream(&self, request: LlmRequest) -> Result<LlmStream>;
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
}

#[async_trait]
pub trait AgentPort: Send + Sync {
    async fn create(&self, config: AgentConfig) -> Result<AgentId>;
    async fn run(&self, id: &AgentId, input: Content) -> Result<AgentOutput>;
    async fn stop(&self, id: &AgentId) -> Result<()>;
    async fn status(&self, id: &AgentId) -> Result<AgentStatus>;
}

#[async_trait]
pub trait StoragePort: Send + Sync {
    async fn put(&self, key: &str, value: &[u8]) -> Result<()>;
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn query(&self, filter: StorageQuery) -> Result<Vec<StorageEntry>>;
    async fn migrate(&self) -> Result<()>;
}

#[async_trait]
pub trait EventPort: Send + Sync {
    async fn emit(&self, event: Event) -> Result<()>;
    fn subscribe(&self) -> EventReceiver;
    async fn replay(&self, since: DateTime<Utc>) -> Result<Vec<Event>>;
}

#[async_trait]
pub trait MemoryPort: Send + Sync {
    async fn store(&self, entry: MemoryEntry) -> Result<MemoryId>;
    async fn recall(&self, id: &MemoryId) -> Result<Option<MemoryEntry>>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryEntry>>;
    async fn forget(&self, id: &MemoryId) -> Result<()>;
}

#[async_trait]
pub trait ToolPort: Send + Sync {
    fn register(&mut self, tool: ToolDefinition) -> Result<()>;
    async fn call(&self, name: &str, args: serde_json::Value) -> Result<ToolResult>;
    fn list(&self) -> Vec<ToolDefinition>;
    fn schema(&self, name: &str) -> Option<serde_json::Value>;
}

#[async_trait]
pub trait SchedulePort: Send + Sync {
    async fn schedule(&self, job: ScheduleJob) -> Result<ScheduleId>;
    async fn cancel(&self, id: &ScheduleId) -> Result<()>;
    async fn list(&self) -> Result<Vec<ScheduleJob>>;
    async fn trigger(&self, id: &ScheduleId) -> Result<()>;
}

#[async_trait]
pub trait HarvestPort: Send + Sync {
    async fn scan(&self, source: HarvestSource) -> Result<Vec<RawOpportunity>>;
    async fn extract(&self, raw: &RawOpportunity) -> Result<Opportunity>;
    async fn score(&self, opportunity: &Opportunity) -> Result<f64>;
    async fn ingest(&self, opportunities: Vec<Opportunity>) -> Result<usize>;
}

#[async_trait]
pub trait PublishPort: Send + Sync {
    async fn publish(&self, content: &ContentPiece, platform: Platform) -> Result<PostId>;
    async fn schedule_post(&self, content: &ContentPiece, platform: Platform, at: DateTime<Utc>) -> Result<ScheduleId>;
    async fn metrics(&self, post_id: &PostId) -> Result<PostMetrics>;
}

#[async_trait]
pub trait AuthPort: Send + Sync {
    async fn store_credential(&self, key: &str, credential: Credential) -> Result<()>;
    async fn get_credential(&self, key: &str) -> Result<Option<Credential>>;
    async fn refresh(&self, key: &str) -> Result<Credential>;
}

#[async_trait]
pub trait ConfigPort: Send + Sync {
    fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>;
    fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()>;
    fn watch(&self, key: &str) -> ConfigWatcher;
}

#[async_trait]
pub trait SessionPort: Send + Sync {
    async fn create(&self, config: SessionConfig) -> Result<SessionId>;
    async fn load(&self, id: &SessionId) -> Result<Session>;
    async fn save(&self, session: &Session) -> Result<()>;
    async fn switch(&self, id: &SessionId) -> Result<Session>;
    async fn list(&self) -> Result<Vec<SessionSummary>>;
}

#[async_trait]
pub trait AutomationPort: Send + Sync {
    async fn define_workflow(&self, workflow: Workflow) -> Result<WorkflowId>;
    async fn execute(&self, id: &WorkflowId, input: Content) -> Result<WorkflowRun>;
    async fn pause(&self, run_id: &RunId) -> Result<()>;
    async fn resume(&self, run_id: &RunId) -> Result<()>;
    async fn status(&self, run_id: &RunId) -> Result<WorkflowStatus>;
}
```

**Shared domain types:**
```rust
// Universal content (inspired by adk-rust)
pub struct Content { pub parts: Vec<Part> }
pub enum Part { Text(String), Image(Bytes), Audio(Bytes), Video(Bytes), File { name: String, data: Bytes } }

// Cross-engine types
pub struct Opportunity { pub id: OpportunityId, pub source: String, pub title: String, pub score: f64, pub status: OpportunityStatus, pub metadata: serde_json::Value }
pub struct Contact { pub id: ContactId, pub name: String, pub channels: Vec<Channel>, pub relationship_score: f64 }
pub struct ContentPiece { pub id: ContentId, pub body: String, pub format: ContentFormat, pub platform_variants: HashMap<Platform, String> }
pub struct Goal { pub id: GoalId, pub description: String, pub deadline: Option<DateTime<Utc>>, pub progress: f64 }
pub struct AgentTask { pub id: TaskId, pub agent_id: AgentId, pub status: TaskStatus, pub cost: f64, pub events: Vec<Event> }
pub struct Session { pub id: SessionId, pub workspace: String, pub agents: Vec<AgentId>, pub memory_scope: MemoryScope }
```

**Status enums:**
```rust
pub enum TaskStatus { Submitted, Working, InputRequired, Completed, Failed, Cancelled }
pub enum OpportunityStatus { Discovered, Scored, Qualified, Proposed, Won, Lost, Archived }
pub enum WorkflowStatus { Pending, Running, Paused, Completed, Failed }
```

### Engine Contract

Every engine implements this trait:

```rust
#[async_trait]
pub trait Engine: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> Vec<Capability>;
    async fn initialize(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    async fn health(&self) -> Result<HealthStatus>;
}
```

Engines receive port implementations via constructor injection:

```rust
pub struct MissionEngine {
    llm: Arc<dyn LlmPort>,
    memory: Arc<dyn MemoryPort>,
    storage: Arc<dyn StoragePort>,
    schedule: Arc<dyn SchedulePort>,
    events: Arc<dyn EventPort>,
}
```

## Database Strategy

- **One SQLite file** per RUSVEL installation: `~/.rusvel/rusvel.db`
- **WAL mode** for concurrent reads
- **Schema per engine** — each engine owns its tables, prefixed: `forge_*`, `code_*`, `harvest_*`, `content_*`, `ops_*`, `mission_*`, `connect_*`
- **Shared tables** — `events`, `sessions`, `config`, `credentials` (owned by foundation crates)
- **Migration system** — each crate ships numbered SQL migrations, run at startup

## Event System

Every significant action emits a typed event:

```rust
pub struct Event {
    pub id: EventId,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<SessionId>,
    pub source: String,          // "forge-engine", "harvest-engine", etc.
    pub kind: EventKind,
    pub payload: serde_json::Value,
}

pub enum EventKind {
    // Agent events
    AgentCreated, AgentStarted, AgentCompleted, AgentFailed, AgentStopped,
    // Harvest events
    ScanStarted, OpportunityDiscovered, OpportunityScored, ProposalGenerated,
    // Content events
    ContentDrafted, ContentAdapted, ContentPublished, MetricsUpdated,
    // Mission events
    GoalCreated, GoalUpdated, DailyPlanGenerated, ReviewCompleted,
    // Ops events
    ContactCreated, DealUpdated, InvoiceCreated,
    // Connect events
    OutreachSent, FollowUpScheduled, ResponseReceived,
    // System events
    SessionCreated, ConfigChanged, Error,
}
```

## Safety Controls (from forge-project)

Built into the foundation, not bolted on:

```rust
pub struct SafetyConfig {
    pub circuit_breaker: CircuitBreakerConfig,  // error threshold → open circuit
    pub rate_limiter: RateLimiterConfig,         // requests per second per provider
    pub cost_tracker: CostTrackerConfig,         // daily/monthly budget caps
    pub loop_detector: LoopDetectorConfig,       // detect agent infinite loops
}
```

Applied as middleware in rusvel-agent, wrapping every LLM call and tool execution.

## Frontend Architecture

Single SvelteKit 5 app with route-based domain separation:

```
frontend/src/routes/
├── +layout.svelte          ← sidebar nav, session switcher
├── +page.svelte            ← dashboard (today's brief)
├── forge/                  ← agent management, live streams
├── code/                   ← codebase explorer, graphs
├── harvest/                ← opportunity pipeline
├── content/                ← editor, calendar, analytics
├── ops/                    ← CRM, invoices, SOPs
├── mission/                ← goals, daily plans, reviews
├── connect/                ← contacts, outreach, sequences
└── settings/               ← config, API keys, preferences
```

Built to static → embedded via rust-embed → served by Axum.

## API Design

RESTful with WebSocket for streaming:

```
GET    /api/health
GET    /api/session/current
POST   /api/session

# Per-engine endpoints follow same pattern:
GET    /api/{engine}/status
POST   /api/{engine}/{action}
WS     /api/{engine}/stream

# Examples:
POST   /api/forge/run         { "prompt": "find rust gigs" }
GET    /api/harvest/pipeline
POST   /api/content/publish   { "id": "...", "platforms": ["twitter"] }
GET    /api/mission/today
POST   /api/connect/outreach  { "campaign_id": "..." }
```

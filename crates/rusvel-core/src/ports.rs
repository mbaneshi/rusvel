//! The 12 core port traits that define RUSVEL's hexagonal boundary.
//!
//! Engines depend **only** on these traits. Concrete implementations
//! live in adapter crates (`rusvel-llm`, `rusvel-db`, …) and are
//! injected via the composition root (`rusvel-app`).
//!
//! ## Port inventory (architecture-v2)
//!
//! | Port | Responsibility |
//! |------|---------------|
//! | [`LlmPort`] | Raw model access: generate, stream, embed |
//! | [`AgentPort`] | Agent orchestration (wraps LLM + Tool + Memory) |
//! | [`ToolPort`] | Tool registry + execution |
//! | [`EventPort`] | System-wide event bus |
//! | [`StoragePort`] | 5 canonical sub-stores (ADR-004) |
//! | [`MemoryPort`] | Context, knowledge, semantic search |
//! | [`JobPort`] | Central job queue (ADR-003) |
//! | [`SessionPort`] | Session → Run → Thread hierarchy |
//! | [`AuthPort`] | Opaque credential handles |
//! | [`ConfigPort`] | Settings + per-session overrides |
//! | [`EmbeddingPort`] | Text → dense vector embeddings |
//! | [`VectorStorePort`] | Similarity search over embeddings |
//!
//! **Not here (ADR-006):** `HarvestPort` and `PublishPort` are
//! engine-internal traits, not cross-cutting concerns.

use async_trait::async_trait;
use serde::{Serialize, de::DeserializeOwned};

use crate::domain::*;
use crate::error::Result;
use crate::id::*;

// ════════════════════════════════════════════════════════════════════
//  1. LlmPort — raw model access
// ════════════════════════════════════════════════════════════════════

/// Raw access to language model providers.
///
/// **ADR-009:** Engines never call this directly — they go through
/// [`AgentPort`] which wraps LLM + Tool + Memory into a coherent
/// orchestration layer.
#[async_trait]
pub trait LlmPort: Send + Sync {
    /// One-shot generation.
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse>;

    /// Generate text embeddings for semantic search.
    async fn embed(&self, model: &ModelRef, text: &str) -> Result<Vec<f32>>;

    /// List available models on this provider.
    async fn list_models(&self) -> Result<Vec<ModelRef>>;
}

// ════════════════════════════════════════════════════════════════════
//  2. AgentPort — agent orchestration
// ════════════════════════════════════════════════════════════════════

/// High-level agent orchestration.
///
/// This is the port engines actually use for AI work (ADR-009).
/// Implementations wrap [`LlmPort`] + [`ToolPort`] + [`MemoryPort`]
/// into a coherent agent loop with retries, tool routing, and memory.
#[async_trait]
pub trait AgentPort: Send + Sync {
    /// Create a new agent instance from config.
    async fn create(&self, config: AgentConfig) -> Result<RunId>;

    /// Execute an agent run with the given input.
    async fn run(&self, run_id: &RunId, input: Content) -> Result<AgentOutput>;

    /// Stop a running agent.
    async fn stop(&self, run_id: &RunId) -> Result<()>;

    /// Query agent status.
    async fn status(&self, run_id: &RunId) -> Result<AgentStatus>;
}

// ════════════════════════════════════════════════════════════════════
//  3. ToolPort — tool registry + execution
// ════════════════════════════════════════════════════════════════════

/// Registry of tools that agents can invoke.
///
/// Uses `&self` (not `&mut self`) for object-safety with `Arc<dyn ToolPort>`.
/// Implementations use interior mutability for `register`.
#[async_trait]
pub trait ToolPort: Send + Sync {
    /// Register a new tool. Uses interior mutability.
    async fn register(&self, tool: ToolDefinition) -> Result<()>;

    /// Call a tool by name with JSON arguments.
    async fn call(&self, name: &str, args: serde_json::Value) -> Result<ToolResult>;

    /// List all registered tools.
    fn list(&self) -> Vec<ToolDefinition>;

    /// Get the JSON Schema for a tool's parameters.
    fn schema(&self, name: &str) -> Option<serde_json::Value>;
}

// ════════════════════════════════════════════════════════════════════
//  4. EventPort — system-wide event bus
// ════════════════════════════════════════════════════════════════════

/// Append-only event bus for domain events.
///
/// Events use `kind: String` (ADR-005) so rusvel-core stays minimal.
#[async_trait]
pub trait EventPort: Send + Sync {
    /// Emit a new event and return its assigned ID.
    async fn emit(&self, event: Event) -> Result<EventId>;

    /// Retrieve a single event by ID.
    async fn get(&self, id: &EventId) -> Result<Option<Event>>;

    /// Query events matching a filter.
    async fn query(&self, filter: EventFilter) -> Result<Vec<Event>>;
}

// ════════════════════════════════════════════════════════════════════
//  5. StoragePort — 5 canonical sub-stores  (ADR-004)
// ════════════════════════════════════════════════════════════════════

/// Unified access to the 5 canonical persistence stores.
///
/// Each sub-store has a focused API optimised for its access pattern:
/// append-only events, CRUD objects, session hierarchy, job queue
/// semantics, and time-series metrics.
pub trait StoragePort: Send + Sync {
    /// Append-only event log.
    fn events(&self) -> &dyn EventStore;

    /// CRUD store for domain objects (Opportunity, Contact, …).
    fn objects(&self) -> &dyn ObjectStore;

    /// Session → Run → Thread hierarchy store.
    fn sessions(&self) -> &dyn SessionStore;

    /// Job queue persistence.
    fn jobs(&self) -> &dyn JobStore;

    /// Time-series metric store.
    fn metrics(&self) -> &dyn MetricStore;
}

// ── Sub-store: EventStore ──────────────────────────────────────────

/// Append-only persistence for domain events.
#[async_trait]
pub trait EventStore: Send + Sync {
    async fn append(&self, event: &Event) -> Result<()>;
    async fn get(&self, id: &EventId) -> Result<Option<Event>>;
    async fn query(&self, filter: EventFilter) -> Result<Vec<Event>>;
}

// ── Sub-store: ObjectStore ─────────────────────────────────────────

/// CRUD persistence for domain objects.
///
/// Objects are stored as JSON values keyed by `(kind, id)`.
/// Callers serialize/deserialize to concrete types.
#[async_trait]
pub trait ObjectStore: Send + Sync {
    async fn put(&self, kind: &str, id: &str, object: serde_json::Value) -> Result<()>;
    async fn get(&self, kind: &str, id: &str) -> Result<Option<serde_json::Value>>;
    async fn delete(&self, kind: &str, id: &str) -> Result<()>;
    async fn list(&self, kind: &str, filter: ObjectFilter) -> Result<Vec<serde_json::Value>>;
}

// ── Sub-store: SessionStore ────────────────────────────────────────

/// Persistence for the Session → Run → Thread hierarchy.
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn put_session(&self, session: &Session) -> Result<()>;
    async fn get_session(&self, id: &SessionId) -> Result<Option<Session>>;
    async fn list_sessions(&self) -> Result<Vec<SessionSummary>>;

    async fn put_run(&self, run: &Run) -> Result<()>;
    async fn get_run(&self, id: &RunId) -> Result<Option<Run>>;
    async fn list_runs(&self, session_id: &SessionId) -> Result<Vec<Run>>;

    async fn put_thread(&self, thread: &Thread) -> Result<()>;
    async fn get_thread(&self, id: &ThreadId) -> Result<Option<Thread>>;
    async fn list_threads(&self, run_id: &RunId) -> Result<Vec<Thread>>;
}

// ── Sub-store: JobStore ────────────────────────────────────────────

/// Persistence layer for the central job queue.
#[async_trait]
pub trait JobStore: Send + Sync {
    async fn enqueue(&self, job: &Job) -> Result<()>;
    async fn dequeue(&self, kinds: &[JobKind]) -> Result<Option<Job>>;
    async fn update(&self, job: &Job) -> Result<()>;
    async fn get(&self, id: &JobId) -> Result<Option<Job>>;
    async fn list(&self, filter: JobFilter) -> Result<Vec<Job>>;
}

// ── Sub-store: MetricStore ─────────────────────────────────────────

/// Time-series metric persistence (engagement, spend, velocity, …).
#[async_trait]
pub trait MetricStore: Send + Sync {
    async fn record(&self, point: &MetricPoint) -> Result<()>;
    async fn query(&self, filter: MetricFilter) -> Result<Vec<MetricPoint>>;
}

// ════════════════════════════════════════════════════════════════════
//  6. MemoryPort — context + semantic search
// ════════════════════════════════════════════════════════════════════

/// Session-namespaced context and knowledge store.
#[async_trait]
pub trait MemoryPort: Send + Sync {
    /// Store a memory entry and return its UUID.
    async fn store(&self, entry: MemoryEntry) -> Result<uuid::Uuid>;

    /// Recall a specific entry by ID.
    async fn recall(&self, id: &uuid::Uuid) -> Result<Option<MemoryEntry>>;

    /// Semantic search within a session's memory.
    async fn search(
        &self,
        session_id: &SessionId,
        query: &str,
        limit: usize,
    ) -> Result<Vec<MemoryEntry>>;

    /// Delete a memory entry.
    async fn forget(&self, id: &uuid::Uuid) -> Result<()>;
}

// ════════════════════════════════════════════════════════════════════
//  7. JobPort — central job queue  (ADR-003)
// ════════════════════════════════════════════════════════════════════

/// Central job queue replacing `AutomationPort` + `SchedulePort`.
///
/// All async work goes through this one substrate. A worker pool
/// routes jobs to the correct engine based on [`JobKind`].
#[async_trait]
pub trait JobPort: Send + Sync {
    /// Submit a new job.
    async fn enqueue(&self, job: NewJob) -> Result<JobId>;

    /// Claim the next available job matching the given kinds.
    async fn dequeue(&self, kinds: &[JobKind]) -> Result<Option<Job>>;

    /// Mark a job as successfully completed.
    async fn complete(&self, id: &JobId, result: JobResult) -> Result<()>;

    /// Mark a job as failed.
    async fn fail(&self, id: &JobId, error: String) -> Result<()>;

    /// Schedule a recurring job with a cron expression.
    async fn schedule(&self, job: NewJob, cron: &str) -> Result<JobId>;

    /// Cancel a queued or scheduled job.
    async fn cancel(&self, id: &JobId) -> Result<()>;

    /// Human approves a job waiting at the approval gate (ADR-008).
    async fn approve(&self, id: &JobId) -> Result<()>;

    /// List jobs matching a filter.
    async fn list(&self, filter: JobFilter) -> Result<Vec<Job>>;
}

// ════════════════════════════════════════════════════════════════════
//  8. SessionPort — session hierarchy
// ════════════════════════════════════════════════════════════════════

/// Manage the Session → Run → Thread hierarchy.
#[async_trait]
pub trait SessionPort: Send + Sync {
    /// Create a new session.
    async fn create(&self, session: Session) -> Result<SessionId>;

    /// Load a session by ID.
    async fn load(&self, id: &SessionId) -> Result<Session>;

    /// Persist updates to an existing session.
    async fn save(&self, session: &Session) -> Result<()>;

    /// List all sessions.
    async fn list(&self) -> Result<Vec<SessionSummary>>;
}

// ════════════════════════════════════════════════════════════════════
//  9. AuthPort — opaque credential handles
// ════════════════════════════════════════════════════════════════════

/// Secure credential storage. Engines never see raw tokens/secrets.
#[async_trait]
pub trait AuthPort: Send + Sync {
    /// Store or update a credential under a key.
    async fn store_credential(&self, key: &str, credential: Credential) -> Result<()>;

    /// Retrieve a credential by key.
    async fn get_credential(&self, key: &str) -> Result<Option<Credential>>;

    /// Refresh an expiring credential.
    async fn refresh(&self, key: &str) -> Result<Credential>;

    /// Delete a credential.
    async fn delete_credential(&self, key: &str) -> Result<()>;
}

// ════════════════════════════════════════════════════════════════════
//  10. ConfigPort — settings + per-session overrides  (original #10)
// ════════════════════════════════════════════════════════════════════

// ════════════════════════════════════════════════════════════════════
//  11. EmbeddingPort — text → vector embeddings
// ════════════════════════════════════════════════════════════════════

/// Generate dense vector embeddings from text.
///
/// Used by the RAG knowledge system to embed documents and queries.
/// Adapter implementations wrap local models (fastembed) or remote
/// APIs (`OpenAI`, Cohere, etc.).
#[async_trait]
pub trait EmbeddingPort: Send + Sync {
    /// Embed multiple texts in a single batch call.
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// Convenience: embed a single text string.
    async fn embed_one(&self, text: &str) -> Result<Vec<f32>>;

    /// The model name used for embeddings (e.g. `"all-MiniLM-L6-v2"`).
    fn model_name(&self) -> &str;

    /// The dimensionality of the embedding vectors.
    fn dimensions(&self) -> usize;
}

// ════════════════════════════════════════════════════════════════════
//  12. VectorStorePort — similarity search over embeddings
// ════════════════════════════════════════════════════════════════════

/// Persistent vector store for RAG knowledge retrieval.
///
/// Stores document chunks alongside their embedding vectors and
/// supports similarity search. Adapter implementations wrap `LanceDB`,
/// Qdrant, or in-memory stores.
#[async_trait]
pub trait VectorStorePort: Send + Sync {
    /// Insert or update a vector entry.
    async fn upsert(
        &self,
        id: &str,
        content: &str,
        embedding: Vec<f32>,
        metadata: serde_json::Value,
    ) -> Result<()>;

    /// Find the most similar entries to a query embedding.
    async fn search(
        &self,
        query_embedding: &[f32],
        limit: usize,
    ) -> Result<Vec<VectorSearchResult>>;

    /// Delete an entry by ID.
    async fn delete(&self, id: &str) -> Result<()>;

    /// List entries (most recent first).
    async fn list(&self, limit: usize) -> Result<Vec<VectorEntry>>;

    /// Return the total number of entries.
    async fn count(&self) -> Result<usize>;
}

// ════════════════════════════════════════════════════════════════════
//  13. ConfigPort — settings + per-session overrides
// ════════════════════════════════════════════════════════════════════

/// Application and per-session configuration.
///
/// Values are stored as JSON and deserialized on read.
pub trait ConfigPort: Send + Sync {
    /// Read a typed config value by dotted key (e.g. `"llm.default_model"`).
    fn get_value(&self, key: &str) -> Result<Option<serde_json::Value>>;

    /// Write a config value.
    fn set_value(&self, key: &str, value: serde_json::Value) -> Result<()>;

    /// Convenience: read and deserialize a typed value.
    fn get_typed<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>>
    where
        Self: Sized,
    {
        match self.get_value(key)? {
            Some(v) => Ok(Some(serde_json::from_value(v)?)),
            None => Ok(None),
        }
    }

    /// Convenience: serialize and write a typed value.
    fn set_typed<T: Serialize>(&self, key: &str, value: &T) -> Result<()>
    where
        Self: Sized,
    {
        let v = serde_json::to_value(value)?;
        self.set_value(key, v)
    }
}

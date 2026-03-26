# Task #33: Self-Improving Knowledge Base

> Read this file, then do the task. Only modify files listed below.

## Goal

Auto-index every engine output (code analyses, content drafts, harvest discoveries) into the knowledge base for cross-department semantic search.

## Files to Read First

- `crates/rusvel-api/src/knowledge.rs` — existing knowledge routes, hybrid RAG
- `crates/rusvel-core/src/ports.rs` — MemoryPort, VectorStorePort, EmbeddingPort
- `crates/rusvel-core/src/domain.rs` — Event type, MemoryEntry
- `crates/rusvel-event/src/lib.rs` — EventBus

## What to Build

### 1. Auto-indexer in `crates/rusvel-api/src/knowledge.rs`

Add a background task that subscribes to the event bus and auto-indexes relevant events:

```rust
pub async fn spawn_knowledge_indexer(
    events: Arc<dyn EventPort>,
    memory: Arc<dyn MemoryPort>,
    embedding: Option<Arc<dyn EmbeddingPort>>,
    vector_store: Option<Arc<dyn VectorStorePort>>,
)
```

Event patterns to index:
- `code.analyzed` → index the analysis summary
- `content.drafted` → index the draft content
- `content.published` → index with platform metadata
- `harvest.scored` → index opportunity descriptions
- `flow.completed` → index flow results

For each: extract text content from event payload, create a MemoryEntry, store via MemoryPort, and if embedding+vector available, embed and store in vector DB.

### 2. Cross-dept search endpoint

Add to knowledge routes:
- `GET /api/knowledge/related?query=X&dept=Y` — search across all departments, optionally filter by dept
- Returns results with department attribution

### 3. Wire the indexer

In `crates/rusvel-app/src/main.rs`, spawn the knowledge indexer after event bus is created.

## Files to Modify

- `crates/rusvel-api/src/knowledge.rs` — add spawn_knowledge_indexer + related endpoint
- `crates/rusvel-app/src/main.rs` — spawn the indexer

## Verify

```bash
cargo check -p rusvel-api && cargo check --workspace
```

## Depends On

- #14 Hybrid RAG (done)

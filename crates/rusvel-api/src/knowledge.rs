//! Knowledge base API — ingest, search, browse, and manage knowledge entries.
//!
//! Uses `EmbeddingPort` for text embedding and `VectorStorePort` for storage/retrieval.
//! Chunking splits text by paragraphs (double newline), capping at 500 chars per chunk.
//!
//! Hybrid search fuses FTS5 session memory ([`rusvel_core::ports::MemoryPort`]) with
//! LanceDB vector hits via [`rusvel_core::reciprocal_rank_fusion`] (v1: RRF only).

use std::collections::HashMap;
use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use rusvel_core::domain::{
    Event, HybridHitSource, HybridSearchHit, MemoryEntry, MemoryKind, VectorEntry,
    VectorSearchResult,
};
use rusvel_core::id::SessionId;
use rusvel_core::ports::{EmbeddingPort, MemoryPort, StoragePort, VectorStorePort};
use rusvel_core::{RRF_K_DEFAULT, reciprocal_rank_fusion};

use crate::AppState;

/// Session used for FTS when the source event has no `session_id` (e.g. code analysis, flow).
fn kb_session_id(event: &Event) -> SessionId {
    event
        .session_id
        .unwrap_or_else(|| SessionId::from_uuid(Uuid::nil()))
}

/// Background task: subscribe to [`rusvel_event::EventBus`] and index matching engine events
/// into session memory and (when configured) the vector store. Task #33.
pub fn spawn_knowledge_indexer(
    event_bus: Arc<rusvel_event::EventBus>,
    memory: Arc<dyn MemoryPort>,
    storage: Arc<dyn StoragePort>,
    embedding: Option<Arc<dyn EmbeddingPort>>,
    vector_store: Option<Arc<dyn VectorStorePort>>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut rx = event_bus.subscribe();
        loop {
            match rx.recv().await {
                Ok(event) => {
                    if let Err(e) = index_event_for_kb(
                        &memory,
                        &storage,
                        embedding.as_ref(),
                        vector_store.as_ref(),
                        &event,
                    )
                    .await
                    {
                        tracing::warn!(
                            error = %e,
                            kind = %event.kind,
                            "knowledge auto-index skipped"
                        );
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    tracing::warn!("knowledge indexer lagged {n} events");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    })
}

async fn index_event_for_kb(
    memory: &Arc<dyn MemoryPort>,
    storage: &Arc<dyn StoragePort>,
    embedding: Option<&Arc<dyn EmbeddingPort>>,
    vector_store: Option<&Arc<dyn VectorStorePort>>,
    event: &Event,
) -> rusvel_core::error::Result<()> {
    if !should_index_kind(&event.kind) {
        return Ok(());
    }

    let Some(text) = indexable_text(storage, event)
        .await
        .map(|s| truncate_kb_text(s))
    else {
        tracing::debug!(kind = %event.kind, "knowledge indexer: no text to index");
        return Ok(());
    };

    let session_id = kb_session_id(event);
    let dept = event.source.clone();
    let meta = serde_json::json!({
        "department": dept,
        "event_kind": event.kind,
        "event_id": event.id.to_string(),
        "indexed_at": Utc::now().to_rfc3339(),
    });

    let entry = MemoryEntry {
        id: None,
        session_id,
        kind: MemoryKind::Custom("kb.auto".into()),
        content: text.clone(),
        embedding: None,
        created_at: Utc::now(),
        metadata: meta.clone(),
    };
    memory.store(entry).await?;

    if let (Some(embed), Some(vs)) = (embedding, vector_store) {
        let chunks = chunk_text(&text, 500);
        for chunk in chunks {
            if chunk.trim().is_empty() {
                continue;
            }
            let emb = embed.embed_one(&chunk).await?;
            let id = Uuid::now_v7().to_string();
            let mut chunk_meta = meta.clone();
            if let Some(obj) = chunk_meta.as_object_mut() {
                obj.insert("chunk".into(), serde_json::Value::String(chunk.clone()));
            }
            vs.upsert(&id, &chunk, emb, chunk_meta).await?;
        }
    }

    Ok(())
}

fn should_index_kind(kind: &str) -> bool {
    matches!(
        kind,
        "code.analyzed"
            | "content.drafted"
            | "content.published"
            | "harvest.opportunity.discovered"
            | "harvest.opportunity.scored"
            | "flow.execution.completed"
    )
}

const KB_TEXT_MAX: usize = 50_000;

fn truncate_kb_text(s: String) -> String {
    if s.len() <= KB_TEXT_MAX {
        return s;
    }
    format!("{}…", &s[..KB_TEXT_MAX.saturating_sub(1)])
}

async fn indexable_text(storage: &Arc<dyn StoragePort>, event: &Event) -> Option<String> {
    let objects = storage.objects();
    match event.kind.as_str() {
        "code.analyzed" => {
            let sid = event.payload.get("snapshot_id")?.as_str()?;
            let json = objects.get("code_analysis", sid).await.ok()??;
            Some(format!(
                "Code analysis snapshot `{}`:\n{}",
                sid,
                serde_json::to_string_pretty(&json).unwrap_or_else(|_| json.to_string())
            ))
        }
        "content.drafted" | "content.published" => {
            let cid = event.payload.get("content_id")?.as_str()?;
            let v = objects.get("content", cid).await.ok()??;
            let title = v.get("title")?.as_str()?;
            let body = v.get("body_markdown")?.as_str()?;
            let label = if event.kind.as_str() == "content.published" {
                "Published content"
            } else {
                "Content draft"
            };
            Some(format!("{label}: {title}\n\n{body}"))
        }
        "harvest.opportunity.discovered" | "harvest.opportunity.scored" => {
            let oid = event.payload.get("id")?.as_str()?;
            let Some(v) = objects.get("opportunity", oid).await.ok().flatten() else {
                let title = event
                    .payload
                    .get("title")
                    .and_then(|x| x.as_str())
                    .unwrap_or(oid);
                return Some(format!("Harvest opportunity (id {oid}): {title}"));
            };
            let title = v.get("title")?.as_str()?;
            let desc = v.get("description")?.as_str()?;
            let url = v.get("url").and_then(|x| x.as_str()).unwrap_or("");
            Some(format!("Opportunity: {title}\nURL: {url}\n\n{desc}"))
        }
        "flow.execution.completed" => {
            let flow_id = event.payload.get("flow_id")?.as_str()?;
            let exec_id = event.payload.get("execution_id")?.as_str()?;
            let status = event.payload.get("status")?.as_str()?;
            Some(format!(
                "Flow execution completed\nflow_id: {flow_id}\nexecution_id: {exec_id}\nstatus: {status}"
            ))
        }
        _ => None,
    }
}

/// Max candidates per leg and max fused results returned (explicit caps).
pub const HYBRID_SEARCH_MAX_LIMIT: usize = 50;
pub const HYBRID_DEFAULT_FTS_LIMIT: usize = 20;
pub const HYBRID_DEFAULT_VECTOR_LIMIT: usize = 20;
pub const HYBRID_DEFAULT_OUTPUT_LIMIT: usize = 10;

// ── Request / Response types ─────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub content: String,
    pub source: String,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub chunks_stored: usize,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct HybridSearchRequest {
    pub query: String,
    /// Required — FTS leg is session-scoped.
    pub session_id: String,
    pub limit: Option<usize>,
    pub fts_limit: Option<usize>,
    pub vector_limit: Option<usize>,
    pub rrf_k: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct HybridSearchResponse {
    pub hits: Vec<HybridSearchHit>,
}

#[derive(Debug, Serialize)]
pub struct KnowledgeStatsResponse {
    pub total_entries: usize,
    pub model_name: String,
    pub dimensions: usize,
}

#[derive(Debug, Deserialize)]
pub struct RelatedQuery {
    pub query: String,
    pub dept: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct RelatedKnowledgeHit {
    pub content: String,
    pub department: String,
    pub score: f64,
    pub entry_id: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct RelatedKnowledgeResponse {
    pub hits: Vec<RelatedKnowledgeHit>,
}

// ── Chunking helper ──────────────────────────────────────────

/// Split text into chunks by double-newline paragraphs, each at most `max_chars`.
fn chunk_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let paragraphs: Vec<&str> = text.split("\n\n").collect();

    let mut current = String::new();
    for para in paragraphs {
        let trimmed = para.trim();
        if trimmed.is_empty() {
            continue;
        }

        if current.is_empty() {
            current = trimmed.to_string();
        } else if current.len() + trimmed.len() + 2 <= max_chars {
            current.push_str("\n\n");
            current.push_str(trimmed);
        } else {
            chunks.push(current);
            current = trimmed.to_string();
        }

        // If current chunk exceeds max, split it further at word boundaries
        while current.len() > max_chars {
            let split_at = current[..max_chars]
                .rfind(|c: char| c.is_whitespace())
                .unwrap_or(max_chars);
            let (left, right) = current.split_at(split_at);
            chunks.push(left.trim().to_string());
            current = right.trim().to_string();
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

// ── Handlers ─────────────────────────────────────────────────

/// POST /api/knowledge/ingest — chunk text, embed each chunk, store in vector DB.
pub async fn ingest_knowledge(
    State(state): State<Arc<AppState>>,
    Json(body): Json<IngestRequest>,
) -> Result<Json<IngestResponse>, (StatusCode, String)> {
    let embed_port = state.embedding.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Embedding service not available".into(),
        )
    })?;
    let vector_store = state.vector_store.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector store not available".into(),
        )
    })?;

    let chunks = chunk_text(&body.content, 500);
    if chunks.is_empty() {
        return Ok(Json(IngestResponse { chunks_stored: 0 }));
    }

    let mut stored = 0usize;
    for chunk in &chunks {
        let embedding = embed_port.embed_one(chunk).await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Embedding failed: {e}"),
            )
        })?;

        let id = uuid::Uuid::now_v7().to_string();
        let metadata = serde_json::json!({
            "source": body.source,
        });

        vector_store
            .upsert(&id, chunk, embedding, metadata)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Store failed: {e}"),
                )
            })?;
        stored += 1;
    }

    tracing::info!("Ingested {} chunks from source '{}'", stored, body.source);
    Ok(Json(IngestResponse {
        chunks_stored: stored,
    }))
}

/// GET /api/knowledge — list all knowledge entries.
pub async fn list_knowledge(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<VectorEntry>>, (StatusCode, String)> {
    let vector_store = state.vector_store.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector store not available".into(),
        )
    })?;

    let entries = vector_store.list(1000).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("List failed: {e}"),
        )
    })?;

    Ok(Json(entries))
}

/// DELETE /api/knowledge/{id} — delete a knowledge entry.
pub async fn delete_knowledge(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let vector_store = state.vector_store.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector store not available".into(),
        )
    })?;

    vector_store.delete(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Delete failed: {e}"),
        )
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/knowledge/search — embed query, search vector store, return results.
pub async fn search_knowledge(
    State(state): State<Arc<AppState>>,
    Json(body): Json<SearchRequest>,
) -> Result<Json<Vec<VectorSearchResult>>, (StatusCode, String)> {
    let embed_port = state.embedding.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Embedding service not available".into(),
        )
    })?;
    let vector_store = state.vector_store.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector store not available".into(),
        )
    })?;

    let query_embedding = embed_port.embed_one(&body.query).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Embedding failed: {e}"),
        )
    })?;

    let limit = body.limit.unwrap_or(5);
    let results = vector_store
        .search(&query_embedding, limit)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search failed: {e}"),
            )
        })?;

    Ok(Json(results))
}

/// POST /api/knowledge/hybrid-search — RRF fusion of FTS memory + vector similarity (v1: no rerank).
pub async fn hybrid_search_knowledge(
    State(state): State<Arc<AppState>>,
    Json(body): Json<HybridSearchRequest>,
) -> Result<Json<HybridSearchResponse>, (StatusCode, String)> {
    let query = body.query.trim();
    if query.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "query must not be empty".into()));
    }

    let session_uuid = uuid::Uuid::parse_str(body.session_id.trim()).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            "session_id must be a valid UUID".into(),
        )
    })?;
    let session_id = SessionId::from_uuid(session_uuid);

    let embed_port = state.embedding.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Embedding service not available (required for hybrid vector leg)".into(),
        )
    })?;
    let vector_store = state.vector_store.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector store not available (required for hybrid vector leg)".into(),
        )
    })?;

    let fts_limit = body
        .fts_limit
        .unwrap_or(HYBRID_DEFAULT_FTS_LIMIT)
        .clamp(1, HYBRID_SEARCH_MAX_LIMIT);
    let vector_limit = body
        .vector_limit
        .unwrap_or(HYBRID_DEFAULT_VECTOR_LIMIT)
        .clamp(1, HYBRID_SEARCH_MAX_LIMIT);
    let out_limit = body
        .limit
        .unwrap_or(HYBRID_DEFAULT_OUTPUT_LIMIT)
        .clamp(1, HYBRID_SEARCH_MAX_LIMIT);
    let rrf_k = body.rrf_k.unwrap_or(RRF_K_DEFAULT).max(1);

    let mem_results = state
        .memory
        .search(&session_id, query, fts_limit)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Memory search failed: {e}"),
            )
        })?;

    let query_embedding = embed_port.embed_one(query).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Embedding failed: {e}"),
        )
    })?;

    let vec_results = vector_store
        .search(&query_embedding, vector_limit)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Vector search failed: {e}"),
            )
        })?;

    let mut fts_map: HashMap<String, rusvel_core::domain::MemoryEntry> = HashMap::new();
    let mut fts_ids: Vec<String> = Vec::new();
    for e in mem_results {
        let Some(id) = e.id else {
            continue;
        };
        let key = format!("fts:{}", id);
        fts_ids.push(key.clone());
        fts_map.insert(key, e);
    }

    let mut vec_map: HashMap<String, VectorSearchResult> = HashMap::new();
    let mut vec_ids: Vec<String> = Vec::new();
    for r in vec_results {
        let key = format!("vec:{}", r.entry.id);
        vec_ids.push(key.clone());
        vec_map.insert(key, r);
    }

    let fused = reciprocal_rank_fusion(&[fts_ids, vec_ids], rrf_k);
    let mut hits = Vec::new();
    for (key, score) in fused.into_iter().take(out_limit) {
        if let Some(entry) = fts_map.get(&key) {
            hits.push(HybridSearchHit {
                fusion_key: key,
                rrf_score: score,
                source: HybridHitSource::Fts,
                content: entry.content.clone(),
                metadata: serde_json::json!({
                    "memory_kind": format!("{:?}", entry.kind),
                    "memory": entry.metadata,
                }),
            });
        } else if let Some(vr) = vec_map.get(&key) {
            hits.push(HybridSearchHit {
                fusion_key: key,
                rrf_score: score,
                source: HybridHitSource::Vector,
                content: vr.entry.content.clone(),
                metadata: serde_json::json!({
                    "vector_score": vr.score,
                    "source": vr.entry.source,
                    "entry_metadata": vr.entry.metadata,
                }),
            });
        }
    }

    Ok(Json(HybridSearchResponse { hits }))
}

/// GET /api/knowledge/stats — return knowledge base statistics.
pub async fn knowledge_stats(
    State(state): State<Arc<AppState>>,
) -> Result<Json<KnowledgeStatsResponse>, (StatusCode, String)> {
    let embed_port = state.embedding.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Embedding service not available".into(),
        )
    })?;
    let vector_store = state.vector_store.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector store not available".into(),
        )
    })?;

    let total_entries = vector_store.count().await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Count failed: {e}"),
        )
    })?;

    let model_name = embed_port.model_name().to_string();
    let dimensions = embed_port.dimensions();

    Ok(Json(KnowledgeStatsResponse {
        total_entries,
        model_name,
        dimensions,
    }))
}

/// GET /api/knowledge/related — vector search over auto-indexed entries; optional `dept` filters
/// by emitting department (`event.source`, e.g. `code`, `content`).
pub async fn related_knowledge(
    State(state): State<Arc<AppState>>,
    Query(q): Query<RelatedQuery>,
) -> Result<Json<RelatedKnowledgeResponse>, (StatusCode, String)> {
    let query = q.query.trim();
    if query.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "query must not be empty".into()));
    }

    let embed_port = state.embedding.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Embedding service not available".into(),
        )
    })?;
    let vector_store = state.vector_store.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "Vector store not available".into(),
        )
    })?;

    let out_limit = q.limit.unwrap_or(10).clamp(1, HYBRID_SEARCH_MAX_LIMIT);
    let fetch_limit = (out_limit * 4).clamp(20, 80);

    let query_embedding = embed_port.embed_one(query).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Embedding failed: {e}"),
        )
    })?;

    let raw = vector_store
        .search(&query_embedding, fetch_limit)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Search failed: {e}"),
            )
        })?;

    let dept_filter = q.dept.as_ref().map(|s| s.trim().to_lowercase());

    let mut hits = Vec::new();
    for r in raw {
        let dept_meta = r
            .entry
            .metadata
            .get("department")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if let Some(ref want) = dept_filter {
            if !want.is_empty() && dept_meta.to_lowercase() != *want {
                continue;
            }
        }
        hits.push(RelatedKnowledgeHit {
            content: r.entry.content.clone(),
            department: dept_meta.to_string(),
            score: r.score,
            entry_id: r.entry.id.clone(),
            metadata: r.entry.metadata.clone(),
        });
        if hits.len() >= out_limit {
            break;
        }
    }

    Ok(Json(RelatedKnowledgeResponse { hits }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_text_basic() {
        let text = "First paragraph.\n\nSecond paragraph.\n\nThird paragraph.";
        let chunks = chunk_text(text, 500);
        assert_eq!(chunks.len(), 1); // All fit in one chunk
        assert!(chunks[0].contains("First"));
        assert!(chunks[0].contains("Third"));
    }

    #[test]
    fn test_chunk_text_splits_long() {
        let text = "A".repeat(300) + "\n\n" + &"B".repeat(300);
        let chunks = chunk_text(&text, 500);
        assert_eq!(chunks.len(), 2);
    }

    #[test]
    fn test_chunk_text_empty() {
        let chunks = chunk_text("", 500);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_text_single_long_paragraph() {
        let words: Vec<String> = (0..200).map(|i| format!("word{i}")).collect();
        let text = words.join(" ");
        let chunks = chunk_text(&text, 100);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 100 + 10); // small tolerance for word boundary
        }
    }
}

//! Knowledge base API — ingest, search, browse, and manage knowledge entries.
//!
//! Uses `EmbeddingPort` for text embedding and `VectorStorePort` for storage/retrieval.
//! Chunking splits text by paragraphs (double newline), capping at 500 chars per chunk.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use rusvel_core::domain::{VectorEntry, VectorSearchResult};

use crate::AppState;

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

#[derive(Debug, Serialize)]
pub struct KnowledgeStatsResponse {
    pub total_entries: usize,
    pub model_name: String,
    pub dimensions: usize,
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

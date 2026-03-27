//! Memory tools: memory_write, memory_read, memory_search, memory_delete.

use std::sync::Arc;

use chrono::Utc;
use rusvel_core::SessionId;
use rusvel_core::domain::{Content, MemoryEntry, MemoryKind, ToolDefinition, ToolResult};
use rusvel_core::ports::MemoryPort;
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(registry: &ToolRegistry, memory: Arc<dyn MemoryPort>) {
    // ── memory_write ───────────────────────────────────────────
    let mem = memory.clone();
    registry
        .register_with_handler(
            ToolDefinition {
                name: "memory_write".into(),
                description: "Store a memory entry. Returns the UUID of the stored entry.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID to scope this memory to"
                        },
                        "content": {
                            "type": "string",
                            "description": "The text content to remember"
                        },
                        "kind": {
                            "type": "string",
                            "enum": ["fact", "conversation", "decision", "preference"],
                            "description": "Type of memory entry. Defaults to 'fact'."
                        }
                    },
                    "required": ["session_id", "content"]
                }),
                searchable: true,
                metadata: json!({"category": "memory"}),
            },
            Arc::new(move |args| {
                let mem = mem.clone();
                Box::pin(async move {
                    let session_str = args["session_id"].as_str().unwrap_or_default();
                    let session_id: SessionId = session_str
                        .parse::<uuid::Uuid>()
                        .map(SessionId::from_uuid)
                        .map_err(|e| {
                            rusvel_core::error::RusvelError::Tool(format!(
                                "memory_write: invalid session_id: {e}"
                            ))
                        })?;

                    let content = args["content"].as_str().unwrap_or_default().to_string();
                    let kind = match args.get("kind").and_then(|v| v.as_str()) {
                        Some("conversation") => MemoryKind::Conversation,
                        Some("decision") => MemoryKind::Decision,
                        Some("preference") => MemoryKind::Preference,
                        _ => MemoryKind::Fact,
                    };

                    let entry = MemoryEntry {
                        id: None,
                        session_id,
                        kind,
                        content,
                        embedding: None,
                        created_at: Utc::now(),
                        metadata: json!({}),
                    };

                    let id = mem.store(entry).await.map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!("memory_write: {e}"))
                    })?;

                    Ok(ToolResult {
                        success: true,
                        output: Content::text(format!("Stored memory entry {id}")),
                        metadata: json!({"id": id.to_string()}),
                    })
                })
            }),
        )
        .await
        .unwrap();

    // ── memory_read ────────────────────────────────────────────
    let mem = memory.clone();
    registry
        .register_with_handler(
            ToolDefinition {
                name: "memory_read".into(),
                description: "Recall a specific memory entry by its UUID.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "UUID of the memory entry to recall"
                        }
                    },
                    "required": ["id"]
                }),
                searchable: true,
                metadata: json!({"category": "memory", "read_only": true}),
            },
            Arc::new(move |args| {
                let mem = mem.clone();
                Box::pin(async move {
                    let id_str = args["id"].as_str().unwrap_or_default();
                    let id: uuid::Uuid = id_str.parse().map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!(
                            "memory_read: invalid id: {e}"
                        ))
                    })?;

                    let entry = mem.recall(&id).await.map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!("memory_read: {e}"))
                    })?;

                    match entry {
                        Some(e) => Ok(ToolResult {
                            success: true,
                            output: Content::text(
                                serde_json::to_string_pretty(&e).unwrap_or_default(),
                            ),
                            metadata: json!({"found": true}),
                        }),
                        None => Ok(ToolResult {
                            success: true,
                            output: Content::text("No memory entry found with that ID"),
                            metadata: json!({"found": false}),
                        }),
                    }
                })
            }),
        )
        .await
        .unwrap();

    // ── memory_search ──────────────────────────────────────────
    let mem = memory.clone();
    registry
        .register_with_handler(
            ToolDefinition {
                name: "memory_search".into(),
                description:
                    "Search memory entries within a session using a text query. Returns matching entries."
                        .into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "session_id": {
                            "type": "string",
                            "description": "Session UUID to search within"
                        },
                        "query": {
                            "type": "string",
                            "description": "Search query text"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results. Defaults to 10."
                        }
                    },
                    "required": ["session_id", "query"]
                }),
                searchable: true,
                metadata: json!({"category": "memory", "read_only": true}),
            },
            Arc::new(move |args| {
                let mem = mem.clone();
                Box::pin(async move {
                    let session_str = args["session_id"].as_str().unwrap_or_default();
                    let session_id: SessionId = session_str
                        .parse::<uuid::Uuid>()
                        .map(SessionId::from_uuid)
                        .map_err(|e| {
                            rusvel_core::error::RusvelError::Tool(format!(
                                "memory_search: invalid session_id: {e}"
                            ))
                        })?;

                    let query = args["query"].as_str().unwrap_or_default();
                    let limit = args
                        .get("limit")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(10) as usize;

                    let entries = mem.search(&session_id, query, limit).await.map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!("memory_search: {e}"))
                    })?;

                    let result = if entries.is_empty() {
                        "No matching memory entries found".to_string()
                    } else {
                        serde_json::to_string_pretty(&entries).unwrap_or_default()
                    };

                    Ok(ToolResult {
                        success: true,
                        output: Content::text(result),
                        metadata: json!({"count": entries.len()}),
                    })
                })
            }),
        )
        .await
        .unwrap();

    // ── memory_delete ──────────────────────────────────────────
    let mem = memory.clone();
    registry
        .register_with_handler(
            ToolDefinition {
                name: "memory_delete".into(),
                description: "Delete a memory entry by its UUID.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "UUID of the memory entry to delete"
                        }
                    },
                    "required": ["id"]
                }),
                searchable: true,
                metadata: json!({"category": "memory", "destructive": true}),
            },
            Arc::new(move |args| {
                let mem = mem.clone();
                Box::pin(async move {
                    let id_str = args["id"].as_str().unwrap_or_default();
                    let id: uuid::Uuid = id_str.parse().map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!(
                            "memory_delete: invalid id: {e}"
                        ))
                    })?;

                    mem.forget(&id).await.map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!("memory_delete: {e}"))
                    })?;

                    Ok(ToolResult {
                        success: true,
                        output: Content::text(format!("Deleted memory entry {id}")),
                        metadata: json!({}),
                    })
                })
            }),
        )
        .await
        .unwrap();
}

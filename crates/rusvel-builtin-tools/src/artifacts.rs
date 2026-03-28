//! Persist Forge doc artifacts (S-049).

use std::sync::Arc;

use forge_engine::save_artifact;
use rusvel_core::domain::{Content, ToolDefinition, ToolResult};
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;
use rusvel_core::ports::StoragePort;
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(registry: &ToolRegistry, storage: Arc<dyn StoragePort>) -> Result<()> {
    let st = storage.clone();
    registry
        .register_with_handler(
            ToolDefinition {
                name: "forge_save_artifact".into(),
                description: "Save a markdown document as a Forge artifact for this session."
                    .into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "session_id": { "type": "string" },
                        "title": { "type": "string" },
                        "body_markdown": { "type": "string" }
                    },
                    "required": ["session_id", "title", "body_markdown"]
                }),
                searchable: false,
                metadata: json!({"category": "forge"}),
            },
            Arc::new(move |args| {
                let storage = st.clone();
                Box::pin(async move {
                    let session_str = args["session_id"].as_str().unwrap_or_default();
                    let session_id: SessionId = session_str
                        .parse::<uuid::Uuid>()
                        .map(SessionId::from_uuid)
                        .map_err(|e| {
                            rusvel_core::error::RusvelError::Tool(format!(
                                "forge_save_artifact: invalid session_id: {e}"
                            ))
                        })?;
                    let title = args["title"].as_str().unwrap_or_default();
                    let body = args["body_markdown"].as_str().unwrap_or_default();
                    let rec = save_artifact(&storage, session_id, title, body)
                        .await
                        .map_err(|e| rusvel_core::error::RusvelError::Tool(e.to_string()))?;
                    Ok(ToolResult {
                        success: true,
                        output: Content::text(format!("Saved artifact {}", rec.id)),
                        metadata: json!({
                            "id": rec.id,
                            "title": rec.title,
                            "created_at": rec.created_at.to_rfc3339(),
                        }),
                    })
                })
            }),
        )
        .await
}

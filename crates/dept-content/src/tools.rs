//! Namespaced agent tools (`content.draft`, `content.adapt`) for registration context.

use std::sync::Arc;

use content_engine::ContentEngine;
use rusvel_core::department::{ToolOutput, ToolRegistrar};
use rusvel_core::domain::{ContentKind, Platform};

fn parse_platform_tool(s: &str) -> Platform {
    match s.to_ascii_lowercase().as_str() {
        "twitter" | "x" => Platform::Twitter,
        "linkedin" => Platform::LinkedIn,
        "devto" | "dev.to" => Platform::DevTo,
        "medium" => Platform::Medium,
        "youtube" => Platform::YouTube,
        "substack" => Platform::Substack,
        "email" => Platform::Email,
        other => Platform::Custom(other.into()),
    }
}

/// Register `content.draft` and `content.adapt` on the department tool registrar.
pub fn register_tools(reg: &mut ToolRegistrar, engine: Arc<ContentEngine>) {
    let eng = engine.clone();
    reg.add(
        "content",
        "content.draft",
        "Draft a blog post or article on a given topic",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Session UUID" },
                "topic": { "type": "string", "description": "Topic to write about" },
                "kind": {
                    "type": "string",
                    "description": "Content kind",
                    "enum": ["LongForm", "Tweet", "Thread", "LinkedInPost", "Blog", "VideoScript", "Email", "Proposal"]
                }
            },
            "required": ["session_id", "topic"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .map(rusvel_core::id::SessionId::from_uuid)
                    .ok_or_else(|| {
                        rusvel_core::error::RusvelError::Validation("session_id required".into())
                    })?;
                let topic = args
                    .get("topic")
                    .and_then(|v| v.as_str())
                    .unwrap_or("general");
                let kind = args
                    .get("kind")
                    .and_then(|v| v.as_str())
                    .and_then(|s| serde_json::from_value(serde_json::json!(s)).ok())
                    .unwrap_or(ContentKind::Blog);
                let item = eng.draft(&session_id, topic, kind).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&item)
                        .unwrap_or_else(|_| "drafted".into()),
                    is_error: false,
                    metadata: serde_json::json!({"content_id": item.id.to_string()}),
                })
            })
        }),
    );

    let eng = engine.clone();
    reg.add(
        "content",
        "content.adapt",
        "Adapt existing content for a specific platform",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Session UUID" },
                "content_id": { "type": "string", "description": "Content item UUID" },
                "platform": {
                    "type": "string",
                    "description": "Target platform",
                    "enum": ["twitter", "linkedin", "devto", "medium", "youtube", "substack", "email"]
                }
            },
            "required": ["session_id", "content_id", "platform"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let session_id = args
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .map(rusvel_core::id::SessionId::from_uuid)
                    .ok_or_else(|| {
                        rusvel_core::error::RusvelError::Validation("session_id required".into())
                    })?;
                let content_id = args
                    .get("content_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse().ok())
                    .map(rusvel_core::id::ContentId::from_uuid)
                    .ok_or_else(|| {
                        rusvel_core::error::RusvelError::Validation("content_id required".into())
                    })?;
                let platform_str = args
                    .get("platform")
                    .and_then(|v| v.as_str())
                    .unwrap_or("twitter");
                let platform = parse_platform_tool(platform_str);
                let item = eng.adapt(&session_id, content_id, platform).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&item)
                        .unwrap_or_else(|_| "adapted".into()),
                    is_error: false,
                    metadata: serde_json::json!({"content_id": item.id.to_string()}),
                })
            })
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_mapping_lowercase() {
        assert_eq!(parse_platform_tool("twitter"), Platform::Twitter);
        assert_eq!(parse_platform_tool("LinkedIn"), Platform::LinkedIn);
        assert_eq!(parse_platform_tool("DEVTO"), Platform::DevTo);
    }
}

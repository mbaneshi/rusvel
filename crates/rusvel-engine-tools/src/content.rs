//! Content engine tools: draft, adapt, publish, list, approve.

use std::sync::Arc;

use rusvel_core::domain::{
    Content, ContentKind, ContentStatus, Platform, ToolDefinition, ToolResult,
};
use rusvel_core::id::{ContentId, SessionId};
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(registry: &ToolRegistry, engine: Arc<content_engine::ContentEngine>) {
    // ── content_draft ─────────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "content_draft".into(),
                    description: "Draft new AI-generated content on a given topic.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" },
                            "topic": { "type": "string", "description": "Content topic" },
                            "kind": {
                                "type": "string",
                                "description": "Content kind",
                                "enum": ["LongForm", "Tweet", "Thread", "LinkedInPost", "Blog", "VideoScript", "Email", "Proposal"],
                                "default": "Blog"
                            }
                        },
                        "required": ["session_id", "topic"]
                    }),
                    searchable: false,
                metadata: json!({"category": "content", "engine": "content"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args)?;
                        let topic = args["topic"].as_str().unwrap_or_default();
                        let kind = args
                            .get("kind")
                            .and_then(|v| v.as_str())
                            .map(parse_content_kind)
                            .unwrap_or(ContentKind::Blog);
                        let deltas = vec![json!({
                            "tool": "content_draft",
                            "phase": "started",
                            "topic": topic,
                            "kind": format!("{kind:?}"),
                        })];
                        match engine.draft(&sid, topic, kind).await {
                            Ok(item) => {
                                let mut deltas = deltas;
                                deltas.push(json!({
                                    "tool": "content_draft",
                                    "phase": "completed",
                                    "content_id": item.id.to_string(),
                                    "title": item.title,
                                }));
                                ok_json_with_deltas(&item, deltas)
                            }
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── content_adapt ─────────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "content_adapt".into(),
                    description: "Adapt existing content for a target platform.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" },
                            "content_id": { "type": "string", "description": "Content item UUID" },
                            "platform": {
                                "type": "string",
                                "description": "Target platform",
                                "enum": ["Twitter", "LinkedIn", "DevTo", "Medium", "YouTube", "Substack", "Email"]
                            }
                        },
                        "required": ["session_id", "content_id", "platform"]
                    }),
                    searchable: false,
                metadata: json!({"category": "content", "engine": "content"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args)?;
                        let cid = parse_content_id(&args)?;
                        let platform = parse_platform(args["platform"].as_str().unwrap_or_default());
                        match engine.adapt(&sid, cid, platform).await {
                            Ok(item) => ok_json(&item),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── content_publish ───────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "content_publish".into(),
                    description: "Publish approved content to a platform. Content must be approved first.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" },
                            "content_id": { "type": "string", "description": "Content item UUID" },
                            "platform": {
                                "type": "string",
                                "description": "Target platform",
                                "enum": ["Twitter", "LinkedIn", "DevTo", "Medium", "YouTube", "Substack", "Email"]
                            }
                        },
                        "required": ["session_id", "content_id", "platform"]
                    }),
                    searchable: false,
                metadata: json!({"category": "content", "engine": "content"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args)?;
                        let cid = parse_content_id(&args)?;
                        let platform = parse_platform(args["platform"].as_str().unwrap_or_default());
                        match engine.publish(&sid, cid, platform).await {
                            Ok(result) => ok_json(&result),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── content_list ──────────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "content_list".into(),
                    description: "List content items for a session, optionally filtered by status."
                        .into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" },
                            "status": {
                                "type": "string",
                                "description": "Filter by status",
                                "enum": ["Draft", "Adapted", "Scheduled", "Published", "Archived"]
                            }
                        },
                        "required": ["session_id"]
                    }),
                    searchable: false,
                    metadata: json!({"category": "content", "engine": "content"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args)?;
                        let status = args
                            .get("status")
                            .and_then(|v| v.as_str())
                            .map(parse_content_status);
                        match engine.list_content(&sid, status).await {
                            Ok(items) => ok_json(&items),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── content_approve ───────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "content_approve".into(),
                    description: "Mark a content item as human-approved (required before publishing).".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "content_id": { "type": "string", "description": "Content item UUID" }
                        },
                        "required": ["content_id"]
                    }),
                    searchable: false,
                metadata: json!({"category": "content", "engine": "content"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let cid = parse_content_id(&args)?;
                        match engine.approve_content(cid).await {
                            Ok(item) => ok_json(&item),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }
}

fn parse_session_id(args: &serde_json::Value) -> rusvel_core::error::Result<SessionId> {
    let s = args["session_id"].as_str().unwrap_or_default();
    let uuid = s.parse::<uuid::Uuid>().map_err(|e| {
        rusvel_core::error::RusvelError::Validation(format!("invalid session_id: {e}"))
    })?;
    Ok(SessionId::from_uuid(uuid))
}

fn parse_content_id(args: &serde_json::Value) -> rusvel_core::error::Result<ContentId> {
    let s = args["content_id"].as_str().unwrap_or_default();
    let uuid = s.parse::<uuid::Uuid>().map_err(|e| {
        rusvel_core::error::RusvelError::Validation(format!("invalid content_id: {e}"))
    })?;
    Ok(ContentId::from_uuid(uuid))
}

fn parse_content_kind(s: &str) -> ContentKind {
    match s {
        "LongForm" => ContentKind::LongForm,
        "Tweet" => ContentKind::Tweet,
        "Thread" => ContentKind::Thread,
        "LinkedInPost" => ContentKind::LinkedInPost,
        "Blog" => ContentKind::Blog,
        "VideoScript" => ContentKind::VideoScript,
        "Email" => ContentKind::Email,
        "Proposal" => ContentKind::Proposal,
        _ => ContentKind::Blog,
    }
}

fn parse_content_status(s: &str) -> ContentStatus {
    match s {
        "Draft" => ContentStatus::Draft,
        "Adapted" => ContentStatus::Adapted,
        "Scheduled" => ContentStatus::Scheduled,
        "Published" => ContentStatus::Published,
        "Archived" => ContentStatus::Archived,
        _ => ContentStatus::Draft,
    }
}

fn parse_platform(s: &str) -> Platform {
    match s {
        "Twitter" => Platform::Twitter,
        "LinkedIn" => Platform::LinkedIn,
        "DevTo" => Platform::DevTo,
        "Medium" => Platform::Medium,
        "YouTube" => Platform::YouTube,
        "Substack" => Platform::Substack,
        "Email" => Platform::Email,
        other => Platform::Custom(other.into()),
    }
}

fn ok_json<T: serde::Serialize>(val: &T) -> rusvel_core::error::Result<ToolResult> {
    ok_json_with_deltas(val, vec![])
}

fn ok_json_with_deltas<T: serde::Serialize>(
    val: &T,
    deltas: Vec<serde_json::Value>,
) -> rusvel_core::error::Result<ToolResult> {
    let output = serde_json::to_string_pretty(val).unwrap_or_default();
    let metadata = if deltas.is_empty() {
        json!({})
    } else {
        json!({ "ag_ui_state_deltas": deltas })
    };
    Ok(ToolResult {
        success: true,
        output: Content::text(output),
        metadata,
    })
}

fn err_result(e: rusvel_core::error::RusvelError) -> rusvel_core::error::Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        output: Content::text(format!("Error: {e}")),
        metadata: json!({}),
    })
}

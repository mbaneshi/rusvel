//! Agent tools for the Support department.

use std::sync::Arc;

use rusvel_core::department::*;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::SessionId;
use support_engine::SupportEngine;
use support_engine::TicketPriority;

fn parse_session_id(args: &serde_json::Value) -> Result<SessionId> {
    args.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(SessionId::from_uuid)
        .ok_or_else(|| RusvelError::Validation("session_id required or invalid".into()))
}

pub fn register(engine: &Arc<SupportEngine>, ctx: &mut RegistrationContext) {
    let eng = engine.clone();
    ctx.tools.add(
        "support",
        "support.tickets.create",
        "Create a support ticket",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "subject": { "type": "string" },
                "description": { "type": "string" },
                "priority": { "type": "string", "enum": ["Low", "Medium", "High", "Urgent"] },
                "requester_email": { "type": "string" }
            },
            "required": ["session_id", "subject", "description", "priority", "requester_email"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let subject = args
                    .get("subject")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let priority: TicketPriority = serde_json::from_value(
                    args.get("priority")
                        .cloned()
                        .unwrap_or(serde_json::json!("Medium")),
                )
                .map_err(|e| RusvelError::Validation(format!("priority: {e}")))?;
                let requester_email = args
                    .get("requester_email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let t = eng
                    .tickets()
                    .create_ticket(sid, subject, description, priority, requester_email)
                    .await?;
                Ok(ToolOutput {
                    content: serde_json::to_string(&t).unwrap_or_default(),
                    is_error: false,
                    metadata: serde_json::json!({ "ticket_id": t.id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "support",
        "support.tickets.list",
        "List support tickets for a session",
        serde_json::json!({
            "type": "object",
            "properties": { "session_id": { "type": "string" } },
            "required": ["session_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let list = eng.tickets().list_tickets(sid).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".into()),
                    is_error: false,
                    metadata: serde_json::json!({ "count": list.len() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "support",
        "support.knowledge.search",
        "Search knowledge base articles by keyword in title or body",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "query": { "type": "string" }
            },
            "required": ["session_id", "query"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let q = args
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_lowercase();
                let articles = eng.knowledge().list_articles(sid).await?;
                let filtered: Vec<_> = articles
                    .into_iter()
                    .filter(|a| {
                        a.title.to_lowercase().contains(&q) || a.content.to_lowercase().contains(&q)
                    })
                    .collect();
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&filtered)
                        .unwrap_or_else(|_| "[]".into()),
                    is_error: false,
                    metadata: serde_json::json!({ "count": filtered.len() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "support",
        "support.nps.calculate_score",
        "Calculate Net Promoter Score from recorded responses",
        serde_json::json!({
            "type": "object",
            "properties": { "session_id": { "type": "string" } },
            "required": ["session_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let score = eng.nps().calculate_nps(sid).await?;
                Ok(ToolOutput {
                    content: format!("{score:.2}"),
                    is_error: false,
                    metadata: serde_json::json!({ "nps": score }),
                })
            })
        }),
    );

    tracing::debug!("support agent tools registered");
}

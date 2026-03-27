//! Agent tools for the GTM department.

use std::sync::Arc;

/// Registered tool names (Sprint 1 GTM stories S-003–S-005).
pub const GTM_TOOL_IDS: &[&str] = &[
    "gtm.crm.add_contact",
    "gtm.crm.list_contacts",
    "gtm.crm.add_deal",
    "gtm.outreach.create_sequence",
    "gtm.invoices.create_invoice",
];

use gtm_engine::GtmEngine;
use gtm_engine::invoice::LineItem;
use gtm_engine::outreach::SequenceStep;
use rusvel_core::department::*;
use rusvel_core::domain::Contact;
use rusvel_core::error::RusvelError;
use rusvel_core::id::{ContactId, SessionId};

fn parse_session_id(args: &serde_json::Value) -> rusvel_core::error::Result<SessionId> {
    args.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(SessionId::from_uuid)
        .ok_or_else(|| RusvelError::Validation("session_id required or invalid".into()))
}

fn parse_contact_id(args: &serde_json::Value) -> rusvel_core::error::Result<ContactId> {
    args.get("contact_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(ContactId::from_uuid)
        .ok_or_else(|| RusvelError::Validation("contact_id required or invalid".into()))
}

pub fn register(engine: &Arc<GtmEngine>, ctx: &mut RegistrationContext) {
    // ── gtm.crm.add_contact ──────────────────────────────────────
    let eng = engine.clone();
    ctx.tools.add(
        "gtm",
        "gtm.crm.add_contact",
        "Add a new contact to the CRM",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Session UUID" },
                "name": { "type": "string", "description": "Contact full name" },
                "email": { "type": "string", "description": "Contact email address" },
                "company": { "type": "string", "description": "Company name (optional)" },
                "metadata": { "type": "object", "description": "Extra metadata (optional)" }
            },
            "required": ["session_id", "name", "email"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let name = args
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let email = args
                    .get("email")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let company = args
                    .get("company")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                let metadata = args
                    .get("metadata")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                let contact = Contact {
                    id: ContactId::new(),
                    session_id: sid,
                    name,
                    emails: vec![email],
                    links: vec![],
                    company,
                    role: None,
                    tags: vec![],
                    last_contacted_at: None,
                    metadata,
                };
                let id = eng.crm().add_contact(sid, contact).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "contact_id": id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "contact_id": id.to_string() }),
                })
            })
        }),
    );

    // ── gtm.crm.list_contacts ────────────────────────────────────
    let eng = engine.clone();
    ctx.tools.add(
        "gtm",
        "gtm.crm.list_contacts",
        "List all contacts in the CRM for a session",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Session UUID" }
            },
            "required": ["session_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let contacts = eng.crm().list_contacts(sid).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&contacts)
                        .unwrap_or_else(|_| "[]".into()),
                    is_error: false,
                    metadata: serde_json::json!({ "count": contacts.len() }),
                })
            })
        }),
    );

    // ── gtm.crm.add_deal ─────────────────────────────────────────
    let eng = engine.clone();
    ctx.tools.add(
        "gtm",
        "gtm.crm.add_deal",
        "Add a new deal to the CRM pipeline",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Session UUID" },
                "contact_id": { "type": "string", "description": "Contact UUID" },
                "title": { "type": "string", "description": "Deal title" },
                "value": { "type": "number", "description": "Deal value in currency units" },
                "stage": { "type": "string", "enum": ["Lead", "Qualified", "Proposal", "Negotiation", "Won", "Lost"], "description": "Pipeline stage" }
            },
            "required": ["session_id", "contact_id", "title", "value", "stage"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let cid = parse_contact_id(&args)?;
                let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let value = args
                    .get("value")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);
                let stage_str = args.get("stage").and_then(|v| v.as_str()).unwrap_or("Lead");
                let stage: gtm_engine::DealStage = serde_json::from_value(
                    serde_json::Value::String(stage_str.to_string()),
                )
                .map_err(|_| RusvelError::Validation(format!("invalid stage: {stage_str}")))?;

                let deal = gtm_engine::Deal {
                    id: gtm_engine::DealId::new(),
                    session_id: sid,
                    contact_id: cid,
                    title,
                    value,
                    stage,
                    notes: String::new(),
                    created_at: chrono::Utc::now(),
                    metadata: serde_json::json!({}),
                };
                let deal_id = eng.crm().add_deal(sid, deal).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "deal_id": deal_id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "deal_id": deal_id.to_string() }),
                })
            })
        }),
    );

    // ── gtm.outreach.create_sequence ─────────────────────────────
    let eng = engine.clone();
    ctx.tools.add(
        "gtm",
        "gtm.outreach.create_sequence",
        "Create a multi-step outreach sequence",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Session UUID" },
                "name": { "type": "string", "description": "Sequence name" },
                "steps": {
                    "type": "array",
                    "description": "Sequence steps",
                    "items": {
                        "type": "object",
                        "properties": {
                            "delay_days": { "type": "integer", "description": "Days to wait before this step" },
                            "channel": { "type": "string", "description": "Channel (email, linkedin, etc.)" },
                            "template": { "type": "string", "description": "Message template name" }
                        },
                        "required": ["delay_days", "channel", "template"]
                    }
                }
            },
            "required": ["session_id", "name", "steps"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let steps_val = args.get("steps").cloned().unwrap_or(serde_json::json!([]));
                let steps: Vec<SequenceStep> = serde_json::from_value(steps_val)
                    .map_err(|e| RusvelError::Validation(format!("invalid steps: {e}")))?;

                let seq_id = eng.outreach().create_sequence(sid, name, steps).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "sequence_id": seq_id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "sequence_id": seq_id.to_string() }),
                })
            })
        }),
    );

    // ── gtm.invoices.create_invoice ──────────────────────────────
    let eng = engine.clone();
    ctx.tools.add(
        "gtm",
        "gtm.invoices.create_invoice",
        "Create an invoice with line items",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string", "description": "Session UUID" },
                "client_name": { "type": "string", "description": "Client name (used to look up or create contact)" },
                "line_items": {
                    "type": "array",
                    "description": "Invoice line items",
                    "items": {
                        "type": "object",
                        "properties": {
                            "description": { "type": "string" },
                            "quantity": { "type": "number" },
                            "unit_price": { "type": "number" }
                        },
                        "required": ["description", "quantity", "unit_price"]
                    }
                },
                "due_date": { "type": "string", "description": "ISO 8601 due date (e.g. 2026-04-30T00:00:00Z)" }
            },
            "required": ["session_id", "client_name", "line_items", "due_date"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let _client_name = args.get("client_name").and_then(|v| v.as_str()).unwrap_or("");
                let items_val = args.get("line_items").cloned().unwrap_or(serde_json::json!([]));
                let items: Vec<LineItem> = serde_json::from_value(items_val)
                    .map_err(|e| RusvelError::Validation(format!("invalid line_items: {e}")))?;
                let due_str = args.get("due_date").and_then(|v| v.as_str()).unwrap_or("");
                let due_date: chrono::DateTime<chrono::Utc> = due_str.parse()
                    .map_err(|_| RusvelError::Validation(format!("invalid due_date: {due_str}")))?;

                // Use a placeholder ContactId; real usage would resolve from client_name
                let contact_id = ContactId::new();
                let inv_id = eng.invoices().create_invoice(sid, contact_id, items, due_date).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "invoice_id": inv_id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "invoice_id": inv_id.to_string() }),
                })
            })
        }),
    );

    tracing::debug!(count = GTM_TOOL_IDS.len(), "gtm agent tools registered");
}

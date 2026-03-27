//! Agent tools for the Legal department.

use std::sync::Arc;

use legal_engine::{ComplianceArea, IpKind, LegalEngine};
use rusvel_core::department::*;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::SessionId;

fn parse_session_id(args: &serde_json::Value) -> Result<SessionId> {
    args.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(SessionId::from_uuid)
        .ok_or_else(|| RusvelError::Validation("session_id required or invalid".into()))
}

pub fn register(engine: &Arc<LegalEngine>, ctx: &mut RegistrationContext) {
    let eng = engine.clone();
    ctx.tools.add(
        "legal",
        "legal.contracts.create",
        "Create a new contract draft",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "title": { "type": "string" },
                "counterparty": { "type": "string" },
                "template": { "type": "string" }
            },
            "required": ["session_id", "title", "counterparty", "template"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let title = args
                    .get("title")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let counterparty = args
                    .get("counterparty")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let template = args
                    .get("template")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let c = eng
                    .contracts()
                    .create_contract(sid, title, counterparty, template)
                    .await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "contract_id": c.id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "contract_id": c.id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "legal",
        "legal.compliance.check",
        "Record a compliance check outcome",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "area": { "type": "string", "enum": ["GDPR", "Privacy", "Licensing", "Tax"] },
                "description": { "type": "string" },
                "passed": { "type": "boolean" },
                "notes": { "type": "string" }
            },
            "required": ["session_id", "area", "description", "passed", "notes"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let area: ComplianceArea = serde_json::from_value(
                    args.get("area")
                        .cloned()
                        .unwrap_or(serde_json::json!("GDPR")),
                )
                .map_err(|e| RusvelError::Validation(format!("area: {e}")))?;
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let passed = args
                    .get("passed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let notes = args
                    .get("notes")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let check = eng
                    .compliance()
                    .add_check(sid, area, description, passed, notes)
                    .await?;
                Ok(ToolOutput {
                    content: serde_json::to_string(&check).unwrap_or_default(),
                    is_error: false,
                    metadata: serde_json::json!({ "check_id": check.id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "legal",
        "legal.ip.register",
        "File an intellectual property asset",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "kind": { "type": "string", "enum": ["Patent", "Trademark", "Copyright", "TradeSecret"] },
                "name": { "type": "string" },
                "description": { "type": "string" }
            },
            "required": ["session_id", "kind", "name", "description"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let kind: IpKind = serde_json::from_value(
                    args.get("kind").cloned().unwrap_or(serde_json::json!("Copyright")),
                )
                .map_err(|e| RusvelError::Validation(format!("kind: {e}")))?;
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let asset = eng.ip().file_asset(sid, kind, name, description).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "asset_id": asset.id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "asset_id": asset.id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "legal",
        "legal.contracts.list",
        "List contracts for a session",
        serde_json::json!({
            "type": "object",
            "properties": { "session_id": { "type": "string" } },
            "required": ["session_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let list = eng.contracts().list_contracts(sid).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".into()),
                    is_error: false,
                    metadata: serde_json::json!({ "count": list.len() }),
                })
            })
        }),
    );

    tracing::debug!("legal agent tools registered");
}

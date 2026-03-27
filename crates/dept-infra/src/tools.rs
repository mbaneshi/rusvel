//! Agent tools for the Infra department.

use std::sync::Arc;

use infra_engine::{CheckStatus, InfraEngine, Severity};
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

pub fn register(engine: &Arc<InfraEngine>, ctx: &mut RegistrationContext) {
    let eng = engine.clone();
    ctx.tools.add(
        "infra",
        "infra.deploy.trigger",
        "Record a deployment (starts as Pending)",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "service": { "type": "string" },
                "version": { "type": "string" },
                "environment": { "type": "string", "description": "e.g. staging, production" }
            },
            "required": ["session_id", "service", "version", "environment"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let service = args
                    .get("service")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let version = args
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let environment = args
                    .get("environment")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let d = eng
                    .deploy()
                    .record_deployment(sid, service, version, environment)
                    .await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "deployment_id": d.id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "deployment_id": d.id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "infra",
        "infra.monitor.status",
        "List health checks and a simple status summary",
        serde_json::json!({
            "type": "object",
            "properties": { "session_id": { "type": "string" } },
            "required": ["session_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let checks = eng.monitor().list_checks(sid).await?;
                let down = checks
                    .iter()
                    .filter(|c| c.status == CheckStatus::Down)
                    .count();
                let degraded = checks
                    .iter()
                    .filter(|c| c.status == CheckStatus::Degraded)
                    .count();
                let summary = serde_json::json!({
                    "check_count": checks.len(),
                    "down": down,
                    "degraded": degraded,
                    "checks": checks,
                });
                Ok(ToolOutput {
                    content: summary.to_string(),
                    is_error: false,
                    metadata: summary,
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "infra",
        "infra.incidents.create",
        "Open an incident",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "title": { "type": "string" },
                "description": { "type": "string" },
                "severity": { "type": "string", "enum": ["P1", "P2", "P3", "P4"] }
            },
            "required": ["session_id", "title", "description", "severity"]
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
                let description = args
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let severity: Severity = serde_json::from_value(
                    args.get("severity")
                        .cloned()
                        .unwrap_or(serde_json::json!("P3")),
                )
                .map_err(|e| RusvelError::Validation(format!("severity: {e}")))?;
                let inc = eng
                    .incidents()
                    .open_incident(sid, title, description, severity)
                    .await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "incident_id": inc.id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "incident_id": inc.id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "infra",
        "infra.incidents.list",
        "List incidents for a session",
        serde_json::json!({
            "type": "object",
            "properties": { "session_id": { "type": "string" } },
            "required": ["session_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let list = eng.incidents().list_incidents(sid).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".into()),
                    is_error: false,
                    metadata: serde_json::json!({ "count": list.len() }),
                })
            })
        }),
    );

    tracing::debug!("infra agent tools registered");
}

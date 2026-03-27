//! Agent tools for the Growth department.

use std::sync::Arc;

use growth_engine::GrowthEngine;
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

pub fn register(engine: &Arc<GrowthEngine>, ctx: &mut RegistrationContext) {
    let eng = engine.clone();
    ctx.tools.add(
        "growth",
        "growth.funnel.add_stage",
        "Add a funnel stage for conversion tracking",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "name": { "type": "string" },
                "order": { "type": "integer" }
            },
            "required": ["session_id", "name", "order"]
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
                let order = args.get("order").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                let id = eng.funnel().add_stage(sid, name, order).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "stage_id": id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "stage_id": id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "growth",
        "growth.cohort.create_cohort",
        "Create a user cohort for retention analysis",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "name": { "type": "string" },
                "size": { "type": "integer" }
            },
            "required": ["session_id", "name", "size"]
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
                let size = args.get("size").and_then(|v| v.as_u64()).unwrap_or(0);
                let id = eng.cohort().create_cohort(sid, name, size).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "cohort_id": id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "cohort_id": id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "growth",
        "growth.kpi.record_kpi",
        "Record a KPI measurement",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "name": { "type": "string" },
                "value": { "type": "number" },
                "unit": { "type": "string" }
            },
            "required": ["session_id", "name", "value", "unit"]
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
                let value = args
                    .get("value")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0);
                let unit = args
                    .get("unit")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let id = eng.kpi().record_kpi(sid, name, value, unit).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "kpi_id": id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "kpi_id": id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "growth",
        "growth.kpi.get_trend",
        "Compare the last two KPI readings for a named metric (simple trend)",
        serde_json::json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" },
                "kpi_name": { "type": "string", "description": "Name of the KPI to compare" }
            },
            "required": ["session_id", "kpi_name"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let sid = parse_session_id(&args)?;
                let kpi_name = args
                    .get("kpi_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let mut kpis = eng.kpi().list_kpis(sid).await?;
                kpis.retain(|k| k.name == kpi_name);
                kpis.sort_by_key(|k| k.recorded_at);
                let trend = if kpis.len() >= 2 {
                    let a = kpis[kpis.len() - 2].value;
                    let b = kpis[kpis.len() - 1].value;
                    serde_json::json!({
                        "kpi_name": kpi_name,
                        "previous": a,
                        "latest": b,
                        "delta": b - a
                    })
                } else {
                    serde_json::json!({
                        "kpi_name": kpi_name,
                        "message": "need at least two readings with the same name",
                        "count": kpis.len()
                    })
                };
                Ok(ToolOutput {
                    content: trend.to_string(),
                    is_error: false,
                    metadata: trend,
                })
            })
        }),
    );

    tracing::debug!("growth agent tools registered");
}

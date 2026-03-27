//! Agent tools for the Flow department.

use std::sync::Arc;

use flow_engine::FlowEngine;
use rusvel_core::department::*;
use rusvel_core::domain::FlowDef;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::{FlowExecutionId, FlowId};

fn parse_flow_id(args: &serde_json::Value) -> Result<FlowId> {
    args.get("flow_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(FlowId::from_uuid)
        .ok_or_else(|| RusvelError::Validation("flow_id required or invalid".into()))
}

fn parse_execution_id(args: &serde_json::Value) -> Result<FlowExecutionId> {
    args.get("execution_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(FlowExecutionId::from_uuid)
        .ok_or_else(|| RusvelError::Validation("execution_id required or invalid".into()))
}

pub fn register(engine: &Arc<FlowEngine>, ctx: &mut RegistrationContext) {
    let eng = engine.clone();
    ctx.tools.add(
        "flow",
        "flow.save",
        "Save a flow definition (full FlowDef JSON)",
        serde_json::json!({
            "type": "object",
            "properties": {
                "flow": { "type": "object", "description": "FlowDef JSON (id, name, nodes, connections, ...)" }
            },
            "required": ["flow"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let flow_val = args.get("flow").cloned().ok_or_else(|| {
                    RusvelError::Validation("flow object required".into())
                })?;
                let flow: FlowDef = serde_json::from_value(flow_val)
                    .map_err(|e| RusvelError::Validation(format!("invalid FlowDef: {e}")))?;
                let id = eng.save_flow(&flow).await?;
                Ok(ToolOutput {
                    content: serde_json::json!({ "flow_id": id.to_string() }).to_string(),
                    is_error: false,
                    metadata: serde_json::json!({ "flow_id": id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "flow",
        "flow.get",
        "Load a flow definition by ID",
        serde_json::json!({
            "type": "object",
            "properties": { "flow_id": { "type": "string" } },
            "required": ["flow_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let fid = parse_flow_id(&args)?;
                let f = eng.get_flow(&fid).await?;
                let content = match f {
                    Some(flow) => serde_json::to_string_pretty(&flow).unwrap_or_default(),
                    None => "null".into(),
                };
                Ok(ToolOutput {
                    content,
                    is_error: false,
                    metadata: serde_json::json!({}),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "flow",
        "flow.list",
        "List all saved flow definitions",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
        Arc::new(move |_args| {
            let eng = eng.clone();
            Box::pin(async move {
                let flows = eng.list_flows().await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&flows).unwrap_or_else(|_| "[]".into()),
                    is_error: false,
                    metadata: serde_json::json!({ "count": flows.len() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "flow",
        "flow.run",
        "Run a flow with trigger payload JSON",
        serde_json::json!({
            "type": "object",
            "properties": {
                "flow_id": { "type": "string" },
                "trigger_data": { "type": "object" }
            },
            "required": ["flow_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let fid = parse_flow_id(&args)?;
                let trigger = args
                    .get("trigger_data")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));
                let exec = eng.run_flow(&fid, trigger).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&exec).unwrap_or_default(),
                    is_error: false,
                    metadata: serde_json::json!({ "execution_id": exec.id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "flow",
        "flow.resume",
        "Resume a flow from checkpoint (execution id string)",
        serde_json::json!({
            "type": "object",
            "properties": {
                "execution_id": { "type": "string", "description": "Flow execution UUID string" }
            },
            "required": ["execution_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let eid = args
                    .get("execution_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| RusvelError::Validation("execution_id required".into()))?;
                let exec = eng.resume_flow(eid).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&exec).unwrap_or_default(),
                    is_error: false,
                    metadata: serde_json::json!({ "execution_id": exec.id.to_string() }),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "flow",
        "flow.get_execution",
        "Get a flow execution record by ID",
        serde_json::json!({
            "type": "object",
            "properties": { "execution_id": { "type": "string" } },
            "required": ["execution_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let eid = parse_execution_id(&args)?;
                let ex = eng.get_execution(&eid).await?;
                let content = match ex {
                    Some(e) => serde_json::to_string_pretty(&e).unwrap_or_default(),
                    None => "null".into(),
                };
                Ok(ToolOutput {
                    content,
                    is_error: false,
                    metadata: serde_json::json!({}),
                })
            })
        }),
    );

    let eng = engine.clone();
    ctx.tools.add(
        "flow",
        "flow.list_executions",
        "List executions for a flow",
        serde_json::json!({
            "type": "object",
            "properties": { "flow_id": { "type": "string" } },
            "required": ["flow_id"]
        }),
        Arc::new(move |args| {
            let eng = eng.clone();
            Box::pin(async move {
                let fid = parse_flow_id(&args)?;
                let list = eng.list_executions(&fid).await?;
                Ok(ToolOutput {
                    content: serde_json::to_string_pretty(&list).unwrap_or_else(|_| "[]".into()),
                    is_error: false,
                    metadata: serde_json::json!({ "count": list.len() }),
                })
            })
        }),
    );

    tracing::debug!("flow agent tools registered");
}

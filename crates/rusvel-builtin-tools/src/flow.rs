//! Flow invocation tool: invoke_flow.

use std::sync::Arc;

use flow_engine::FlowEngine;
use rusvel_core::domain::{Content, ToolDefinition, ToolResult};
use rusvel_core::id::FlowId;
use rusvel_core::ports::{AgentPort, EventPort, StoragePort};
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(
    registry: &ToolRegistry,
    storage: Arc<dyn StoragePort>,
    events: Arc<dyn EventPort>,
    agent: Arc<dyn AgentPort>,
) {
    let engine = Arc::new(FlowEngine::new(storage, events, agent, None, None));

    registry
        .register_with_handler(
            ToolDefinition {
                name: "invoke_flow".into(),
                description:
                    "Execute a DAG workflow flow by its ID. Returns the execution result as JSON."
                        .into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "flow_id": {
                            "type": "string",
                            "description": "UUID of the flow definition to execute"
                        },
                        "input": {
                            "type": "object",
                            "description": "Optional input data passed as trigger_data to the flow"
                        }
                    },
                    "required": ["flow_id"]
                }),
                searchable: true,
                metadata: json!({"category": "flow"}),
            },
            Arc::new(move |args| {
                let engine = engine.clone();
                Box::pin(async move {
                    let flow_id_str = args["flow_id"].as_str().unwrap_or_default();
                    let flow_id: FlowId = flow_id_str
                        .parse::<uuid::Uuid>()
                        .map(FlowId::from_uuid)
                        .map_err(|e| {
                            rusvel_core::error::RusvelError::Tool(format!(
                                "invoke_flow: invalid flow_id: {e}"
                            ))
                        })?;

                    let input = args.get("input").cloned().unwrap_or_else(|| json!({}));

                    let execution = engine.run_flow(&flow_id, input).await.map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!("invoke_flow: {e}"))
                    })?;

                    let output = serde_json::to_string_pretty(&execution).unwrap_or_default();

                    Ok(ToolResult {
                        success: true,
                        output: Content::text(output),
                        metadata: json!({
                            "execution_id": execution.id.to_string(),
                            "flow_id": flow_id.to_string(),
                            "status": format!("{:?}", execution.status),
                        }),
                    })
                })
            }),
        )
        .await
        .unwrap();
}

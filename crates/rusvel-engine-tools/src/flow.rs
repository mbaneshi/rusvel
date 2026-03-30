//! Flow engine tool: invoke_flow — let agents trigger DAG workflows.

use std::sync::Arc;

use rusvel_core::domain::{Content, ToolDefinition, ToolResult};
use rusvel_core::id::FlowId;
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(registry: &ToolRegistry, engine: Arc<flow_engine::FlowEngine>) {
    registry
        .register_with_handler(
            ToolDefinition {
                name: "invoke_flow".into(),
                description: "Execute a DAG workflow (flow) by ID and return its execution result.\n\n\
                    WHEN TO USE: Triggering multi-step automations, running predefined pipelines, \
                    orchestrating cross-department workflows.\n\
                    WHEN NOT TO USE: Simple single-tool operations (call the tool directly).\n\n\
                    TIPS:\n\
                    - Get available flows first: use the API or ask the user\n\
                    - trigger_data is passed to the first node as input\n\
                    - Returns execution status, node results, and any errors".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "flow_id": {
                            "type": "string",
                            "description": "UUID of the flow to execute"
                        },
                        "trigger_data": {
                            "type": "object",
                            "description": "JSON data passed as input to the flow's entry node. Optional.",
                            "default": {}
                        }
                    },
                    "required": ["flow_id"]
                }),
                searchable: true,
                metadata: json!({"category": "flow", "read_only": false}),
            },
            Arc::new(move |args| {
                let engine = engine.clone();
                Box::pin(async move {
                    let flow_id_str = args["flow_id"]
                        .as_str()
                        .ok_or_else(|| rusvel_core::error::RusvelError::Tool("flow_id required".into()))?;

                    let uuid = uuid::Uuid::parse_str(flow_id_str).map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!("invalid flow_id: {e}"))
                    })?;
                    let flow_id = FlowId::from_uuid(uuid);

                    let trigger_data = args
                        .get("trigger_data")
                        .cloned()
                        .unwrap_or(json!({}));

                    let execution = engine.run_flow(&flow_id, trigger_data).await.map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!("flow execution failed: {e}"))
                    })?;

                    let summary = json!({
                        "execution_id": execution.id.to_string(),
                        "status": format!("{:?}", execution.status),
                        "nodes_executed": execution.node_results.len(),
                        "error": execution.error,
                        "node_results": execution.node_results.iter().map(|(node_id, nr)| {
                            json!({
                                "node_id": node_id,
                                "status": format!("{:?}", nr.status),
                                "output": nr.output,
                                "error": nr.error,
                            })
                        }).collect::<Vec<_>>(),
                    });

                    Ok(ToolResult {
                        success: true,
                        output: Content::text(serde_json::to_string_pretty(&summary).unwrap_or_default()),
                        metadata: json!({"execution_id": execution.id.to_string()}),
                    })
                })
            }),
        )
        .await
        .unwrap();
}

//! DAG executor — builds a petgraph from a FlowDef and walks it.

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use petgraph::visit::{EdgeRef, Topo};
use tokio::sync::Mutex;

use rusvel_core::domain::*;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::{FlowExecutionId, FlowNodeId};

use crate::nodes::{NodeContext, NodeRegistry};

/// Execute a flow definition, producing a FlowExecution record.
pub async fn execute_flow(
    flow: &FlowDef,
    trigger_data: serde_json::Value,
    registry: &NodeRegistry,
) -> Result<FlowExecution> {
    let exec_id = FlowExecutionId::new();
    let started_at = Utc::now();
    let node_results: Arc<Mutex<HashMap<String, FlowNodeResult>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // Map node IDs to graph indices
    let mut graph: StableDiGraph<FlowNodeId, String> = StableDiGraph::new();
    let mut id_to_index: HashMap<FlowNodeId, NodeIndex> = HashMap::new();
    let mut id_to_node: HashMap<FlowNodeId, &FlowNodeDef> = HashMap::new();
    // Map from node ID string (for expression resolution) to FlowNodeId
    let mut name_to_id: HashMap<String, FlowNodeId> = HashMap::new();

    for node in &flow.nodes {
        let idx = graph.add_node(node.id);
        id_to_index.insert(node.id, idx);
        id_to_node.insert(node.id, node);
        name_to_id.insert(node.id.to_string(), node.id);
        // Also allow lookup by node name
        name_to_id.insert(node.name.clone(), node.id);
    }

    // Add edges
    for conn in &flow.connections {
        if let (Some(&src_idx), Some(&tgt_idx)) = (
            id_to_index.get(&conn.source_node),
            id_to_index.get(&conn.target_node),
        ) {
            graph.add_edge(src_idx, tgt_idx, conn.source_output.clone());
        }
    }

    // Detect cycles
    if petgraph::algo::is_cyclic_directed(&graph) {
        return Err(RusvelError::Validation("Flow contains a cycle".into()));
    }

    // Topological walk
    let outputs: Arc<Mutex<HashMap<FlowNodeId, serde_json::Value>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let mut topo = Topo::new(&graph);
    while let Some(node_idx) = topo.next(&graph) {
        let node_id = graph[node_idx];
        let node_def = match id_to_node.get(&node_id) {
            Some(n) => *n,
            None => continue,
        };

        // Mark as running
        {
            let mut results = node_results.lock().await;
            results.insert(
                node_id.to_string(),
                FlowNodeResult {
                    status: FlowNodeStatus::Running,
                    output: None,
                    error: None,
                    started_at: Some(Utc::now()),
                    finished_at: None,
                },
            );
        }

        // Gather inputs from upstream nodes
        let mut inputs: HashMap<String, serde_json::Value> = HashMap::new();
        for edge in graph.edges_directed(node_idx, petgraph::Direction::Incoming) {
            let source_id = graph[edge.source()];
            let output_name = edge.weight();
            let outs = outputs.lock().await;
            if let Some(data) = outs.get(&source_id) {
                // Check if this edge matches the output that was produced
                inputs.insert(source_id.to_string(), data.clone());
            }
            drop(outs);
            let _ = output_name; // used for routing in connection matching
        }

        // If no upstream inputs, use trigger data
        if inputs.is_empty() {
            inputs.insert("trigger".into(), trigger_data.clone());
        }

        // Find handler
        let handler = match registry.get(&node_def.node_type) {
            Some(h) => h,
            None => {
                let mut results = node_results.lock().await;
                results.insert(
                    node_id.to_string(),
                    FlowNodeResult {
                        status: FlowNodeStatus::Failed,
                        output: None,
                        error: Some(format!("Unknown node type: {}", node_def.node_type)),
                        started_at: Some(Utc::now()),
                        finished_at: Some(Utc::now()),
                    },
                );
                return Err(RusvelError::Internal(format!(
                    "Unknown node type: {}",
                    node_def.node_type
                )));
            }
        };

        let ctx = NodeContext {
            node: node_def.clone(),
            inputs,
            variables: flow.variables.clone(),
        };

        match handler.execute(&ctx).await {
            Ok(output) => {
                // Store output for downstream nodes
                outputs.lock().await.insert(node_id, output.data.clone());

                let mut results = node_results.lock().await;
                results.insert(
                    node_id.to_string(),
                    FlowNodeResult {
                        status: FlowNodeStatus::Succeeded,
                        output: Some(output.data),
                        error: None,
                        started_at: Some(Utc::now()),
                        finished_at: Some(Utc::now()),
                    },
                );

                // For condition nodes, skip edges that don't match the output_name
                if node_def.node_type == "condition" {
                    // Mark downstream nodes on non-matching branches as skipped
                    for edge in graph.edges_directed(node_idx, petgraph::Direction::Outgoing) {
                        let edge_output = edge.weight();
                        if *edge_output != output.output_name {
                            let target_id = graph[edge.target()];
                            mark_subtree_skipped(
                                &graph,
                                edge.target(),
                                &node_results,
                            )
                            .await;
                            let _ = target_id;
                        }
                    }
                }
            }
            Err(e) => {
                let mut results = node_results.lock().await;
                results.insert(
                    node_id.to_string(),
                    FlowNodeResult {
                        status: FlowNodeStatus::Failed,
                        output: None,
                        error: Some(e.to_string()),
                        started_at: Some(Utc::now()),
                        finished_at: Some(Utc::now()),
                    },
                );

                match node_def.on_error {
                    FlowErrorBehavior::StopFlow => {
                        return Ok(FlowExecution {
                            id: exec_id,
                            flow_id: flow.id,
                            status: FlowExecutionStatus::Failed,
                            trigger_data,
                            node_results: results.clone(),
                            started_at,
                            finished_at: Some(Utc::now()),
                            error: Some(e.to_string()),
                            metadata: serde_json::json!({}),
                        });
                    }
                    FlowErrorBehavior::ContinueOnFail => {
                        // Continue to next node
                    }
                    FlowErrorBehavior::UseErrorOutput => {
                        // Store error as output for "error" connections
                        outputs.lock().await.insert(
                            node_id,
                            serde_json::json!({"error": e.to_string()}),
                        );
                    }
                }
            }
        }

        // Skip nodes already marked as skipped
        let results = node_results.lock().await;
        if let Some(r) = results.get(&node_id.to_string()) {
            if r.status == FlowNodeStatus::Skipped {
                continue;
            }
        }
    }

    let final_results = node_results.lock().await.clone();
    let all_succeeded = final_results
        .values()
        .all(|r| r.status == FlowNodeStatus::Succeeded || r.status == FlowNodeStatus::Skipped);

    Ok(FlowExecution {
        id: exec_id,
        flow_id: flow.id,
        status: if all_succeeded {
            FlowExecutionStatus::Succeeded
        } else {
            FlowExecutionStatus::Failed
        },
        trigger_data,
        node_results: final_results,
        started_at,
        finished_at: Some(Utc::now()),
        error: None,
        metadata: serde_json::json!({}),
    })
}

/// Recursively mark all downstream nodes as skipped.
async fn mark_subtree_skipped(
    graph: &StableDiGraph<FlowNodeId, String>,
    start: NodeIndex,
    results: &Arc<Mutex<HashMap<String, FlowNodeResult>>>,
) {
    let node_id = graph[start];
    let mut r = results.lock().await;
    r.insert(
        node_id.to_string(),
        FlowNodeResult {
            status: FlowNodeStatus::Skipped,
            output: None,
            error: None,
            started_at: None,
            finished_at: None,
        },
    );
    drop(r);

    for edge in graph.edges_directed(start, petgraph::Direction::Outgoing) {
        Box::pin(mark_subtree_skipped(graph, edge.target(), results)).await;
    }
}

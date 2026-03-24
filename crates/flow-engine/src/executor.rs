//! DAG executor — builds a petgraph from a FlowDef and walks it.

use std::collections::{HashMap, HashSet};

use chrono::Utc;
use petgraph::stable_graph::{NodeIndex, StableDiGraph};
use petgraph::visit::{EdgeRef, Topo};

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

    // Build the graph
    let mut graph: StableDiGraph<FlowNodeId, String> = StableDiGraph::new();
    let mut id_to_index: HashMap<FlowNodeId, NodeIndex> = HashMap::new();
    let mut id_to_node: HashMap<FlowNodeId, &FlowNodeDef> = HashMap::new();

    for node in &flow.nodes {
        let idx = graph.add_node(node.id);
        id_to_index.insert(node.id, idx);
        id_to_node.insert(node.id, node);
    }

    for conn in &flow.connections {
        if let (Some(&src_idx), Some(&tgt_idx)) = (
            id_to_index.get(&conn.source_node),
            id_to_index.get(&conn.target_node),
        ) {
            graph.add_edge(src_idx, tgt_idx, conn.source_output.clone());
        }
    }

    if petgraph::algo::is_cyclic_directed(&graph) {
        return Err(RusvelError::Validation("Flow contains a cycle".into()));
    }

    // State: node outputs and results (no mutex needed — sequential walk)
    let mut outputs: HashMap<FlowNodeId, serde_json::Value> = HashMap::new();
    let mut node_results: HashMap<String, FlowNodeResult> = HashMap::new();
    let mut skipped: HashSet<FlowNodeId> = HashSet::new();

    // Walk in topological order
    let mut topo = Topo::new(&graph);
    while let Some(node_idx) = topo.next(&graph) {
        let node_id = graph[node_idx];
        let node_def = match id_to_node.get(&node_id) {
            Some(n) => *n,
            None => continue,
        };

        // Skip if marked by a condition branch
        if skipped.contains(&node_id) {
            node_results.insert(
                node_id.to_string(),
                FlowNodeResult {
                    status: FlowNodeStatus::Skipped,
                    output: None,
                    error: None,
                    started_at: None,
                    finished_at: None,
                },
            );
            continue;
        }

        // Gather inputs from upstream nodes
        let mut inputs: HashMap<String, serde_json::Value> = HashMap::new();
        for edge in graph.edges_directed(node_idx, petgraph::Direction::Incoming) {
            let source_id = graph[edge.source()];
            if let Some(data) = outputs.get(&source_id) {
                inputs.insert(source_id.to_string(), data.clone());
            }
        }

        // Root nodes get trigger data
        if inputs.is_empty() {
            inputs.insert("trigger".into(), trigger_data.clone());
        }

        // Find handler
        let handler = match registry.get(&node_def.node_type) {
            Some(h) => h,
            None => {
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
                outputs.insert(node_id, output.data.clone());
                node_results.insert(
                    node_id.to_string(),
                    FlowNodeResult {
                        status: FlowNodeStatus::Succeeded,
                        output: Some(output.data),
                        error: None,
                        started_at: Some(Utc::now()),
                        finished_at: Some(Utc::now()),
                    },
                );

                // Condition routing: mark non-matching branches as skipped
                if node_def.node_type == "condition" {
                    for edge in graph.edges_directed(node_idx, petgraph::Direction::Outgoing) {
                        if *edge.weight() != output.output_name {
                            mark_skipped(&graph, edge.target(), &mut skipped);
                        }
                    }
                }
            }
            Err(e) => {
                node_results.insert(
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
                            node_results,
                            started_at,
                            finished_at: Some(Utc::now()),
                            error: Some(e.to_string()),
                            metadata: serde_json::json!({}),
                        });
                    }
                    FlowErrorBehavior::ContinueOnFail => {}
                    FlowErrorBehavior::UseErrorOutput => {
                        outputs
                            .insert(node_id, serde_json::json!({"error": e.to_string()}));
                    }
                }
            }
        }
    }

    let all_ok = node_results
        .values()
        .all(|r| r.status == FlowNodeStatus::Succeeded || r.status == FlowNodeStatus::Skipped);

    Ok(FlowExecution {
        id: exec_id,
        flow_id: flow.id,
        status: if all_ok {
            FlowExecutionStatus::Succeeded
        } else {
            FlowExecutionStatus::Failed
        },
        trigger_data,
        node_results,
        started_at,
        finished_at: Some(Utc::now()),
        error: None,
        metadata: serde_json::json!({}),
    })
}

/// Recursively mark a node and all its downstream descendants as skipped.
fn mark_skipped(
    graph: &StableDiGraph<FlowNodeId, String>,
    node_idx: NodeIndex,
    skipped: &mut HashSet<FlowNodeId>,
) {
    let node_id = graph[node_idx];
    if skipped.insert(node_id) {
        for edge in graph.edges_directed(node_idx, petgraph::Direction::Outgoing) {
            mark_skipped(graph, edge.target(), skipped);
        }
    }
}

//! Node types for the flow engine.
//!
//! Each node implements `NodeHandler` and is registered in `NodeRegistry`.

pub mod agent;
pub mod browser;
pub mod code;
pub mod condition;
pub mod parallel;

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::domain::FlowNodeDef;
use rusvel_core::error::Result;

/// Context passed to a node handler during execution.
pub struct NodeContext {
    /// The node definition being executed.
    pub node: FlowNodeDef,
    /// Outputs from upstream nodes, keyed by node ID string.
    pub inputs: HashMap<String, serde_json::Value>,
    /// Flow-level variables.
    pub variables: HashMap<String, String>,
}

/// Output produced by a node handler.
pub struct NodeOutput {
    /// The data produced by this node.
    pub data: serde_json::Value,
    /// Which output port this routes to: "main", "true", "false", "error".
    pub output_name: String,
}

/// Trait for all node types in a flow.
#[async_trait]
pub trait NodeHandler: Send + Sync {
    /// The node type string this handler matches (e.g. "agent", "code", "condition").
    fn node_type(&self) -> &str;
    /// Execute the node with the given context.
    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput>;
}

/// Registry mapping node_type strings to handler implementations.
pub struct NodeRegistry {
    handlers: HashMap<String, Arc<dyn NodeHandler>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    pub fn register(&mut self, handler: Arc<dyn NodeHandler>) {
        self.handlers
            .insert(handler.node_type().to_string(), handler);
    }

    pub fn get(&self, node_type: &str) -> Option<&Arc<dyn NodeHandler>> {
        self.handlers.get(node_type)
    }

    pub fn node_types(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }
}

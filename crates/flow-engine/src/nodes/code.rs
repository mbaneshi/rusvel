//! Code node — returns a literal JSON value or extracts a field from inputs.
//!
//! Parameters:
//!   - `"value"`: a literal JSON value to output
//!   - `"extract"`: a dot-path to extract from the first input (e.g. "text", "score")

use async_trait::async_trait;
use rusvel_core::error::{Result, RusvelError};

use super::{NodeContext, NodeHandler, NodeOutput};

pub struct CodeNode;

#[async_trait]
impl NodeHandler for CodeNode {
    fn node_type(&self) -> &str {
        "code"
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        // If "value" parameter exists, return it directly
        if let Some(value) = ctx.node.parameters.get("value") {
            return Ok(NodeOutput {
                data: value.clone(),
                output_name: "main".into(),
            });
        }

        // If "extract" parameter exists, extract field from first input
        if let Some(field) = ctx.node.parameters.get("extract").and_then(|v| v.as_str()) {
            let input = ctx.inputs.values().next().ok_or_else(|| {
                RusvelError::Validation("Code node has no inputs to extract from".into())
            })?;

            let extracted = walk_json_path(input, field);
            return Ok(NodeOutput {
                data: extracted,
                output_name: "main".into(),
            });
        }

        // Default: pass through first input
        let data = ctx
            .inputs
            .values()
            .next()
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        Ok(NodeOutput {
            data,
            output_name: "main".into(),
        })
    }
}

/// Walk a dot-separated path into a JSON value.
/// e.g. `walk_json_path({"a": {"b": 5}}, "a.b")` → `5`
fn walk_json_path(value: &serde_json::Value, path: &str) -> serde_json::Value {
    let mut current = value;
    for segment in path.split('.') {
        match current.get(segment) {
            Some(v) => current = v,
            None => return serde_json::Value::Null,
        }
    }
    current.clone()
}

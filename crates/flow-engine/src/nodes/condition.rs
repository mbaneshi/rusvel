//! Condition node — evaluates an expression and routes to "true" or "false" output.

use async_trait::async_trait;
use rusvel_core::error::{Result, RusvelError};

use super::{NodeContext, NodeHandler, NodeOutput};

pub struct ConditionNode;

#[async_trait]
impl NodeHandler for ConditionNode {
    fn node_type(&self) -> &str {
        "condition"
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let expression = ctx
            .node
            .parameters
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                RusvelError::Validation("Condition node requires 'expression' parameter".into())
            })?;

        let mut engine = rhai::Engine::new();
        engine.set_max_operations(1_000);

        let mut scope = rhai::Scope::new();
        for (key, value) in &ctx.inputs {
            let json_str = serde_json::to_string(value).unwrap_or_default();
            scope.push(key.clone(), json_str);
        }
        for (key, value) in &ctx.variables {
            scope.push(key.clone(), value.clone());
        }

        let result = engine
            .eval_with_scope::<bool>(&mut scope, expression)
            .map_err(|e| RusvelError::Internal(format!("Condition eval error: {e}")))?;

        // Pass through the first input as data
        let data = ctx
            .inputs
            .values()
            .next()
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        Ok(NodeOutput {
            data,
            output_name: if result { "true" } else { "false" }.into(),
        })
    }
}

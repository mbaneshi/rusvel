//! Code node — evaluates Rhai scripts for data transformation.

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
        let script = ctx
            .node
            .parameters
            .get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RusvelError::Validation("Code node requires 'script' parameter".into()))?;

        let mut engine = rhai::Engine::new();
        // Inject inputs as variables
        let mut scope = rhai::Scope::new();
        for (key, value) in &ctx.inputs {
            let json_str = serde_json::to_string(value).unwrap_or_default();
            scope.push(key.clone(), json_str);
        }
        for (key, value) in &ctx.variables {
            scope.push(key.clone(), value.clone());
        }

        // Limit execution to prevent infinite loops
        engine.set_max_operations(10_000);

        let result = engine
            .eval_with_scope::<rhai::Dynamic>(&mut scope, script)
            .map_err(|e| RusvelError::Engine(format!("Rhai error: {e}")))?;

        // Convert Rhai result to JSON
        let output = rhai_to_json(&result);

        Ok(NodeOutput {
            data: output,
            output_name: "main".into(),
        })
    }
}

fn rhai_to_json(val: &rhai::Dynamic) -> serde_json::Value {
    if val.is_string() {
        serde_json::Value::String(val.clone().into_string().unwrap_or_default())
    } else if val.is_int() {
        serde_json::json!(val.as_int().unwrap_or(0))
    } else if val.is_float() {
        serde_json::json!(val.as_float().unwrap_or(0.0))
    } else if val.is_bool() {
        serde_json::json!(val.as_bool().unwrap_or(false))
    } else if val.is_unit() {
        serde_json::Value::Null
    } else {
        // Fallback: convert to string
        serde_json::Value::String(format!("{val:?}"))
    }
}

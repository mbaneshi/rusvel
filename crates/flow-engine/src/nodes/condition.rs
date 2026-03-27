//! Condition node — evaluates a simple expression and routes to "true" or "false".
//!
//! Parameters:
//!   - `"field"`: JSON path to extract from input (e.g. "score")
//!   - `"op"`: comparison operator ("==", "!=", ">", ">=", "<", "<=")
//!   - `"value"`: value to compare against
//!
//! Or simply:
//!   - `"result"`: literal boolean (true/false) — always routes to that branch

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
        let params = &ctx.node.parameters;

        // Literal boolean shortcut
        if let Some(result) = params.get("result").and_then(|v| v.as_bool()) {
            return ok_branch(result, &ctx.inputs);
        }

        // Field + operator + value comparison
        let field = params
            .get("field")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                RusvelError::Validation(
                    "Condition needs 'result' (bool) or 'field' + 'op' + 'value'".into(),
                )
            })?;
        let op = params.get("op").and_then(|v| v.as_str()).unwrap_or("==");
        let compare_to = params
            .get("value")
            .cloned()
            .unwrap_or(serde_json::Value::Null);

        // Extract field from first input
        let input = ctx
            .inputs
            .values()
            .next()
            .ok_or_else(|| RusvelError::Validation("Condition node has no inputs".into()))?;

        let actual = walk_path(input, field);
        let result = compare(&actual, op, &compare_to);

        ok_branch(result, &ctx.inputs)
    }
}

fn ok_branch(
    result: bool,
    inputs: &std::collections::HashMap<String, serde_json::Value>,
) -> Result<NodeOutput> {
    let data = inputs
        .values()
        .next()
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    Ok(NodeOutput {
        data,
        output_name: if result { "true" } else { "false" }.into(),
    })
}

fn walk_path(value: &serde_json::Value, path: &str) -> serde_json::Value {
    let mut current = value;
    for segment in path.split('.') {
        match current.get(segment) {
            Some(v) => current = v,
            None => return serde_json::Value::Null,
        }
    }
    current.clone()
}

fn compare(actual: &serde_json::Value, op: &str, expected: &serde_json::Value) -> bool {
    match op {
        "==" => actual == expected,
        "!=" => actual != expected,
        ">" | ">=" | "<" | "<=" => compare_numeric(actual, op, expected),
        _ => false,
    }
}

fn compare_numeric(actual: &serde_json::Value, op: &str, expected: &serde_json::Value) -> bool {
    let a = as_f64(actual);
    let b = as_f64(expected);
    match (a, b) {
        (Some(a), Some(b)) => match op {
            ">" => a > b,
            ">=" => a >= b,
            "<" => a < b,
            "<=" => a <= b,
            _ => false,
        },
        _ => false,
    }
}

fn as_f64(v: &serde_json::Value) -> Option<f64> {
    v.as_f64()
        .or_else(|| v.as_i64().map(|i| i as f64))
        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
}

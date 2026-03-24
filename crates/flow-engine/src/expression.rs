//! Template expression resolver.
//!
//! Resolves `{{node_id}}` placeholders in strings by looking up
//! outputs from previously executed nodes.

use std::collections::HashMap;

use rusvel_core::id::FlowNodeId;

/// Resolve `{{node_id}}` placeholders in a template string.
///
/// Looks up each referenced node ID in the outputs map and substitutes
/// the JSON value as a string.
pub fn resolve_template(
    template: &str,
    outputs: &HashMap<FlowNodeId, serde_json::Value>,
    node_id_map: &HashMap<String, FlowNodeId>,
) -> String {
    let mut result = template.to_string();

    // Find all {{...}} patterns
    while let Some(start) = result.find("{{") {
        if let Some(end) = result[start..].find("}}") {
            let end = start + end + 2;
            let key = result[start + 2..end - 2].trim();

            // Try to resolve: key might be "node_id" or "node_id.field"
            let replacement = resolve_key(key, outputs, node_id_map);
            result.replace_range(start..end, &replacement);
        } else {
            break;
        }
    }

    result
}

fn resolve_key(
    key: &str,
    outputs: &HashMap<FlowNodeId, serde_json::Value>,
    node_id_map: &HashMap<String, FlowNodeId>,
) -> String {
    let parts: Vec<&str> = key.splitn(2, '.').collect();
    let node_key = parts[0];

    let node_id = match node_id_map.get(node_key) {
        Some(id) => id,
        None => return format!("{{{{{key}}}}}"), // leave unresolved
    };

    let value = match outputs.get(node_id) {
        Some(v) => v,
        None => return format!("{{{{{key}}}}}"),
    };

    if parts.len() == 1 {
        match value {
            serde_json::Value::String(s) => s.clone(),
            other => serde_json::to_string(other).unwrap_or_default(),
        }
    } else {
        // Drill into the JSON: "node_id.field"
        let field = parts[1];
        match value.get(field) {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(other) => serde_json::to_string(other).unwrap_or_default(),
            None => format!("{{{{{key}}}}}"),
        }
    }
}

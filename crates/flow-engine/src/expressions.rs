//! MiniJinja `{{ ... }}` resolution inside JSON node parameters.

use minijinja::Environment;
use serde_json::Value;

/// Resolve `{{` … `}}` in strings inside `value` using `context` as template variables.
/// On render error, leaves the original string unchanged.
pub fn resolve_expressions(value: &Value, context: &Value) -> Value {
    match value {
        Value::String(s) => Value::String(resolve_string(s, context)),
        Value::Array(items) => Value::Array(
            items
                .iter()
                .map(|v| resolve_expressions(v, context))
                .collect(),
        ),
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), resolve_expressions(v, context));
            }
            Value::Object(out)
        }
        _ => value.clone(),
    }
}

fn resolve_string(s: &str, context: &Value) -> String {
    if !s.contains("{{") {
        return s.to_string();
    }
    let ctx = minijinja::value::Value::from_serialize(context);
    let env = Environment::new();
    match env.render_str(s, ctx) {
        Ok(out) => out,
        Err(_) => s.to_string(),
    }
}

/// Merge trigger payload and upstream node outputs into one template context object.
pub fn flow_parameter_context(
    trigger_data: &Value,
    inputs: &std::collections::HashMap<String, Value>,
) -> Value {
    let mut map = serde_json::Map::new();
    match trigger_data {
        Value::Object(o) => {
            for (k, v) in o {
                map.insert(k.clone(), v.clone());
            }
        }
        _ => {
            map.insert("trigger".into(), trigger_data.clone());
        }
    }
    for (k, v) in inputs {
        map.insert(k.clone(), v.clone());
    }
    Value::Object(map)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_string_template() {
        let v = serde_json::json!("Hello {{ name }}");
        let ctx = serde_json::json!({ "name": "World" });
        assert_eq!(
            resolve_expressions(&v, &ctx),
            serde_json::json!("Hello World")
        );
    }

    #[test]
    fn nested_object() {
        let v = serde_json::json!({ "x": { "y": "Hi {{ name }}" } });
        let ctx = serde_json::json!({ "name": "there" });
        assert_eq!(
            resolve_expressions(&v, &ctx),
            serde_json::json!({ "x": { "y": "Hi there" } })
        );
    }

    #[test]
    fn plain_string_unchanged() {
        let v = serde_json::json!("no templates here");
        let ctx = serde_json::json!({});
        assert_eq!(resolve_expressions(&v, &ctx), v);
    }

    #[test]
    fn invalid_template_passthrough() {
        let v = serde_json::json!("{{ this is not valid minijinja }}");
        let ctx = serde_json::json!({});
        assert_eq!(resolve_expressions(&v, &ctx), v);
    }
}

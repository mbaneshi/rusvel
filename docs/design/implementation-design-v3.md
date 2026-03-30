# Implementation Design — Reference Repo Integration

> Code samples, domain types, trait changes, and wiring for ADRs 015-019.
> Date: 2026-03-30 | Tracks: Sprints 3-6

---

## Table of Contents

1. [Flow Node Extension (ADR-015)](#1-flow-node-extension-adr-015)
2. [Expression Language (ADR-018)](#2-expression-language-adr-018)
3. [Channel Expansion (ADR-016)](#3-channel-expansion-adr-016)
4. [Cost Tracking (ADR-017)](#4-cost-tracking-adr-017)
5. [Claude Code Hooks (ADR-019)](#5-claude-code-hooks-adr-019)
6. [Wiring Changes (rusvel-app)](#6-wiring-changes-rusvel-app)

---

## 1. Flow Node Extension (ADR-015)

### 1.1 Enhanced NodeHandler Trait

**File:** `crates/flow-engine/src/nodes/mod.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::domain::FlowNodeDef;
use rusvel_core::error::Result;

/// Context passed to a node handler during execution.
pub struct NodeContext {
    pub node: FlowNodeDef,
    pub inputs: HashMap<String, serde_json::Value>,
    pub variables: HashMap<String, String>,
}

/// Output produced by a node handler.
pub struct NodeOutput {
    pub data: serde_json::Value,
    pub output_name: String,
}

/// Port definition for a node type.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PortDef {
    pub name: String,
    pub label: String,
}

/// Input/output port definitions for a node type.
#[derive(Debug, Clone, serde::Serialize)]
pub struct NodePorts {
    pub inputs: Vec<PortDef>,
    pub outputs: Vec<PortDef>,
}

impl NodePorts {
    pub fn single_main() -> Self {
        Self {
            inputs: vec![PortDef { name: "main".into(), label: "Input".into() }],
            outputs: vec![PortDef { name: "main".into(), label: "Output".into() }],
        }
    }
}

/// Trait for all node types in a flow.
#[async_trait]
pub trait NodeHandler: Send + Sync {
    /// The node type string this handler matches.
    fn node_type(&self) -> &str;

    /// Human-readable display name for the UI palette.
    fn display_name(&self) -> &str { self.node_type() }

    /// JSON Schema for this node's parameters (drives frontend config UI).
    fn parameter_schema(&self) -> serde_json::Value {
        serde_json::json!({ "type": "object", "properties": {} })
    }

    /// Input/output port definitions.
    fn ports(&self) -> NodePorts { NodePorts::single_main() }

    /// Execute the node.
    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput>;
}

/// Registry mapping node_type strings to handler implementations.
pub struct NodeRegistry {
    handlers: HashMap<String, Arc<dyn NodeHandler>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self { handlers: HashMap::new() }
    }

    pub fn register(&mut self, handler: Arc<dyn NodeHandler>) {
        self.handlers.insert(handler.node_type().to_string(), handler);
    }

    pub fn get(&self, node_type: &str) -> Option<&Arc<dyn NodeHandler>> {
        self.handlers.get(node_type)
    }

    pub fn node_types(&self) -> Vec<String> {
        self.handlers.keys().cloned().collect()
    }

    /// Return all node type metadata for the frontend palette.
    pub fn node_type_descriptors(&self) -> Vec<serde_json::Value> {
        self.handlers.values().map(|h| {
            serde_json::json!({
                "type": h.node_type(),
                "display_name": h.display_name(),
                "parameter_schema": h.parameter_schema(),
                "ports": h.ports(),
            })
        }).collect()
    }
}
```

### 1.2 LoopNode

**File:** `crates/flow-engine/src/nodes/loop_node.rs`

```rust
use async_trait::async_trait;
use serde_json::json;

use super::{NodeContext, NodeHandler, NodeOutput, NodePorts, PortDef};
use rusvel_core::error::Result;

/// Iterates over an array from upstream, executing downstream per item.
///
/// Parameters:
///   - `items_path`: dot-path to array in inputs (default: first input root)
///   - `max_iterations`: safety limit (default: 1000)
///
/// Outputs:
///   - All items collected into an array on "main"
pub struct LoopNode;

#[async_trait]
impl NodeHandler for LoopNode {
    fn node_type(&self) -> &str { "loop" }
    fn display_name(&self) -> &str { "Loop Over Items" }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "items_path": {
                    "type": "string",
                    "description": "Dot-path to array in inputs (e.g. 'data.results')",
                    "default": ""
                },
                "max_iterations": {
                    "type": "integer",
                    "description": "Safety limit on iterations",
                    "default": 1000
                }
            }
        })
    }

    fn ports(&self) -> NodePorts {
        NodePorts {
            inputs: vec![PortDef { name: "main".into(), label: "Items".into() }],
            outputs: vec![
                PortDef { name: "main".into(), label: "Collected".into() },
            ],
        }
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let params = &ctx.node.parameters;
        let items_path = params.get("items_path")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let max_iter = params.get("max_iterations")
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as usize;

        // Collect all input data into one merged value
        let merged: serde_json::Value = if ctx.inputs.len() == 1 {
            ctx.inputs.values().next().cloned().unwrap_or(json!(null))
        } else {
            json!(ctx.inputs)
        };

        // Extract array via dot-path
        let items = if items_path.is_empty() {
            merged
        } else {
            dot_path(&merged, items_path).unwrap_or(json!(null))
        };

        let arr = items.as_array().cloned().unwrap_or_default();
        let limited = &arr[..arr.len().min(max_iter)];

        // For a pure loop node, we pass items through with index metadata.
        // In a full implementation, each item would trigger downstream execution.
        // Here we collect with loop metadata for downstream consumption.
        let results: Vec<serde_json::Value> = limited.iter().enumerate().map(|(i, item)| {
            json!({
                "index": i,
                "item": item,
            })
        }).collect();

        Ok(NodeOutput {
            data: json!({ "items": results, "count": results.len() }),
            output_name: "main".into(),
        })
    }
}

/// Simple dot-path extraction: "a.b.c" -> value["a"]["b"]["c"]
fn dot_path(value: &serde_json::Value, path: &str) -> Option<serde_json::Value> {
    let mut current = value.clone();
    for segment in path.split('.') {
        current = current.get(segment)?.clone();
    }
    Some(current)
}
```

### 1.3 DelayNode

**File:** `crates/flow-engine/src/nodes/delay.rs`

```rust
use async_trait::async_trait;
use serde_json::json;
use std::time::Duration;

use super::{NodeContext, NodeHandler, NodeOutput};
use rusvel_core::error::Result;

/// Pauses execution for a configurable duration.
///
/// Parameters:
///   - `seconds`: delay in seconds (supports fractional)
///   - `max_seconds`: safety cap (default: 300)
pub struct DelayNode;

#[async_trait]
impl NodeHandler for DelayNode {
    fn node_type(&self) -> &str { "delay" }
    fn display_name(&self) -> &str { "Delay / Wait" }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "seconds": { "type": "number", "description": "Seconds to wait", "default": 1.0 },
                "max_seconds": { "type": "number", "description": "Safety cap", "default": 300.0 }
            },
            "required": ["seconds"]
        })
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let seconds = ctx.node.parameters.get("seconds")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0);
        let max = ctx.node.parameters.get("max_seconds")
            .and_then(|v| v.as_f64())
            .unwrap_or(300.0);

        let clamped = seconds.min(max).max(0.0);
        tokio::time::sleep(Duration::from_secs_f64(clamped)).await;

        // Pass through upstream data
        let upstream = ctx.inputs.values().next().cloned().unwrap_or(json!(null));
        Ok(NodeOutput {
            data: json!({ "delayed_seconds": clamped, "data": upstream }),
            output_name: "main".into(),
        })
    }
}
```

### 1.4 HttpRequestNode

**File:** `crates/flow-engine/src/nodes/http.rs`

```rust
use async_trait::async_trait;
use serde_json::json;

use super::{NodeContext, NodeHandler, NodeOutput, NodePorts, PortDef};
use rusvel_core::error::Result;

/// Makes an HTTP request. Supports GET, POST, PUT, DELETE.
///
/// Parameters:
///   - `url`: request URL (supports MiniJinja templates)
///   - `method`: GET|POST|PUT|DELETE (default: GET)
///   - `headers`: object of header key→value
///   - `body`: request body (for POST/PUT)
///   - `timeout_seconds`: request timeout (default: 30)
pub struct HttpRequestNode;

#[async_trait]
impl NodeHandler for HttpRequestNode {
    fn node_type(&self) -> &str { "http_request" }
    fn display_name(&self) -> &str { "HTTP Request" }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "Request URL" },
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE"],
                    "default": "GET"
                },
                "headers": {
                    "type": "object",
                    "description": "HTTP headers",
                    "additionalProperties": { "type": "string" }
                },
                "body": { "description": "Request body (JSON)" },
                "timeout_seconds": { "type": "integer", "default": 30 }
            },
            "required": ["url"]
        })
    }

    fn ports(&self) -> NodePorts {
        NodePorts {
            inputs: vec![PortDef { name: "main".into(), label: "Input".into() }],
            outputs: vec![
                PortDef { name: "main".into(), label: "Response".into() },
                PortDef { name: "error".into(), label: "Error".into() },
            ],
        }
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let params = &ctx.node.parameters;
        let url = params.get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("http_request: missing 'url' parameter"))?;
        let method = params.get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET");
        let timeout = params.get("timeout_seconds")
            .and_then(|v| v.as_u64())
            .unwrap_or(30);

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .build()?;

        let mut req = match method {
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => client.get(url),
        };

        // Apply headers
        if let Some(headers) = params.get("headers").and_then(|v| v.as_object()) {
            for (k, v) in headers {
                if let Some(val) = v.as_str() {
                    req = req.header(k.as_str(), val);
                }
            }
        }

        // Apply body
        if let Some(body) = params.get("body") {
            req = req.json(body);
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();
        let headers: serde_json::Map<String, serde_json::Value> = resp.headers().iter()
            .filter_map(|(k, v)| {
                v.to_str().ok().map(|val| (k.as_str().to_string(), json!(val)))
            })
            .collect();
        let body_text = resp.text().await.unwrap_or_default();

        // Try parse as JSON, fall back to text
        let body_value = serde_json::from_str::<serde_json::Value>(&body_text)
            .unwrap_or(json!(body_text));

        let output_port = if status >= 400 { "error" } else { "main" };

        Ok(NodeOutput {
            data: json!({
                "status": status,
                "headers": headers,
                "body": body_value,
            }),
            output_name: output_port.into(),
        })
    }
}
```

### 1.5 ToolCallNode

**File:** `crates/flow-engine/src/nodes/tool_call.rs`

```rust
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use super::{NodeContext, NodeHandler, NodeOutput};
use rusvel_core::error::Result;
use rusvel_core::ports::ToolPort;

/// Invokes a registered tool by name.
///
/// Parameters:
///   - `tool_name`: name of the tool to call
///   - `arguments`: JSON arguments to pass (supports templates)
pub struct ToolCallNode {
    pub tool_port: Arc<dyn ToolPort>,
}

#[async_trait]
impl NodeHandler for ToolCallNode {
    fn node_type(&self) -> &str { "tool_call" }
    fn display_name(&self) -> &str { "Call Tool" }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "tool_name": { "type": "string", "description": "Registered tool name" },
                "arguments": { "type": "object", "description": "Arguments to pass to the tool" }
            },
            "required": ["tool_name"]
        })
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let tool_name = ctx.node.parameters.get("tool_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("tool_call: missing 'tool_name'"))?;

        let args = ctx.node.parameters.get("arguments")
            .cloned()
            .unwrap_or(json!({}));

        let result = self.tool_port.call(tool_name, args).await?;

        Ok(NodeOutput {
            data: json!({
                "tool": tool_name,
                "result": result.content.to_string(),
                "is_error": result.is_error,
            }),
            output_name: if result.is_error { "error" } else { "main" }.into(),
        })
    }
}
```

### 1.6 SwitchNode

**File:** `crates/flow-engine/src/nodes/switch.rs`

```rust
use async_trait::async_trait;
use serde_json::json;

use super::{NodeContext, NodeHandler, NodeOutput, NodePorts, PortDef};
use rusvel_core::error::Result;

/// Multi-way branching based on a field value.
///
/// Parameters:
///   - `field`: dot-path to the value to switch on
///   - `cases`: object mapping value → output port name
///   - `default_output`: fallback port (default: "default")
pub struct SwitchNode;

#[async_trait]
impl NodeHandler for SwitchNode {
    fn node_type(&self) -> &str { "switch" }
    fn display_name(&self) -> &str { "Switch / Route" }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "field": { "type": "string", "description": "Dot-path to switch value" },
                "cases": {
                    "type": "object",
                    "description": "Mapping of value → output port name",
                    "additionalProperties": { "type": "string" }
                },
                "default_output": { "type": "string", "default": "default" }
            },
            "required": ["field", "cases"]
        })
    }

    fn ports(&self) -> NodePorts {
        // Dynamic ports — the executor uses output_name to route
        NodePorts {
            inputs: vec![PortDef { name: "main".into(), label: "Input".into() }],
            outputs: vec![
                PortDef { name: "default".into(), label: "Default".into() },
                // Additional ports added dynamically based on cases
            ],
        }
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let params = &ctx.node.parameters;
        let field_path = params.get("field")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let cases = params.get("cases")
            .and_then(|v| v.as_object())
            .cloned()
            .unwrap_or_default();
        let default = params.get("default_output")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let merged = ctx.inputs.values().next().cloned().unwrap_or(json!(null));

        // Extract switch value via dot-path
        let switch_val = if field_path.is_empty() {
            merged.clone()
        } else {
            let mut current = merged.clone();
            for seg in field_path.split('.') {
                current = current.get(seg).cloned().unwrap_or(json!(null));
            }
            current
        };

        let switch_str = match &switch_val {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => format!("{}", switch_val),
        };

        let output_port = cases.get(&switch_str)
            .and_then(|v| v.as_str())
            .unwrap_or(default);

        Ok(NodeOutput {
            data: merged,
            output_name: output_port.to_string(),
        })
    }
}
```

### 1.7 MergeNode

**File:** `crates/flow-engine/src/nodes/merge.rs`

```rust
use async_trait::async_trait;
use serde_json::json;

use super::{NodeContext, NodeHandler, NodeOutput, NodePorts, PortDef};
use rusvel_core::error::Result;

/// Combines outputs from multiple upstream branches into one.
///
/// Parameters:
///   - `mode`: "append" (array concat) | "merge" (object merge) | "wait_all" (collect all)
pub struct MergeNode;

#[async_trait]
impl NodeHandler for MergeNode {
    fn node_type(&self) -> &str { "merge" }
    fn display_name(&self) -> &str { "Merge Branches" }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "mode": {
                    "type": "string",
                    "enum": ["append", "merge", "wait_all"],
                    "default": "wait_all"
                }
            }
        })
    }

    fn ports(&self) -> NodePorts {
        NodePorts {
            inputs: vec![
                PortDef { name: "input_1".into(), label: "Branch 1".into() },
                PortDef { name: "input_2".into(), label: "Branch 2".into() },
            ],
            outputs: vec![PortDef { name: "main".into(), label: "Merged".into() }],
        }
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let mode = ctx.node.parameters.get("mode")
            .and_then(|v| v.as_str())
            .unwrap_or("wait_all");

        let data = match mode {
            "append" => {
                let mut arr = Vec::new();
                for val in ctx.inputs.values() {
                    if let Some(items) = val.as_array() {
                        arr.extend(items.iter().cloned());
                    } else {
                        arr.push(val.clone());
                    }
                }
                json!(arr)
            }
            "merge" => {
                let mut merged = serde_json::Map::new();
                for val in ctx.inputs.values() {
                    if let Some(obj) = val.as_object() {
                        merged.extend(obj.iter().map(|(k, v)| (k.clone(), v.clone())));
                    }
                }
                json!(merged)
            }
            _ => {
                // wait_all: collect all inputs keyed by source
                json!(ctx.inputs)
            }
        };

        Ok(NodeOutput { data, output_name: "main".into() })
    }
}
```

### 1.8 NotifyNode

**File:** `crates/flow-engine/src/nodes/notify.rs`

```rust
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::json;

use super::{NodeContext, NodeHandler, NodeOutput};
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;
use rusvel_core::ports::ChannelPort;

/// Sends a notification via ChannelPort.
///
/// Parameters:
///   - `message`: text to send (supports templates)
///   - `channel`: optional channel kind override
pub struct NotifyNode {
    pub channel: Option<Arc<dyn ChannelPort>>,
}

#[async_trait]
impl NodeHandler for NotifyNode {
    fn node_type(&self) -> &str { "notify" }
    fn display_name(&self) -> &str { "Send Notification" }

    fn parameter_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "message": { "type": "string", "description": "Notification text" },
                "channel": { "type": "string", "description": "Channel kind (optional)" }
            },
            "required": ["message"]
        })
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let message = ctx.node.parameters.get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("(empty notification)");

        let sent = if let Some(ref ch) = self.channel {
            let session_id = SessionId::new();
            let payload = json!({ "text": message });
            ch.send_message(&session_id, payload).await.is_ok()
        } else {
            false
        };

        let upstream = ctx.inputs.values().next().cloned().unwrap_or(json!(null));

        Ok(NodeOutput {
            data: json!({
                "notified": sent,
                "message": message,
                "data": upstream,
            }),
            output_name: "main".into(),
        })
    }
}
```

### 1.9 Registration in dept-flow

**File:** `crates/dept-flow/src/lib.rs` — updated `register()`:

```rust
async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
    use flow_engine::nodes::*;

    let mut registry = NodeRegistry::new();

    // Built-in nodes (existing)
    registry.register(Arc::new(code::CodeNode));
    registry.register(Arc::new(condition::ConditionNode));
    registry.register(Arc::new(agent::AgentNode::new(ctx.agent.clone())));
    registry.register(Arc::new(browser::BrowserTriggerNode));
    registry.register(Arc::new(browser::BrowserActionNode::new(None)));
    registry.register(Arc::new(parallel::ParallelEvaluateNode::new(ctx.agent.clone())));

    // New Tier 1 nodes (ADR-015)
    registry.register(Arc::new(loop_node::LoopNode));
    registry.register(Arc::new(delay::DelayNode));
    registry.register(Arc::new(http::HttpRequestNode));
    registry.register(Arc::new(tool_call::ToolCallNode { tool_port: ctx.tools.clone() }));

    // New Tier 2 nodes
    registry.register(Arc::new(switch::SwitchNode));
    registry.register(Arc::new(merge::MergeNode));
    registry.register(Arc::new(notify::NotifyNode { channel: None }));

    let engine = Arc::new(FlowEngine::new_with_registry(
        ctx.storage.clone(),
        ctx.events.clone(),
        registry,
        None,
        None,
    ));
    let _ = self.engine.set(engine.clone());
    tools::register(&engine, ctx);
    Ok(())
}
```

---

## 2. Expression Language (ADR-018)

### 2.1 Template Resolution Module

**File:** `crates/flow-engine/src/expression.rs`

```rust
use std::collections::HashMap;

use minijinja::{Environment, Value};
use serde_json;

/// Resolve all template strings in a JSON value.
///
/// Context variables:
///   - `inputs.<node_id>.<path>` — upstream node outputs
///   - `env.<KEY>` — environment variables (filtered allowlist)
///   - `variables.<name>` — flow-level variables
///   - `trigger.<path>` — trigger data
///   - `results.<node_id>.<path>` — all node results so far
pub fn resolve_parameters(
    params: &serde_json::Value,
    inputs: &HashMap<String, serde_json::Value>,
    variables: &HashMap<String, String>,
    trigger_data: &serde_json::Value,
    results: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    let mut env = Environment::new();
    // No auto-escaping for non-HTML context
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);

    let context = minijinja::context! {
        inputs => Value::from_serialize(inputs),
        env => env_vars_filtered(),
        variables => Value::from_serialize(variables),
        trigger => Value::from_serialize(trigger_data),
        results => Value::from_serialize(results),
    };

    resolve_value(&env, params, &context)
}

fn resolve_value(
    env: &Environment,
    value: &serde_json::Value,
    context: &Value,
) -> Result<serde_json::Value, String> {
    match value {
        serde_json::Value::String(s) => {
            if s.contains("{{") {
                let rendered = env
                    .render_str(s, context.clone())
                    .map_err(|e| format!("Expression error: {e}"))?;
                // Try parsing rendered string as JSON (for numeric/bool results)
                Ok(serde_json::from_str(&rendered).unwrap_or(serde_json::Value::String(rendered)))
            } else {
                Ok(value.clone())
            }
        }
        serde_json::Value::Object(map) => {
            let mut resolved = serde_json::Map::new();
            for (k, v) in map {
                resolved.insert(k.clone(), resolve_value(env, v, context)?);
            }
            Ok(serde_json::Value::Object(resolved))
        }
        serde_json::Value::Array(arr) => {
            let resolved: Result<Vec<_>, _> = arr.iter()
                .map(|v| resolve_value(env, v, context))
                .collect();
            Ok(serde_json::Value::Array(resolved?))
        }
        _ => Ok(value.clone()),
    }
}

/// Return a filtered set of environment variables safe for templates.
fn env_vars_filtered() -> HashMap<String, String> {
    const ALLOWED_PREFIXES: &[&str] = &["RUSVEL_", "APP_"];
    std::env::vars()
        .filter(|(k, _)| ALLOWED_PREFIXES.iter().any(|p| k.starts_with(p)))
        .collect()
}
```

### 2.2 Integration in Executor

**File:** `crates/flow-engine/src/executor.rs` — add before `handler.execute()`:

```rust
// Before executing each node, resolve template expressions
let resolved_params = expression::resolve_parameters(
    &node_def.parameters,
    &upstream_outputs,
    &flow.variables,
    &trigger_data,
    &completed_outputs,
)?;
let resolved_node = FlowNodeDef {
    parameters: resolved_params,
    ..node_def.clone()
};
let ctx = NodeContext {
    node: resolved_node,
    inputs: upstream_outputs.clone(),
    variables: flow.variables.clone(),
};
let output = handler.execute(&ctx).await?;
```

---

## 3. Channel Expansion (ADR-016)

### 3.1 Expanded ChannelPort Trait

**File:** `crates/rusvel-core/src/ports.rs` — replace existing ChannelPort:

```rust
/// Channel communication port (outbound + optional inbound).
#[async_trait]
pub trait ChannelPort: Send + Sync {
    /// Channel identifier (e.g., "telegram", "discord", "slack").
    fn channel_kind(&self) -> &'static str;

    /// What this channel supports.
    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities::text_only()
    }

    /// Send a plain text message.
    async fn send_message(
        &self,
        session_id: &SessionId,
        payload: serde_json::Value,
    ) -> Result<()>;

    /// Send a rich message (embeds, buttons, media). Default: extract text, send plain.
    async fn send_rich(
        &self,
        session_id: &SessionId,
        payload: RichPayload,
    ) -> Result<DeliveryReceipt> {
        self.send_message(session_id, serde_json::json!({ "text": payload.text })).await?;
        Ok(DeliveryReceipt::sent())
    }

    /// Handle an inbound webhook payload. Default: unsupported.
    async fn handle_inbound(&self, _raw: serde_json::Value) -> Result<InboundMessage> {
        anyhow::bail!("Inbound not supported for {}", self.channel_kind())
    }
}
```

### 3.2 Domain Types

**File:** `crates/rusvel-core/src/domain.rs` — add:

```rust
// ════════════════════════════════════════════════════════════════════
//  Channel types (ADR-016)
// ════════════════════════════════════════════════════════════════════

/// What a channel adapter supports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelCapabilities {
    pub inbound: bool,
    pub rich_text: bool,
    pub embeds: bool,
    pub buttons: bool,
    pub threads: bool,
    pub reactions: bool,
    pub images: bool,
    pub audio: bool,
    pub video: bool,
    pub files: bool,
    pub max_message_length: usize,
}

impl ChannelCapabilities {
    pub fn text_only() -> Self {
        Self {
            inbound: false, rich_text: false, embeds: false, buttons: false,
            threads: false, reactions: false, images: false, audio: false,
            video: false, files: false, max_message_length: 4096,
        }
    }
}

/// Rich outbound message payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RichPayload {
    pub text: String,
    #[serde(default)]
    pub embeds: Vec<Embed>,
    #[serde(default)]
    pub buttons: Vec<Button>,
    #[serde(default)]
    pub media: Vec<MediaAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Embed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub color: Option<u32>,
    pub fields: Vec<EmbedField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbedField {
    pub name: String,
    pub value: String,
    pub inline: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Button {
    pub label: String,
    pub action: String,
    pub style: ButtonStyle,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ButtonStyle {
    #[default]
    Primary,
    Secondary,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAttachment {
    pub kind: MediaKind,
    pub url: Option<String>,
    pub data: Option<Vec<u8>>,
    pub filename: Option<String>,
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaKind {
    Image, Audio, Video, File,
}

/// Normalized inbound message from any channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundMessage {
    pub channel_kind: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub text: Option<String>,
    pub media: Vec<MediaAttachment>,
    pub thread_id: Option<String>,
    pub reply_to_id: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub raw: serde_json::Value,
    pub metadata: serde_json::Value,
}

/// Delivery confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub message_id: String,
    pub timestamp: DateTime<Utc>,
    pub status: DeliveryStatus,
}

impl DeliveryReceipt {
    pub fn sent() -> Self {
        Self {
            message_id: uuid::Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            status: DeliveryStatus::Sent,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Failed(String),
}
```

### 3.3 ChannelRouter

**File:** `crates/rusvel-channel/src/router.rs`

```rust
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;
use rusvel_core::ports::ChannelPort;

/// Routes messages to the appropriate channel adapter(s).
pub struct ChannelRouter {
    channels: HashMap<String, Arc<dyn ChannelPort>>,
    default_channel: Option<String>,
}

impl ChannelRouter {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            default_channel: None,
        }
    }

    pub fn register(&mut self, channel: Arc<dyn ChannelPort>) {
        let kind = channel.channel_kind().to_string();
        if self.default_channel.is_none() {
            self.default_channel = Some(kind.clone());
        }
        self.channels.insert(kind, channel);
    }

    pub fn list_channels(&self) -> Vec<&str> {
        self.channels.keys().map(|s| s.as_str()).collect()
    }

    pub fn get(&self, kind: &str) -> Option<&Arc<dyn ChannelPort>> {
        self.channels.get(kind)
    }

    /// Broadcast to all registered channels.
    pub async fn broadcast(
        &self,
        session_id: &SessionId,
        payload: serde_json::Value,
    ) -> Vec<Result<()>> {
        let mut results = Vec::new();
        for ch in self.channels.values() {
            results.push(ch.send_message(session_id, payload.clone()).await);
        }
        results
    }
}

/// ChannelRouter itself implements ChannelPort, routing by payload["channel"].
#[async_trait]
impl ChannelPort for ChannelRouter {
    fn channel_kind(&self) -> &'static str { "router" }

    async fn send_message(
        &self,
        session_id: &SessionId,
        payload: serde_json::Value,
    ) -> Result<()> {
        let target_kind = payload.get("channel")
            .and_then(|v| v.as_str())
            .or(self.default_channel.as_deref());

        if let Some(kind) = target_kind {
            if let Some(ch) = self.channels.get(kind) {
                return ch.send_message(session_id, payload).await;
            }
        }

        // Fallback: send to default
        if let Some(ref default) = self.default_channel {
            if let Some(ch) = self.channels.get(default) {
                return ch.send_message(session_id, payload).await;
            }
        }

        anyhow::bail!("No channel configured")
    }
}
```

### 3.4 Discord Adapter

**File:** `crates/rusvel-channel/src/discord.rs`

```rust
use async_trait::async_trait;
use rusvel_core::domain::{ChannelCapabilities, DeliveryReceipt, RichPayload};
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;
use rusvel_core::ports::ChannelPort;
use serde_json::json;

/// Discord webhook adapter. Sends messages via Discord webhook URL.
///
/// Env: `RUSVEL_DISCORD_WEBHOOK_URL`
pub struct DiscordChannel {
    webhook_url: String,
    client: reqwest::Client,
}

impl DiscordChannel {
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("RUSVEL_DISCORD_WEBHOOK_URL").ok()?;
        Some(Self {
            webhook_url: url,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl ChannelPort for DiscordChannel {
    fn channel_kind(&self) -> &'static str { "discord" }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities {
            inbound: false, // Webhook is outbound-only; bot mode needed for inbound
            rich_text: true,
            embeds: true,
            buttons: false, // Webhooks don't support components
            threads: false,
            reactions: false,
            images: true,
            audio: false,
            video: false,
            files: true,
            max_message_length: 2000,
        }
    }

    async fn send_message(
        &self,
        _session_id: &SessionId,
        payload: serde_json::Value,
    ) -> Result<()> {
        let text = payload.get("text")
            .or_else(|| payload.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("(empty)");

        let body = json!({ "content": text });

        self.client
            .post(&self.webhook_url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    async fn send_rich(
        &self,
        _session_id: &SessionId,
        payload: RichPayload,
    ) -> Result<DeliveryReceipt> {
        let embeds: Vec<serde_json::Value> = payload.embeds.iter().map(|e| {
            json!({
                "title": e.title,
                "description": e.description,
                "url": e.url,
                "color": e.color,
                "fields": e.fields.iter().map(|f| json!({
                    "name": f.name,
                    "value": f.value,
                    "inline": f.inline,
                })).collect::<Vec<_>>(),
            })
        }).collect();

        let body = json!({
            "content": payload.text,
            "embeds": embeds,
        });

        self.client
            .post(&self.webhook_url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(DeliveryReceipt::sent())
    }
}
```

### 3.5 Slack Adapter

**File:** `crates/rusvel-channel/src/slack.rs`

```rust
use async_trait::async_trait;
use rusvel_core::domain::ChannelCapabilities;
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;
use rusvel_core::ports::ChannelPort;
use serde_json::json;

/// Slack incoming webhook adapter.
///
/// Env: `RUSVEL_SLACK_WEBHOOK_URL`
pub struct SlackChannel {
    webhook_url: String,
    client: reqwest::Client,
}

impl SlackChannel {
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("RUSVEL_SLACK_WEBHOOK_URL").ok()?;
        Some(Self {
            webhook_url: url,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl ChannelPort for SlackChannel {
    fn channel_kind(&self) -> &'static str { "slack" }

    fn capabilities(&self) -> ChannelCapabilities {
        ChannelCapabilities {
            inbound: false,
            rich_text: true,
            embeds: true,   // Slack blocks
            buttons: true,  // Slack interactive (with bot token)
            threads: false,
            reactions: false,
            images: true,
            audio: false,
            video: false,
            files: false,
            max_message_length: 40000,
        }
    }

    async fn send_message(
        &self,
        _session_id: &SessionId,
        payload: serde_json::Value,
    ) -> Result<()> {
        let text = payload.get("text")
            .or_else(|| payload.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("(empty)");

        let body = json!({ "text": text });

        self.client
            .post(&self.webhook_url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}
```

---

## 4. Cost Tracking (ADR-017)

### 4.1 Domain Types

**File:** `crates/rusvel-core/src/domain.rs` — add:

```rust
// ════════════════════════════════════════════════════════════════════
//  Cost tracking (ADR-017)
// ════════════════════════════════════════════════════════════════════

/// A single billable operation record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEvent {
    pub id: String,
    pub department: String,
    pub operation: CostOperation,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub cost_usd: f64,
    pub model: String,
    pub session_id: Option<String>,
    pub context: CostContext,
    pub timestamp: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostOperation {
    LlmCall,
    Embedding,
    ToolExecution,
    FlowNode,
    VectorSearch,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CostContext {
    Chat { message_id: String },
    Flow { execution_id: String, node_id: String },
    Job { job_id: String },
    Agent { run_id: String },
    System,
}

/// Aggregated cost summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub total_usd: f64,
    pub total_tokens_in: u64,
    pub total_tokens_out: u64,
    pub by_department: HashMap<String, f64>,
    pub by_operation: HashMap<String, f64>,
    pub by_model: HashMap<String, f64>,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
}
```

### 4.2 Recording in CostTrackingLlm

**File:** `crates/rusvel-llm/src/cost.rs` — extend existing wrapper:

```rust
// After each LLM call, emit a CostEvent via MetricStore
impl CostTrackingLlm {
    async fn record_cost(
        &self,
        model: &str,
        usage: &TokenUsage,
        department: &str,
        context: CostContext,
    ) {
        let price = self.price_per_token(model);
        let cost_usd = (usage.input_tokens as f64 * price.input)
            + (usage.output_tokens as f64 * price.output);

        let event = CostEvent {
            id: uuid::Uuid::new_v4().to_string(),
            department: department.to_string(),
            operation: CostOperation::LlmCall,
            tokens_in: usage.input_tokens,
            tokens_out: usage.output_tokens,
            cost_usd,
            model: model.to_string(),
            session_id: None,
            context,
            timestamp: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        };

        // Store as metric point for queryable analytics
        let point = MetricPoint {
            name: "cost".to_string(),
            value: cost_usd,
            tags: serde_json::json!({
                "department": event.department,
                "operation": "llm_call",
                "model": event.model,
            }),
            timestamp: event.timestamp,
        };

        if let Err(e) = self.metrics.record(point).await {
            tracing::warn!("Failed to record cost metric: {e}");
        }
    }

    fn price_per_token(&self, model: &str) -> TokenPrice {
        // Pricing per 1M tokens (approximate)
        match model {
            m if m.contains("haiku") => TokenPrice { input: 0.25 / 1_000_000.0, output: 1.25 / 1_000_000.0 },
            m if m.contains("sonnet") => TokenPrice { input: 3.0 / 1_000_000.0, output: 15.0 / 1_000_000.0 },
            m if m.contains("opus") => TokenPrice { input: 15.0 / 1_000_000.0, output: 75.0 / 1_000_000.0 },
            _ => TokenPrice { input: 1.0 / 1_000_000.0, output: 3.0 / 1_000_000.0 },
        }
    }
}

struct TokenPrice {
    input: f64,
    output: f64,
}
```

### 4.3 Cost API Routes

**File:** `crates/rusvel-api/src/analytics.rs` — add:

```rust
use axum::{extract::{Query, State}, Json};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::sync::Arc;

use crate::AppState;

#[derive(Deserialize)]
pub struct CostQuery {
    pub department: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub operation: Option<String>,
}

pub async fn get_costs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CostQuery>,
) -> Json<serde_json::Value> {
    let filter = rusvel_core::domain::MetricFilter {
        name: Some("cost".to_string()),
        since: q.since,
        until: q.until,
        tags: q.department.map(|d| serde_json::json!({ "department": d })),
    };

    let points = state.storage.metrics().query(filter).await.unwrap_or_default();

    let total_usd: f64 = points.iter().map(|p| p.value).sum();

    Json(serde_json::json!({
        "total_usd": total_usd,
        "count": points.len(),
        "points": points,
    }))
}

pub async fn get_cost_summary(
    State(state): State<Arc<AppState>>,
    Query(q): Query<CostQuery>,
) -> Json<serde_json::Value> {
    let filter = rusvel_core::domain::MetricFilter {
        name: Some("cost".to_string()),
        since: q.since,
        until: q.until,
        tags: None,
    };

    let points = state.storage.metrics().query(filter).await.unwrap_or_default();

    let mut by_dept: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    let mut by_model: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    for p in &points {
        if let Some(dept) = p.tags.get("department").and_then(|v| v.as_str()) {
            *by_dept.entry(dept.to_string()).or_default() += p.value;
        }
        if let Some(model) = p.tags.get("model").and_then(|v| v.as_str()) {
            *by_model.entry(model.to_string()).or_default() += p.value;
        }
    }

    Json(serde_json::json!({
        "total_usd": points.iter().map(|p| p.value).sum::<f64>(),
        "by_department": by_dept,
        "by_model": by_model,
        "period": {
            "since": q.since,
            "until": q.until,
        }
    }))
}
```

---

## 5. Claude Code Hooks (ADR-019)

### 5.1 Hook: Pre-Commit Quality Gate

**File:** `.claude/hooks/pre-bash-commit-quality.sh`

```bash
#!/usr/bin/env bash
# PreToolUse hook: Block commits with secrets or non-conventional messages.
# Exit 2 = block. Exit 0 = allow.

set -euo pipefail

input=$(cat)
tool_name=$(echo "$input" | jq -r '.tool_name // ""')
command=$(echo "$input" | jq -r '.tool_input.command // ""')

if [[ "$tool_name" != "Bash" ]]; then
    echo "$input"
    exit 0
fi

if [[ "$command" != git\ commit* ]]; then
    echo "$input"
    exit 0
fi

# Check staged diff for secrets
if git diff --cached --diff-filter=d 2>/dev/null | grep -iE '(api_key|secret_key|password|private_key|token)\s*[:=]\s*["\x27][^"\x27]{8,}' >/dev/null 2>&1; then
    echo "BLOCKED: Possible secrets detected in staged changes. Remove before committing." >&2
    exit 2
fi

# Validate conventional commit format
msg_flag=""
for arg in $command; do
    if [[ "$msg_flag" == "1" ]]; then
        # Simple check: message should start with type:
        if ! echo "$arg" | grep -qE '^"?(feat|fix|refactor|docs|test|chore|perf|ci|build):'; then
            echo "WARNING: Commit message may not follow conventional commits format (feat:, fix:, etc.)" >&2
        fi
        break
    fi
    if [[ "$arg" == "-m" ]]; then
        msg_flag="1"
    fi
done

echo "$input"
exit 0
```

### 5.2 Hook: Post-Edit Auto-Format

**File:** `.claude/hooks/post-edit-format.sh`

```bash
#!/usr/bin/env bash
# PostToolUse hook: Auto-run rustfmt after editing .rs files.

set -euo pipefail

input=$(cat)
tool_name=$(echo "$input" | jq -r '.tool_name // ""')
file_path=$(echo "$input" | jq -r '.tool_input.file_path // ""')

if [[ "$tool_name" != "Edit" && "$tool_name" != "Write" ]]; then
    echo "$input"
    exit 0
fi

if [[ "$file_path" == *.rs ]]; then
    rustfmt "$file_path" 2>/dev/null || true
fi

echo "$input"
exit 0
```

### 5.3 Hook: Block npm

**File:** `.claude/hooks/pre-bash-no-npm.sh`

```bash
#!/usr/bin/env bash
# PreToolUse hook: Block npm commands (enforce pnpm).

set -euo pipefail

input=$(cat)
command=$(echo "$input" | jq -r '.tool_input.command // ""')

if echo "$command" | grep -qE '(^|\s)npm\s'; then
    echo "BLOCKED: Use pnpm, not npm. (Project rule: CLAUDE.md)" >&2
    exit 2
fi

echo "$input"
exit 0
```

### 5.4 Hook: Session Save on Stop

**File:** `.claude/hooks/stop-session-save.sh`

```bash
#!/usr/bin/env bash
# Stop hook: Persist session state for future resumption.
# Non-blocking (async in settings.json).

set -euo pipefail

input=$(cat)
transcript=$(echo "$input" | jq -r '.session.transcript_path // ""')

if [[ -z "$transcript" || ! -f "$transcript" ]]; then
    echo "$input"
    exit 0
fi

SESSION_DIR="$HOME/.claude/session-data"
mkdir -p "$SESSION_DIR"

DATE=$(date +%Y-%m-%d)
SHORT_ID=$(head -c 8 /dev/urandom | xxd -p | head -c 8)
SESSION_FILE="$SESSION_DIR/${DATE}-${SHORT_ID}-session.md"

# Extract project from cwd
PROJECT=$(basename "$(pwd)")

cat > "$SESSION_FILE" << MARKDOWN
# Session: ${DATE}

**Project:** ${PROJECT}
**Last Updated:** $(date -u +"%Y-%m-%dT%H:%M:%SZ")

---

## Transcript Path
${transcript}

## Working Directory
$(pwd)
MARKDOWN

echo "$input"
exit 0
```

### 5.5 Settings Configuration

**File:** `.claude/settings.json` — hooks section:

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "bash .claude/hooks/pre-bash-commit-quality.sh"
          },
          {
            "type": "command",
            "command": "bash .claude/hooks/pre-bash-no-npm.sh"
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit",
        "hooks": [
          {
            "type": "command",
            "command": "bash .claude/hooks/post-edit-format.sh"
          }
        ]
      },
      {
        "matcher": "Write",
        "hooks": [
          {
            "type": "command",
            "command": "bash .claude/hooks/post-edit-format.sh"
          }
        ]
      }
    ],
    "Stop": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "bash .claude/hooks/stop-session-save.sh",
            "async": true,
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

### 5.6 Learn Command

**File:** `.claude/commands/learn.md`

```markdown
---
description: Extract reusable patterns from the current session
---

# Learn from Session

Review this session and extract non-obvious, reusable patterns.

## What to Look For

1. **Error resolution patterns** — specific error → root cause → fix
2. **Architecture decisions** — why an approach was chosen over alternatives
3. **Tool combinations** — non-obvious sequences that solved problems
4. **Workarounds** — library quirks, API limitations, version-specific fixes
5. **Performance insights** — what was slow, what optimized it

## For Each Pattern Found

Create a file at `.claude/skills/learned/<pattern-name>/SKILL.md`:

```
---
name: <pattern-name>
description: <when this pattern applies>
origin: session
---

# <Pattern Title>

## When to Activate
- <trigger condition 1>
- <trigger condition 2>

## Solution
<the pattern/technique with code examples>

## Why This Works
<explanation>
```

Also create `.claude/skills/learned/<pattern-name>/.provenance.json`:

```json
{
  "source": "session",
  "created_at": "<ISO timestamp>",
  "confidence": 0.5,
  "author": "session-extraction"
}
```

## Rules

- Skip trivial patterns (typos, simple syntax)
- Skip patterns already documented in CLAUDE.md
- Focus on patterns that would save 5+ minutes if encountered again
- Set confidence to 0.5 for new patterns (increases with validation)
- Maximum 3 patterns per session (quality over quantity)
```

---

## 6. Wiring Changes (rusvel-app)

### 6.1 Channel Router in main.rs

```rust
// In main.rs, replace single channel with router:

// Before (current):
// let channel: Option<Arc<dyn ChannelPort>> = TelegramChannel::from_env()
//     .map(|c| Arc::new(c) as Arc<dyn ChannelPort>);

// After (ADR-016):
let mut channel_router = ChannelRouter::new();

if let Some(telegram) = TelegramChannel::from_env() {
    channel_router.register(Arc::new(telegram));
}
if let Some(discord) = DiscordChannel::from_env() {
    channel_router.register(Arc::new(discord));
}
if let Some(slack) = SlackChannel::from_env() {
    channel_router.register(Arc::new(slack));
}

let channel: Option<Arc<dyn ChannelPort>> = if channel_router.list_channels().is_empty() {
    None
} else {
    Some(Arc::new(channel_router))
};
```

### 6.2 Cost Analytics Routes

```rust
// In rusvel-api/src/lib.rs, add routes:
.route("/api/analytics/costs", get(analytics::get_costs))
.route("/api/analytics/costs/summary", get(analytics::get_cost_summary))
```

### 6.3 Enhanced Node Types Endpoint

```rust
// In flow_routes.rs, update list_node_types to return full descriptors:
pub async fn list_node_types(
    State(state): State<Arc<AppState>>,
) -> Json<serde_json::Value> {
    if let Some(ref flow_engine) = state.flow_engine {
        // New: return full descriptors with parameter schemas
        Json(serde_json::json!({
            "node_types": flow_engine.node_type_descriptors()
        }))
    } else {
        Json(serde_json::json!({ "node_types": [] }))
    }
}
```

---

## File Impact Summary

| File | Change | ADR | Sprint |
|------|--------|-----|--------|
| `crates/flow-engine/src/nodes/mod.rs` | Add `parameter_schema()`, `ports()`, `NodePorts` | 015 | 3 |
| `crates/flow-engine/src/nodes/loop_node.rs` | New file | 015 | 3 |
| `crates/flow-engine/src/nodes/delay.rs` | New file | 015 | 3 |
| `crates/flow-engine/src/nodes/http.rs` | New file | 015 | 3 |
| `crates/flow-engine/src/nodes/tool_call.rs` | New file | 015 | 3 |
| `crates/flow-engine/src/nodes/switch.rs` | New file | 015 | 4 |
| `crates/flow-engine/src/nodes/merge.rs` | New file | 015 | 4 |
| `crates/flow-engine/src/nodes/notify.rs` | New file | 015 | 4 |
| `crates/flow-engine/src/expression.rs` | New file (MiniJinja) | 018 | 3 |
| `crates/flow-engine/src/executor.rs` | Template resolution before execute | 018 | 3 |
| `crates/flow-engine/Cargo.toml` | Add `minijinja`, `reqwest` deps | 015,018 | 3 |
| `crates/rusvel-core/src/ports.rs` | Expand `ChannelPort` trait | 016 | 4 |
| `crates/rusvel-core/src/domain.rs` | Add channel + cost domain types | 016,017 | 4 |
| `crates/rusvel-channel/src/router.rs` | New file | 016 | 4 |
| `crates/rusvel-channel/src/discord.rs` | New file | 016 | 4 |
| `crates/rusvel-channel/src/slack.rs` | New file | 016 | 5 |
| `crates/rusvel-llm/src/cost.rs` | Extend CostTrackingLlm | 017 | 4 |
| `crates/rusvel-api/src/analytics.rs` | New routes | 017 | 4 |
| `crates/rusvel-api/src/lib.rs` | Add analytics + channel routes | 016,017 | 4 |
| `crates/rusvel-app/src/main.rs` | ChannelRouter wiring | 016 | 4 |
| `crates/dept-flow/src/lib.rs` | Register new node types | 015 | 3 |
| `.claude/hooks/*.sh` | 4 new hook scripts | 019 | 3 |
| `.claude/settings.json` | Hook configuration | 019 | 3 |
| `.claude/commands/learn.md` | New command | 019 | 5 |

**Dependency additions:**
- `flow-engine/Cargo.toml`: `minijinja = "2"`, `reqwest = { version = "0.12", features = ["json"] }`
- `rusvel-channel/Cargo.toml`: `reqwest = { version = "0.12", features = ["json"] }` (if not already)

**New crate line counts (estimated):**
- `flow-engine` new nodes: ~600 lines across 7 files (within 2000-line budget when spread)
- `expression.rs`: ~80 lines
- `rusvel-channel` new files: ~300 lines across 3 files
- Total new Rust: ~1,000 lines

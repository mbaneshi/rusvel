//! `tool_search` — meta-tool for deferred tool loading.
//!
//! Instead of injecting all tools into every LLM prompt, agents call
//! `tool_search` to discover relevant tools on demand. Discovered tools
//! are dynamically added to subsequent LLM requests.

use std::sync::Arc;

use rusvel_core::domain::{Content, ToolDefinition, ToolResult};
use rusvel_core::ports::ToolPort;
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(registry: &ToolRegistry, tools: Arc<dyn ToolPort>) {
    registry
        .register_with_handler(
            ToolDefinition {
                name: "tool_search".into(),
                description: "Search for available tools by keyword. Use this when you need a \
                    tool that isn't in your current set. Returns matching tool names and \
                    descriptions. After calling this, the discovered tools become available \
                    for use in subsequent turns."
                    .into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Keywords describing the tool you need (e.g. 'read file', 'git history', 'search code')"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results. Default: 5."
                        }
                    },
                    "required": ["query"]
                }),
                searchable: false,
                metadata: json!({"category": "meta", "read_only": true}),
            },
            Arc::new(move |args| {
                let tools = tools.clone();
                Box::pin(async move {
                    let query = args["query"].as_str().unwrap_or("");
                    let limit = args
                        .get("limit")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(5) as usize;

                    let results = tools.search(query, limit);

                    if results.is_empty() {
                        return Ok(ToolResult {
                            success: true,
                            output: Content::text(format!(
                                "No tools found matching \"{query}\". Try different keywords."
                            )),
                            metadata: json!({"discovered_tools": []}),
                        });
                    }

                    let mut lines = vec![format!(
                        "Found {} tool(s) matching \"{query}\":\n",
                        results.len()
                    )];
                    let discovered_names: Vec<String> =
                        results.iter().map(|t| t.name.clone()).collect();

                    for tool in &results {
                        lines.push(format!("• **{}** — {}", tool.name, tool.description));
                    }
                    lines.push(String::new());
                    lines.push(
                        "These tools are now available. You can call them directly.".into(),
                    );

                    Ok(ToolResult {
                        success: true,
                        output: Content::text(lines.join("\n")),
                        metadata: json!({"discovered_tools": discovered_names}),
                    })
                })
            }),
        )
        .await
        .unwrap();
}

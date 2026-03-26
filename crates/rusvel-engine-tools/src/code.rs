//! Code engine tools: analyze, search.

use std::path::Path;
use std::sync::Arc;

use rusvel_core::domain::{Content, ToolDefinition, ToolResult};
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(registry: &ToolRegistry, engine: Arc<code_engine::CodeEngine>) {
    // ── code_analyze ──────────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "code_analyze".into(),
                    description: "Analyze a repository: parse Rust files, build symbol graph, compute metrics, and index for search.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "path": {
                                "type": "string",
                                "description": "Path to the repository or directory to analyze"
                            }
                        },
                        "required": ["path"]
                    }),
                    searchable: false,
                metadata: json!({"category": "code", "engine": "code"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let path = args["path"].as_str().unwrap_or(".");
                        let deltas = vec![json!({
                            "tool": "code_analyze",
                            "phase": "started",
                            "path": path
                        })];
                        match engine.analyze(Path::new(path)).await {
                            Ok(analysis) => {
                                let summary = analysis.summary();
                                let mut deltas = deltas;
                                deltas.push(json!({
                                    "tool": "code_analyze",
                                    "phase": "completed",
                                    "repo_path": summary.repo_path,
                                    "total_files": summary.total_files,
                                    "total_symbols": summary.total_symbols,
                                    "snapshot_id": summary.snapshot_id,
                                }));
                                ok_json_with_deltas(&summary, deltas)
                            }
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── code_search ───────────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "code_search".into(),
                    description: "Search previously indexed code symbols using BM25. Requires analyze() to have been called first.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "query": {
                                "type": "string",
                                "description": "Search query"
                            },
                            "limit": {
                                "type": "integer",
                                "description": "Maximum number of results (default 10)",
                                "default": 10
                            }
                        },
                        "required": ["query"]
                    }),
                    searchable: false,
                metadata: json!({"category": "code", "engine": "code"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let query = args["query"].as_str().unwrap_or_default();
                        let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                        match engine.search(query, limit) {
                            Ok(results) => ok_json(&results),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }
}

fn ok_json<T: serde::Serialize>(val: &T) -> rusvel_core::error::Result<ToolResult> {
    ok_json_with_deltas(val, vec![])
}

fn ok_json_with_deltas<T: serde::Serialize>(
    val: &T,
    deltas: Vec<serde_json::Value>,
) -> rusvel_core::error::Result<ToolResult> {
    let output = serde_json::to_string_pretty(val).unwrap_or_default();
    let metadata = if deltas.is_empty() {
        json!({})
    } else {
        json!({ "ag_ui_state_deltas": deltas })
    };
    Ok(ToolResult {
        success: true,
        output: Content::text(output),
        metadata,
    })
}

fn err_result(e: rusvel_core::error::RusvelError) -> rusvel_core::error::Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        output: Content::text(format!("Error: {e}")),
        metadata: json!({}),
    })
}

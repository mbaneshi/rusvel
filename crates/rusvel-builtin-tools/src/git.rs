//! Git tools: git_status, git_diff, git_log.

use std::sync::Arc;

use rusvel_core::domain::{Content, ToolDefinition, ToolResult};
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(registry: &ToolRegistry) {
    // ── git_status ───────────────────────────────────────────────
    registry
        .register_with_handler(
            ToolDefinition {
                name: "git_status".into(),
                description: "Show the working tree status (staged, unstaged, untracked files)."
                    .into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Repository path. Defaults to current directory."
                        }
                    }
                }),
                searchable: false,
                metadata: json!({"category": "git", "read_only": true}),
            },
            Arc::new(|args| {
                Box::pin(async move {
                    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                    let output = tokio::process::Command::new("git")
                        .args(["status", "--short"])
                        .current_dir(path)
                        .output()
                        .await
                        .map_err(|e| {
                            rusvel_core::error::RusvelError::Tool(format!("git_status: {e}"))
                        })?;

                    let text = String::from_utf8_lossy(&output.stdout);
                    let result = if text.is_empty() {
                        "Working tree clean".into()
                    } else {
                        text.into_owned()
                    };

                    Ok(ToolResult {
                        success: output.status.success(),
                        output: Content::text(result),
                        metadata: json!({}),
                    })
                })
            }),
        )
        .await
        .unwrap();

    // ── git_diff ─────────────────────────────────────────────────
    registry
        .register_with_handler(
            ToolDefinition {
                name: "git_diff".into(),
                description: "Show changes between commits, commit and working tree, etc.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Repository path. Defaults to current directory."
                        },
                        "staged": {
                            "type": "boolean",
                            "description": "Show staged changes (--cached). Default: false."
                        }
                    }
                }),
                searchable: false,
                metadata: json!({"category": "git", "read_only": true}),
            },
            Arc::new(|args| {
                Box::pin(async move {
                    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                    let staged = args
                        .get("staged")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    let mut cmd = tokio::process::Command::new("git");
                    cmd.arg("diff").current_dir(path);
                    if staged {
                        cmd.arg("--cached");
                    }

                    let output = cmd.output().await.map_err(|e| {
                        rusvel_core::error::RusvelError::Tool(format!("git_diff: {e}"))
                    })?;

                    let text = String::from_utf8_lossy(&output.stdout);
                    let result = if text.is_empty() {
                        "No changes".into()
                    } else {
                        text.into_owned()
                    };

                    Ok(ToolResult {
                        success: output.status.success(),
                        output: Content::text(result),
                        metadata: json!({}),
                    })
                })
            }),
        )
        .await
        .unwrap();

    // ── git_log ──────────────────────────────────────────────────
    registry
        .register_with_handler(
            ToolDefinition {
                name: "git_log".into(),
                description: "Show recent commit history.".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Repository path. Defaults to current directory."
                        },
                        "count": {
                            "type": "integer",
                            "description": "Number of commits to show. Default: 10."
                        }
                    }
                }),
                searchable: false,
                metadata: json!({"category": "git", "read_only": true}),
            },
            Arc::new(|args| {
                Box::pin(async move {
                    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                    let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(10);

                    let output = tokio::process::Command::new("git")
                        .args(["log", "--oneline", &format!("-{count}")])
                        .current_dir(path)
                        .output()
                        .await
                        .map_err(|e| {
                            rusvel_core::error::RusvelError::Tool(format!("git_log: {e}"))
                        })?;

                    let text = String::from_utf8_lossy(&output.stdout);
                    Ok(ToolResult {
                        success: output.status.success(),
                        output: Content::text(text.into_owned()),
                        metadata: json!({}),
                    })
                })
            }),
        )
        .await
        .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusvel_core::ports::ToolPort;

    #[tokio::test]
    async fn git_status_runs() {
        let registry = ToolRegistry::new();
        register(&registry).await;

        // Should work in the repo root.
        let result = registry.call("git_status", json!({})).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn git_log_runs() {
        let registry = ToolRegistry::new();
        register(&registry).await;

        let result = registry.call("git_log", json!({"count": 3})).await.unwrap();
        assert!(result.success);
    }
}

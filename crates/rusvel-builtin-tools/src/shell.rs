//! Shell execution tool: bash.

use std::sync::Arc;

use rusvel_core::domain::{Content, ToolDefinition, ToolResult};
use rusvel_tool::ToolRegistry;
use serde_json::json;

/// Default timeout for shell commands: 120 seconds.
const DEFAULT_TIMEOUT_MS: u64 = 120_000;

pub async fn register(registry: &ToolRegistry) {
    registry
        .register_with_handler(
            ToolDefinition {
                name: "bash".into(),
                description:
                    "Execute a bash command and return its stdout/stderr. Commands have a timeout."
                        .into(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The bash command to execute"
                        },
                        "timeout_ms": {
                            "type": "integer",
                            "description": "Timeout in milliseconds. Default: 120000 (2 minutes)."
                        }
                    },
                    "required": ["command"]
                }),
                searchable: false,
                metadata: json!({"category": "shell", "destructive": true}),
            },
            Arc::new(|args| {
                Box::pin(async move {
                    let command = args["command"].as_str().unwrap_or("");
                    let timeout_ms = args
                        .get("timeout_ms")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(DEFAULT_TIMEOUT_MS);

                    let result = tokio::time::timeout(
                        std::time::Duration::from_millis(timeout_ms),
                        tokio::process::Command::new("bash")
                            .arg("-c")
                            .arg(command)
                            .output(),
                    )
                    .await;

                    match result {
                        Ok(Ok(output)) => {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            let exit_code = output.status.code().unwrap_or(-1);

                            let mut text = String::new();
                            if !stdout.is_empty() {
                                text.push_str(&stdout);
                            }
                            if !stderr.is_empty() {
                                if !text.is_empty() {
                                    text.push('\n');
                                }
                                text.push_str("STDERR:\n");
                                text.push_str(&stderr);
                            }
                            if text.is_empty() {
                                text = "(no output)".into();
                            }

                            Ok(ToolResult {
                                success: exit_code == 0,
                                output: Content::text(text),
                                metadata: json!({"exit_code": exit_code}),
                            })
                        }
                        Ok(Err(e)) => Err(rusvel_core::error::RusvelError::Tool(format!(
                            "bash exec error: {e}"
                        ))),
                        Err(_) => Err(rusvel_core::error::RusvelError::Tool(format!(
                            "bash command timed out after {timeout_ms}ms"
                        ))),
                    }
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
    async fn bash_echo() {
        let registry = ToolRegistry::new();
        register(&registry).await;

        let result = registry
            .call("bash", json!({"command": "echo hello"}))
            .await
            .unwrap();

        assert!(result.success);
        let text = match &result.output.parts[0] {
            rusvel_core::domain::Part::Text(t) => t.clone(),
            _ => panic!("expected text"),
        };
        assert!(text.contains("hello"));
    }

    #[tokio::test]
    async fn bash_exit_code() {
        let registry = ToolRegistry::new();
        register(&registry).await;

        let result = registry
            .call("bash", json!({"command": "exit 42"}))
            .await
            .unwrap();

        assert!(!result.success);
        assert_eq!(result.metadata["exit_code"], 42);
    }
}

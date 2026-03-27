//! MCP Client — connects to external MCP servers and proxies their tools
//! into RUSVEL's [`ToolRegistry`](rusvel_tool::ToolRegistry).
//!
//! External tools appear identically to built-in tools. The agent runtime
//! does not need to know whether a tool is local or proxied via MCP.

mod transport;

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use rusvel_core::domain::{Content, ToolDefinition, ToolResult};
use rusvel_core::error::{Result, RusvelError};
use rusvel_tool::{ToolHandler, ToolRegistry};
use transport::StdioTransport;

// ════════════════════════════════════════════════════════════════════
//  MCP Server Config (matches rusvel-api's McpServerConfig)
// ════════════════════════════════════════════════════════════════════

/// Configuration for an external MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub server_type: String,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub env: serde_json::Value,
    pub enabled: bool,
    pub metadata: serde_json::Value,
}

// ════════════════════════════════════════════════════════════════════
//  MCP Client
// ════════════════════════════════════════════════════════════════════

/// A connected MCP client that proxies tools from an external MCP server.
pub struct McpClient {
    name: String,
    transport: Arc<StdioTransport>,
    tools: Vec<McpToolInfo>,
}

/// Info about a tool discovered from an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl McpClient {
    /// Connect to an MCP server, perform the initialize handshake, and
    /// discover available tools.
    pub async fn connect(config: &McpServerConfig) -> Result<Self> {
        let command = config.command.as_deref().ok_or_else(|| {
            RusvelError::Tool(format!(
                "MCP server '{}' has no command configured",
                config.name
            ))
        })?;

        // Build environment variables from config.
        let env: HashMap<String, String> = if let Some(obj) = config.env.as_object() {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        } else {
            HashMap::new()
        };

        let transport = StdioTransport::spawn(command, &config.args, env).await?;
        let transport = Arc::new(transport);

        // Initialize handshake.
        let init_result = transport
            .request(
                "initialize",
                serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {
                        "name": "rusvel-mcp-client",
                        "version": "0.1.0"
                    }
                }),
            )
            .await?;

        let server_name = init_result
            .get("serverInfo")
            .and_then(|s| s.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");

        info!(
            server = %config.name,
            mcp_server = %server_name,
            "MCP server initialized"
        );

        // Send initialized notification.
        transport
            .notify("notifications/initialized", serde_json::json!({}))
            .await?;

        // Discover tools.
        let tools_result = transport
            .request("tools/list", serde_json::json!({}))
            .await?;

        let tools: Vec<McpToolInfo> = tools_result
            .get("tools")
            .and_then(|t| serde_json::from_value(t.clone()).ok())
            .unwrap_or_default();

        info!(
            server = %config.name,
            tool_count = tools.len(),
            "Discovered MCP tools"
        );

        for tool in &tools {
            debug!(server = %config.name, tool = %tool.name, "  tool: {}", tool.description);
        }

        Ok(Self {
            name: config.name.clone(),
            transport,
            tools,
        })
    }

    /// Call a tool on the remote MCP server.
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let result = self
            .transport
            .request(
                "tools/call",
                serde_json::json!({
                    "name": name,
                    "arguments": args,
                }),
            )
            .await?;

        Ok(result)
    }

    /// Get the list of discovered tools.
    pub fn tools(&self) -> &[McpToolInfo] {
        &self.tools
    }

    /// Get the server name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gracefully shut down the MCP server process.
    pub async fn shutdown(&self) {
        if let Err(e) = self.transport.shutdown().await {
            warn!(server = %self.name, error = %e, "Error shutting down MCP server");
        }
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tool bridge — register MCP tools into ToolRegistry
// ════════════════════════════════════════════════════════════════════

/// Register all tools from an MCP client into a ToolRegistry.
///
/// Tools are namespaced as `{server_name}__{tool_name}` to avoid conflicts.
pub async fn register_mcp_tools(client: &Arc<McpClient>, registry: &ToolRegistry) {
    for tool in client.tools() {
        let namespaced_name = format!("{}__{}", client.name(), tool.name);
        let description = format!("[{}] {}", client.name(), tool.description);

        let definition = ToolDefinition {
            name: namespaced_name.clone(),
            description,
            parameters: tool.input_schema.clone(),
            searchable: false,
            metadata: serde_json::json!({
                "mcp_server": client.name(),
                "mcp_tool": tool.name,
            }),
        };

        let client_ref = Arc::clone(client);
        let tool_name = tool.name.clone();

        let handler: ToolHandler = Arc::new(move |args: serde_json::Value| {
            let client = Arc::clone(&client_ref);
            let name = tool_name.clone();
            Box::pin(async move {
                let result = client.call_tool(&name, args).await?;

                // Parse MCP tool result format: { content: [{ type: "text", text: "..." }] }
                let text = result
                    .get("content")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|block| block.get("text").and_then(|t| t.as_str()))
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
                    .unwrap_or_else(|| result.to_string());

                let is_error = result
                    .get("isError")
                    .and_then(|e| e.as_bool())
                    .unwrap_or(false);

                Ok(ToolResult {
                    success: !is_error,
                    output: Content::text(text),
                    metadata: serde_json::json!({"mcp_server": client.name()}),
                })
            })
        });

        if let Err(e) = registry.register_with_handler(definition, handler).await {
            error!(tool = %namespaced_name, error = %e, "Failed to register MCP tool");
        } else {
            debug!(tool = %namespaced_name, "Registered MCP tool");
        }
    }
}

// ════════════════════════════════════════════════════════════════════
//  Manager — lifecycle for all MCP clients
// ════════════════════════════════════════════════════════════════════

/// Manages the lifecycle of multiple MCP client connections.
pub struct McpClientManager {
    clients: RwLock<HashMap<String, Arc<McpClient>>>,
}

impl McpClientManager {
    pub fn new() -> Self {
        Self {
            clients: RwLock::new(HashMap::new()),
        }
    }

    /// Connect to an MCP server and register its tools.
    pub async fn connect(&self, config: &McpServerConfig, registry: &ToolRegistry) -> Result<()> {
        match McpClient::connect(config).await {
            Ok(client) => {
                let client = Arc::new(client);
                register_mcp_tools(&client, registry).await;
                self.clients.write().await.insert(config.id.clone(), client);
                Ok(())
            }
            Err(e) => {
                error!(
                    server = %config.name,
                    error = %e,
                    "Failed to connect to MCP server"
                );
                Err(e)
            }
        }
    }

    /// Disconnect from an MCP server.
    pub async fn disconnect(&self, id: &str) {
        if let Some(client) = self.clients.write().await.remove(id) {
            client.shutdown().await;
            info!(server = %client.name(), "MCP server disconnected");
        }
    }

    /// List connected MCP servers and their tool counts.
    pub async fn list(&self) -> Vec<(String, String, usize)> {
        let clients = self.clients.read().await;
        clients
            .iter()
            .map(|(id, c)| (id.clone(), c.name().to_string(), c.tools().len()))
            .collect()
    }

    /// Shut down all connections.
    pub async fn shutdown_all(&self) {
        let clients: Vec<Arc<McpClient>> = {
            let mut map = self.clients.write().await;
            map.drain().map(|(_, c)| c).collect()
        };
        for client in clients {
            client.shutdown().await;
        }
    }
}

impl Default for McpClientManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Connect to all enabled MCP servers from a list of configs.
pub async fn connect_all(
    configs: Vec<McpServerConfig>,
    registry: &ToolRegistry,
) -> McpClientManager {
    let manager = McpClientManager::new();

    for config in &configs {
        if !config.enabled {
            debug!(server = %config.name, "Skipping disabled MCP server");
            continue;
        }
        if config.server_type != "stdio" {
            warn!(
                server = %config.name,
                server_type = %config.server_type,
                "Only stdio MCP servers are supported — skipping"
            );
            continue;
        }
        if let Err(e) = manager.connect(config, registry).await {
            error!(server = %config.name, error = %e, "Failed to connect MCP server");
        }
    }

    manager
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_server_config_deserialize() {
        let json = serde_json::json!({
            "id": "test-1",
            "name": "test-server",
            "description": "A test server",
            "server_type": "stdio",
            "command": "echo",
            "args": ["hello"],
            "url": null,
            "env": {"API_KEY": "secret"},
            "enabled": true,
            "metadata": {"engine": "code"}
        });
        let config: McpServerConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.name, "test-server");
        assert_eq!(config.command.as_deref(), Some("echo"));
        assert!(config.enabled);
    }

    #[test]
    fn mcp_tool_info_deserialize() {
        let json = serde_json::json!({
            "name": "search",
            "description": "Search for items",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "query": {"type": "string"}
                },
                "required": ["query"]
            }
        });
        let tool: McpToolInfo = serde_json::from_value(json).unwrap();
        assert_eq!(tool.name, "search");
    }

    #[test]
    fn manager_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<McpClientManager>();
    }
}

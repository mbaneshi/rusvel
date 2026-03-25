//! Built-in tools for RUSVEL agents.
//!
//! Registers file operations, shell execution, web fetching, and git tools
//! into a [`ToolRegistry`](rusvel_tool::ToolRegistry).

mod file_ops;
mod git;
mod shell;
pub mod tool_search;

use rusvel_tool::ToolRegistry;

/// Register all built-in tools into the given registry.
pub async fn register_all(registry: &ToolRegistry) {
    file_ops::register(registry).await;
    shell::register(registry).await;
    git::register(registry).await;
}

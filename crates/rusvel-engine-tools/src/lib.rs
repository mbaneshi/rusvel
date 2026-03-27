//! Engine-specific tools for RUSVEL agent execution.
//!
//! Wraps harvest, content, and code engine methods as agent-callable tools
//! registered into a [`ToolRegistry`](rusvel_tool::ToolRegistry).

mod code;
mod content;
mod harvest;

use std::sync::Arc;

use rusvel_tool::ToolRegistry;

pub async fn register_harvest_tools(
    registry: &ToolRegistry,
    engine: Arc<harvest_engine::HarvestEngine>,
) {
    harvest::register(registry, engine).await;
}

pub async fn register_content_tools(
    registry: &ToolRegistry,
    engine: Arc<content_engine::ContentEngine>,
) {
    content::register(registry, engine).await;
}

pub async fn register_code_tools(registry: &ToolRegistry, engine: Arc<code_engine::CodeEngine>) {
    code::register(registry, engine).await;
}

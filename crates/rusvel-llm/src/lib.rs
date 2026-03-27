//! `rusvel-llm` — LLM provider adapters implementing [`LlmPort`].
//!
//! Included adapters:
//! - [`OllamaProvider`] — local Ollama instance
//! - [`ClaudeProvider`] — Anthropic Claude API
//! - [`OpenAiProvider`] — `OpenAI` API (also works with compatible proxies)
//! - [`MultiProvider`] — routes requests by [`ModelProvider`] to the right adapter

mod claude;
mod claude_cli;
mod cursor_agent;
mod flat_prompt;
pub mod cost;
mod cost_tracking;
mod multi;
mod ollama;
mod openai;
mod tier_routing;
pub mod stream;

pub use claude::ClaudeProvider;
pub use claude_cli::ClaudeCliProvider;
pub use cursor_agent::CursorAgentProvider;
pub use cost::{aggregate_spend, SpendAggregation, LLM_COST_METRIC_NAME};
pub use cost_tracking::CostTrackingLlm;
pub use multi::MultiProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use stream::{ClaudeCliStreamer, StreamEvent};
pub use tier_routing::apply_model_tier;

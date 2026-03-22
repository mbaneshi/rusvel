//! `rusvel-llm` — LLM provider adapters implementing [`LlmPort`].
//!
//! Included adapters:
//! - [`OllamaProvider`] — local Ollama instance
//! - [`ClaudeProvider`] — Anthropic Claude API
//! - [`OpenAiProvider`] — OpenAI API (also works with compatible proxies)
//! - [`MultiProvider`] — routes requests by [`ModelProvider`] to the right adapter

mod claude;
mod claude_cli;
mod multi;
mod ollama;
mod openai;
pub mod stream;

pub use claude::ClaudeProvider;
pub use claude_cli::ClaudeCliProvider;
pub use multi::MultiProvider;
pub use ollama::OllamaProvider;
pub use openai::OpenAiProvider;
pub use stream::{ClaudeCliStreamer, StreamEvent};

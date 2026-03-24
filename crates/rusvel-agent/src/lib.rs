//! # rusvel-agent
//!
//! Implements [`AgentPort`] by orchestrating [`LlmPort`] + [`ToolPort`] +
//! [`MemoryPort`] into a request/response agent loop (ADR-009).
//!
//! The core type is [`AgentRuntime`] which holds `Arc` references to the
//! three backing ports and manages active runs in a concurrent map.
//!
//! ## Workflow patterns
//!
//! The [`workflow`] module adds Sequential, Parallel, and Loop execution
//! patterns for composing multiple agent runs. The [`persona`] module
//! provides a catalog of reusable agent personas.

pub mod persona;
pub mod workflow;

pub use persona::PersonaCatalog;
pub use workflow::{Workflow, WorkflowRunner, WorkflowStep};

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, info, warn};

use rusvel_core::domain::*;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::*;
use rusvel_core::ports::{AgentPort, LlmPort, MemoryPort, ToolPort};

/// Maximum iterations in the agent loop to prevent runaways.
const MAX_ITERATIONS: u32 = 10;

/// Streaming events emitted by [`AgentRuntime::run_streaming`].
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum AgentEvent {
    TextDelta { text: String },
    ToolCallStart { id: String, name: String, args: serde_json::Value },
    ToolCallEnd { id: String, name: String, result: String, is_error: bool },
    Done { output: AgentOutput },
    Error { message: String },
}

/// Internal state for a single agent run.
struct RunState {
    config: AgentConfig,
    status: AgentStatus,
}

/// Agent orchestration runtime.
///
/// Wraps [`LlmPort`], [`ToolPort`], and [`MemoryPort`] into a coherent
/// agent loop. Each call to [`create`](AgentPort::create) registers a new
/// run; [`run`](AgentPort::run) executes the loop.
pub struct AgentRuntime {
    llm: Arc<dyn LlmPort>,
    tools: Arc<dyn ToolPort>,
    #[allow(dead_code)] // will be used for context retrieval in future iterations
    memory: Arc<dyn MemoryPort>,
    runs: RwLock<HashMap<RunId, RunState>>,
}

impl AgentRuntime {
    /// Build a new runtime from the three backing ports.
    pub fn new(
        llm: Arc<dyn LlmPort>,
        tools: Arc<dyn ToolPort>,
        memory: Arc<dyn MemoryPort>,
    ) -> Self {
        Self {
            llm,
            tools,
            memory,
            runs: RwLock::new(HashMap::new()),
        }
    }

    /// Build an [`LlmRequest`] from config, messages, and available tool definitions.
    fn build_request(
        config: &AgentConfig,
        messages: &[LlmMessage],
        tool_defs: &[ToolDefinition],
    ) -> LlmRequest {
        let model = config.model.clone().unwrap_or_else(|| ModelRef {
            provider: ModelProvider::Claude,
            model: "claude-sonnet-4-20250514".into(),
        });

        // Convert ToolDefinitions to the JSON Schema format LLM providers expect.
        let tools: Vec<serde_json::Value> = tool_defs
            .iter()
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "input_schema": t.parameters,
                })
            })
            .collect();

        LlmRequest {
            model,
            messages: messages.to_vec(),
            tools,
            temperature: None,
            max_tokens: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Extract the first tool call from an LLM response.
    ///
    /// Scans `Part::ToolCall` variants in the response content.
    fn extract_tool_call(
        response: &LlmResponse,
    ) -> Option<(String, String, serde_json::Value)> {
        for part in &response.content.parts {
            if let Part::ToolCall { id, name, args } = part {
                return Some((id.clone(), name.clone(), args.clone()));
            }
        }
        None
    }

    /// Extract text from LLM response content parts.
    fn extract_text(content: &Content) -> String {
        content
            .parts
            .iter()
            .filter_map(|p| match p {
                Part::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }

    /// Run the agent loop, streaming [`AgentEvent`]s to the provided channel.
    ///
    /// This mirrors [`AgentPort::run`] but emits incremental events so the
    /// caller can forward them as SSE to a frontend.
    pub async fn run_streaming(
        &self,
        run_id: &RunId,
        input: Content,
        tx: mpsc::Sender<AgentEvent>,
    ) -> Result<AgentOutput> {
        // Transition to Running.
        {
            let mut runs = self.runs.write().await;
            let state = runs.get_mut(run_id).ok_or_else(|| RusvelError::NotFound {
                kind: "AgentRun".into(),
                id: run_id.to_string(),
            })?;
            if state.status == AgentStatus::Stopped {
                return Err(RusvelError::Agent("run has been stopped".into()));
            }
            state.status = AgentStatus::Running;
        }

        // Snapshot config.
        let config = {
            let runs = self.runs.read().await;
            runs.get(run_id)
                .ok_or_else(|| RusvelError::NotFound {
                    kind: "AgentRun".into(),
                    id: run_id.to_string(),
                })?
                .config
                .clone()
        };

        // Seed conversation.
        let mut messages: Vec<LlmMessage> = Vec::new();
        if let Some(ref instructions) = config.instructions {
            messages.push(LlmMessage {
                role: LlmRole::System,
                content: Content::text(instructions),
            });
        }
        messages.push(LlmMessage {
            role: LlmRole::User,
            content: input,
        });

        let mut total_usage = LlmUsage::default();
        let mut tool_calls: u32 = 0;
        let tool_defs = self.tools.list();

        for iteration in 0..MAX_ITERATIONS {
            debug!(%run_id, iteration, "agent streaming loop iteration");

            // Check if stopped.
            {
                let runs = self.runs.read().await;
                if let Some(state) = runs.get(run_id)
                    && state.status == AgentStatus::Stopped
                {
                    let _ = tx.send(AgentEvent::Error { message: "run was stopped".into() }).await;
                    return Err(RusvelError::Agent("run was stopped".into()));
                }
            }

            let request = Self::build_request(&config, &messages, &tool_defs);
            let response = match self.llm.generate(request).await {
                Ok(r) => r,
                Err(e) => {
                    let _ = tx.send(AgentEvent::Error { message: e.to_string() }).await;
                    return Err(e);
                }
            };

            total_usage.input_tokens += response.usage.input_tokens;
            total_usage.output_tokens += response.usage.output_tokens;

            match response.finish_reason {
                FinishReason::Stop | FinishReason::Length | FinishReason::ContentFilter => {
                    let text = Self::extract_text(&response.content);
                    if !text.is_empty() {
                        let _ = tx.send(AgentEvent::TextDelta { text }).await;
                    }

                    let output = AgentOutput {
                        run_id: *run_id,
                        content: response.content,
                        tool_calls,
                        usage: total_usage,
                        cost_estimate: 0.0,
                        metadata: serde_json::json!({}),
                    };

                    self.runs.write().await.entry(*run_id).and_modify(|s| {
                        s.status = AgentStatus::Completed;
                    });

                    let _ = tx.send(AgentEvent::Done { output: output.clone() }).await;
                    info!(%run_id, tool_calls, "agent streaming run completed");
                    return Ok(output);
                }
                FinishReason::ToolUse => {
                    // Emit any text before the tool call.
                    let text = Self::extract_text(&response.content);
                    if !text.is_empty() {
                        let _ = tx.send(AgentEvent::TextDelta { text }).await;
                    }

                    self.runs.write().await.entry(*run_id).and_modify(|s| {
                        s.status = AgentStatus::AwaitingTool;
                    });

                    let (tool_call_id, tool_name, tool_args) =
                        Self::extract_tool_call(&response).ok_or_else(|| {
                            RusvelError::Agent(
                                "ToolUse finish_reason but no Part::ToolCall found".into(),
                            )
                        })?;

                    let _ = tx.send(AgentEvent::ToolCallStart {
                        id: tool_call_id.clone(),
                        name: tool_name.clone(),
                        args: tool_args.clone(),
                    }).await;

                    messages.push(LlmMessage {
                        role: LlmRole::Assistant,
                        content: response.content,
                    });

                    let tool_result = self.tools.call(&tool_name, tool_args).await;
                    tool_calls += 1;

                    let (result_text, is_error) = match &tool_result {
                        Ok(r) => {
                            let t = Self::extract_text(&r.output);
                            (t, !r.success)
                        }
                        Err(e) => (format!("Tool error: {e}"), true),
                    };

                    let _ = tx.send(AgentEvent::ToolCallEnd {
                        id: tool_call_id.clone(),
                        name: tool_name.clone(),
                        result: result_text.clone(),
                        is_error,
                    }).await;

                    messages.push(LlmMessage {
                        role: LlmRole::Tool,
                        content: Content {
                            parts: vec![Part::ToolResult {
                                tool_call_id,
                                content: result_text,
                                is_error,
                            }],
                        },
                    });

                    self.runs.write().await.entry(*run_id).and_modify(|s| {
                        s.status = AgentStatus::Running;
                    });
                }
                FinishReason::Other(ref reason) => {
                    let msg = format!("unexpected finish reason: {reason}");
                    let _ = tx.send(AgentEvent::Error { message: msg.clone() }).await;
                    warn!(%run_id, %reason, "unexpected finish reason");
                    return Err(RusvelError::Agent(msg));
                }
            }
        }

        // Exhausted iterations.
        self.runs.write().await.entry(*run_id).and_modify(|s| {
            s.status = AgentStatus::Failed;
        });
        let msg = format!("agent loop exceeded {MAX_ITERATIONS} iterations");
        let _ = tx.send(AgentEvent::Error { message: msg.clone() }).await;
        Err(RusvelError::Agent(msg))
    }
}

#[async_trait]
impl AgentPort for AgentRuntime {
    async fn create(&self, config: AgentConfig) -> Result<RunId> {
        let run_id = RunId::new();
        let state = RunState {
            config,
            status: AgentStatus::Idle,
        };
        self.runs.write().await.insert(run_id, state);
        info!(%run_id, "agent run created");
        Ok(run_id)
    }

    async fn run(&self, run_id: &RunId, input: Content) -> Result<AgentOutput> {
        // Transition to Running.
        {
            let mut runs = self.runs.write().await;
            let state = runs.get_mut(run_id).ok_or_else(|| RusvelError::NotFound {
                kind: "AgentRun".into(),
                id: run_id.to_string(),
            })?;
            if state.status == AgentStatus::Stopped {
                return Err(RusvelError::Agent("run has been stopped".into()));
            }
            state.status = AgentStatus::Running;
        }

        // Snapshot config for use outside the lock.
        let config = {
            let runs = self.runs.read().await;
            runs.get(run_id)
                .ok_or_else(|| RusvelError::NotFound {
                    kind: "AgentRun".into(),
                    id: run_id.to_string(),
                })?
                .config
                .clone()
        };

        // Seed the conversation with system instructions + user input.
        let mut messages: Vec<LlmMessage> = Vec::new();
        if let Some(ref instructions) = config.instructions {
            messages.push(LlmMessage {
                role: LlmRole::System,
                content: Content::text(instructions),
            });
        }
        messages.push(LlmMessage {
            role: LlmRole::User,
            content: input,
        });

        let mut total_usage = LlmUsage::default();
        let mut tool_calls: u32 = 0;

        // Resolve available tool definitions once before the loop.
        let tool_defs = self.tools.list();

        // ── Agent loop ───────────────────────────────────────────────
        for iteration in 0..MAX_ITERATIONS {
            debug!(%run_id, iteration, "agent loop iteration");

            // Check if stopped between iterations.
            {
                let runs = self.runs.read().await;
                if let Some(state) = runs.get(run_id)
                    && state.status == AgentStatus::Stopped
                {
                    return Err(RusvelError::Agent("run was stopped".into()));
                }
            }

            let request = Self::build_request(&config, &messages, &tool_defs);
            let response = self.llm.generate(request).await?;

            // Accumulate usage.
            total_usage.input_tokens += response.usage.input_tokens;
            total_usage.output_tokens += response.usage.output_tokens;

            match response.finish_reason {
                FinishReason::Stop | FinishReason::Length | FinishReason::ContentFilter => {
                    // Terminal — return the final content.
                    let output = AgentOutput {
                        run_id: *run_id,
                        content: response.content,
                        tool_calls,
                        usage: total_usage,
                        cost_estimate: 0.0,
                        metadata: serde_json::json!({}),
                    };

                    self.runs.write().await.entry(*run_id).and_modify(|s| {
                        s.status = AgentStatus::Completed;
                    });
                    info!(%run_id, tool_calls, "agent run completed");
                    return Ok(output);
                }
                FinishReason::ToolUse => {
                    // Update status.
                    self.runs.write().await.entry(*run_id).and_modify(|s| {
                        s.status = AgentStatus::AwaitingTool;
                    });

                    let (tool_call_id, tool_name, tool_args) =
                        Self::extract_tool_call(&response).ok_or_else(|| {
                            RusvelError::Agent(
                                "ToolUse finish_reason but no Part::ToolCall found".into(),
                            )
                        })?;

                    debug!(%run_id, %tool_name, "calling tool");

                    // Append the assistant message (includes the ToolCall part).
                    messages.push(LlmMessage {
                        role: LlmRole::Assistant,
                        content: response.content,
                    });

                    // Execute the tool.
                    let tool_result = self.tools.call(&tool_name, tool_args).await;
                    tool_calls += 1;

                    // Build a ToolResult part with the proper tool_call_id.
                    let (result_text, is_error) = match &tool_result {
                        Ok(r) => {
                            let text = r
                                .output
                                .parts
                                .iter()
                                .filter_map(|p| match p {
                                    Part::Text(t) => Some(t.as_str()),
                                    _ => None,
                                })
                                .collect::<Vec<_>>()
                                .join("");
                            (text, !r.success)
                        }
                        Err(e) => (format!("Tool error: {e}"), true),
                    };

                    messages.push(LlmMessage {
                        role: LlmRole::Tool,
                        content: Content {
                            parts: vec![Part::ToolResult {
                                tool_call_id,
                                content: result_text,
                                is_error,
                            }],
                        },
                    });

                    // Transition back to Running.
                    self.runs.write().await.entry(*run_id).and_modify(|s| {
                        s.status = AgentStatus::Running;
                    });
                }
                FinishReason::Other(ref reason) => {
                    warn!(%run_id, %reason, "unexpected finish reason");
                    return Err(RusvelError::Agent(format!(
                        "unexpected finish reason: {reason}"
                    )));
                }
            }
        }

        // Exhausted iterations.
        self.runs.write().await.entry(*run_id).and_modify(|s| {
            s.status = AgentStatus::Failed;
        });
        Err(RusvelError::Agent(format!(
            "agent loop exceeded {MAX_ITERATIONS} iterations"
        )))
    }

    async fn stop(&self, run_id: &RunId) -> Result<()> {
        let mut runs = self.runs.write().await;
        let state = runs.get_mut(run_id).ok_or_else(|| RusvelError::NotFound {
            kind: "AgentRun".into(),
            id: run_id.to_string(),
        })?;
        state.status = AgentStatus::Stopped;
        info!(%run_id, "agent run stopped");
        Ok(())
    }

    async fn status(&self, run_id: &RunId) -> Result<AgentStatus> {
        let runs = self.runs.read().await;
        let state = runs.get(run_id).ok_or_else(|| RusvelError::NotFound {
            kind: "AgentRun".into(),
            id: run_id.to_string(),
        })?;
        Ok(state.status.clone())
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── Mock LlmPort ─────────────────────────────────────────────

    struct MockLlm {
        /// Responses to return, consumed in order.
        responses: RwLock<Vec<LlmResponse>>,
    }

    impl MockLlm {
        fn new(responses: Vec<LlmResponse>) -> Self {
            Self {
                responses: RwLock::new(responses),
            }
        }
    }

    #[async_trait]
    impl LlmPort for MockLlm {
        async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse> {
            let mut resps = self.responses.write().await;
            if resps.is_empty() {
                return Err(RusvelError::Llm("no more mock responses".into()));
            }
            Ok(resps.remove(0))
        }

        async fn embed(&self, _model: &ModelRef, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.0; 128])
        }

        async fn list_models(&self) -> Result<Vec<ModelRef>> {
            Ok(vec![])
        }
    }

    // ── Mock ToolPort ────────────────────────────────────────────

    struct MockTool;

    #[async_trait]
    impl ToolPort for MockTool {
        async fn register(&self, _tool: ToolDefinition) -> Result<()> {
            Ok(())
        }

        async fn call(&self, _name: &str, _args: serde_json::Value) -> Result<ToolResult> {
            Ok(ToolResult {
                success: true,
                output: Content::text("tool result"),
                metadata: serde_json::json!({}),
            })
        }

        fn list(&self) -> Vec<ToolDefinition> {
            vec![]
        }

        fn schema(&self, _name: &str) -> Option<serde_json::Value> {
            None
        }
    }

    // ── Mock MemoryPort ──────────────────────────────────────────

    struct MockMemory;

    #[async_trait]
    impl MemoryPort for MockMemory {
        async fn store(&self, _entry: MemoryEntry) -> Result<uuid::Uuid> {
            Ok(uuid::Uuid::now_v7())
        }

        async fn recall(&self, _id: &uuid::Uuid) -> Result<Option<MemoryEntry>> {
            Ok(None)
        }

        async fn search(
            &self,
            _session_id: &SessionId,
            _query: &str,
            _limit: usize,
        ) -> Result<Vec<MemoryEntry>> {
            Ok(vec![])
        }

        async fn forget(&self, _id: &uuid::Uuid) -> Result<()> {
            Ok(())
        }
    }

    // ── Helpers ──────────────────────────────────────────────────

    fn make_config() -> AgentConfig {
        AgentConfig {
            profile_id: None,
            session_id: SessionId::new(),
            model: None,
            tools: vec![],
            instructions: Some("You are a helpful assistant.".into()),
            budget_limit: None,
            metadata: serde_json::json!({}),
        }
    }

    fn stop_response(text: &str) -> LlmResponse {
        LlmResponse {
            content: Content::text(text),
            finish_reason: FinishReason::Stop,
            usage: LlmUsage {
                input_tokens: 10,
                output_tokens: 20,
            },
            metadata: serde_json::json!({}),
        }
    }

    fn tool_use_response(tool_name: &str) -> LlmResponse {
        LlmResponse {
            content: Content {
                parts: vec![Part::ToolCall {
                    id: format!("call_{tool_name}"),
                    name: tool_name.into(),
                    args: serde_json::json!({"query": "test"}),
                }],
            },
            finish_reason: FinishReason::ToolUse,
            usage: LlmUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
            metadata: serde_json::json!({}),
        }
    }

    fn make_runtime(responses: Vec<LlmResponse>) -> AgentRuntime {
        AgentRuntime::new(
            Arc::new(MockLlm::new(responses)),
            Arc::new(MockTool),
            Arc::new(MockMemory),
        )
    }

    // ── Tests ────────────────────────────────────────────────────

    #[tokio::test]
    async fn create_and_status() {
        let rt = make_runtime(vec![]);
        let run_id = rt.create(make_config()).await.unwrap();
        assert_eq!(rt.status(&run_id).await.unwrap(), AgentStatus::Idle);
    }

    #[tokio::test]
    async fn simple_run_returns_output() {
        let rt = make_runtime(vec![stop_response("Hello!")]);
        let run_id = rt.create(make_config()).await.unwrap();
        let output = rt.run(&run_id, Content::text("Hi")).await.unwrap();

        assert_eq!(output.run_id, run_id);
        assert_eq!(output.tool_calls, 0);
        assert_eq!(output.usage.input_tokens, 10);
        assert_eq!(output.usage.output_tokens, 20);
        assert_eq!(rt.status(&run_id).await.unwrap(), AgentStatus::Completed);
    }

    #[tokio::test]
    async fn tool_use_loop() {
        let rt = make_runtime(vec![
            tool_use_response("search"),
            stop_response("Here is your answer"),
        ]);
        let run_id = rt.create(make_config()).await.unwrap();
        let output = rt
            .run(&run_id, Content::text("Find something"))
            .await
            .unwrap();

        assert_eq!(output.tool_calls, 1);
        assert_eq!(output.usage.input_tokens, 20); // 10 + 10
        assert_eq!(output.usage.output_tokens, 25); // 5 + 20
    }

    #[tokio::test]
    async fn stop_prevents_run() {
        let rt = make_runtime(vec![stop_response("ok")]);
        let run_id = rt.create(make_config()).await.unwrap();
        rt.stop(&run_id).await.unwrap();

        let err = rt.run(&run_id, Content::text("Hi")).await.unwrap_err();
        assert!(err.to_string().contains("stopped"));
    }

    #[tokio::test]
    async fn unknown_run_id_errors() {
        let rt = make_runtime(vec![]);
        let fake = RunId::new();
        let err = rt.status(&fake).await.unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[tokio::test]
    async fn max_iterations_exceeded() {
        // 11 tool-use responses should exceed the 10-iteration limit.
        let responses: Vec<LlmResponse> = (0..11).map(|_| tool_use_response("search")).collect();
        let rt = make_runtime(responses);
        let run_id = rt.create(make_config()).await.unwrap();
        let err = rt.run(&run_id, Content::text("loop")).await.unwrap_err();
        assert!(err.to_string().contains("exceeded"));
        assert_eq!(rt.status(&run_id).await.unwrap(), AgentStatus::Failed);
    }
}

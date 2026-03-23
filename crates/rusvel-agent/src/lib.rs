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
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use rusvel_core::domain::*;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::*;
use rusvel_core::ports::{AgentPort, LlmPort, MemoryPort, ToolPort};

/// Maximum iterations in the agent loop to prevent runaways.
const MAX_ITERATIONS: u32 = 10;

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

    /// Build the initial [`LlmRequest`] from config instructions + user input.
    fn build_request(config: &AgentConfig, messages: &[LlmMessage]) -> LlmRequest {
        let model = config.model.clone().unwrap_or_else(|| ModelRef {
            provider: ModelProvider::Claude,
            model: "claude-sonnet-4-20250514".into(),
        });

        LlmRequest {
            model,
            messages: messages.to_vec(),
            tools: vec![],
            temperature: None,
            max_tokens: None,
            metadata: serde_json::json!({}),
        }
    }

    /// Extract tool name and arguments from the LLM response content.
    ///
    /// Expects the response metadata to carry `tool_name` and `tool_args`
    /// fields when `finish_reason == ToolUse`.
    fn extract_tool_call(response: &LlmResponse) -> Option<(String, serde_json::Value)> {
        let name = response.metadata.get("tool_name")?.as_str()?;
        let args = response
            .metadata
            .get("tool_args")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        Some((name.to_string(), args))
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

            let request = Self::build_request(&config, &messages);
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

                    let (tool_name, tool_args) =
                        Self::extract_tool_call(&response).ok_or_else(|| {
                            RusvelError::Agent(
                                "ToolUse finish_reason but no tool call in metadata".into(),
                            )
                        })?;

                    debug!(%run_id, %tool_name, "calling tool");

                    // Append the assistant message.
                    messages.push(LlmMessage {
                        role: LlmRole::Assistant,
                        content: response.content,
                    });

                    let tool_result = self.tools.call(&tool_name, tool_args).await?;
                    tool_calls += 1;

                    // Append tool result as a Tool message.
                    messages.push(LlmMessage {
                        role: LlmRole::Tool,
                        content: tool_result.output,
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
            content: Content::text("I need to call a tool"),
            finish_reason: FinishReason::ToolUse,
            usage: LlmUsage {
                input_tokens: 10,
                output_tokens: 5,
            },
            metadata: serde_json::json!({
                "tool_name": tool_name,
                "tool_args": {"query": "test"},
            }),
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

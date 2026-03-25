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

/// Events emitted during a streaming agent run.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    /// Incremental text chunk from the LLM.
    TextDelta { text: String },
    /// A tool is being called.
    ToolCall { name: String, args: serde_json::Value },
    /// Tool execution completed.
    ToolResult { name: String, output: String, is_error: bool },
    /// The agent run completed successfully.
    Done { output: AgentOutput },
    /// An error occurred.
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

    /// Execute a streaming agent run.
    ///
    /// Unlike [`AgentPort::run()`] which returns a complete `AgentOutput`,
    /// this emits incremental [`AgentEvent`]s via an `mpsc::Receiver`:
    /// - `TextDelta` for each LLM text chunk
    /// - `ToolCall` / `ToolResult` for tool interactions
    /// - `Done` with the final `AgentOutput`
    /// - `Error` on failure
    ///
    /// The `run_id` must have been created via [`AgentPort::create()`] first.
    pub async fn run_streaming(
        &self,
        run_id: &RunId,
        input: Content,
    ) -> Result<tokio::sync::mpsc::Receiver<AgentEvent>> {
        // Transition to Running + snapshot config.
        let config = {
            let mut runs = self.runs.write().await;
            let state = runs.get_mut(run_id).ok_or_else(|| RusvelError::NotFound {
                kind: "AgentRun".into(),
                id: run_id.to_string(),
            })?;
            if state.status == AgentStatus::Stopped {
                return Err(RusvelError::Agent("run has been stopped".into()));
            }
            state.status = AgentStatus::Running;
            state.config.clone()
        };

        let (tx, rx) = tokio::sync::mpsc::channel(64);
        let llm = self.llm.clone();
        let tools = self.tools.clone();
        let run_id = *run_id;

        tokio::spawn(async move {
            let result = run_streaming_loop(&llm, &tools, &config, &run_id, input, &tx).await;
            match result {
                Ok(output) => {
                    let _ = tx.send(AgentEvent::Done { output }).await;
                }
                Err(e) => {
                    let _ = tx
                        .send(AgentEvent::Error {
                            message: e.to_string(),
                        })
                        .await;
                }
            }
        });

        Ok(rx)
    }
}

/// Inner streaming agent loop, factored out so it can be spawned as a task.
async fn run_streaming_loop(
    llm: &Arc<dyn LlmPort>,
    tools: &Arc<dyn ToolPort>,
    config: &AgentConfig,
    run_id: &RunId,
    input: Content,
    tx: &tokio::sync::mpsc::Sender<AgentEvent>,
) -> Result<AgentOutput> {
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
    let tool_defs = tools.list();

    for iteration in 0..MAX_ITERATIONS {
        debug!(%run_id, iteration, "streaming agent loop iteration");

        let request = AgentRuntime::build_request(config, &messages, &tool_defs);

        // Use stream() for incremental deltas
        let mut stream_rx = llm.stream(request).await?;
        let mut response: Option<LlmResponse> = None;

        while let Some(event) = stream_rx.recv().await {
            match event {
                LlmStreamEvent::Delta(text) => {
                    let _ = tx.send(AgentEvent::TextDelta { text }).await;
                }
                LlmStreamEvent::ToolUse { id: _, name, args } => {
                    // Tool use events from stream are informational;
                    // we handle them via the Done response below.
                    let _ = tx
                        .send(AgentEvent::ToolCall {
                            name: name.clone(),
                            args: args.clone(),
                        })
                        .await;
                }
                LlmStreamEvent::Done(resp) => {
                    response = Some(resp);
                    break;
                }
                LlmStreamEvent::Error(msg) => {
                    return Err(RusvelError::Llm(msg));
                }
            }
        }

        let response = response
            .ok_or_else(|| RusvelError::Llm("stream ended without Done event".into()))?;

        total_usage.input_tokens += response.usage.input_tokens;
        total_usage.output_tokens += response.usage.output_tokens;

        match response.finish_reason {
            FinishReason::Stop | FinishReason::Length | FinishReason::ContentFilter => {
                return Ok(AgentOutput {
                    run_id: *run_id,
                    content: response.content,
                    tool_calls,
                    usage: total_usage,
                    cost_estimate: 0.0,
                    metadata: serde_json::json!({}),
                });
            }
            FinishReason::ToolUse => {
                let (tool_call_id, tool_name, tool_args) =
                    AgentRuntime::extract_tool_call(&response).ok_or_else(|| {
                        RusvelError::Agent(
                            "ToolUse finish_reason but no Part::ToolCall found".into(),
                        )
                    })?;

                debug!(%run_id, %tool_name, "streaming: calling tool");

                messages.push(LlmMessage {
                    role: LlmRole::Assistant,
                    content: response.content,
                });

                let tool_result = tools.call(&tool_name, tool_args).await;
                tool_calls += 1;

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

                let _ = tx
                    .send(AgentEvent::ToolResult {
                        name: tool_name,
                        output: result_text.clone(),
                        is_error,
                    })
                    .await;

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
            }
            FinishReason::Other(ref reason) => {
                return Err(RusvelError::Agent(format!(
                    "unexpected finish reason: {reason}"
                )));
            }
        }
    }

    Err(RusvelError::Agent(format!(
        "agent loop exceeded {MAX_ITERATIONS} iterations"
    )))
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

    // ── Streaming mock ─────────────────────────────────────────

    /// Mock LLM that emits multiple Delta events before Done.
    struct MockStreamingLlm {
        responses: RwLock<Vec<LlmResponse>>,
        /// Number of delta chunks to emit per response.
        deltas_per_response: usize,
    }

    impl MockStreamingLlm {
        fn new(responses: Vec<LlmResponse>, deltas_per_response: usize) -> Self {
            Self {
                responses: RwLock::new(responses),
                deltas_per_response,
            }
        }
    }

    #[async_trait]
    impl LlmPort for MockStreamingLlm {
        async fn generate(&self, _request: LlmRequest) -> Result<LlmResponse> {
            let mut resps = self.responses.write().await;
            if resps.is_empty() {
                return Err(RusvelError::Llm("no more mock responses".into()));
            }
            Ok(resps.remove(0))
        }

        async fn stream(
            &self,
            _request: LlmRequest,
        ) -> Result<tokio::sync::mpsc::Receiver<LlmStreamEvent>> {
            let mut resps = self.responses.write().await;
            if resps.is_empty() {
                return Err(RusvelError::Llm("no more mock responses".into()));
            }
            let response = resps.remove(0);
            let deltas = self.deltas_per_response;

            let (tx, rx) = tokio::sync::mpsc::channel(64);
            tokio::spawn(async move {
                // Extract full text from response
                let full_text: String = response
                    .content
                    .parts
                    .iter()
                    .filter_map(|p| match p {
                        Part::Text(t) => Some(t.as_str()),
                        _ => None,
                    })
                    .collect();

                // Split into N delta chunks
                if !full_text.is_empty() && deltas > 0 {
                    let chunk_size = (full_text.len() + deltas - 1) / deltas;
                    for chunk in full_text.as_bytes().chunks(chunk_size) {
                        let text = String::from_utf8_lossy(chunk).to_string();
                        let _ = tx.send(LlmStreamEvent::Delta(text)).await;
                    }
                }

                let _ = tx.send(LlmStreamEvent::Done(response)).await;
            });

            Ok(rx)
        }

        async fn embed(&self, _model: &ModelRef, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.0; 128])
        }

        async fn list_models(&self) -> Result<Vec<ModelRef>> {
            Ok(vec![])
        }
    }

    fn make_streaming_runtime(
        responses: Vec<LlmResponse>,
        deltas: usize,
    ) -> AgentRuntime {
        AgentRuntime::new(
            Arc::new(MockStreamingLlm::new(responses, deltas)),
            Arc::new(MockTool),
            Arc::new(MockMemory),
        )
    }

    // ── Streaming tests ────────────────────────────────────────

    #[tokio::test]
    async fn streaming_emits_deltas_then_done() {
        let rt = make_streaming_runtime(vec![stop_response("Hello world!")], 3);
        let run_id = rt.create(make_config()).await.unwrap();
        let mut rx = rt.run_streaming(&run_id, Content::text("Hi")).await.unwrap();

        let mut deltas = Vec::new();
        let mut done = false;

        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::TextDelta { text } => deltas.push(text),
                AgentEvent::Done { output } => {
                    assert_eq!(output.run_id, run_id);
                    assert_eq!(output.tool_calls, 0);
                    done = true;
                }
                AgentEvent::Error { message } => panic!("unexpected error: {message}"),
                _ => {}
            }
        }

        assert!(done, "should receive Done event");
        assert!(deltas.len() >= 2, "should receive multiple deltas, got {}", deltas.len());
        let reassembled: String = deltas.into_iter().collect();
        assert_eq!(reassembled, "Hello world!");
    }

    #[tokio::test]
    async fn streaming_with_tool_use() {
        let rt = make_streaming_runtime(
            vec![tool_use_response("search"), stop_response("Answer found")],
            2,
        );
        let run_id = rt.create(make_config()).await.unwrap();
        let mut rx = rt.run_streaming(&run_id, Content::text("Search")).await.unwrap();

        let mut got_tool_result = false;
        let mut got_done = false;

        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::ToolResult { name, .. } => {
                    assert_eq!(name, "search");
                    got_tool_result = true;
                }
                AgentEvent::Done { output } => {
                    assert_eq!(output.tool_calls, 1);
                    got_done = true;
                }
                AgentEvent::Error { message } => panic!("unexpected error: {message}"),
                _ => {}
            }
        }

        assert!(got_tool_result, "should receive ToolResult");
        assert!(got_done, "should receive Done");
    }

    #[tokio::test]
    async fn streaming_batch_fallback() {
        // Using the non-streaming MockLlm (uses default stream() impl)
        let rt = make_runtime(vec![stop_response("Batch response")]);
        let run_id = rt.create(make_config()).await.unwrap();
        let mut rx = rt.run_streaming(&run_id, Content::text("Hi")).await.unwrap();

        let mut got_done = false;
        while let Some(event) = rx.recv().await {
            match event {
                AgentEvent::Done { output } => {
                    assert_eq!(output.run_id, run_id);
                    got_done = true;
                }
                AgentEvent::Error { message } => panic!("unexpected error: {message}"),
                _ => {}
            }
        }
        assert!(got_done, "batch fallback should emit Done");
    }
}

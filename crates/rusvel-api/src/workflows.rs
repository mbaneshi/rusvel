//! Workflows CRUD + execution — multi-step agent pipelines stored in `ObjectStore`.
//!
//! A workflow defines a sequence of steps, each referencing an agent by name
//! and providing a prompt template. Execution runs each step through `claude -p`
//! with the referenced agent's configuration.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use rusvel_core::domain::{AgentProfile, ObjectFilter};
use rusvel_llm::stream::ClaudeCliStreamer;

use crate::AppState;

const STORE_KIND: &str = "workflows";

// ── Types ────────────────────────────────────────────────────

/// A single step in a workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStepDef {
    pub agent_name: String,
    pub prompt_template: String,
    /// "sequential" or "parallel" (parallel is reserved for future use).
    #[serde(default = "default_step_type")]
    pub step_type: String,
}

fn default_step_type() -> String {
    "sequential".into()
}

/// A persisted workflow definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub steps: Vec<WorkflowStepDef>,
    #[serde(default = "default_metadata")]
    pub metadata: serde_json::Value,
}

fn default_metadata() -> serde_json::Value {
    serde_json::json!({})
}

/// Request body for creating a workflow.
#[derive(Debug, Deserialize)]
pub struct CreateWorkflowBody {
    pub name: String,
    pub description: Option<String>,
    pub steps: Vec<WorkflowStepDef>,
    pub metadata: Option<serde_json::Value>,
}

/// Result of a single step execution.
#[derive(Debug, Serialize)]
pub struct StepResult {
    pub step_index: usize,
    pub agent_name: String,
    pub prompt: String,
    pub output: String,
    pub cost_usd: f64,
}

/// Result of running an entire workflow.
#[derive(Debug, Serialize)]
pub struct WorkflowRunResult {
    pub workflow_id: String,
    pub workflow_name: String,
    pub steps: Vec<StepResult>,
    pub total_cost_usd: f64,
}

/// Optional body for workflow run (variable substitution, etc.)
#[derive(Debug, Deserialize, Default)]
pub struct RunWorkflowBody {
    /// Optional variables to substitute into prompt templates ({{key}} -> value).
    #[serde(default)]
    pub variables: std::collections::HashMap<String, String>,
}

// ── CRUD Handlers ────────────────────────────────────────────

/// `GET /api/workflows` — list all workflows.
pub async fn list_workflows(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WorkflowDefinition>>, (StatusCode, String)> {
    let all = state
        .storage
        .objects()
        .list(STORE_KIND, ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let workflows: Vec<WorkflowDefinition> = all
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    Ok(Json(workflows))
}

/// `POST /api/workflows` — create a new workflow.
pub async fn create_workflow(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateWorkflowBody>,
) -> Result<(StatusCode, Json<WorkflowDefinition>), (StatusCode, String)> {
    let wf = WorkflowDefinition {
        id: uuid::Uuid::now_v7().to_string(),
        name: body.name,
        description: body.description.unwrap_or_default(),
        steps: body.steps,
        metadata: body.metadata.unwrap_or_else(|| serde_json::json!({})),
    };

    state
        .storage
        .objects()
        .put(
            STORE_KIND,
            &wf.id,
            serde_json::to_value(&wf).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(wf)))
}

/// `GET /api/workflows/{id}` — get a single workflow.
pub async fn get_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<WorkflowDefinition>, (StatusCode, String)> {
    let val = state
        .storage
        .objects()
        .get(STORE_KIND, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "workflow not found".into()))?;

    let wf: WorkflowDefinition = serde_json::from_value(val)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(wf))
}

/// `PUT /api/workflows/{id}` — update a workflow.
pub async fn update_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(wf): Json<WorkflowDefinition>,
) -> Result<Json<WorkflowDefinition>, (StatusCode, String)> {
    state
        .storage
        .objects()
        .put(
            STORE_KIND,
            &id,
            serde_json::to_value(&wf).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(wf))
}

/// `DELETE /api/workflows/{id}` — delete a workflow.
pub async fn delete_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .storage
        .objects()
        .delete(STORE_KIND, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Workflow Execution ───────────────────────────────────────

/// `POST /api/workflows/{id}/run` — execute a workflow sequentially.
///
/// For each step:
/// 1. Look up the agent by name in ObjectStore("agents")
/// 2. Build CLI args from the agent's config (model, tools, etc.)
/// 3. Run the prompt through `claude -p`
/// 4. Collect the result; feed output into next step's context
pub async fn run_workflow(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    body: Option<Json<RunWorkflowBody>>,
) -> Result<Json<WorkflowRunResult>, (StatusCode, String)> {
    let body = body.map(|b| b.0).unwrap_or_default();

    // Load workflow
    let wf_val = state
        .storage
        .objects()
        .get(STORE_KIND, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "workflow not found".into()))?;

    let wf: WorkflowDefinition = serde_json::from_value(wf_val)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Load all agents
    let agent_values = state
        .storage
        .objects()
        .list("agents", ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let agents: Vec<AgentProfile> = agent_values
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    let mut step_results = Vec::new();
    let mut total_cost = 0.0;
    let mut previous_output = String::new();

    for (i, step) in wf.steps.iter().enumerate() {
        // Find agent by name (case-insensitive)
        let agent = agents
            .iter()
            .find(|a| a.name.eq_ignore_ascii_case(&step.agent_name))
            .ok_or((
                StatusCode::BAD_REQUEST,
                format!("agent '{}' not found for step {}", step.agent_name, i),
            ))?;

        // Build prompt: substitute variables and inject previous output
        let mut prompt = step.prompt_template.clone();
        for (key, value) in &body.variables {
            prompt = prompt.replace(&format!("{{{{{key}}}}}"), value);
        }
        if !previous_output.is_empty() {
            prompt = format!(
                "<previous_step_output>\n{previous_output}\n</previous_step_output>\n\n{prompt}"
            );
        }

        // Prepend agent instructions as system prompt
        let full_prompt = if agent.instructions.is_empty() {
            prompt.clone()
        } else {
            format!("<system>\n{}\n</system>\n\n{}", agent.instructions, prompt)
        };

        // Build CLI args from agent config
        let mut cli_args: Vec<String> = vec!["--model".into(), agent.default_model.model.clone()];
        if !agent.allowed_tools.is_empty() {
            cli_args.push("--allowedTools".into());
            cli_args.push(agent.allowed_tools.join(" "));
        }

        // Execute via claude -p (blocking collect, not streaming)
        let (output_text, cost) = execute_claude_step(&full_prompt, &cli_args).await?;

        previous_output = output_text.clone();
        total_cost += cost;

        step_results.push(StepResult {
            step_index: i,
            agent_name: step.agent_name.clone(),
            prompt: prompt.clone(),
            output: output_text,
            cost_usd: cost,
        });
    }

    Ok(Json(WorkflowRunResult {
        workflow_id: wf.id,
        workflow_name: wf.name,
        steps: step_results,
        total_cost_usd: total_cost,
    }))
}

/// Run a single step through claude CLI and collect the full result.
async fn execute_claude_step(
    prompt: &str,
    cli_args: &[String],
) -> Result<(String, f64), (StatusCode, String)> {
    use tokio_stream::StreamExt;
    use tokio_stream::wrappers::ReceiverStream;

    let streamer = ClaudeCliStreamer::new();
    let rx = streamer.stream_with_args(prompt, cli_args);

    let mut stream = ReceiverStream::new(rx);
    let mut full_text = String::new();
    let mut cost = 0.0;

    while let Some(event) = stream.next().await {
        match event {
            rusvel_llm::stream::StreamEvent::Delta { text } => {
                // Accumulate deltas (they are initial chunks before result)
                let _ = text; // deltas are partial; full text comes in Done
            }
            rusvel_llm::stream::StreamEvent::Done {
                full_text: text,
                cost_usd,
            } => {
                full_text = text;
                cost = cost_usd;
            }
            rusvel_llm::stream::StreamEvent::Error { message } => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("claude step failed: {message}"),
                ));
            }
        }
    }

    if full_text.is_empty() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "no output from claude CLI step".into(),
        ));
    }

    Ok((full_text, cost))
}

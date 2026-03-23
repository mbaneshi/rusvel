//! Workflow execution engine: Sequential, Parallel, and Loop patterns.
//!
//! Composes multiple agent runs into higher-level orchestration flows.

use std::sync::Arc;

use tracing::{debug, info};

use rusvel_core::domain::*;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::ports::AgentPort;

// ════════════════════════════════════════════════════════════════════
//  Workflow types
// ════════════════════════════════════════════════════════════════════

/// A single step in a workflow graph.
#[derive(Debug, Clone)]
pub enum WorkflowStep {
    /// Run a single agent with the given config.
    Agent {
        config: AgentConfig,
        /// Optional label used for logging / debugging.
        input_mapping: Option<String>,
    },
    /// Run steps one after another; output of step N feeds into step N+1.
    Sequential { steps: Vec<WorkflowStep> },
    /// Run all steps concurrently and collect every result.
    Parallel { steps: Vec<WorkflowStep> },
    /// Repeat a step up to `max_iterations` times, or until the output
    /// contains the `until` sentinel string (default: `"DONE"`).
    Loop {
        step: Box<WorkflowStep>,
        max_iterations: u32,
        until: Option<String>,
    },
}

/// A named workflow composed of one or more steps.
#[derive(Debug, Clone)]
pub struct Workflow {
    pub name: String,
    pub steps: Vec<WorkflowStep>,
}

// ════════════════════════════════════════════════════════════════════
//  WorkflowRunner
// ════════════════════════════════════════════════════════════════════

/// Executes [`Workflow`] graphs against an [`AgentPort`].
pub struct WorkflowRunner {
    agent_port: Arc<dyn AgentPort>,
}

impl WorkflowRunner {
    /// Create a new runner backed by the given agent port.
    pub fn new(agent_port: Arc<dyn AgentPort>) -> Self {
        Self { agent_port }
    }

    /// Execute a complete workflow and return all collected outputs.
    pub async fn run_workflow(
        &self,
        workflow: &Workflow,
        input: Content,
    ) -> Result<Vec<AgentOutput>> {
        info!(workflow = %workflow.name, "starting workflow");
        let mut results = Vec::new();
        let mut current_input = input;

        for (i, step) in workflow.steps.iter().enumerate() {
            debug!(workflow = %workflow.name, step = i, "executing top-level step");
            let step_results = self.execute_step(step, current_input.clone()).await?;
            if let Some(last) = step_results.last() {
                current_input = last.content.clone();
            }
            results.extend(step_results);
        }

        info!(workflow = %workflow.name, outputs = results.len(), "workflow complete");
        Ok(results)
    }

    /// Recursively execute a single workflow step.
    fn execute_step<'a>(
        &'a self,
        step: &'a WorkflowStep,
        input: Content,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<AgentOutput>>> + Send + 'a>>
    {
        Box::pin(async move {
            match step {
                WorkflowStep::Agent {
                    config,
                    input_mapping,
                } => {
                    debug!(mapping = ?input_mapping, "running agent step");
                    let run_id = self.agent_port.create(config.clone()).await?;
                    let output = self.agent_port.run(&run_id, input).await?;
                    Ok(vec![output])
                }

                WorkflowStep::Sequential { steps } => {
                    let mut results = Vec::new();
                    let mut current = input;
                    for (i, s) in steps.iter().enumerate() {
                        debug!(seq_step = i, "sequential step");
                        let step_results = self.execute_step(s, current.clone()).await?;
                        if let Some(last) = step_results.last() {
                            current = last.content.clone();
                        }
                        results.extend(step_results);
                    }
                    Ok(results)
                }

                WorkflowStep::Parallel { steps } => {
                    let mut join_set = tokio::task::JoinSet::new();
                    for s in steps.clone() {
                        let port = Arc::clone(&self.agent_port);
                        let inp = input.clone();
                        join_set.spawn(async move {
                            let runner = WorkflowRunner::new(port);
                            runner.execute_step(&s, inp).await
                        });
                    }

                    let mut results = Vec::new();
                    while let Some(join_result) = join_set.join_next().await {
                        let step_results = join_result
                            .map_err(|e| RusvelError::Agent(format!("join error: {e}")))?;
                        results.extend(step_results?);
                    }
                    Ok(results)
                }

                WorkflowStep::Loop {
                    step,
                    max_iterations,
                    until,
                } => {
                    let sentinel = until.as_deref().unwrap_or("DONE");
                    let mut results = Vec::new();
                    let mut current = input;

                    for iteration in 0..*max_iterations {
                        debug!(iteration, max = max_iterations, "loop iteration");
                        let step_results = self.execute_step(step, current.clone()).await?;

                        if let Some(last) = step_results.last() {
                            current = last.content.clone();
                            // Check if any text part contains the sentinel.
                            let done = last
                                .content
                                .parts
                                .iter()
                                .any(|p| matches!(p, Part::Text(t) if t.contains(sentinel)));
                            results.extend(step_results);
                            if done {
                                debug!(iteration, "loop sentinel found");
                                break;
                            }
                        } else {
                            results.extend(step_results);
                            break;
                        }

                        if iteration + 1 == *max_iterations {
                            debug!("loop reached max iterations");
                        }
                    }
                    Ok(results)
                }
            }
        })
    }
}

// Tests live in tests/workflow_tests.rs to keep this file under 200 lines.

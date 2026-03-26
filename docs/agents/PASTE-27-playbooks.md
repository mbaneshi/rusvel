# Task #27: Playbooks

> Read this file, then do the task. Only modify files listed below.

## Goal

Predefined multi-step JSON pipelines that users can browse, run, and track. Thin wrapper over FlowEngine + delegate_agent.

## Files to Read First

- `crates/rusvel-core/src/domain.rs` — Flow, WorkflowDefinition types
- `crates/flow-engine/src/lib.rs` — FlowEngine, run_flow
- `crates/rusvel-agent/src/workflow.rs` — WorkflowStep, WorkflowRunner
- `crates/rusvel-api/src/flow_routes.rs` — existing flow routes pattern

## What to Build

### 1. Playbook types in `crates/rusvel-core/src/domain.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playbook {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,                    // e.g. "content", "harvest", "code"
    pub steps: Vec<PlaybookStep>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookStep {
    pub name: String,
    pub description: String,
    pub action: PlaybookAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlaybookAction {
    Agent { persona: Option<String>, prompt_template: String, tools: Vec<String> },
    Flow { flow_id: String, input_mapping: Option<String> },
    Approval { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybookRun {
    pub id: String,
    pub playbook_id: String,
    pub status: PlaybookRunStatus,
    pub step_results: Vec<serde_json::Value>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlaybookRunStatus { Running, Paused, Completed, Failed }
```

### 2. API routes in `crates/rusvel-api/src/playbooks.rs` (new)

- `GET /api/playbooks` — list all playbooks
- `GET /api/playbooks/:id` — get single playbook
- `POST /api/playbooks` — create playbook
- `POST /api/playbooks/:id/run` — execute a playbook (returns run_id, runs async)
- `GET /api/playbooks/runs/:run_id` — get run status + step results
- `GET /api/playbooks/runs` — list recent runs

### 3. Wire routes

- Add `pub mod playbooks;` to `crates/rusvel-api/src/lib.rs`
- Mount routes in router

### 4. Seed 3 built-in playbooks

Create `crates/rusvel-api/src/playbooks.rs` with 3 hardcoded playbooks:
1. **"Content from Code"** — analyze code → draft blog post → review
2. **"Opportunity Pipeline"** — scan sources → score → draft proposals
3. **"Daily Brief"** — query each dept → summarize → present

## Files to Modify

- `crates/rusvel-core/src/domain.rs` — add Playbook types
- `crates/rusvel-api/src/playbooks.rs` (new)
- `crates/rusvel-api/src/lib.rs` — add module + routes

## Verify

```bash
cargo check -p rusvel-api && cargo check --workspace
```

## Depends On

- #18 delegate_agent (done)
- #19 invoke_flow (done)
- #22 Event triggers

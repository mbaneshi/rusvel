# Task #26: Durable Execution

> Read this file, then do the task. Only modify files listed below.

## Goal

Add checkpoint/resume to FlowEngine so DAG workflows survive crashes and can retry individual nodes.

## Files to Read First

- `crates/flow-engine/src/lib.rs` — FlowEngine, run_flow(), FlowExecution, NodeResult
- `crates/rusvel-core/src/domain.rs` — Flow, FlowNode, FlowExecution types
- `crates/rusvel-core/src/ports.rs` — StoragePort (for persisting checkpoints)

## What to Build

### 1. Checkpoint types in `crates/rusvel-core/src/domain.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowCheckpoint {
    pub flow_id: String,
    pub execution_id: String,
    pub completed_nodes: Vec<String>,       // node IDs that finished
    pub node_outputs: HashMap<String, serde_json::Value>,  // outputs per node
    pub failed_node: Option<String>,        // node that failed (if any)
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
```

### 2. Checkpoint logic in `crates/flow-engine/src/lib.rs`

- After each node completes successfully, persist a `FlowCheckpoint` via `StoragePort::save("flow_checkpoints", ...)`
- Add `pub async fn resume_flow(&self, execution_id: &str) -> Result<FlowExecution>` that:
  1. Loads the checkpoint from StoragePort
  2. Skips already-completed nodes
  3. Resumes from the failed/next node
  4. Returns the combined FlowExecution
- Add `pub async fn retry_node(&self, execution_id: &str, node_id: &str) -> Result<NodeResult>` for single-node retry
- On flow completion (all nodes done), delete the checkpoint

### 3. API routes in `crates/rusvel-api/src/flow_routes.rs`

- `POST /api/flows/:id/resume` — resume a failed flow
- `POST /api/flows/:id/retry/:node_id` — retry a single node
- `GET /api/flows/:id/checkpoint` — get current checkpoint status

## Files to Modify

- `crates/rusvel-core/src/domain.rs` — add FlowCheckpoint
- `crates/flow-engine/src/lib.rs` — add checkpoint persistence, resume_flow, retry_node
- `crates/rusvel-api/src/flow_routes.rs` — add 3 routes

## Verify

```bash
cargo check -p flow-engine && cargo check -p rusvel-api && cargo check --workspace
```

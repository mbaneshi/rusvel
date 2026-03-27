//! Registration smoke: all engine tools appear in the registry with expected names.

use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::domain::{AgentConfig, AgentOutput, AgentStatus, Content, LlmUsage};
use rusvel_core::error::Result;
use rusvel_core::id::RunId;
use rusvel_core::ports::{AgentPort, JobPort, StoragePort, ToolPort};
use rusvel_db::Database;
use rusvel_event::EventBus;
use rusvel_tool::ToolRegistry;
use tempfile::tempdir;

struct StubAgent;

#[async_trait]
impl AgentPort for StubAgent {
    async fn create(&self, _: AgentConfig) -> Result<RunId> {
        Ok(RunId::new())
    }

    async fn run(&self, _: &RunId, _: Content) -> Result<AgentOutput> {
        Ok(AgentOutput {
            run_id: RunId::new(),
            content: Content::text("stub"),
            tool_calls: 0,
            usage: LlmUsage::default(),
            cost_estimate: 0.0,
            metadata: serde_json::json!({}),
        })
    }

    async fn stop(&self, _: &RunId) -> Result<()> {
        Ok(())
    }

    async fn status(&self, _: &RunId) -> Result<AgentStatus> {
        Ok(AgentStatus::Completed)
    }
}

#[tokio::test]
async fn harvest_tools_register_five_names() {
    let dir = tempdir().unwrap();
    let db = Arc::new(Database::open(dir.path().join("r.db")).unwrap());
    let reg = ToolRegistry::new();
    let engine = Arc::new(harvest_engine::HarvestEngine::new(db));
    rusvel_engine_tools::register_harvest_tools(&reg, engine).await;
    let names: Vec<String> = reg.list().into_iter().map(|d| d.name).collect();
    for n in [
        "harvest_scan",
        "harvest_score",
        "harvest_propose",
        "harvest_list",
        "harvest_pipeline",
    ] {
        assert!(
            names.iter().any(|x| x == n),
            "missing tool {n}, got {names:?}"
        );
    }
}

#[tokio::test]
async fn code_tools_register_two_names() {
    let dir = tempdir().unwrap();
    let db = Arc::new(Database::open(dir.path().join("r.db")).unwrap());
    let events = Arc::new(EventBus::new(
        db.clone() as Arc<dyn rusvel_core::ports::EventStore>
    ));
    let reg = ToolRegistry::new();
    let engine = Arc::new(code_engine::CodeEngine::new(db, events));
    rusvel_engine_tools::register_code_tools(&reg, engine).await;
    let names: Vec<String> = reg.list().into_iter().map(|d| d.name).collect();
    assert!(names.contains(&"code_analyze".into()));
    assert!(names.contains(&"code_search".into()));
}

#[tokio::test]
async fn content_tools_register_five_names() {
    let dir = tempdir().unwrap();
    let db = Arc::new(Database::open(dir.path().join("r.db")).unwrap());
    let events = Arc::new(EventBus::new(
        db.clone() as Arc<dyn rusvel_core::ports::EventStore>
    ));
    let agent: Arc<dyn AgentPort> = Arc::new(StubAgent);
    let jobs: Arc<dyn JobPort> = db.clone() as Arc<dyn JobPort>;
    let reg = ToolRegistry::new();
    let engine = Arc::new(content_engine::ContentEngine::new(
        db.clone() as Arc<dyn StoragePort>,
        events,
        agent,
        jobs,
    ));
    rusvel_engine_tools::register_content_tools(&reg, engine).await;
    let names: Vec<String> = reg.list().into_iter().map(|d| d.name).collect();
    for n in [
        "content_draft",
        "content_adapt",
        "content_publish",
        "content_list",
        "content_approve",
    ] {
        assert!(
            names.iter().any(|x| x == n),
            "missing tool {n}, got {names:?}"
        );
    }
}

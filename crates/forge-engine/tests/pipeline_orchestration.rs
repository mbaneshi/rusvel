//! S-042: orchestrate_pipeline persists FlowExecution and emits events.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use forge_engine::ForgeEngine;
use forge_engine::pipeline::{
    FLOW_EXECUTIONS_OBJECT_KIND, PipelineOrchestrationDef, PipelineStepKind, PipelineStepRunner,
};
use rusvel_core::domain::{ContentKind, Event, EventFilter, ObjectFilter, Session};
use rusvel_core::error::Result;
use rusvel_core::id::{EventId, SessionId};
use rusvel_core::ports::{
    AgentPort, ConfigPort, EventPort, JobPort, MemoryPort, ObjectStore, SessionPort, StoragePort,
};
use rusvel_core::ports::{EventStore, JobStore, MetricStore, SessionStore};
use serde_json::{Value, json};

struct MockAgent;
#[async_trait]
impl AgentPort for MockAgent {
    async fn create(&self, _: rusvel_core::domain::AgentConfig) -> Result<rusvel_core::id::RunId> {
        Ok(rusvel_core::id::RunId::new())
    }
    async fn run(
        &self,
        _: &rusvel_core::id::RunId,
        _: rusvel_core::domain::Content,
    ) -> Result<rusvel_core::domain::AgentOutput> {
        Ok(rusvel_core::domain::AgentOutput {
            run_id: rusvel_core::id::RunId::new(),
            content: rusvel_core::domain::Content::text("{}"),
            tool_calls: 0,
            usage: rusvel_core::domain::LlmUsage::default(),
            cost_estimate: 0.0,
            metadata: json!({}),
        })
    }
    async fn stop(&self, _: &rusvel_core::id::RunId) -> Result<()> {
        Ok(())
    }
    async fn status(&self, _: &rusvel_core::id::RunId) -> Result<rusvel_core::domain::AgentStatus> {
        Ok(rusvel_core::domain::AgentStatus::Idle)
    }
}

struct MemEvents(Mutex<Vec<Event>>);
#[async_trait]
impl EventPort for MemEvents {
    async fn emit(&self, event: Event) -> Result<EventId> {
        let id = event.id;
        self.0.lock().unwrap().push(event);
        Ok(id)
    }
    async fn get(&self, _: &EventId) -> Result<Option<Event>> {
        Ok(None)
    }
    async fn query(&self, _: EventFilter) -> Result<Vec<Event>> {
        Ok(vec![])
    }
}

struct MemStore {
    data: Mutex<Vec<(String, String, Value)>>,
}
#[async_trait]
impl ObjectStore for MemStore {
    async fn put(&self, kind: &str, id: &str, obj: Value) -> Result<()> {
        let mut d = self.data.lock().unwrap();
        d.retain(|(k, i, _)| !(k == kind && i == id));
        d.push((kind.into(), id.into(), obj));
        Ok(())
    }
    async fn get(&self, kind: &str, id: &str) -> Result<Option<Value>> {
        Ok(self
            .data
            .lock()
            .unwrap()
            .iter()
            .find(|(k, i, _)| k == kind && i == id)
            .map(|(_, _, v)| v.clone()))
    }
    async fn delete(&self, kind: &str, id: &str) -> Result<()> {
        self.data
            .lock()
            .unwrap()
            .retain(|(k, i, _)| !(k == kind && i == id));
        Ok(())
    }
    async fn list(&self, kind: &str, _: ObjectFilter) -> Result<Vec<Value>> {
        Ok(self
            .data
            .lock()
            .unwrap()
            .iter()
            .filter(|(k, _, _)| k == kind)
            .map(|(_, _, v)| v.clone())
            .collect())
    }
}

struct St(Arc<MemStore>);
impl StoragePort for St {
    fn events(&self) -> &dyn EventStore {
        panic!()
    }
    fn objects(&self) -> &dyn ObjectStore {
        self.0.as_ref()
    }
    fn sessions(&self) -> &dyn SessionStore {
        panic!()
    }
    fn jobs(&self) -> &dyn JobStore {
        panic!()
    }
    fn metrics(&self) -> &dyn MetricStore {
        panic!()
    }
}

struct MockJobs;
#[async_trait]
impl JobPort for MockJobs {
    async fn enqueue(&self, _: rusvel_core::domain::NewJob) -> Result<rusvel_core::id::JobId> {
        Ok(rusvel_core::id::JobId::new())
    }
    async fn dequeue(
        &self,
        _: &[rusvel_core::domain::JobKind],
    ) -> Result<Option<rusvel_core::domain::Job>> {
        Ok(None)
    }
    async fn complete(
        &self,
        _: &rusvel_core::id::JobId,
        _: rusvel_core::domain::JobResult,
    ) -> Result<()> {
        Ok(())
    }
    async fn hold_for_approval(
        &self,
        _: &rusvel_core::id::JobId,
        _: rusvel_core::domain::JobResult,
    ) -> Result<()> {
        Ok(())
    }
    async fn fail(&self, _: &rusvel_core::id::JobId, _: String) -> Result<()> {
        Ok(())
    }
    async fn schedule(
        &self,
        _: rusvel_core::domain::NewJob,
        _: &str,
    ) -> Result<rusvel_core::id::JobId> {
        Ok(rusvel_core::id::JobId::new())
    }
    async fn cancel(&self, _: &rusvel_core::id::JobId) -> Result<()> {
        Ok(())
    }
    async fn approve(&self, _: &rusvel_core::id::JobId) -> Result<()> {
        Ok(())
    }
    async fn list(
        &self,
        _: rusvel_core::domain::JobFilter,
    ) -> Result<Vec<rusvel_core::domain::Job>> {
        Ok(vec![])
    }
}

struct MockSess;
#[async_trait]
impl SessionPort for MockSess {
    async fn create(&self, _: Session) -> Result<SessionId> {
        Ok(SessionId::new())
    }
    async fn load(&self, _: &SessionId) -> Result<Session> {
        panic!()
    }
    async fn save(&self, _: &Session) -> Result<()> {
        Ok(())
    }
    async fn list(&self) -> Result<Vec<rusvel_core::domain::SessionSummary>> {
        Ok(vec![])
    }
}

struct MockCfg;
impl ConfigPort for MockCfg {
    fn get_value(&self, _: &str) -> Result<Option<Value>> {
        Ok(None)
    }
    fn set_value(&self, _: &str, _: Value) -> Result<()> {
        Ok(())
    }
}

struct MockMem;
#[async_trait]
impl MemoryPort for MockMem {
    async fn store(&self, _: rusvel_core::domain::MemoryEntry) -> Result<uuid::Uuid> {
        Ok(uuid::Uuid::now_v7())
    }
    async fn recall(&self, _: &uuid::Uuid) -> Result<Option<rusvel_core::domain::MemoryEntry>> {
        Ok(None)
    }
    async fn search(
        &self,
        _: &SessionId,
        _: &str,
        _: usize,
    ) -> Result<Vec<rusvel_core::domain::MemoryEntry>> {
        Ok(vec![])
    }
    async fn forget(&self, _: &uuid::Uuid) -> Result<()> {
        Ok(())
    }
}

struct SeqRunner;

#[async_trait]
impl PipelineStepRunner for SeqRunner {
    async fn scan(&self, _: &SessionId) -> Result<Value> {
        Ok(json!({"opportunity_ids":["x"],"titles":["T"],"count":1}))
    }
    async fn score(&self, _: &SessionId, _: &Value) -> Result<Value> {
        Ok(json!({"rescored":1}))
    }
    async fn propose(&self, _: &SessionId, _: &Value, _: &str) -> Result<Value> {
        Ok(json!({"opportunity_id":"x"}))
    }
    async fn draft_content(
        &self,
        _: &SessionId,
        _: &Value,
        _: Option<&str>,
        _: ContentKind,
    ) -> Result<Value> {
        Ok(json!({"content_id":"c1"}))
    }
}

#[tokio::test]
async fn orchestrate_pipeline_persists_and_succeeds() {
    let store = Arc::new(MemStore {
        data: Mutex::new(vec![]),
    });
    let storage: Arc<dyn StoragePort> = Arc::new(St(store.clone()));
    let mem_events = Arc::new(MemEvents(Mutex::new(vec![])));
    let events: Arc<dyn EventPort> = mem_events.clone();

    let forge = Arc::new(ForgeEngine::new(
        Arc::new(MockAgent),
        events.clone(),
        Arc::new(MockMem),
        storage,
        Arc::new(MockJobs),
        Arc::new(MockSess),
        Arc::new(MockCfg),
    ));

    let sid = SessionId::new();
    let def = PipelineOrchestrationDef {
        steps: vec![
            PipelineStepKind::Scan,
            PipelineStepKind::Score,
            PipelineStepKind::Propose,
            PipelineStepKind::DraftContent,
        ],
        proposal_profile: "p".into(),
        draft_topic: None,
        draft_kind: ContentKind::Blog,
    };

    let exec = forge
        .orchestrate_pipeline(sid, def, &SeqRunner)
        .await
        .expect("pipeline");

    assert_eq!(
        exec.status,
        rusvel_core::domain::FlowExecutionStatus::Succeeded
    );
    let found = store
        .data
        .lock()
        .unwrap()
        .iter()
        .any(|(k, _, _)| k == FLOW_EXECUTIONS_OBJECT_KIND);
    assert!(found);

    let kinds: Vec<String> = {
        let guard = mem_events.0.lock().unwrap();
        guard.iter().map(|e| e.kind.clone()).collect()
    };
    assert!(
        kinds
            .iter()
            .any(|k| k == forge_engine::events::PIPELINE_STARTED)
    );
    assert!(
        kinds
            .iter()
            .any(|k| k == forge_engine::events::PIPELINE_COMPLETED)
    );
}

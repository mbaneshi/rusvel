//! Infra Engine — deployments, monitoring, and incident management.

use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::domain::{Capability, Event, HealthStatus};
use rusvel_core::error::Result;
use rusvel_core::id::EventId;
use rusvel_core::ports::{AgentPort, EventPort, JobPort, StoragePort};

pub mod deploy;
pub mod incident;
pub mod monitor;

pub use deploy::{DeployManager, DeployStatus, Deployment, DeploymentId};
pub use incident::{Incident, IncidentId, IncidentManager, IncidentStatus, Severity};
pub use monitor::{CheckStatus, HealthCheck, HealthCheckId, MonitorManager};

pub mod events {
    pub const DEPLOY_COMPLETED: &str = "infra.deploy.completed";
    pub const ALERT_FIRED: &str = "infra.alert.fired";
    pub const INCIDENT_OPENED: &str = "infra.incident.opened";
    pub const INCIDENT_RESOLVED: &str = "infra.incident.resolved";
}

pub struct InfraEngine {
    #[allow(dead_code)]
    storage: Arc<dyn StoragePort>,
    events: Arc<dyn EventPort>,
    #[allow(dead_code)]
    agent: Arc<dyn AgentPort>,
    #[allow(dead_code)]
    jobs: Arc<dyn JobPort>,
    deploy: DeployManager,
    monitor: MonitorManager,
    incidents: IncidentManager,
}

impl InfraEngine {
    pub fn new(
        storage: Arc<dyn StoragePort>,
        events: Arc<dyn EventPort>,
        agent: Arc<dyn AgentPort>,
        jobs: Arc<dyn JobPort>,
    ) -> Self {
        let deploy = DeployManager::new(Arc::clone(&storage));
        let monitor = MonitorManager::new(Arc::clone(&storage));
        let incidents = IncidentManager::new(Arc::clone(&storage));
        Self {
            storage,
            events,
            agent,
            jobs,
            deploy,
            monitor,
            incidents,
        }
    }

    pub fn deploy(&self) -> &DeployManager {
        &self.deploy
    }
    pub fn monitor(&self) -> &MonitorManager {
        &self.monitor
    }
    pub fn incidents(&self) -> &IncidentManager {
        &self.incidents
    }

    /// Emit a domain event on the event bus.
    pub async fn emit_event(&self, kind: &str, payload: serde_json::Value) -> Result<EventId> {
        let event = Event {
            id: EventId::new(),
            session_id: None,
            run_id: None,
            source: "infra".into(),
            kind: kind.into(),
            payload,
            created_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        };
        self.events.emit(event).await
    }
}

#[async_trait]
impl rusvel_core::engine::Engine for InfraEngine {
    fn kind(&self) -> &str {
        "infra"
    }
    fn name(&self) -> &'static str {
        "Infra Engine"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Custom("deploy".into()),
            Capability::Custom("monitoring".into()),
            Capability::Custom("incident_management".into()),
        ]
    }

    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn health(&self) -> Result<HealthStatus> {
        Ok(HealthStatus {
            healthy: true,
            message: None,
            metadata: serde_json::json!({}),
        })
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use rusvel_core::domain::*;
    use rusvel_core::id::*;
    use rusvel_core::ports::*;
    use std::sync::Mutex;

    struct StubStore {
        objects: StubObjects,
    }
    impl StubStore {
        fn new() -> Self {
            Self {
                objects: StubObjects::new(),
            }
        }
    }
    struct StubEvents;
    struct StubSessions;
    struct StubJobStore;
    struct StubMetrics;
    struct StubObjects {
        data: Mutex<Vec<(String, String, serde_json::Value)>>,
    }
    impl StubObjects {
        fn new() -> Self {
            Self {
                data: Mutex::new(Vec::new()),
            }
        }
    }

    impl StoragePort for StubStore {
        fn events(&self) -> &dyn EventStore {
            &StubEvents
        }
        fn objects(&self) -> &dyn ObjectStore {
            &self.objects
        }
        fn sessions(&self) -> &dyn SessionStore {
            &StubSessions
        }
        fn jobs(&self) -> &dyn JobStore {
            &StubJobStore
        }
        fn metrics(&self) -> &dyn MetricStore {
            &StubMetrics
        }
    }

    #[async_trait]
    impl EventStore for StubEvents {
        async fn append(&self, _: &Event) -> Result<()> {
            Ok(())
        }
        async fn get(&self, _: &EventId) -> Result<Option<Event>> {
            Ok(None)
        }
        async fn query(&self, _: EventFilter) -> Result<Vec<Event>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl ObjectStore for StubObjects {
        async fn put(&self, kind: &str, id: &str, object: serde_json::Value) -> Result<()> {
            let mut data = self.data.lock().unwrap();
            data.retain(|(k, i, _)| !(k == kind && i == id));
            data.push((kind.into(), id.into(), object));
            Ok(())
        }
        async fn get(&self, kind: &str, id: &str) -> Result<Option<serde_json::Value>> {
            let data = self.data.lock().unwrap();
            Ok(data
                .iter()
                .find(|(k, i, _)| k == kind && i == id)
                .map(|(_, _, v)| v.clone()))
        }
        async fn delete(&self, kind: &str, id: &str) -> Result<()> {
            let mut data = self.data.lock().unwrap();
            data.retain(|(k, i, _)| !(k == kind && i == id));
            Ok(())
        }
        async fn list(&self, kind: &str, _filter: ObjectFilter) -> Result<Vec<serde_json::Value>> {
            let data = self.data.lock().unwrap();
            Ok(data
                .iter()
                .filter(|(k, _, _)| k == kind)
                .map(|(_, _, v)| v.clone())
                .collect())
        }
    }

    #[async_trait]
    impl SessionStore for StubSessions {
        async fn put_session(&self, _: &Session) -> Result<()> {
            Ok(())
        }
        async fn get_session(&self, _: &SessionId) -> Result<Option<Session>> {
            Ok(None)
        }
        async fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
            Ok(vec![])
        }
        async fn put_run(&self, _: &Run) -> Result<()> {
            Ok(())
        }
        async fn get_run(&self, _: &RunId) -> Result<Option<Run>> {
            Ok(None)
        }
        async fn list_runs(&self, _: &SessionId) -> Result<Vec<Run>> {
            Ok(vec![])
        }
        async fn put_thread(&self, _: &Thread) -> Result<()> {
            Ok(())
        }
        async fn get_thread(&self, _: &ThreadId) -> Result<Option<Thread>> {
            Ok(None)
        }
        async fn list_threads(&self, _: &RunId) -> Result<Vec<Thread>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl JobStore for StubJobStore {
        async fn enqueue(&self, _: &Job) -> Result<()> {
            Ok(())
        }
        async fn dequeue(&self, _: &[JobKind]) -> Result<Option<Job>> {
            Ok(None)
        }
        async fn update(&self, _: &Job) -> Result<()> {
            Ok(())
        }
        async fn get(&self, _: &JobId) -> Result<Option<Job>> {
            Ok(None)
        }
        async fn list(&self, _: JobFilter) -> Result<Vec<Job>> {
            Ok(vec![])
        }
    }

    #[async_trait]
    impl MetricStore for StubMetrics {
        async fn record(&self, _: &MetricPoint) -> Result<()> {
            Ok(())
        }
        async fn query(&self, _: MetricFilter) -> Result<Vec<MetricPoint>> {
            Ok(vec![])
        }
    }

    struct StubEventPort;

    #[async_trait]
    impl EventPort for StubEventPort {
        async fn emit(&self, event: Event) -> Result<EventId> {
            Ok(event.id)
        }
        async fn get(&self, _: &EventId) -> Result<Option<Event>> {
            Ok(None)
        }
        async fn query(&self, _: EventFilter) -> Result<Vec<Event>> {
            Ok(vec![])
        }
    }

    struct StubAgentPort;

    #[async_trait]
    impl AgentPort for StubAgentPort {
        async fn create(&self, _: AgentConfig) -> Result<RunId> {
            Ok(RunId::new())
        }
        async fn run(&self, _: &RunId, _: Content) -> Result<AgentOutput> {
            Ok(AgentOutput {
                run_id: RunId::new(),
                content: Content::text("ok"),
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
            Ok(AgentStatus::Idle)
        }
    }

    struct StubJobPort;

    #[async_trait]
    impl JobPort for StubJobPort {
        async fn enqueue(&self, _: NewJob) -> Result<JobId> {
            Ok(JobId::new())
        }
        async fn dequeue(&self, _: &[JobKind]) -> Result<Option<Job>> {
            Ok(None)
        }
        async fn complete(&self, _: &JobId, _: JobResult) -> Result<()> {
            Ok(())
        }
        async fn hold_for_approval(&self, _: &JobId, _: JobResult) -> Result<()> {
            Ok(())
        }
        async fn fail(&self, _: &JobId, _: String) -> Result<()> {
            Ok(())
        }
        async fn schedule(&self, _: NewJob, _: &str) -> Result<JobId> {
            Ok(JobId::new())
        }
        async fn cancel(&self, _: &JobId) -> Result<()> {
            Ok(())
        }
        async fn approve(&self, _: &JobId) -> Result<()> {
            Ok(())
        }
        async fn list(&self, _: JobFilter) -> Result<Vec<Job>> {
            Ok(vec![])
        }
    }

    fn make_engine() -> InfraEngine {
        InfraEngine::new(
            Arc::new(StubStore::new()),
            Arc::new(StubEventPort),
            Arc::new(StubAgentPort),
            Arc::new(StubJobPort),
        )
    }

    #[tokio::test]
    async fn deploy_record() {
        let engine = make_engine();
        let sid = SessionId::new();

        let deployment = engine
            .deploy()
            .record_deployment(
                sid,
                "rusvel-api".into(),
                "v0.2.0".into(),
                "production".into(),
            )
            .await
            .unwrap();

        assert_eq!(deployment.service, "rusvel-api");
        assert_eq!(deployment.status, crate::DeployStatus::Pending);

        let all = engine.deploy().list_deployments(sid).await.unwrap();
        assert_eq!(all.len(), 1);
    }

    #[tokio::test]
    async fn health_returns_healthy() {
        use rusvel_core::engine::Engine;
        let engine = make_engine();
        let status = engine.health().await.unwrap();
        assert!(status.healthy);
    }
}

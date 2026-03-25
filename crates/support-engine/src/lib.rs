//! Support Engine — ticket management, knowledge base, and NPS tracking.

use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::domain::{Capability, Event, HealthStatus};
use rusvel_core::error::Result;
use rusvel_core::id::EventId;
use rusvel_core::ports::{AgentPort, EventPort, JobPort, StoragePort};

pub mod knowledge;
pub mod nps;
pub mod ticket;

pub use knowledge::{Article, ArticleId, KnowledgeManager};
pub use nps::{NpsManager, NpsResponse, NpsResponseId};
pub use ticket::{Ticket, TicketId, TicketManager, TicketPriority, TicketStatus};

pub mod events {
    pub const TICKET_CREATED: &str = "support.ticket.created";
    pub const TICKET_RESOLVED: &str = "support.ticket.resolved";
    pub const ARTICLE_PUBLISHED: &str = "support.article.published";
    pub const NPS_RECORDED: &str = "support.nps.recorded";
}

pub struct SupportEngine {
    #[allow(dead_code)]
    storage: Arc<dyn StoragePort>,
    events: Arc<dyn EventPort>,
    #[allow(dead_code)]
    agent: Arc<dyn AgentPort>,
    #[allow(dead_code)]
    jobs: Arc<dyn JobPort>,
    tickets: TicketManager,
    knowledge: KnowledgeManager,
    nps: NpsManager,
}

impl SupportEngine {
    pub fn new(
        storage: Arc<dyn StoragePort>,
        events: Arc<dyn EventPort>,
        agent: Arc<dyn AgentPort>,
        jobs: Arc<dyn JobPort>,
    ) -> Self {
        let tickets = TicketManager::new(Arc::clone(&storage));
        let knowledge = KnowledgeManager::new(Arc::clone(&storage));
        let nps = NpsManager::new(Arc::clone(&storage));
        Self {
            storage,
            events,
            agent,
            jobs,
            tickets,
            knowledge,
            nps,
        }
    }

    pub fn tickets(&self) -> &TicketManager {
        &self.tickets
    }
    pub fn knowledge(&self) -> &KnowledgeManager {
        &self.knowledge
    }
    pub fn nps(&self) -> &NpsManager {
        &self.nps
    }

    /// Emit a domain event on the event bus.
    pub async fn emit_event(&self, kind: &str, payload: serde_json::Value) -> Result<EventId> {
        let event = Event {
            id: EventId::new(),
            session_id: None,
            run_id: None,
            source: "support".into(),
            kind: kind.into(),
            payload,
            created_at: chrono::Utc::now(),
            metadata: serde_json::json!({}),
        };
        self.events.emit(event).await
    }
}

#[async_trait]
impl rusvel_core::engine::Engine for SupportEngine {
    fn kind(&self) -> &str {
        "support"
    }
    fn name(&self) -> &'static str {
        "Support Engine"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability::Custom("tickets".into()),
            Capability::Custom("knowledge_base".into()),
            Capability::Custom("nps".into()),
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

    fn make_engine() -> SupportEngine {
        SupportEngine::new(
            Arc::new(StubStore::new()),
            Arc::new(StubEventPort),
            Arc::new(StubAgentPort),
            Arc::new(StubJobPort),
        )
    }

    #[tokio::test]
    async fn ticket_create() {
        let engine = make_engine();
        let sid = SessionId::new();

        let ticket = engine
            .tickets()
            .create_ticket(
                sid,
                "Login broken".into(),
                "Cannot log in with SSO".into(),
                TicketPriority::High,
                "user@example.com".into(),
            )
            .await
            .unwrap();

        assert_eq!(ticket.subject, "Login broken");
        assert_eq!(ticket.status, TicketStatus::Open);

        let all = engine.tickets().list_tickets(sid).await.unwrap();
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

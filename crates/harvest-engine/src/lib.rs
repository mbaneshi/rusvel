//! Harvest Engine — opportunity discovery, scoring, and pipeline management.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use rusvel_core::engine::Engine;
use rusvel_core::ports::{AgentPort, EventPort, StoragePort};
use rusvel_core::{
    Capability, EngineKind, Event, EventId, HealthStatus, ObjectFilter, Opportunity,
    OpportunityId, OpportunityStage, Result, RusvelError, SessionId,
};

pub mod events;
pub mod pipeline;
pub mod proposal;
pub mod scorer;
pub mod source;

use pipeline::{Pipeline, PipelineStats};
use proposal::{Proposal, ProposalGenerator};

/// Persisted proposal row in the object store (`kind`: `"proposal"`).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct StoredProposalRecord {
    session_id: SessionId,
    opportunity_id: String,
    proposal: Proposal,
}
use scorer::OpportunityScorer;
use source::HarvestSource;

/// Configuration for the Harvest engine.
#[derive(Debug, Clone)]
pub struct HarvestConfig {
    pub skills: Vec<String>,
    pub min_budget: Option<f64>,
}

impl Default for HarvestConfig {
    fn default() -> Self {
        Self {
            skills: vec!["rust".into()],
            min_budget: None,
        }
    }
}

/// The Harvest engine discovers, scores, and manages freelance opportunities.
pub struct HarvestEngine {
    storage: Arc<dyn StoragePort>,
    event_port: Option<Arc<dyn EventPort>>,
    agent: Option<Arc<dyn AgentPort>>,
    config: HarvestConfig,
}

impl HarvestEngine {
    pub fn new(storage: Arc<dyn StoragePort>) -> Self {
        Self {
            storage,
            event_port: None,
            agent: None,
            config: HarvestConfig::default(),
        }
    }

    pub fn with_events(mut self, events: Arc<dyn EventPort>) -> Self {
        self.event_port = Some(events);
        self
    }

    pub fn with_agent(mut self, agent: Arc<dyn AgentPort>) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn with_config(mut self, config: HarvestConfig) -> Self {
        self.config = config;
        self
    }

    /// Skills used for scoring and RSS query expansion (see [`HarvestConfig`]).
    pub fn harvest_skills(&self) -> &[String] {
        &self.config.skills
    }

    /// Emit a domain event if the event port is configured.
    async fn emit(&self, session_id: &SessionId, kind: &str, payload: serde_json::Value) {
        if let Some(ep) = &self.event_port {
            let event = Event {
                id: EventId::new(),
                session_id: Some(*session_id),
                run_id: None,
                source: EngineKind::Harvest,
                kind: kind.into(),
                payload,
                created_at: Utc::now(),
                metadata: serde_json::json!({}),
            };
            let _ = ep.emit(event).await;
        }
    }

    /// Scan a source, score results, store them, and return opportunities.
    pub async fn scan(
        &self,
        session_id: &SessionId,
        source: &dyn HarvestSource,
    ) -> Result<Vec<Opportunity>> {
        self.emit(
            session_id,
            events::SCAN_STARTED,
            serde_json::json!({"source": source.name()}),
        )
        .await;

        let raw_items = source.scan().await?;
        let scorer = OpportunityScorer::new(
            self.agent.clone(),
            self.config.skills.clone(),
            self.config.min_budget,
        );

        let pipe = Pipeline::new(self.storage.clone());
        let mut results = Vec::new();

        for raw in &raw_items {
            let scored = scorer.score(raw).await?;

            let opportunity = Opportunity {
                id: OpportunityId::new(),
                session_id: *session_id,
                source: source.source_kind(),
                title: scored.raw.title.clone(),
                url: scored.raw.url.clone(),
                description: scored.raw.description.clone(),
                score: scored.score,
                stage: OpportunityStage::Cold,
                value_estimate: parse_budget(&scored.raw.budget),
                metadata: serde_json::json!({
                    "reasoning": scored.reasoning,
                    "skills": scored.raw.skills,
                    "posted_at": scored.raw.posted_at,
                }),
            };

            pipe.add(&opportunity).await?;

            self.emit(
                session_id,
                events::OPPORTUNITY_DISCOVERED,
                serde_json::json!({"id": opportunity.id.to_string(), "title": &opportunity.title}),
            )
            .await;

            results.push(opportunity);
        }

        self.emit(
            session_id,
            events::SCAN_COMPLETED,
            serde_json::json!({"count": results.len()}),
        )
        .await;

        Ok(results)
    }

    /// Re-score an existing opportunity and update its stored score.
    pub async fn score_opportunity(
        &self,
        session_id: &SessionId,
        opportunity_id: &str,
    ) -> Result<f64> {
        let value = self
            .storage
            .objects()
            .get("opportunity", opportunity_id)
            .await?
            .ok_or_else(|| RusvelError::NotFound {
                kind: "opportunity".into(),
                id: opportunity_id.into(),
            })?;

        let mut opp: Opportunity = serde_json::from_value(value)?;
        let raw = source::RawOpportunity {
            title: opp.title.clone(),
            description: opp.description.clone(),
            url: opp.url.clone(),
            budget: opp.value_estimate.map(|v| format!("${v}")),
            skills: opp.metadata["skills"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
            posted_at: opp.metadata["posted_at"].as_str().map(String::from),
            source_data: serde_json::json!({}),
        };

        let scorer = OpportunityScorer::new(
            self.agent.clone(),
            self.config.skills.clone(),
            self.config.min_budget,
        );
        let scored = scorer.score(&raw).await?;
        opp.score = scored.score;

        let updated = serde_json::to_value(&opp)?;
        self.storage
            .objects()
            .put("opportunity", opportunity_id, updated)
            .await?;

        self.emit(
            session_id,
            events::OPPORTUNITY_SCORED,
            serde_json::json!({"id": opportunity_id, "score": scored.score}),
        )
        .await;

        Ok(scored.score)
    }

    /// Generate a proposal for a stored opportunity.
    pub async fn generate_proposal(
        &self,
        session_id: &SessionId,
        opportunity_id: &str,
        profile: &str,
    ) -> Result<Proposal> {
        let agent = self
            .agent
            .as_ref()
            .ok_or_else(|| RusvelError::Internal("No agent configured".into()))?;

        let value = self
            .storage
            .objects()
            .get("opportunity", opportunity_id)
            .await?
            .ok_or_else(|| RusvelError::NotFound {
                kind: "opportunity".into(),
                id: opportunity_id.into(),
            })?;

        let opp: Opportunity = serde_json::from_value(value)?;
        let generator = ProposalGenerator::new(agent.clone());
        let proposal = generator.generate(&opp, profile).await?;

        self.emit(
            session_id,
            events::PROPOSAL_GENERATED,
            serde_json::json!({"opportunity_id": opportunity_id}),
        )
        .await;

        let ts = Utc::now().timestamp_millis();
        let key = format!("{opportunity_id}_{ts}");
        let record = StoredProposalRecord {
            session_id: *session_id,
            opportunity_id: opportunity_id.to_string(),
            proposal: proposal.clone(),
        };
        self.storage
            .objects()
            .put(
                "proposal",
                &key,
                serde_json::to_value(&record)?,
            )
            .await?;

        self.emit(
            session_id,
            events::PROPOSAL_PERSISTED,
            serde_json::json!({
                "key": key,
                "opportunity_id": opportunity_id,
            }),
        )
        .await;

        Ok(proposal)
    }

    /// List persisted proposals for a session (`kind`: `"proposal"`).
    pub async fn get_proposals(&self, session_id: &SessionId) -> Result<Vec<Proposal>> {
        let filter = ObjectFilter {
            session_id: Some(*session_id),
            tags: vec![],
            limit: None,
            offset: None,
        };
        let rows = self.storage.objects().list("proposal", filter).await?;
        let mut out = Vec::new();
        for v in rows {
            let rec: StoredProposalRecord = serde_json::from_value(v)?;
            out.push(rec.proposal);
        }
        Ok(out)
    }

    /// Get pipeline statistics for a session.
    pub async fn pipeline(&self, session_id: &SessionId) -> Result<PipelineStats> {
        Pipeline::new(self.storage.clone()).stats(session_id).await
    }

    /// List opportunities, optionally filtered by stage.
    pub async fn list_opportunities(
        &self,
        session_id: &SessionId,
        stage: Option<&OpportunityStage>,
    ) -> Result<Vec<Opportunity>> {
        Pipeline::new(self.storage.clone())
            .list(session_id, stage)
            .await
    }
}

/// Parse a budget string like "$5000" into an f64.
fn parse_budget(budget: &Option<String>) -> Option<f64> {
    budget.as_ref().and_then(|s| {
        s.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect::<String>()
            .parse()
            .ok()
    })
}

#[async_trait]
impl Engine for HarvestEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::Harvest
    }

    fn name(&self) -> &'static str {
        "Harvest Engine"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::OpportunityDiscovery]
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
            metadata: serde_json::json!({"skills": self.config.skills}),
        })
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use rusvel_core::domain::ObjectFilter;
    use rusvel_core::ports::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    // ── In-memory ObjectStore ──────────────────────────────────────

    #[derive(Default)]
    struct MemObjectStore {
        data: Mutex<HashMap<String, HashMap<String, serde_json::Value>>>,
    }

    #[async_trait]
    impl ObjectStore for MemObjectStore {
        async fn put(&self, kind: &str, id: &str, object: serde_json::Value) -> Result<()> {
            self.data
                .lock()
                .unwrap()
                .entry(kind.into())
                .or_default()
                .insert(id.into(), object);
            Ok(())
        }

        async fn get(&self, kind: &str, id: &str) -> Result<Option<serde_json::Value>> {
            Ok(self
                .data
                .lock()
                .unwrap()
                .get(kind)
                .and_then(|m| m.get(id).cloned()))
        }

        async fn delete(&self, kind: &str, id: &str) -> Result<()> {
            if let Some(m) = self.data.lock().unwrap().get_mut(kind) {
                m.remove(id);
            }
            Ok(())
        }

        async fn list(&self, kind: &str, filter: ObjectFilter) -> Result<Vec<serde_json::Value>> {
            let data = self.data.lock().unwrap();
            let Some(m) = data.get(kind) else {
                return Ok(vec![]);
            };
            let mut out: Vec<serde_json::Value> = m
                .values()
                .filter(|v| {
                    filter.session_id.map_or(true, |sid| {
                        v.get("session_id")
                            .and_then(|x| x.as_str())
                            .is_some_and(|s| s == sid.to_string())
                    })
                })
                .cloned()
                .collect();
            if let Some(lim) = filter.limit {
                out.truncate(lim as usize);
            }
            Ok(out)
        }
    }

    // ── Stub stores ───────────────────────────────────────────────

    struct StubEventStore;
    #[async_trait]
    impl EventStore for StubEventStore {
        async fn append(&self, _: &Event) -> Result<()> {
            Ok(())
        }
        async fn get(&self, _: &EventId) -> Result<Option<Event>> {
            Ok(None)
        }
        async fn query(&self, _: rusvel_core::EventFilter) -> Result<Vec<Event>> {
            Ok(vec![])
        }
    }

    struct StubMetricStore;
    #[async_trait]
    impl MetricStore for StubMetricStore {
        async fn record(&self, _: &rusvel_core::MetricPoint) -> Result<()> {
            Ok(())
        }
        async fn query(
            &self,
            _: rusvel_core::MetricFilter,
        ) -> Result<Vec<rusvel_core::MetricPoint>> {
            Ok(vec![])
        }
    }

    struct StubSessionStore;
    #[async_trait]
    impl SessionStore for StubSessionStore {
        async fn put_session(&self, _: &rusvel_core::Session) -> Result<()> {
            Ok(())
        }
        async fn get_session(&self, _: &SessionId) -> Result<Option<rusvel_core::Session>> {
            Ok(None)
        }
        async fn list_sessions(&self) -> Result<Vec<rusvel_core::SessionSummary>> {
            Ok(vec![])
        }
        async fn put_run(&self, _: &rusvel_core::Run) -> Result<()> {
            Ok(())
        }
        async fn get_run(&self, _: &rusvel_core::RunId) -> Result<Option<rusvel_core::Run>> {
            Ok(None)
        }
        async fn list_runs(&self, _: &SessionId) -> Result<Vec<rusvel_core::Run>> {
            Ok(vec![])
        }
        async fn put_thread(&self, _: &rusvel_core::Thread) -> Result<()> {
            Ok(())
        }
        async fn get_thread(
            &self,
            _: &rusvel_core::ThreadId,
        ) -> Result<Option<rusvel_core::Thread>> {
            Ok(None)
        }
        async fn list_threads(&self, _: &rusvel_core::RunId) -> Result<Vec<rusvel_core::Thread>> {
            Ok(vec![])
        }
    }

    struct StubJobStore;
    #[async_trait]
    impl JobStore for StubJobStore {
        async fn enqueue(&self, _: &rusvel_core::Job) -> Result<()> {
            Ok(())
        }
        async fn dequeue(&self, _: &[rusvel_core::JobKind]) -> Result<Option<rusvel_core::Job>> {
            Ok(None)
        }
        async fn update(&self, _: &rusvel_core::Job) -> Result<()> {
            Ok(())
        }
        async fn get(&self, _: &rusvel_core::JobId) -> Result<Option<rusvel_core::Job>> {
            Ok(None)
        }
        async fn list(&self, _: rusvel_core::JobFilter) -> Result<Vec<rusvel_core::Job>> {
            Ok(vec![])
        }
    }

    // ── TestStorage ───────────────────────────────────────────────

    struct TestStorage {
        objects: MemObjectStore,
        events: StubEventStore,
        metrics: StubMetricStore,
        sessions: StubSessionStore,
        jobs: StubJobStore,
    }

    impl TestStorage {
        fn new() -> Self {
            Self {
                objects: MemObjectStore::default(),
                events: StubEventStore,
                metrics: StubMetricStore,
                sessions: StubSessionStore,
                jobs: StubJobStore,
            }
        }
    }

    impl StoragePort for TestStorage {
        fn events(&self) -> &dyn EventStore {
            &self.events
        }
        fn objects(&self) -> &dyn ObjectStore {
            &self.objects
        }
        fn metrics(&self) -> &dyn MetricStore {
            &self.metrics
        }
        fn sessions(&self) -> &dyn SessionStore {
            &self.sessions
        }
        fn jobs(&self) -> &dyn JobStore {
            &self.jobs
        }
    }

    // ── Tests ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn scan_with_mock_source_returns_opportunities() {
        let storage = Arc::new(TestStorage::new());
        let engine = HarvestEngine::new(storage.clone()).with_config(HarvestConfig {
            skills: vec!["rust".into(), "axum".into()],
            min_budget: Some(1000.0),
        });

        let session_id = SessionId::new();
        let results = engine.scan(&session_id, &source::MockSource).await.unwrap();

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].title, "Build a REST API in Rust");
        assert!(results[0].score > 0.0);
        assert_eq!(results[0].stage, OpportunityStage::Cold);
    }

    #[tokio::test]
    async fn pipeline_stores_and_lists() {
        let storage = Arc::new(TestStorage::new());
        let engine = HarvestEngine::new(storage.clone()).with_config(HarvestConfig {
            skills: vec!["rust".into()],
            min_budget: None,
        });

        let session_id = SessionId::new();
        let results = engine.scan(&session_id, &source::MockSource).await.unwrap();
        assert!(!results.is_empty());

        let listed = engine.list_opportunities(&session_id, None).await.unwrap();
        assert_eq!(listed.len(), results.len());
    }

    #[tokio::test]
    async fn pipeline_stats_correct() {
        let storage = Arc::new(TestStorage::new());
        let engine = HarvestEngine::new(storage.clone()).with_config(HarvestConfig {
            skills: vec!["rust".into()],
            min_budget: None,
        });

        let session_id = SessionId::new();
        engine.scan(&session_id, &source::MockSource).await.unwrap();

        let stats = engine.pipeline(&session_id).await.unwrap();
        assert_eq!(stats.total, 3);
        // All should be in Cold stage
        assert_eq!(*stats.by_stage.get("Cold").unwrap_or(&0), 3);
    }

    #[tokio::test]
    async fn health_returns_healthy() {
        let engine = HarvestEngine::new(Arc::new(TestStorage::new()));
        let status = engine.health().await.unwrap();
        assert!(status.healthy);
    }

    #[tokio::test]
    async fn get_proposals_respects_session_filter() {
        let storage = Arc::new(TestStorage::new());
        let engine = HarvestEngine::new(storage.clone());
        let sid = SessionId::new();
        let other = SessionId::new();
        let proposal = Proposal {
            body: "hello".into(),
            estimated_value: None,
            tone: "pro".into(),
            metadata: serde_json::json!({}),
        };
        let rec = StoredProposalRecord {
            session_id: sid,
            opportunity_id: "opp-a".into(),
            proposal,
        };
        storage
            .objects()
            .put(
                "proposal",
                "opp-a_1",
                serde_json::to_value(&rec).unwrap(),
            )
            .await
            .unwrap();

        let mut rec_other = rec.clone();
        rec_other.session_id = other;
        rec_other.opportunity_id = "opp-b".into();
        storage
            .objects()
            .put(
                "proposal",
                "opp-b_1",
                serde_json::to_value(&rec_other).unwrap(),
            )
            .await
            .unwrap();

        let mine = engine.get_proposals(&sid).await.unwrap();
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].body, "hello");

        let empty = engine.get_proposals(&SessionId::new()).await.unwrap();
        assert!(empty.is_empty());
    }
}

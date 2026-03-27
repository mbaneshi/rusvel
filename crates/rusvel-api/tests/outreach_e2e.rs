//! S-039: Outreach sequence end-to-end — contact + sequence + first step, approval job,
//! then send via mock email and follow-up [`JobKind::OutreachSend`] scheduled for step 2.
//!
//! Mirrors `rusvel-app` worker handling for [`JobKind::OutreachSend`] (draft → hold → approve → send + enqueue next).

mod common;

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use common::SessionAdapter;
use common::outreach::process_outreach_like_app_worker;
use gtm_engine::email::MockEmailAdapter;
use gtm_engine::{GtmEngine, SequenceStep};
use rusvel_core::domain::{
    AgentConfig, AgentOutput, AgentStatus, Contact, Content, JobFilter, JobKind, JobStatus,
    Session, SessionConfig, SessionKind,
};
use rusvel_core::error::Result;
use rusvel_core::id::{ContactId, RunId, SessionId};
use rusvel_core::ports::{AgentPort, EventPort, JobPort, SessionPort, StoragePort};
use rusvel_db::Database;
use rusvel_event::EventBus;

/// Returns short text for outreach draft generation.
struct OutreachStubAgent;

#[async_trait]
impl AgentPort for OutreachStubAgent {
    async fn create(&self, _: AgentConfig) -> Result<RunId> {
        Ok(RunId::new())
    }
    async fn run(&self, _: &RunId, _: Content) -> Result<AgentOutput> {
        Ok(AgentOutput {
            run_id: RunId::new(),
            content: Content::text("Hi — following up per our sequence."),
            tool_calls: 0,
            usage: rusvel_core::domain::LlmUsage::default(),
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

#[tokio::test]
async fn outreach_sequence_creates_approval_then_schedules_follow_up_with_mock_email() {
    let base = std::env::temp_dir().join(format!("rusvel-outreach-e2e-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&base).expect("temp dir");
    let db_path = base.join("rusvel.db");
    let db: Arc<Database> = Arc::new(Database::open(&db_path).expect("db"));
    let storage: Arc<dyn StoragePort> = db.clone();
    let jobs: Arc<dyn JobPort> = db.clone();
    let events: Arc<dyn EventPort> = Arc::new(EventBus::new(
        db.clone() as Arc<dyn rusvel_core::ports::EventStore>
    ));
    let sessions: Arc<dyn SessionPort> = Arc::new(SessionAdapter(storage.clone()));

    let agent: Arc<dyn AgentPort> = Arc::new(OutreachStubAgent);
    let engine = Arc::new(GtmEngine::new(
        storage.clone(),
        events.clone(),
        agent,
        jobs.clone(),
    ));

    let sid = SessionId::new();
    let now = Utc::now();
    sessions
        .create(Session {
            id: sid,
            name: "outreach-e2e".into(),
            kind: SessionKind::General,
            tags: vec![],
            config: SessionConfig::default(),
            created_at: now,
            updated_at: now,
            metadata: serde_json::json!({}),
        })
        .await
        .expect("session");

    let contact = Contact {
        id: ContactId::new(),
        session_id: sid,
        name: "Jane Smith".into(),
        emails: vec!["jane@example.com".into()],
        links: vec![],
        company: None,
        role: None,
        tags: vec![],
        last_contacted_at: None,
        metadata: serde_json::json!({}),
    };
    let contact_id = engine
        .crm()
        .add_contact(sid, contact)
        .await
        .expect("contact");

    let steps = vec![
        SequenceStep {
            delay_days: 0,
            channel: "email".into(),
            template: "intro".into(),
        },
        SequenceStep {
            delay_days: 0,
            channel: "email".into(),
            template: "nudge".into(),
        },
    ];
    let seq_id = engine
        .outreach()
        .create_sequence(sid, "Jane nurture".into(), steps)
        .await
        .expect("sequence");
    engine
        .outreach()
        .activate_sequence(&seq_id)
        .await
        .expect("activate");

    let first_job_id = engine
        .outreach()
        .execute_sequence(sid, seq_id, contact_id)
        .await
        .expect("execute_sequence");

    let job1 = jobs
        .dequeue(&[])
        .await
        .expect("dequeue")
        .expect("first OutreachSend job");
    assert_eq!(job1.kind, JobKind::OutreachSend);
    assert_eq!(job1.id, first_job_id);

    let mock_email = MockEmailAdapter::new();

    process_outreach_like_app_worker(
        engine.as_ref(),
        jobs.as_ref(),
        events.as_ref(),
        &mock_email,
        &job1,
    )
    .await
    .expect("draft + hold");

    let pending = jobs
        .list(JobFilter {
            session_id: None,
            kinds: vec![],
            statuses: vec![JobStatus::AwaitingApproval],
            limit: None,
        })
        .await
        .expect("list pending");
    assert_eq!(pending.len(), 1, "approval job should exist");
    assert_eq!(pending[0].id, first_job_id);

    jobs.approve(&first_job_id).await.expect("approve");

    let job_after = jobs
        .dequeue(&[])
        .await
        .expect("dequeue2")
        .expect("same job re-queued after approve");
    assert_eq!(job_after.id, first_job_id);

    process_outreach_like_app_worker(
        engine.as_ref(),
        jobs.as_ref(),
        events.as_ref(),
        &mock_email,
        &job_after,
    )
    .await
    .expect("send + enqueue next");

    assert_eq!(
        mock_email.sent_len(),
        1,
        "mock adapter should record one outbound email after approval"
    );

    let queued_followup = jobs
        .list(JobFilter {
            session_id: Some(sid),
            kinds: vec![JobKind::OutreachSend],
            statuses: vec![JobStatus::Queued],
            limit: None,
        })
        .await
        .expect("list queued");
    let step2 = queued_followup.iter().find(|j| {
        j.payload
            .get("step_index")
            .and_then(|v| v.as_u64())
            .map(|u| u == 1)
            .unwrap_or(false)
    });
    assert!(
        step2.is_some(),
        "follow-up OutreachSend for step 1 should be queued"
    );
}

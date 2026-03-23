//! Outreach sequence and follow-up management.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use rusvel_core::domain::{AgentConfig, Contact, Content, ObjectFilter};
use rusvel_core::error::Result;
use rusvel_core::id::{ContactId, SessionId};
use rusvel_core::ports::{AgentPort, JobPort, StoragePort};

// ── Local ID + domain types ────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SequenceId(Uuid);

impl Default for SequenceId {
    fn default() -> Self {
        Self::new()
    }
}

impl SequenceId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl std::fmt::Display for SequenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FollowUpId(Uuid);

impl Default for FollowUpId {
    fn default() -> Self {
        Self::new()
    }
}

impl FollowUpId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl std::fmt::Display for FollowUpId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SequenceStatus {
    Draft,
    Active,
    Paused,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceStep {
    pub delay_days: u32,
    pub channel: String,
    pub template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutreachSequence {
    pub id: SequenceId,
    pub session_id: SessionId,
    pub name: String,
    pub steps: Vec<SequenceStep>,
    pub status: SequenceStatus,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FollowUp {
    pub id: FollowUpId,
    pub contact_id: ContactId,
    pub session_id: SessionId,
    pub due_date: DateTime<Utc>,
    pub context: String,
    pub completed: bool,
    pub metadata: serde_json::Value,
}

// ── OutreachManager ────────────────────────────────────────────────

const KIND_SEQUENCE: &str = "outreach_sequence";
const KIND_FOLLOWUP: &str = "followup";

pub struct OutreachManager {
    storage: Arc<dyn StoragePort>,
    agent: Arc<dyn AgentPort>,
    #[allow(dead_code)]
    jobs: Arc<dyn JobPort>,
}

impl OutreachManager {
    pub fn new(
        storage: Arc<dyn StoragePort>,
        agent: Arc<dyn AgentPort>,
        jobs: Arc<dyn JobPort>,
    ) -> Self {
        Self {
            storage,
            agent,
            jobs,
        }
    }

    pub async fn create_sequence(
        &self,
        session_id: SessionId,
        name: String,
        steps: Vec<SequenceStep>,
    ) -> Result<SequenceId> {
        let seq = OutreachSequence {
            id: SequenceId::new(),
            session_id,
            name,
            steps,
            status: SequenceStatus::Draft,
            created_at: Utc::now(),
            metadata: serde_json::json!({}),
        };
        let id = seq.id;
        let json = serde_json::to_value(&seq)?;
        self.storage
            .objects()
            .put(KIND_SEQUENCE, &id.to_string(), json)
            .await?;
        Ok(id)
    }

    pub async fn list_sequences(&self, session_id: SessionId) -> Result<Vec<OutreachSequence>> {
        let filter = ObjectFilter {
            session_id: Some(session_id),
            ..Default::default()
        };
        let vals = self.storage.objects().list(KIND_SEQUENCE, filter).await?;
        vals.into_iter()
            .map(|v| Ok(serde_json::from_value(v)?))
            .collect()
    }

    pub async fn schedule_followup(
        &self,
        session_id: SessionId,
        contact_id: ContactId,
        due_date: DateTime<Utc>,
        context: String,
    ) -> Result<()> {
        let fu = FollowUp {
            id: FollowUpId::new(),
            contact_id,
            session_id,
            due_date,
            context,
            completed: false,
            metadata: serde_json::json!({}),
        };
        let json = serde_json::to_value(&fu)?;
        self.storage
            .objects()
            .put(KIND_FOLLOWUP, &fu.id.to_string(), json)
            .await
    }

    pub async fn list_followups(&self, session_id: SessionId) -> Result<Vec<FollowUp>> {
        let filter = ObjectFilter {
            session_id: Some(session_id),
            ..Default::default()
        };
        let vals = self.storage.objects().list(KIND_FOLLOWUP, filter).await?;
        vals.into_iter()
            .map(|v| Ok(serde_json::from_value(v)?))
            .collect()
    }

    pub async fn generate_message(
        &self,
        session_id: SessionId,
        contact: &Contact,
        context: &str,
    ) -> Result<String> {
        let prompt = format!(
            "Write a short, personalized outreach message for {} ({}).\n\
             Company: {}\nContext: {}",
            contact.name,
            contact.emails.first().unwrap_or(&String::new()),
            contact.company.as_deref().unwrap_or("unknown"),
            context,
        );
        let config = AgentConfig {
            profile_id: None,
            session_id,
            model: None,
            tools: vec![],
            instructions: Some("You are a sales outreach assistant.".into()),
            budget_limit: Some(0.05),
            metadata: serde_json::json!({}),
        };
        let run_id = self.agent.create(config).await?;
        let output = self.agent.run(&run_id, Content::text(prompt)).await?;
        let text = output
            .content
            .parts
            .into_iter()
            .filter_map(|p| match p {
                rusvel_core::domain::Part::Text(t) => Some(t),
                _ => None,
            })
            .collect::<String>();
        Ok(text)
    }
}

//! HTTP list for the central job queue (ADR-003).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use chrono::{DateTime, Utc};
use serde::Serialize;

use rusvel_core::domain::{Job, JobFilter, JobKind, JobStatus};
use rusvel_core::id::{JobId, SessionId};

use crate::AppState;

/// Wire shape for [`GET /api/jobs`]: `kind` and `status` are plain strings (same vocabulary as query filters).
#[derive(Debug, Serialize)]
pub struct JobListItem {
    pub id: JobId,
    pub session_id: SessionId,
    pub kind: String,
    pub status: String,
    pub payload: serde_json::Value,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retries: u32,
    pub max_retries: u32,
    pub error: Option<String>,
    pub metadata: serde_json::Value,
}

impl From<Job> for JobListItem {
    fn from(j: Job) -> Self {
        Self {
            id: j.id,
            session_id: j.session_id,
            kind: kind_to_wire(&j.kind),
            status: status_to_wire(&j.status),
            payload: j.payload,
            scheduled_at: j.scheduled_at,
            started_at: j.started_at,
            completed_at: j.completed_at,
            retries: j.retries,
            max_retries: j.max_retries,
            error: j.error,
            metadata: j.metadata,
        }
    }
}

fn kind_to_wire(k: &JobKind) -> String {
    match k {
        JobKind::AgentRun => "AgentRun".into(),
        JobKind::ContentPublish => "ContentPublish".into(),
        JobKind::OutreachSend => "OutreachSend".into(),
        JobKind::HarvestScan => "HarvestScan".into(),
        JobKind::CodeAnalyze => "CodeAnalyze".into(),
        JobKind::ProposalDraft => "ProposalDraft".into(),
        JobKind::ScheduledCron => "ScheduledCron".into(),
        JobKind::Custom(s) => format!("Custom:{s}"),
    }
}

fn status_to_wire(s: &JobStatus) -> String {
    match s {
        JobStatus::Queued => "Queued".into(),
        JobStatus::Running => "Running".into(),
        JobStatus::Succeeded => "Succeeded".into(),
        JobStatus::Failed => "Failed".into(),
        JobStatus::Cancelled => "Cancelled".into(),
        JobStatus::AwaitingApproval => "AwaitingApproval".into(),
    }
}

/// `GET /api/jobs` — list jobs with optional filters.
///
/// Query: `session_id` (UUID), `status` (comma-separated: Queued,Running,...),
/// `kinds` (comma-separated job kinds), `limit` (default 50, max 200).
/// Empty `status` / `kinds` means no filter on that axis.
pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<JobsQuery>,
) -> Result<Json<Vec<JobListItem>>, (StatusCode, String)> {
    let filter = build_filter(q).map_err(|e: String| (StatusCode::BAD_REQUEST, e))?;
    let jobs = state
        .jobs
        .list(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(jobs.into_iter().map(JobListItem::from).collect()))
}

#[derive(Debug, serde::Deserialize)]
pub struct JobsQuery {
    pub session_id: Option<String>,
    /// Comma-separated: Queued,Running,Succeeded,Failed,Cancelled,AwaitingApproval
    pub status: Option<String>,
    /// Comma-separated: AgentRun,ContentPublish,...,ProposalDraft,...
    pub kinds: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

fn build_filter(q: JobsQuery) -> Result<JobFilter, String> {
    let session_id = match q
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(s) => Some(
            s.parse::<uuid::Uuid>()
                .map(SessionId::from_uuid)
                .map_err(|_| "invalid session_id".to_string())?,
        ),
        None => None,
    };

    let statuses = parse_statuses(q.status.as_deref().unwrap_or(""))?;
    let kinds = parse_kinds(q.kinds.as_deref().unwrap_or(""))?;

    let limit = Some(q.limit.unwrap_or(50).min(200));

    Ok(JobFilter {
        session_id,
        kinds,
        statuses,
        limit,
    })
}

fn parse_statuses(s: &str) -> Result<Vec<JobStatus>, String> {
    let parts: Vec<_> = s
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return Ok(vec![]);
    }
    parts.into_iter().map(parse_one_status).collect()
}

fn parse_one_status(s: &str) -> Result<JobStatus, String> {
    match s {
        "Queued" => Ok(JobStatus::Queued),
        "Running" => Ok(JobStatus::Running),
        "Succeeded" => Ok(JobStatus::Succeeded),
        "Failed" => Ok(JobStatus::Failed),
        "Cancelled" => Ok(JobStatus::Cancelled),
        "AwaitingApproval" => Ok(JobStatus::AwaitingApproval),
        _ => Err(format!("unknown job status: {s}")),
    }
}

fn parse_kinds(s: &str) -> Result<Vec<JobKind>, String> {
    let parts: Vec<_> = s
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .collect();
    if parts.is_empty() {
        return Ok(vec![]);
    }
    parts.into_iter().map(parse_one_kind).collect()
}

fn parse_one_kind(s: &str) -> Result<JobKind, String> {
    match s {
        "AgentRun" => Ok(JobKind::AgentRun),
        "ContentPublish" => Ok(JobKind::ContentPublish),
        "OutreachSend" => Ok(JobKind::OutreachSend),
        "HarvestScan" => Ok(JobKind::HarvestScan),
        "CodeAnalyze" => Ok(JobKind::CodeAnalyze),
        "ProposalDraft" => Ok(JobKind::ProposalDraft),
        "ScheduledCron" => Ok(JobKind::ScheduledCron),
        _ if s.starts_with("Custom:") => {
            Ok(JobKind::Custom(s.trim_start_matches("Custom:").to_string()))
        }
        _ => Err(format!("unknown job kind: {s}")),
    }
}

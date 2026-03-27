//! HTTP list for the central job queue (ADR-003).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;

use rusvel_core::domain::{Job, JobFilter, JobKind, JobStatus};
use rusvel_core::id::SessionId;

use crate::AppState;

/// `GET /api/jobs` — list jobs with optional filters.
///
/// Query: `session_id` (UUID), `status` (comma-separated: Queued,Running,...),
/// `kinds` (comma-separated job kinds), `limit` (default 50, max 200).
/// Empty `status` / `kinds` means no filter on that axis.
pub async fn list_jobs(
    State(state): State<Arc<AppState>>,
    Query(q): Query<JobsQuery>,
) -> Result<Json<Vec<Job>>, (StatusCode, String)> {
    let filter = build_filter(q).map_err(|e: String| (StatusCode::BAD_REQUEST, e))?;
    let jobs = state
        .jobs
        .list(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(jobs))
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
    let session_id = match q.session_id.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
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
        _ if s.starts_with("Custom:") => Ok(JobKind::Custom(s.trim_start_matches("Custom:").to_string())),
        _ => Err(format!("unknown job kind: {s}")),
    }
}

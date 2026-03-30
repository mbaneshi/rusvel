//! Aggregated “active work” view (Cowork-style tasks + queue).

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::json;

use rusvel_core::domain::{Job, JobFilter, JobStatus};
use rusvel_core::id::SessionId;
use uuid::Uuid;

use crate::AppState;
use crate::jobs::JobListItem;

#[derive(Debug, Deserialize, Default)]
pub struct ActiveQuery {
    #[serde(default)]
    pub session_id: Option<String>,
}

/// `GET /api/dashboard/active` — in-flight jobs, pending approvals, cron schedules.
pub async fn active_dashboard(
    State(state): State<Arc<AppState>>,
    Query(q): Query<ActiveQuery>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let session_filter = if let Some(ref s) = q.session_id {
        Some(
            Uuid::parse_str(s.trim())
                .map(SessionId::from_uuid)
                .map_err(|_| (StatusCode::BAD_REQUEST, "invalid session_id".into()))?,
        )
    } else {
        None
    };

    let jobs = state
        .jobs
        .list(JobFilter {
            session_id: session_filter,
            kinds: vec![],
            statuses: vec![
                JobStatus::Queued,
                JobStatus::Running,
                JobStatus::AwaitingApproval,
            ],
            limit: Some(64),
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let pending = state
        .jobs
        .list(JobFilter {
            session_id: session_filter,
            kinds: vec![],
            statuses: vec![JobStatus::AwaitingApproval],
            limit: Some(32),
        })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let cron = state
        .cron_scheduler
        .list()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let job_items: Vec<JobListItem> = jobs.into_iter().map(Into::into).collect();
    let approval_items: Vec<Job> = pending;

    Ok(Json(json!({
        "jobs": job_items,
        "pending_approvals": approval_items,
        "cron_schedules": cron,
    })))
}

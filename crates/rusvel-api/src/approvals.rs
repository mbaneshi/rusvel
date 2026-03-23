//! Approvals — human-in-the-loop approval flow for jobs (ADR-008).
//!
//! Lists jobs in `AwaitingApproval` status and allows approving or rejecting them.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use rusvel_core::domain::{Job, JobFilter, JobStatus};
use rusvel_core::id::JobId;

use crate::AppState;

/// `GET /api/approvals` — list jobs awaiting human approval.
pub async fn list_pending(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Job>>, (StatusCode, String)> {
    let filter = JobFilter {
        session_id: None,
        kinds: vec![],
        statuses: vec![JobStatus::AwaitingApproval],
        limit: None,
    };

    let jobs = state
        .storage
        .jobs()
        .list(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(jobs))
}

/// `POST /api/approvals/{id}/approve` — approve a pending job.
pub async fn approve_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid job id".into()))?;
    let job_id = JobId::from_uuid(uuid);

    let mut job = state
        .storage
        .jobs()
        .get(&job_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "job not found".into()))?;

    if job.status != JobStatus::AwaitingApproval {
        return Err((
            StatusCode::CONFLICT,
            format!("job is not awaiting approval (status: {:?})", job.status),
        ));
    }

    job.status = JobStatus::Queued;
    state
        .storage
        .jobs()
        .update(&job)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

/// `POST /api/approvals/{id}/reject` — reject a pending job (sets status to Cancelled).
pub async fn reject_job(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let uuid = uuid::Uuid::parse_str(&id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid job id".into()))?;
    let job_id = JobId::from_uuid(uuid);

    let mut job = state
        .storage
        .jobs()
        .get(&job_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "job not found".into()))?;

    if job.status != JobStatus::AwaitingApproval {
        return Err((
            StatusCode::CONFLICT,
            format!("job is not awaiting approval (status: {:?})", job.status),
        ));
    }

    job.status = JobStatus::Cancelled;
    state
        .storage
        .jobs()
        .update(&job)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::OK)
}

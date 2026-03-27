//! CRUD for persisted cron schedules (`/api/cron/*`).

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use rusvel_core::error::RusvelError;
use rusvel_core::id::SessionId;
use serde::Deserialize;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct CreateCronBody {
    pub name: String,
    pub session_id: String,
    pub schedule: String,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub event_kind: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct UpdateCronBody {
    pub name: Option<String>,
    pub schedule: Option<String>,
    pub enabled: Option<bool>,
    pub payload: Option<Value>,
    pub event_kind: Option<String>,
}

/// `GET /api/cron` — list schedules (no secrets).
pub async fn list_schedules(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let list = state
        .cron_scheduler
        .list()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(list).unwrap()))
}

/// `POST /api/cron` — create schedule.
pub async fn create_schedule(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateCronBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let sid = Uuid::parse_str(body.session_id.trim())
        .map(SessionId::from_uuid)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid session_id".to_string()))?;

    let created = state
        .cron_scheduler
        .create(
            body.name,
            sid,
            body.schedule,
            body.payload,
            body.event_kind,
            body.enabled,
        )
        .await
        .map_err(|e| match e {
            RusvelError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    Ok(Json(serde_json::to_value(created).unwrap()))
}

/// `GET /api/cron/{id}` — get one schedule.
pub async fn get_schedule(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let row = state
        .cron_scheduler
        .get(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "cron schedule not found".into()))?;
    Ok(Json(serde_json::to_value(row).unwrap()))
}

/// `PUT /api/cron/{id}` — update fields.
pub async fn update_schedule(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(body): Json<UpdateCronBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let updated = state
        .cron_scheduler
        .update(
            &id,
            body.name,
            body.schedule,
            body.enabled,
            body.payload,
            body.event_kind,
        )
        .await
        .map_err(|e| match e {
            RusvelError::NotFound { .. } => (StatusCode::NOT_FOUND, e.to_string()),
            RusvelError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    Ok(Json(serde_json::to_value(updated).unwrap()))
}

/// `DELETE /api/cron/{id}` — remove schedule.
pub async fn delete_schedule(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, (StatusCode, String)> {
    state
        .cron_scheduler
        .delete(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({ "ok": true })))
}

/// `POST /api/cron/tick` — run one scheduler evaluation (tests / ops).
pub async fn tick_now(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    state
        .cron_scheduler
        .tick()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(json!({ "ok": true })))
}

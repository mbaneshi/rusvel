//! Incoming webhooks — list/create endpoints (authenticated) and receive (HMAC).

use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use rusvel_core::domain::{JobKind, NewJob};
use rusvel_core::error::RusvelError;
use rusvel_core::id::SessionId;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;

const SIGNATURE_HEADER: &str = "x-rusvel-signature";

/// Register a webhook with this `event_kind` to enqueue a forge pipeline job (body: `session_id`, optional `def`).
pub const FORGE_PIPELINE_WEBHOOK_KIND: &str = "forge.pipeline.requested";

#[derive(Debug, Deserialize)]
pub struct CreateWebhookBody {
    pub name: String,
    pub event_kind: String,
}

/// `GET /api/webhooks` — list registered webhooks (no secrets).
pub async fn list_webhooks(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let list = state
        .webhook_receiver
        .list()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(serde_json::to_value(list).unwrap()))
}

/// `POST /api/webhooks` — create endpoint; response includes `secret` once.
pub async fn create_webhook(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateWebhookBody>,
) -> Result<Json<Value>, (StatusCode, String)> {
    let created = state
        .webhook_receiver
        .create(body.name, body.event_kind)
        .await
        .map_err(|e| match e {
            RusvelError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;
    Ok(Json(serde_json::to_value(created).unwrap()))
}

fn signature_from_headers(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(SIGNATURE_HEADER)
        .or_else(|| headers.get("X-Rusvel-Signature"))
        .and_then(|v| v.to_str().ok())
}

/// `POST /api/webhooks/{id}` — receive payload; requires `X-Rusvel-Signature: sha256=<hex>`.
pub async fn receive_webhook(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Value>, (StatusCode, String)> {
    let sig = signature_from_headers(&headers);
    let outcome = state
        .webhook_receiver
        .receive(&id, body.as_ref(), sig)
        .await
        .map_err(|e| match e {
            RusvelError::NotFound { .. } => (StatusCode::NOT_FOUND, e.to_string()),
            RusvelError::Validation(_) | RusvelError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, e.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })?;

    if outcome.event_kind == FORGE_PIPELINE_WEBHOOK_KIND {
        let sid = outcome
            .body
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                (
                    StatusCode::BAD_REQUEST,
                    "body must include session_id (string)".into(),
                )
            })
            .and_then(|s| {
                uuid::Uuid::parse_str(s.trim())
                    .map(SessionId::from_uuid)
                    .map_err(|_| (StatusCode::BAD_REQUEST, "invalid session_id".into()))
            })?;
        let def = outcome.body.get("def").cloned().unwrap_or(Value::Null);
        state
            .jobs
            .enqueue(NewJob {
                session_id: sid,
                kind: JobKind::Custom("forge.pipeline".into()),
                payload: json!({ "def": def }),
                max_retries: 2,
                metadata: json!({
                    "source": "webhook",
                    "event_id": outcome.event_id.to_string(),
                }),
                scheduled_at: None,
            })
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(Json(json!({
        "ok": true,
        "event_id": outcome.event_id.to_string(),
    })))
}

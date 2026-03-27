//! Incoming webhooks — list/create endpoints (authenticated) and receive (HMAC).

use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use rusvel_core::error::RusvelError;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;

const SIGNATURE_HEADER: &str = "x-rusvel-signature";

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
    let event_id = state
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

    Ok(Json(json!({
        "ok": true,
        "event_id": event_id.to_string(),
    })))
}

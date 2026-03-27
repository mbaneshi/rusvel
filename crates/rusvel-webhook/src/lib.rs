//! Incoming webhooks: persist registrations in the object store, validate
//! `X-Rusvel-Signature: sha256=<hex>` (HMAC-SHA256 over the raw body), and emit
//! [`Event`](rusvel_core::domain::Event) records on the event bus.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use rusvel_core::domain::{Event, ObjectFilter};
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::EventId;
use rusvel_core::ports::{EventPort, ObjectStore, StoragePort};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::Sha256;
use subtle::ConstantTimeEq;

/// Object-store `kind` for [`WebhookRecord`] rows.
pub const WEBHOOK_OBJECT_KIND: &str = "webhook";

type HmacSha256 = Hmac<Sha256>;

/// Persisted webhook endpoint (includes secret for HMAC verification).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRecord {
    pub id: String,
    pub name: String,
    pub secret: String,
    pub event_kind: String,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub metadata: Value,
}

/// Response from [`WebhookReceiver::create`] — `secret` is shown once.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedWebhook {
    pub id: String,
    pub name: String,
    pub event_kind: String,
    pub secret: String,
}

/// Public summary (no secret) for [`WebhookReceiver::list`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookSummary {
    pub id: String,
    pub name: String,
    pub event_kind: String,
    pub created_at: DateTime<Utc>,
}

/// Result of a verified webhook receive: event persisted plus routing metadata for side effects.
#[derive(Debug, Clone)]
pub struct WebhookReceiveOutcome {
    pub event_id: EventId,
    pub event_kind: String,
    /// Parsed JSON body from the HTTP request (or string/null if not JSON).
    pub body: Value,
}

/// Maps webhook HTTP receives to event emissions (ADR-005: `Event.kind` is caller-defined).
#[derive(Clone)]
pub struct WebhookReceiver {
    storage: Arc<dyn StoragePort>,
    events: Arc<dyn EventPort>,
}

impl WebhookReceiver {
    pub fn new(storage: Arc<dyn StoragePort>, events: Arc<dyn EventPort>) -> Self {
        Self { storage, events }
    }

    fn objects(&self) -> &dyn ObjectStore {
        self.storage.objects()
    }

    /// Register a new endpoint; generates id (UUID v7) and a random hex secret.
    pub async fn create(&self, name: String, event_kind: String) -> Result<CreatedWebhook> {
        if name.trim().is_empty() {
            return Err(RusvelError::Validation("name must not be empty".into()));
        }
        if event_kind.trim().is_empty() {
            return Err(RusvelError::Validation(
                "event_kind must not be empty".into(),
            ));
        }

        let id = uuid::Uuid::now_v7().to_string();
        let secret = format!(
            "{}{}",
            uuid::Uuid::now_v7().simple(),
            uuid::Uuid::now_v7().simple()
        );
        let now = Utc::now();
        let record = WebhookRecord {
            id: id.clone(),
            name: name.clone(),
            secret: secret.clone(),
            event_kind: event_kind.clone(),
            created_at: now,
            metadata: json!({}),
        };

        self.objects()
            .put(
                WEBHOOK_OBJECT_KIND,
                &id,
                serde_json::to_value(&record)
                    .map_err(|e| RusvelError::Serialization(e.to_string()))?,
            )
            .await?;

        Ok(CreatedWebhook {
            id,
            name,
            event_kind,
            secret,
        })
    }

    pub async fn list(&self) -> Result<Vec<WebhookSummary>> {
        let rows = self
            .objects()
            .list(WEBHOOK_OBJECT_KIND, ObjectFilter::default())
            .await?;

        let mut out: Vec<WebhookSummary> = rows
            .into_iter()
            .filter_map(|v| serde_json::from_value::<WebhookRecord>(v).ok())
            .map(|r| WebhookSummary {
                id: r.id,
                name: r.name,
                event_kind: r.event_kind,
                created_at: r.created_at,
            })
            .collect();
        out.sort_by_key(|s| s.created_at);
        Ok(out)
    }

    /// Verify `X-Rusvel-Signature` (`sha256=<hex>`), parse JSON body when possible, emit event.
    pub async fn receive(
        &self,
        webhook_id: &str,
        body: &[u8],
        signature_header: Option<&str>,
    ) -> Result<WebhookReceiveOutcome> {
        let Some(header) = signature_header else {
            return Err(RusvelError::Validation(
                "missing X-Rusvel-Signature header (expected sha256=<hex>)".into(),
            ));
        };

        let raw = self
            .objects()
            .get(WEBHOOK_OBJECT_KIND, webhook_id)
            .await?
            .ok_or_else(|| RusvelError::NotFound {
                kind: WEBHOOK_OBJECT_KIND.into(),
                id: webhook_id.into(),
            })?;

        let record: WebhookRecord =
            serde_json::from_value(raw).map_err(|e| RusvelError::Serialization(e.to_string()))?;

        verify_hmac_sha256(record.secret.as_bytes(), body, header.trim())?;

        let body_value: Value = if body.is_empty() {
            json!(null)
        } else {
            serde_json::from_slice(body)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(body).into_owned()))
        };

        let event_kind = record.event_kind.clone();
        let event = Event {
            id: EventId::new(),
            session_id: None,
            run_id: None,
            source: "webhook".into(),
            kind: record.event_kind,
            payload: json!({
                "webhook_id": webhook_id,
                "body": body_value.clone(),
            }),
            created_at: Utc::now(),
            metadata: json!({ "webhook_id": webhook_id }),
        };

        let event_id = self.events.emit(event).await?;
        Ok(WebhookReceiveOutcome {
            event_id,
            event_kind,
            body: body_value,
        })
    }
}

/// HMAC-SHA256 over `body` using `secret`; `signature_header` must be `sha256=<64 hex chars>`.
pub fn verify_hmac_sha256(secret: &[u8], body: &[u8], signature_header: &str) -> Result<()> {
    let hex_sig = signature_header
        .strip_prefix("sha256=")
        .ok_or_else(|| RusvelError::Validation("signature must start with sha256=".into()))?;
    if hex_sig.len() != 64 {
        return Err(RusvelError::Validation(
            "signature must be sha256= followed by 64 hex characters".into(),
        ));
    }
    if !hex_sig.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(RusvelError::Validation("signature hex is malformed".into()));
    }

    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|_| RusvelError::Internal("invalid HMAC key length".into()))?;
    mac.update(body);
    let expected_hex = hex::encode(mac.finalize().into_bytes());

    if !bool::from(expected_hex.as_bytes().ct_eq(hex_sig.as_bytes())) {
        return Err(RusvelError::Unauthorized(
            "webhook HMAC signature mismatch".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_accepts_valid_signature() {
        let secret = b"test-secret";
        let body = br#"{"x":1}"#;
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(body);
        let sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        verify_hmac_sha256(secret, body, &sig).unwrap();
    }

    #[test]
    fn verify_rejects_tampered_body() {
        let secret = b"test-secret";
        let body = br#"{"x":1}"#;
        let mut mac = HmacSha256::new_from_slice(secret).unwrap();
        mac.update(body);
        let sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        let err = verify_hmac_sha256(secret, br#"{"x":2}"#, &sig).unwrap_err();
        assert!(matches!(err, RusvelError::Unauthorized(_)));
    }
}

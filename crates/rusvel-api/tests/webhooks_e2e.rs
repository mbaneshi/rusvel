//! Webhook registration, HMAC receive, and event persistence.

mod common;

use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use hmac::{Hmac, Mac};
use rusvel_core::id::EventId;
use serde_json::{Value, json};
use sha2::Sha256;
use tower::ServiceExt;

type HmacSha256 = Hmac<Sha256>;

#[tokio::test]
async fn webhook_hmac_emits_event() {
    let mut h = common::build_harness().await;

    let (status, bytes) = common::json_request(
        &mut h.router,
        "POST",
        "/api/webhooks",
        Some(json!({
            "name": "t",
            "event_kind": "webhook.e2e.ping"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    let wid = v["id"].as_str().unwrap();
    let secret = v["secret"].as_str().unwrap();

    let body = br#"{"x":1}"#;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(body);
    let sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/webhooks/{wid}"))
        .header("content-type", "application/json")
        .header("x-rusvel-signature", &sig)
        .body(Body::from(body.as_slice()))
        .unwrap();
    let res = h.router.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let out = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let ev: Value = serde_json::from_slice(&out).unwrap();
    let eid_str = ev["event_id"].as_str().unwrap();
    let eid = EventId::from_uuid(uuid::Uuid::parse_str(eid_str).unwrap());

    let stored = h.events.get(&eid).await.unwrap().expect("persisted");
    assert_eq!(stored.kind, "webhook.e2e.ping");
    assert_eq!(stored.source, "webhook");
}

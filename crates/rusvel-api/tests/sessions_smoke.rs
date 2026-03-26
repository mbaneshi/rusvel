mod common;

use axum::http::StatusCode;
use rusvel_core::domain::SessionKind;
use rusvel_core::domain::SessionSummary;
use serde_json::json;

use common::{build_harness, json_request};

#[tokio::test]
async fn get_sessions_includes_seed_session() {
    let mut h = build_harness().await;
    let (status, body) = json_request(&mut h.router, "GET", "/api/sessions", None).await;
    assert_eq!(status, StatusCode::OK);
    let list: Vec<SessionSummary> = serde_json::from_slice(&body).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, h.session_id);
}

#[tokio::test]
async fn post_sessions_creates_second_session() {
    let mut h = build_harness().await;
    let (status, body) = json_request(
        &mut h.router,
        "POST",
        "/api/sessions",
        Some(json!({
            "name": "e2e",
            "kind": "General"
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.get("id").is_some());

    let (st2, b2) = json_request(&mut h.router, "GET", "/api/sessions", None).await;
    assert_eq!(st2, StatusCode::OK);
    let list: Vec<SessionSummary> = serde_json::from_slice(&b2).unwrap();
    assert_eq!(list.len(), 2);
    assert!(list.iter().any(|s| s.name == "e2e"));
    assert!(list.iter().any(|s| s.kind == SessionKind::General));
}

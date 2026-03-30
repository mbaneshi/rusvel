mod common;

use axum::http::StatusCode;
use serde_json::json;

use common::{build_harness, json_request};

#[tokio::test]
async fn get_config_ok() {
    let mut h = build_harness().await;
    let (status, _) = json_request(&mut h.router, "GET", "/api/config", None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn get_config_models() {
    let mut h = build_harness().await;
    let (status, body) = json_request(&mut h.router, "GET", "/api/config/models", None).await;
    assert_eq!(status, StatusCode::OK);
    let models: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(models.is_array());
}

#[tokio::test]
async fn get_config_tools() {
    let mut h = build_harness().await;
    let (status, body) = json_request(&mut h.router, "GET", "/api/config/tools", None).await;
    assert_eq!(status, StatusCode::OK);
    let tools: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(tools.is_array());
}

#[tokio::test]
async fn update_chat_config_roundtrip() {
    let mut h = build_harness().await;

    // Update with full ChatConfig shape
    let (status, _) = json_request(
        &mut h.router,
        "PUT",
        "/api/config",
        Some(json!({
            "model": "claude/sonnet",
            "effort": "medium",
            "permission_mode": "auto",
            "allowed_tools": [],
            "disallowed_tools": []
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Verify
    let (st2, b2) = json_request(&mut h.router, "GET", "/api/config", None).await;
    assert_eq!(st2, StatusCode::OK);
    let cfg: serde_json::Value = serde_json::from_slice(&b2).unwrap();
    assert_eq!(cfg["model"], "claude/sonnet");
}

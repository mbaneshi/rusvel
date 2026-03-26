mod common;

use axum::http::StatusCode;
use common::{build_harness, json_request};

#[tokio::test]
async fn get_api_health_ok() {
    let mut h = build_harness().await;
    let (status, body) = json_request(&mut h.router, "GET", "/api/health", None).await;
    assert_eq!(status, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["status"], "ok");
}

#[tokio::test]
async fn get_system_status_ok() {
    let mut h = build_harness().await;
    let (status, body) = json_request(&mut h.router, "GET", "/api/system/status", None).await;
    assert_eq!(status, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.is_object());
}

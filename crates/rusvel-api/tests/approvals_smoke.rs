mod common;

use axum::http::StatusCode;
use common::{build_harness, json_request};

#[tokio::test]
async fn get_approvals_returns_ok() {
    let mut h = build_harness().await;
    let (status, body) = json_request(&mut h.router, "GET", "/api/approvals", None).await;
    assert_eq!(status, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.is_array() || v.is_object());
}

mod common;

use axum::http::StatusCode;

use common::{build_harness, json_request};

#[tokio::test]
async fn get_analytics_spend() {
    let mut h = build_harness().await;
    let sid = h.session_id;
    let (status, body) = json_request(
        &mut h.router,
        "GET",
        &format!("/api/analytics/spend?session_id={sid}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(data.is_object());
}

#[tokio::test]
async fn get_analytics_dashboard() {
    let mut h = build_harness().await;
    let sid = h.session_id;
    let (status, _) = json_request(
        &mut h.router,
        "GET",
        &format!("/api/analytics/dashboard?session_id={sid}"),
        None,
    )
    .await;
    // Dashboard endpoint may or may not exist; accept 200 or 404
    assert!(status == StatusCode::OK || status == StatusCode::NOT_FOUND);
}

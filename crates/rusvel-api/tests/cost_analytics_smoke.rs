mod common;

use axum::http::StatusCode;

use common::{build_harness, json_request};

#[tokio::test]
async fn get_costs_returns_empty_array() {
    let mut h = build_harness().await;
    let (status, body) = json_request(&mut h.router, "GET", "/api/analytics/costs", None).await;
    assert_eq!(status, StatusCode::OK);
    let data: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(data.is_empty());
}

#[tokio::test]
async fn get_costs_summary_by_department_ok() {
    let mut h = build_harness().await;
    let (status, body) = json_request(
        &mut h.router,
        "GET",
        "/api/analytics/costs/summary?group_by=department",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let data: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert!(data.is_empty() || data[0].get("key").is_some());
}

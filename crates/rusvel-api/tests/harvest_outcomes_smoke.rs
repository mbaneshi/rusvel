//! S-044: harvest outcomes list + record API.

mod common;

use axum::http::StatusCode;
use common::{build_harness_with_gtm, json_request};
use serde_json::json;

#[tokio::test]
async fn get_harvest_outcomes_empty_array() {
    let mut h = build_harness_with_gtm().await;
    let sid = h.session_id.to_string();
    let uri = format!("/api/dept/harvest/outcomes?session_id={sid}");
    let (st, body) = json_request(&mut h.router, "GET", &uri, None).await;
    assert_eq!(st, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.is_array());
    assert_eq!(v.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn post_harvest_outcome_404_unknown_opportunity() {
    let mut h = build_harness_with_gtm().await;
    let sid = h.session_id.to_string();
    let (st, _) = json_request(
        &mut h.router,
        "POST",
        "/api/dept/harvest/outcome",
        Some(json!({
            "session_id": sid,
            "opportunity_id": "00000000-0000-0000-0000-000000000099",
            "result": "lost",
            "notes": "test"
        })),
    )
    .await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

//! S-043 / S-042: cached brief 404 + cross-engine flow template JSON.

mod common;

use axum::http::StatusCode;
use common::{build_harness_with_gtm, json_request};

#[tokio::test]
async fn get_brief_latest_404_without_prior_generate() {
    let mut h = build_harness_with_gtm().await;
    let sid = h.session_id.to_string();
    let uri = format!("/api/brief/latest?session_id={sid}");
    let (st, _) = json_request(&mut h.router, "GET", &uri, None).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_flow_cross_engine_template_ok() {
    let mut h = build_harness_with_gtm().await;
    let (st, body) = json_request(
        &mut h.router,
        "GET",
        "/api/flows/templates/cross-engine-handoff",
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(
        v["metadata"]["template"].as_str(),
        Some("cross_engine_handoff")
    );
    assert!(v["nodes"].as_array().map(|a| a.len() >= 2).unwrap_or(false));
}

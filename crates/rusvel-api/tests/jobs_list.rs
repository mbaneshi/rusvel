//! GET /api/jobs — filtered job list.

mod common;

use axum::http::StatusCode;
use common::{build_harness, json_request};
use rusvel_core::domain::{JobKind, NewJob};
use serde_json::json;

#[tokio::test]
async fn get_jobs_empty_ok() {
    let mut h = build_harness().await;
    let sid = h.session_id;
    let (status, body) = json_request(
        &mut h.router,
        "GET",
        &format!("/api/jobs?session_id={sid}"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.as_array().is_some());
}

#[tokio::test]
async fn get_jobs_bad_status_400() {
    let mut h = build_harness().await;
    let (status, _) = json_request(
        &mut h.router,
        "GET",
        "/api/jobs?status=NotARealStatus",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_jobs_filters_by_session_and_kind() {
    let mut h = build_harness().await;
    let sid = h.session_id;

    let job_id = h
        .jobs
        .enqueue(NewJob {
            session_id: sid,
            kind: JobKind::ProposalDraft,
            payload: json!({
                "opportunity_id": "opp-1",
                "profile": "default",
            }),
            max_retries: 3,
            metadata: json!({}),
            scheduled_at: None,
        })
        .await
        .unwrap();

    let uri = format!("/api/jobs?session_id={sid}&kinds=ProposalDraft&status=Queued&limit=10");
    let (status, body) = json_request(&mut h.router, "GET", &uri, None).await;
    assert_eq!(status, StatusCode::OK);
    let arr: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"].as_str().unwrap(), job_id.to_string());
    assert_eq!(arr[0]["status"].as_str().unwrap(), "Queued");
    assert_eq!(arr[0]["kind"].as_str().unwrap(), "ProposalDraft");
}

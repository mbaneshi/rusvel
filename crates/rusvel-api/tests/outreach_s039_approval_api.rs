//! S-039: HTTP surface — POST `/api/dept/gtm/outreach/execute` then worker hold → `GET /api/approvals`.

mod common;

use axum::http::StatusCode;
use common::outreach::process_outreach_like_app_worker;
use common::{build_harness_with_gtm, json_request};
use gtm_engine::SequenceStep;
use gtm_engine::email::MockEmailAdapter;
use rusvel_core::domain::JobKind;
use serde_json::json;

#[tokio::test]
async fn s039_outreach_execute_surfaces_on_approvals_api_after_draft_hold() {
    let mut h = build_harness_with_gtm().await;
    let sid = h.session_id;
    let ge = h.gtm_engine.as_ref().expect("gtm");

    let (st, contact_bytes) = json_request(
        &mut h.router,
        "POST",
        "/api/dept/gtm/contacts",
        Some(json!({
            "session_id": sid.to_string(),
            "name": "Jane Smith",
            "email": "jane@example.com",
        })),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let contact_json: serde_json::Value = serde_json::from_slice(&contact_bytes).unwrap();
    let contact_id = contact_json["id"].as_str().unwrap();

    let steps = vec![
        SequenceStep {
            delay_days: 0,
            channel: "email".into(),
            template: "intro".into(),
        },
        SequenceStep {
            delay_days: 1,
            channel: "email".into(),
            template: "followup".into(),
        },
    ];
    let seq_id = ge
        .outreach()
        .create_sequence(sid, "S-039 sequence".into(), steps)
        .await
        .unwrap();
    ge.outreach().activate_sequence(&seq_id).await.unwrap();

    let (status, body) = json_request(
        &mut h.router,
        "POST",
        "/api/dept/gtm/outreach/execute",
        Some(json!({
            "session_id": sid.to_string(),
            "sequence_id": seq_id.to_string(),
            "contact_id": contact_id,
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let job_id_str = v["job_id"].as_str().expect("job_id");
    assert_eq!(v["count"], json!(1));

    let job1 = h
        .jobs
        .dequeue(&[])
        .await
        .expect("dequeue")
        .expect("OutreachSend job");
    assert_eq!(job1.kind, JobKind::OutreachSend);
    assert_eq!(job1.id.to_string(), job_id_str);

    let mock_email = MockEmailAdapter::new();
    process_outreach_like_app_worker(
        ge.as_ref(),
        h.jobs.as_ref(),
        h.events.as_ref(),
        &mock_email,
        &job1,
    )
    .await
    .expect("draft → hold_for_approval");

    let (ap_st, ap_bytes) = json_request(&mut h.router, "GET", "/api/approvals", None).await;
    assert_eq!(ap_st, StatusCode::OK);
    let pending: Vec<serde_json::Value> = serde_json::from_slice(&ap_bytes).unwrap();
    assert_eq!(
        pending.len(),
        1,
        "approvals API should list the held outreach job"
    );
    assert_eq!(
        pending[0]["id"].as_str().unwrap(),
        job_id_str,
        "pending job id should match execute response"
    );
    assert_eq!(pending[0]["status"], json!("AwaitingApproval"));
}

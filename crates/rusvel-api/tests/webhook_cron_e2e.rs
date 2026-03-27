//! S-046: cron tick enqueues `ScheduledCron` jobs (webhook HMAC coverage: `webhooks_e2e.rs`).

mod common;

use std::time::Duration;

use axum::http::StatusCode;
use common::{build_harness, json_request};
use serde_json::json;

#[tokio::test]
async fn cron_tick_enqueues_scheduled_cron_job() {
    let mut h = build_harness().await;
    let sid = h.session_id;

    let (st, body) = json_request(
        &mut h.router,
        "POST",
        "/api/cron",
        Some(json!({
            "name": "s046 every second",
            "session_id": sid.to_string(),
            "schedule": "*/1 * * * * * *",
            "payload": { "s046": true },
            "event_kind": "s046.cron.fired",
            "enabled": true,
        })),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let schedule_id = created["id"].as_str().unwrap().to_string();

    let mut found = false;
    for _ in 0..30 {
        let (tick_st, _) = json_request(&mut h.router, "POST", "/api/cron/tick", None).await;
        assert_eq!(tick_st, StatusCode::OK);

        let uri = format!(
            "/api/jobs?session_id={sid}&kinds=ScheduledCron&status=Queued&limit=20"
        );
        let (st2, list_bytes) = json_request(&mut h.router, "GET", &uri, None).await;
        assert_eq!(st2, StatusCode::OK);
        let arr: Vec<serde_json::Value> = serde_json::from_slice(&list_bytes).unwrap();
        if arr.iter().any(|row| {
            row["kind"].as_str() == Some("ScheduledCron")
                && row["payload"]["schedule_id"].as_str() == Some(&schedule_id)
        }) {
            found = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(15)).await;
    }

    assert!(
        found,
        "expected POST /api/cron/tick to enqueue ScheduledCron for schedule {schedule_id}"
    );
}

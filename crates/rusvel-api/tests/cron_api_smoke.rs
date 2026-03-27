//! S-041: `/api/cron` CRUD smoke (list + create with session).

mod common;

use axum::http::StatusCode;
use common::{build_harness_with_gtm, json_request};
use serde_json::json;

#[tokio::test]
async fn get_cron_list_ok() {
    let mut h = build_harness_with_gtm().await;
    let (st, body) = json_request(&mut h.router, "GET", "/api/cron", None).await;
    assert_eq!(st, StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v.is_array());
}

#[tokio::test]
async fn post_cron_create_hourly_schedule() {
    let mut h = build_harness_with_gtm().await;
    let sid = h.session_id.to_string();
    let (st, body) = json_request(
        &mut h.router,
        "POST",
        "/api/cron",
        Some(json!({
            "name": "smoke hourly",
            "session_id": sid,
            "schedule": "hourly",
            "payload": { "x": 1 },
            "event_kind": "cron.smoke.test",
            "enabled": true
        })),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(v["id"].as_str().is_some());
    assert_eq!(v["name"], json!("smoke hourly"));
}

#[tokio::test]
async fn post_cron_tick_ok() {
    let mut h = build_harness_with_gtm().await;
    let (st, body) = json_request(&mut h.router, "POST", "/api/cron/tick", None).await;
    assert_eq!(st, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["ok"], json!(true));
}

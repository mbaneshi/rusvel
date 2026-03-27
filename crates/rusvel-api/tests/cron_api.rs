//! S-041 / S-046: `/api/cron` CRUD + manual tick (enqueue timing is covered in `rusvel-cron`).

mod common;

use axum::http::StatusCode;
use common::{build_harness_with_gtm, json_request};
use serde_json::json;

#[tokio::test]
async fn cron_crud_and_tick_ok() {
    let mut h = build_harness_with_gtm().await;
    let sid = h.session_id;

    let (st, body) = json_request(
        &mut h.router,
        "POST",
        "/api/cron",
        Some(json!({
            "name": "every second (test)",
            "session_id": sid.to_string(),
            "schedule": "*/1 * * * * * *",
            "payload": { "hello": "world" },
            "event_kind": "test.cron.tick",
            "enabled": true,
        })),
    )
    .await;
    assert_eq!(st, StatusCode::OK);
    let created: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let id = created["id"].as_str().unwrap();

    let (st2, list_bytes) = json_request(&mut h.router, "GET", "/api/cron", None).await;
    assert_eq!(st2, StatusCode::OK);
    let list: Vec<serde_json::Value> = serde_json::from_slice(&list_bytes).unwrap();
    assert!(list.iter().any(|row| row["id"].as_str() == Some(id)));

    let (stg, one) = json_request(&mut h.router, "GET", &format!("/api/cron/{id}"), None).await;
    assert_eq!(stg, StatusCode::OK);
    let row: serde_json::Value = serde_json::from_slice(&one).unwrap();
    assert_eq!(row["name"], "every second (test)");

    let (stu, _) = json_request(
        &mut h.router,
        "PUT",
        &format!("/api/cron/{id}"),
        Some(json!({ "enabled": false })),
    )
    .await;
    assert_eq!(stu, StatusCode::OK);

    let (st3, _) = json_request(&mut h.router, "POST", "/api/cron/tick", None).await;
    assert_eq!(st3, StatusCode::OK);

    let (std, _) = json_request(&mut h.router, "DELETE", &format!("/api/cron/{id}"), None).await;
    assert_eq!(std, StatusCode::OK);
}

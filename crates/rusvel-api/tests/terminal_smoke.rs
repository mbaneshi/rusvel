mod common;

use axum::http::StatusCode;

use common::{build_harness, json_request};

#[tokio::test]
async fn terminal_dept_pane_without_terminal_port() {
    let mut h = build_harness().await;
    let sid = h.session_id;
    let (status, _) = json_request(
        &mut h.router,
        "GET",
        &format!("/api/terminal/dept/forge?session_id={sid}"),
        None,
    )
    .await;
    // Terminal port is None in test harness, should return 503
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn terminal_resize_without_terminal_port() {
    let mut h = build_harness().await;
    let (status, _) = json_request(
        &mut h.router,
        "POST",
        "/api/terminal/pane/00000000-0000-0000-0000-000000000000/resize",
        Some(serde_json::json!({"rows": 24, "cols": 80})),
    )
    .await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
}

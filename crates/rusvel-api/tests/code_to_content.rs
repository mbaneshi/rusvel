//! Code → content → approve → publish (mock platform), no real LLM or external HTTP.

mod common;

use std::path::PathBuf;

use axum::http::StatusCode;
use rusvel_core::domain::{ApprovalStatus, ContentItem, ContentKind};
use serde_json::{Value, json};

use common::{build_harness, json_request};

#[tokio::test]
async fn post_from_code_creates_items_for_kinds() {
    let mut h = build_harness().await;
    let code_src: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../code-engine/src");

    let (status, body) = json_request(
        &mut h.router,
        "POST",
        "/api/dept/content/from-code",
        Some(json!({
            "session_id": h.session_id.to_string(),
            "path": code_src.to_string_lossy(),
            "kinds": ["LinkedInPost", "Thread"]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let items: Vec<ContentItem> = serde_json::from_slice(&body).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].kind, ContentKind::LinkedInPost);
    assert_eq!(items[1].kind, ContentKind::Thread);
}

#[tokio::test]
async fn patch_approve_sets_approval_approved() {
    let mut h = build_harness().await;
    let code_src: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../code-engine/src");

    let (_st, b) = json_request(
        &mut h.router,
        "POST",
        "/api/dept/content/from-code",
        Some(json!({
            "session_id": h.session_id.to_string(),
            "path": code_src.to_string_lossy(),
            "kinds": ["Blog"]
        })),
    )
    .await;
    let items: Vec<ContentItem> = serde_json::from_slice(&b).unwrap();
    let id = items[0].id.to_string();

    let (status, body) = json_request(
        &mut h.router,
        "PATCH",
        &format!("/api/dept/content/{id}/approve"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item: ContentItem = serde_json::from_slice(&body).unwrap();
    assert_eq!(item.approval, ApprovalStatus::Approved);
}

#[tokio::test]
async fn publish_uses_mock_platform_after_approve() {
    let mut h = build_harness().await;
    let code_src: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../code-engine/src");

    let (_st, b) = json_request(
        &mut h.router,
        "POST",
        "/api/dept/content/from-code",
        Some(json!({
            "session_id": h.session_id.to_string(),
            "path": code_src.to_string_lossy(),
            "kinds": ["Tweet"]
        })),
    )
    .await;
    let items: Vec<ContentItem> = serde_json::from_slice(&b).unwrap();
    let id = items[0].id.to_string();

    let (st, _) = json_request(
        &mut h.router,
        "PATCH",
        &format!("/api/dept/content/{id}/approve"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    let (pub_st, _) = json_request(
        &mut h.router,
        "POST",
        "/api/dept/content/publish",
        Some(json!({
            "session_id": h.session_id.to_string(),
            "content_id": id,
            "platform": "Twitter"
        })),
    )
    .await;
    assert_eq!(pub_st, StatusCode::OK);
    assert_eq!(h.mock_twitter.published_items().len(), 1);
}

#[tokio::test]
async fn get_db_tables_lists_user_tables() {
    let mut h = build_harness().await;
    let (status, body) = json_request(&mut h.router, "GET", "/api/db/tables", None).await;
    assert_eq!(status, StatusCode::OK);
    let tables: Vec<Value> = serde_json::from_slice(&body).expect("json array");
    let names: Vec<&str> = tables
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.iter().any(|n| *n == "sessions" || *n == "events"),
        "expected core tables, got {names:?}"
    );
}

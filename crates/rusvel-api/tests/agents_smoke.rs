mod common;

use axum::http::StatusCode;
use rusvel_core::domain::AgentProfile;
use serde_json::json;

use common::{build_harness, json_request};

#[tokio::test]
async fn post_get_agent_roundtrip() {
    let mut h = build_harness().await;
    let (status, body) = json_request(
        &mut h.router,
        "POST",
        "/api/agents",
        Some(json!({
            "name": "lint-bot",
            "role": "reviewer",
            "metadata": {"engine": "code"}
        })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    let created: AgentProfile = serde_json::from_slice(&body).unwrap();
    let id = created.id.to_string();

    let (st2, b2) = json_request(&mut h.router, "GET", &format!("/api/agents/{id}"), None).await;
    assert_eq!(st2, StatusCode::OK);
    let got: AgentProfile = serde_json::from_slice(&b2).unwrap();
    assert_eq!(got.name, "lint-bot");

    let (st3, b3) = json_request(&mut h.router, "GET", "/api/agents?engine=code", None).await;
    assert_eq!(st3, StatusCode::OK);
    let list: Vec<AgentProfile> = serde_json::from_slice(&b3).unwrap();
    assert!(list.iter().any(|a| a.name == "lint-bot"));
}

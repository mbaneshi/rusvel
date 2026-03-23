//! Analytics endpoint — aggregate counts across all ObjectStore kinds.
//!
//! `GET /api/analytics` returns a JSON object with counts of agents, skills,
//! rules, MCP servers, hooks, conversations, events, and departments.

use std::collections::HashSet;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;

use rusvel_core::domain::{EventFilter, ObjectFilter};

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct AnalyticsResponse {
    pub agents: usize,
    pub skills: usize,
    pub rules: usize,
    pub mcp_servers: usize,
    pub hooks: usize,
    pub conversations: usize,
    pub events: usize,
    pub departments: usize,
}

/// `GET /api/analytics` — return aggregate counts.
pub async fn get_analytics(
    State(state): State<Arc<AppState>>,
) -> Result<Json<AnalyticsResponse>, (StatusCode, String)> {
    let objects = state.storage.objects();
    let filter = ObjectFilter::default();

    let agents = objects.list("agents", filter.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    let skills = objects.list("skills", filter.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    let rules = objects.list("rules", filter.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    let mcp_servers = objects.list("mcp_servers", filter.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    let hooks = objects.list("hooks", filter.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    // Count unique conversations from chat_message entries
    let chat_messages = objects.list("chat_message", filter.clone()).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let conversations: usize = chat_messages.iter()
        .filter_map(|v| v.get("conversation_id").and_then(|c| c.as_str()))
        .collect::<HashSet<_>>()
        .len();

    // Count events via EventPort
    let events = state.events
        .query(EventFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    Ok(Json(AnalyticsResponse {
        agents,
        skills,
        rules,
        mcp_servers,
        hooks,
        conversations,
        events,
        departments: 5, // Code, Content, Harvest, GTM, Forge
    }))
}

//! Analytics endpoint — aggregate counts across all `ObjectStore` kinds.
//!
//! `GET /api/analytics` returns a JSON object with counts of agents, skills,
//! rules, MCP servers, hooks, conversations, events, and departments.
//!
//! `GET /api/analytics/spend` returns LLM spend from [`MetricStore`] (`llm.cost_usd`),
//! optionally filtered by department and scoped to a session for budget warnings.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use rusvel_core::domain::{EventFilter, MetricFilter, ObjectFilter};
use rusvel_core::id::SessionId;
use rusvel_core::ports::MetricStore;
use rusvel_llm::cost::{aggregate_spend, LLM_COST_METRIC_NAME};
use uuid::Uuid;

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

    let agents = objects
        .list("agents", filter.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    let skills = objects
        .list("skills", filter.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    let rules = objects
        .list("rules", filter.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    let mcp_servers = objects
        .list("mcp_servers", filter.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    let hooks = objects
        .list("hooks", filter.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .len();

    // Count unique conversations from chat_message entries
    let chat_messages = objects
        .list("chat_message", filter.clone())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let conversations: usize = chat_messages
        .iter()
        .filter_map(|v| v.get("conversation_id").and_then(|c| c.as_str()))
        .collect::<HashSet<_>>()
        .len();

    // Count events via EventPort
    let events = state
        .events
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

// ── Spend (S-035) ─────────────────────────────────────────────

const BUDGET_WARN_RATIO: f64 = 0.8;

#[derive(Debug, Deserialize)]
pub struct SpendQuery {
    /// Filter totals to this department id (e.g. `harvest`, `content`). Omit for all departments.
    pub dept: Option<String>,
    /// When set, include session budget from [`Session::config`] and session-wide spend for warnings.
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SpendResponse {
    /// Sum of `llm.cost_usd` for the selected department (or all departments if `dept` omitted).
    pub total_usd: f64,
    /// Aggregated spend per `dept:` tag (from metrics).
    pub by_department: HashMap<String, f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// Total LLM spend attributed to this session (any department), when `session_id` is set.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_total_usd: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_budget_limit_usd: Option<f64>,
    /// True when session spend is at or above 80% of the configured budget.
    pub budget_warning: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_usage_ratio: Option<f64>,
}

/// `GET /api/analytics/spend` — LLM spend breakdown by department; optional session budget warning.
pub async fn get_spend(
    State(state): State<Arc<AppState>>,
    Query(q): Query<SpendQuery>,
) -> Result<Json<SpendResponse>, (StatusCode, String)> {
    let filter = MetricFilter {
        name: Some(LLM_COST_METRIC_NAME.into()),
        limit: Some(50_000),
        ..Default::default()
    };
    let points = state
        .database
        .query(filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let agg = aggregate_spend(&points);
    let by_department = agg.by_department;
    let session_totals = agg.by_session;

    let total_usd = if let Some(ref d) = q.dept {
        *by_department.get(d.as_str()).unwrap_or(&0.0)
    } else {
        agg.total_usd
    };

    let mut session_id_out: Option<String> = None;
    let mut session_total_usd: Option<f64> = None;
    let mut session_budget_limit_usd: Option<f64> = None;
    let mut budget_warning = false;
    let mut budget_usage_ratio: Option<f64> = None;

    if let Some(ref sid_str) = q.session_id {
        if let Ok(uuid) = Uuid::parse_str(sid_str) {
            let sid = SessionId::from(uuid);
            session_id_out = Some(sid_str.clone());
            session_total_usd = session_totals.get(&sid.to_string()).copied().or(Some(0.0));

            if let Ok(session) = state.sessions.load(&sid).await {
                session_budget_limit_usd = session.config.budget_limit;
            }

            if let (Some(spent), Some(limit)) = (session_total_usd, session_budget_limit_usd) {
                if limit > 0.0 {
                    let ratio = spent / limit;
                    budget_usage_ratio = Some(ratio);
                    if spent >= limit * BUDGET_WARN_RATIO {
                        budget_warning = true;
                    }
                }
            }
        }
    }

    Ok(Json(SpendResponse {
        total_usd,
        by_department,
        session_id: session_id_out,
        session_total_usd,
        session_budget_limit_usd,
        budget_warning,
        budget_usage_ratio,
    }))
}

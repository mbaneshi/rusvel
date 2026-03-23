//! Agents CRUD — manage `AgentProfile` definitions stored in `ObjectStore`.
//!
//! Agents are scoped to departments via `metadata.engine` field.
//! Global agents (no engine in metadata) appear in all departments.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use rusvel_core::domain::{AgentProfile, ModelProvider, ModelRef, ObjectFilter};
use rusvel_core::id::AgentProfileId;

use crate::AppState;

const STORE_KIND: &str = "agents";

#[derive(Debug, Deserialize)]
pub struct AgentQuery {
    pub engine: Option<String>,
}

/// `GET /api/agents?engine=code` — list agents, optionally filtered by department.
pub async fn list_agents(
    State(state): State<Arc<AppState>>,
    Query(params): Query<AgentQuery>,
) -> Result<Json<Vec<AgentProfile>>, (StatusCode, String)> {
    let all = state
        .storage
        .objects()
        .list(STORE_KIND, ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut agents: Vec<AgentProfile> = all
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    if let Some(ref engine) = params.engine {
        agents.retain(|a| {
            let agent_engine = a.metadata.get("engine").and_then(|e| e.as_str());
            agent_engine == Some(engine) || agent_engine.is_none() // include global
        });
    }

    Ok(Json(agents))
}

/// `POST /api/agents` — create a new agent.
pub async fn create_agent(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateAgentBody>,
) -> Result<(StatusCode, Json<AgentProfile>), (StatusCode, String)> {
    let agent = AgentProfile {
        id: AgentProfileId::new(),
        name: body.name,
        role: body.role.unwrap_or_default(),
        instructions: body.instructions.unwrap_or_default(),
        default_model: ModelRef {
            provider: ModelProvider::Claude,
            model: body.model.unwrap_or_else(|| "sonnet".into()),
        },
        allowed_tools: body.allowed_tools.unwrap_or_default(),
        capabilities: vec![],
        budget_limit: body.budget_limit,
        metadata: body.metadata.unwrap_or_else(|| serde_json::json!({})),
    };

    state
        .storage
        .objects()
        .put(
            STORE_KIND,
            &agent.id.to_string(),
            serde_json::to_value(&agent).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(agent)))
}

/// `GET /api/agents/{id}` — get a single agent.
pub async fn get_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<AgentProfile>, (StatusCode, String)> {
    let val = state
        .storage
        .objects()
        .get(STORE_KIND, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "agent not found".into()))?;

    let agent: AgentProfile = serde_json::from_value(val)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agent))
}

/// `PUT /api/agents/{id}` — update an agent.
pub async fn update_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(agent): Json<AgentProfile>,
) -> Result<Json<AgentProfile>, (StatusCode, String)> {
    state
        .storage
        .objects()
        .put(
            STORE_KIND,
            &id,
            serde_json::to_value(&agent).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(agent))
}

/// `DELETE /api/agents/{id}` — delete an agent.
pub async fn delete_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state
        .storage
        .objects()
        .delete(STORE_KIND, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ── Request body ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateAgentBody {
    pub name: String,
    pub role: Option<String>,
    pub instructions: Option<String>,
    pub model: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub budget_limit: Option<f64>,
    pub metadata: Option<serde_json::Value>,
}

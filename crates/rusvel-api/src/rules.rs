//! Rules CRUD — instructions injected into department system prompts.
//!
//! Rules are scoped to departments via `metadata.engine` field.
//! Enabled rules get appended to the system prompt before each chat call.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::AppState;

const STORE_KIND: &str = "rules";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub id: String,
    pub name: String,
    pub content: String,
    pub enabled: bool,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct RuleQuery {
    pub engine: Option<String>,
}

pub async fn list_rules(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RuleQuery>,
) -> Result<Json<Vec<RuleDefinition>>, (StatusCode, String)> {
    let all = state
        .storage
        .objects()
        .list(STORE_KIND, rusvel_core::domain::ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut rules: Vec<RuleDefinition> = all
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    if let Some(ref engine) = params.engine {
        rules.retain(|r| {
            let rule_engine = r.metadata.get("engine").and_then(|e| e.as_str());
            rule_engine == Some(engine) || rule_engine.is_none()
        });
    }

    Ok(Json(rules))
}

pub async fn create_rule(
    State(state): State<Arc<AppState>>,
    Json(mut rule): Json<RuleDefinition>,
) -> Result<(StatusCode, Json<RuleDefinition>), (StatusCode, String)> {
    if rule.id.is_empty() {
        rule.id = uuid::Uuid::now_v7().to_string();
    }
    state
        .storage
        .objects()
        .put(
            STORE_KIND,
            &rule.id,
            serde_json::to_value(&rule).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(rule)))
}

pub async fn get_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<RuleDefinition>, (StatusCode, String)> {
    let val = state
        .storage
        .objects()
        .get(STORE_KIND, &id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "rule not found".into()))?;
    Ok(Json(serde_json::from_value(val).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
    })?))
}

pub async fn update_rule(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(rule): Json<RuleDefinition>,
) -> Result<Json<RuleDefinition>, (StatusCode, String)> {
    state
        .storage
        .objects()
        .put(
            STORE_KIND,
            &id,
            serde_json::to_value(&rule).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?,
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(rule))
}

pub async fn delete_rule(
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

/// Load enabled rules for a department (used by `department_chat_handler`).
pub async fn load_rules_for_engine(state: &Arc<AppState>, engine: &str) -> Vec<RuleDefinition> {
    state
        .storage
        .objects()
        .list(STORE_KIND, rusvel_core::domain::ObjectFilter::default())
        .await
        .unwrap_or_default()
        .into_iter()
        .filter_map(|v| serde_json::from_value::<RuleDefinition>(v).ok())
        .filter(|r| r.enabled)
        .filter(|r| {
            let rule_engine = r.metadata.get("engine").and_then(|e| e.as_str());
            rule_engine == Some(engine) || rule_engine.is_none()
        })
        .collect()
}

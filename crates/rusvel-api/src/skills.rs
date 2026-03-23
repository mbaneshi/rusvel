//! Skills CRUD — reusable prompt templates stored in ObjectStore.
//!
//! Skills are scoped to departments via `metadata.engine` field.

use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::AppState;

const STORE_KIND: &str = "skills";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_template: String,
    pub metadata: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct SkillQuery {
    pub engine: Option<String>,
}

pub async fn list_skills(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SkillQuery>,
) -> Result<Json<Vec<SkillDefinition>>, (StatusCode, String)> {
    let all = state.storage.objects()
        .list(STORE_KIND, rusvel_core::domain::ObjectFilter::default())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut skills: Vec<SkillDefinition> = all.into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .collect();

    if let Some(ref engine) = params.engine {
        skills.retain(|s| {
            let skill_engine = s.metadata.get("engine").and_then(|e| e.as_str());
            skill_engine == Some(engine) || skill_engine.is_none()
        });
    }

    Ok(Json(skills))
}

pub async fn create_skill(
    State(state): State<Arc<AppState>>,
    Json(mut skill): Json<SkillDefinition>,
) -> Result<(StatusCode, Json<SkillDefinition>), (StatusCode, String)> {
    if skill.id.is_empty() {
        skill.id = uuid::Uuid::now_v7().to_string();
    }
    state.storage.objects()
        .put(STORE_KIND, &skill.id, serde_json::to_value(&skill)
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok((StatusCode::CREATED, Json(skill)))
}

pub async fn get_skill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<SkillDefinition>, (StatusCode, String)> {
    let val = state.storage.objects()
        .get(STORE_KIND, &id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "skill not found".into()))?;
    Ok(Json(serde_json::from_value(val).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?))
}

pub async fn update_skill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(skill): Json<SkillDefinition>,
) -> Result<Json<SkillDefinition>, (StatusCode, String)> {
    state.storage.objects()
        .put(STORE_KIND, &id, serde_json::to_value(&skill).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?)
        .await.map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(skill))
}

pub async fn delete_skill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.storage.objects().delete(STORE_KIND, &id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

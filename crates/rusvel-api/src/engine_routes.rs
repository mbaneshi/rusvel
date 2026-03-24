//! Engine-specific API routes that call domain engines directly.
//!
//! These routes expose real domain logic (code analysis, content drafting,
//! harvest scoring) — not just CRUD or generic chat.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Query, State};
use axum::http::StatusCode;
use serde::Deserialize;

use rusvel_core::id::{ContentId, SessionId};

use crate::AppState;

type ApiResult<T> = Result<Json<T>, (StatusCode, String)>;

fn parse_session_id(id: &str) -> Result<SessionId, (StatusCode, String)> {
    id.parse::<uuid::Uuid>()
        .map(SessionId::from_uuid)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid session id".into()))
}

fn parse_content_id(id: &str) -> Result<ContentId, (StatusCode, String)> {
    id.parse::<uuid::Uuid>()
        .map(ContentId::from_uuid)
        .map_err(|_| (StatusCode::BAD_REQUEST, "invalid content id".into()))
}

fn engine_err(e: impl std::fmt::Display) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

// ── Code Engine ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub path: String,
}

pub async fn code_analyze(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AnalyzeRequest>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .code_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Code engine not available".into()))?;
    let analysis = engine
        .analyze(std::path::Path::new(&body.path))
        .await
        .map_err(engine_err)?;
    Ok(Json(serde_json::to_value(analysis).map_err(engine_err)?))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub limit: Option<usize>,
}

pub async fn code_search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchQuery>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .code_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Code engine not available".into()))?;
    let results = engine
        .search(&params.q, params.limit.unwrap_or(20))
        .map_err(engine_err)?;
    Ok(Json(serde_json::to_value(results).map_err(engine_err)?))
}

// ── Content Engine ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct DraftRequest {
    pub session_id: String,
    pub topic: String,
    pub kind: Option<String>,
}

pub async fn content_draft(
    State(state): State<Arc<AppState>>,
    Json(body): Json<DraftRequest>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .content_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Content engine not available".into()))?;
    let sid = parse_session_id(&body.session_id)?;
    let kind: rusvel_core::domain::ContentKind = body
        .kind
        .as_deref()
        .and_then(|k| serde_json::from_value(serde_json::json!(k)).ok())
        .unwrap_or(rusvel_core::domain::ContentKind::Blog);
    let item = engine
        .draft(&sid, &body.topic, kind)
        .await
        .map_err(engine_err)?;
    Ok(Json(serde_json::to_value(item).map_err(engine_err)?))
}

#[derive(Debug, Deserialize)]
pub struct PublishRequest {
    pub session_id: String,
    pub content_id: String,
    pub platform: String,
}

pub async fn content_publish(
    State(state): State<Arc<AppState>>,
    Json(body): Json<PublishRequest>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .content_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Content engine not available".into()))?;
    let sid = parse_session_id(&body.session_id)?;
    let cid = parse_content_id(&body.content_id)?;
    let platform: rusvel_core::domain::Platform =
        serde_json::from_value(serde_json::json!(body.platform))
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("invalid platform: {e}")))?;
    let result = engine
        .publish(&sid, cid, platform)
        .await
        .map_err(engine_err)?;
    Ok(Json(serde_json::to_value(result).map_err(engine_err)?))
}

#[derive(Debug, Deserialize)]
pub struct ContentListQuery {
    pub session_id: String,
    pub status: Option<String>,
}

pub async fn content_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ContentListQuery>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .content_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Content engine not available".into()))?;
    let sid = parse_session_id(&params.session_id)?;
    let status_filter: Option<rusvel_core::domain::ContentStatus> = params
        .status
        .as_deref()
        .and_then(|s| serde_json::from_value(serde_json::json!(s)).ok());
    let items = engine
        .list_content(&sid, status_filter)
        .await
        .map_err(engine_err)?;
    Ok(Json(serde_json::to_value(items).map_err(engine_err)?))
}

// ── Harvest Engine ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ScoreRequest {
    pub session_id: String,
    pub opportunity_id: String,
}

pub async fn harvest_score(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ScoreRequest>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .harvest_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Harvest engine not available".into()))?;
    let sid = parse_session_id(&body.session_id)?;
    let score = engine
        .score_opportunity(&sid, &body.opportunity_id)
        .await
        .map_err(engine_err)?;
    Ok(Json(serde_json::json!({ "score": score })))
}

#[derive(Debug, Deserialize)]
pub struct ProposalRequest {
    pub session_id: String,
    pub opportunity_id: String,
    pub profile: String,
}

pub async fn harvest_proposal(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ProposalRequest>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .harvest_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Harvest engine not available".into()))?;
    let sid = parse_session_id(&body.session_id)?;
    let proposal = engine
        .generate_proposal(&sid, &body.opportunity_id, &body.profile)
        .await
        .map_err(engine_err)?;
    Ok(Json(serde_json::to_value(proposal).map_err(engine_err)?))
}

#[derive(Debug, Deserialize)]
pub struct PipelineQuery {
    pub session_id: String,
}

pub async fn harvest_pipeline(
    State(state): State<Arc<AppState>>,
    Query(params): Query<PipelineQuery>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .harvest_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Harvest engine not available".into()))?;
    let sid = parse_session_id(&params.session_id)?;
    let stats = engine.pipeline(&sid).await.map_err(engine_err)?;
    Ok(Json(serde_json::to_value(stats).map_err(engine_err)?))
}

#[derive(Debug, Deserialize)]
pub struct OpportunityListQuery {
    pub session_id: String,
    pub stage: Option<String>,
}

pub async fn harvest_list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<OpportunityListQuery>,
) -> ApiResult<serde_json::Value> {
    let engine = state
        .harvest_engine
        .as_ref()
        .ok_or((StatusCode::SERVICE_UNAVAILABLE, "Harvest engine not available".into()))?;
    let sid = parse_session_id(&params.session_id)?;
    let stage: Option<rusvel_core::domain::OpportunityStage> = params
        .stage
        .as_deref()
        .and_then(|s| serde_json::from_value(serde_json::json!(s)).ok());
    let items = engine
        .list_opportunities(&sid, stage.as_ref())
        .await
        .map_err(engine_err)?;
    Ok(Json(serde_json::to_value(items).map_err(engine_err)?))
}

//! # rusvel-api
//!
//! HTTP API surface for RUSVEL, built on Axum.
//!
//! Provides a JSON REST API for sessions, mission planning, goals,
//! and event queries. All domain logic is delegated to the Forge engine
//! and core port traits.

pub mod agents;
pub mod analytics;
pub mod approvals;
pub mod build_cmd;
pub mod capability;
pub mod chat;
pub mod config;
pub mod department;
pub mod help;
pub mod hook_dispatch;
pub mod hooks;
pub mod knowledge;
pub mod mcp_servers;
pub mod routes;
pub mod rules;
pub mod skills;
pub mod workflows;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use forge_engine::ForgeEngine;
use rusvel_core::domain::UserProfile;
use rusvel_core::ports::{EmbeddingPort, EventPort, SessionPort, StoragePort, VectorStorePort};
use rusvel_core::registry::DepartmentRegistry;

/// Shared application state injected into all handlers.
pub struct AppState {
    pub forge: Arc<ForgeEngine>,
    pub sessions: Arc<dyn SessionPort>,
    pub events: Arc<dyn EventPort>,
    pub storage: Arc<dyn StoragePort>,
    pub profile: Option<UserProfile>,
    pub registry: DepartmentRegistry,
    pub embedding: Option<Arc<dyn EmbeddingPort>>,
    pub vector_store: Option<Arc<dyn VectorStorePort>>,
}

/// Build the Axum router with all routes, CORS, and tracing middleware.
///
/// If `frontend_dir` is provided and exists, the frontend SPA is served
/// at `/` with SPA fallback (all non-API routes serve `index.html`).
pub fn build_router(state: AppState) -> Router {
    build_router_with_frontend(state, None)
}

/// Build the router with optional frontend static file serving.
pub fn build_router_with_frontend(state: AppState, frontend_dir: Option<std::path::PathBuf>) -> Router {
    let shared = Arc::new(state);

    let api = Router::new()
        .route("/api/health", get(routes::health))
        .route("/api/sessions", get(routes::list_sessions))
        .route("/api/sessions", post(routes::create_session))
        .route("/api/sessions/{id}", get(routes::get_session))
        .route("/api/sessions/{id}/mission/today", get(routes::mission_today))
        .route("/api/sessions/{id}/mission/goals", get(routes::list_goals))
        .route("/api/sessions/{id}/mission/goals", post(routes::create_goal))
        .route("/api/sessions/{id}/events", get(routes::query_events))
        // Chat (god agent)
        .route("/api/chat", post(chat::chat_handler))
        .route("/api/chat/conversations", get(chat::list_conversations))
        .route("/api/chat/conversations/{id}", get(chat::get_history))
        // Config
        .route("/api/config", get(config::get_config))
        .route("/api/config", axum::routing::put(config::update_config))
        .route("/api/config/models", get(config::list_models))
        .route("/api/config/tools", get(config::list_tools))
        // Department Registry
        .route("/api/departments", get(department::list_departments))
        // Profile
        .route("/api/profile", get(department::get_profile))
        .route("/api/profile", axum::routing::put(department::update_profile))
        // Departments — 6 parameterized routes replace 72 hardcoded ones
        .route("/api/dept/{dept}/chat", post(department::dept_chat))
        .route("/api/dept/{dept}/chat/conversations", get(department::dept_conversations))
        .route("/api/dept/{dept}/chat/conversations/{id}", get(department::dept_history))
        .route("/api/dept/{dept}/config", get(department::dept_config_get))
        .route("/api/dept/{dept}/config", axum::routing::put(department::dept_config_update))
        .route("/api/dept/{dept}/events", get(department::dept_events))
        // Agents CRUD
        .route("/api/agents", get(agents::list_agents).post(agents::create_agent))
        .route("/api/agents/{id}", get(agents::get_agent).put(agents::update_agent).delete(agents::delete_agent))
        // Skills CRUD
        .route("/api/skills", get(skills::list_skills).post(skills::create_skill))
        .route("/api/skills/{id}", get(skills::get_skill).put(skills::update_skill).delete(skills::delete_skill))
        // Rules CRUD
        .route("/api/rules", get(rules::list_rules).post(rules::create_rule))
        .route("/api/rules/{id}", get(rules::get_rule).put(rules::update_rule).delete(rules::delete_rule))
        // MCP Servers CRUD
        .route("/api/mcp-servers", get(mcp_servers::list_mcp_servers).post(mcp_servers::create_mcp_server))
        .route("/api/mcp-servers/{id}", axum::routing::put(mcp_servers::update_mcp_server).delete(mcp_servers::delete_mcp_server))
        // Workflows CRUD + execution
        .route("/api/workflows", get(workflows::list_workflows).post(workflows::create_workflow))
        .route("/api/workflows/{id}", get(workflows::get_workflow).put(workflows::update_workflow).delete(workflows::delete_workflow))
        .route("/api/workflows/{id}/run", post(workflows::run_workflow))
        // Capability Engine
        .route("/api/capability/build", post(capability::build_capability))
        // Analytics
        .route("/api/analytics", get(analytics::get_analytics))
        // Help (AI-powered)
        .route("/api/help", post(help::help_handler))
        // Approvals (human-in-the-loop, ADR-008)
        .route("/api/approvals", get(approvals::list_pending))
        .route("/api/approvals/{id}/approve", post(approvals::approve_job))
        .route("/api/approvals/{id}/reject", post(approvals::reject_job))
        // Hooks CRUD
        .route("/api/hooks", get(hooks::list_hooks).post(hooks::create_hook))
        .route("/api/hooks/{id}", axum::routing::put(hooks::update_hook).delete(hooks::delete_hook))
        .route("/api/hooks/events", get(hooks::list_hook_events))
        // Knowledge (RAG)
        .route("/api/knowledge", get(knowledge::list_knowledge))
        .route("/api/knowledge/ingest", post(knowledge::ingest_knowledge))
        .route("/api/knowledge/search", post(knowledge::search_knowledge))
        .route("/api/knowledge/stats", get(knowledge::knowledge_stats))
        .route("/api/knowledge/{id}", axum::routing::delete(knowledge::delete_knowledge))
        .with_state(shared);

    // Serve frontend SPA if build directory exists.
    let app = if let Some(dir) = frontend_dir.filter(|d| d.exists()) {
        let index_html = std::fs::read_to_string(dir.join("index.html"))
            .unwrap_or_else(|_| "<h1>Frontend not found</h1>".into());
        let spa_fallback = {
            let html = index_html.clone();
            move |req: axum::extract::Request| {
                let html = html.clone();
                async move {
                    let path = req.uri().path();
                    if path.starts_with("/_app") || path == "/favicon.png" {
                        axum::response::Html("").into_response()
                    } else {
                        axum::response::Html(html).into_response()
                    }
                }
            }
        };
        api.nest_service("/_app", ServeDir::new(dir.join("_app")))
            .nest_service("/favicon.png", ServeDir::new(dir.join("favicon.png")))
            .fallback(spa_fallback)
    } else {
        api
    };

    app.layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    )
    .layer(TraceLayer::new_for_http())
}

/// Start the HTTP server on the given address.
pub async fn start_server(
    router: Router,
    addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("RUSVEL API listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, router).await?;
    Ok(())
}

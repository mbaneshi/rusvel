//! # rusvel-api
//!
//! HTTP API surface for RUSVEL, built on Axum.
//!
//! Provides a JSON REST API for sessions, mission planning, goals,
//! and event queries. All domain logic is delegated to the Forge engine
//! and core port traits.

pub mod agents;
pub mod chat;
pub mod config;
pub mod department;
pub mod routes;
pub mod rules;
pub mod skills;

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
use rusvel_core::ports::{EventPort, SessionPort, StoragePort};

/// Shared application state injected into all handlers.
pub struct AppState {
    pub forge: Arc<ForgeEngine>,
    pub sessions: Arc<dyn SessionPort>,
    pub events: Arc<dyn EventPort>,
    pub storage: Arc<dyn StoragePort>,
    pub profile: Option<UserProfile>,
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
        .route(
            "/api/sessions/{id}/mission/today",
            get(routes::mission_today),
        )
        .route(
            "/api/sessions/{id}/mission/goals",
            get(routes::list_goals),
        )
        .route(
            "/api/sessions/{id}/mission/goals",
            post(routes::create_goal),
        )
        .route("/api/sessions/{id}/events", get(routes::query_events))
        // Chat (god agent)
        .route("/api/chat", post(chat::chat_handler))
        .route("/api/chat/conversations", get(chat::list_conversations))
        .route("/api/chat/conversations/{id}", get(chat::get_history))
        // Config (M02, M03, M04)
        .route("/api/config", get(config::get_config))
        .route("/api/config", axum::routing::put(config::update_config))
        .route("/api/config/models", get(config::list_models))
        .route("/api/config/tools", get(config::list_tools))
        // Departments (Code, Content, Harvest, GTM, Forge)
        .route("/api/dept/code/chat", post(department::code_chat))
        .route("/api/dept/code/chat/conversations", get(department::code_conversations))
        .route("/api/dept/code/chat/conversations/{id}", get(department::code_history))
        .route("/api/dept/code/config", get(department::code_config_get))
        .route("/api/dept/code/config", axum::routing::put(department::code_config_update))
        .route("/api/dept/code/events", get(department::code_events))
        .route("/api/dept/content/chat", post(department::content_chat))
        .route("/api/dept/content/chat/conversations", get(department::content_conversations))
        .route("/api/dept/content/chat/conversations/{id}", get(department::content_history))
        .route("/api/dept/content/config", get(department::content_config_get))
        .route("/api/dept/content/config", axum::routing::put(department::content_config_update))
        .route("/api/dept/content/events", get(department::content_events))
        .route("/api/dept/harvest/chat", post(department::harvest_chat))
        .route("/api/dept/harvest/chat/conversations", get(department::harvest_conversations))
        .route("/api/dept/harvest/chat/conversations/{id}", get(department::harvest_history))
        .route("/api/dept/harvest/config", get(department::harvest_config_get))
        .route("/api/dept/harvest/config", axum::routing::put(department::harvest_config_update))
        .route("/api/dept/harvest/events", get(department::harvest_events))
        .route("/api/dept/gtm/chat", post(department::gtm_chat))
        .route("/api/dept/gtm/chat/conversations", get(department::gtm_conversations))
        .route("/api/dept/gtm/chat/conversations/{id}", get(department::gtm_history))
        .route("/api/dept/gtm/config", get(department::gtm_config_get))
        .route("/api/dept/gtm/config", axum::routing::put(department::gtm_config_update))
        .route("/api/dept/gtm/events", get(department::gtm_events))
        .route("/api/dept/forge/chat", post(department::forge_chat))
        .route("/api/dept/forge/chat/conversations", get(department::forge_conversations))
        .route("/api/dept/forge/chat/conversations/{id}", get(department::forge_history))
        .route("/api/dept/forge/config", get(department::forge_config_get))
        .route("/api/dept/forge/config", axum::routing::put(department::forge_config_update))
        .route("/api/dept/forge/events", get(department::forge_events))
        // Agents CRUD
        .route("/api/agents", get(agents::list_agents).post(agents::create_agent))
        .route("/api/agents/{id}", get(agents::get_agent).put(agents::update_agent).delete(agents::delete_agent))
        // Skills CRUD
        .route("/api/skills", get(skills::list_skills).post(skills::create_skill))
        .route("/api/skills/{id}", get(skills::get_skill).put(skills::update_skill).delete(skills::delete_skill))
        // Rules CRUD
        .route("/api/rules", get(rules::list_rules).post(rules::create_rule))
        .route("/api/rules/{id}", get(rules::get_rule).put(rules::update_rule).delete(rules::delete_rule))
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
                    // Serve static assets directly
                    let path = req.uri().path();
                    if path.starts_with("/_app") || path == "/favicon.png" {
                        // Handled by nest_service below
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

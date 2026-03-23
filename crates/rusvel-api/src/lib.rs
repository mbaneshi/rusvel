//! # rusvel-api
//!
//! HTTP API surface for RUSVEL, built on Axum.
//!
//! Provides a JSON REST API for sessions, mission planning, goals,
//! and event queries. All domain logic is delegated to the Forge engine
//! and core port traits.

pub mod chat;
pub mod config;
pub mod routes;

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
        .with_state(shared);

    // Serve frontend SPA if build directory exists.
    // Static assets (_app/*, favicon) are served directly.
    // All other non-API routes get index.html (SPA client-side routing).
    let app = if let Some(dir) = frontend_dir.filter(|d| d.exists()) {
        let index_html = std::fs::read_to_string(dir.join("index.html"))
            .unwrap_or_else(|_| "<h1>Frontend not found</h1>".into());
        let spa_fallback = move || {
            let html = index_html.clone();
            async move { axum::response::Html(html).into_response() }
        };
        api.nest_service("/_app", ServeDir::new(dir.join("_app")))
            .nest_service("/favicon.png", ServeDir::new(dir.join("favicon.png")))
            .fallback(get(spa_fallback))
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

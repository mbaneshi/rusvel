//! # rusvel-api
//!
//! HTTP API surface for RUSVEL, built on Axum.
//!
//! Provides a JSON REST API for sessions, mission planning, goals,
//! and event queries. All domain logic is delegated to the Forge engine
//! and core port traits.

pub mod agents;
pub mod analytics;
pub mod build_cmd;
pub mod chat;
pub mod config;
pub mod department;
pub mod hooks;
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
        // Finance
        .route("/api/dept/finance/chat", post(department::finance_chat))
        .route("/api/dept/finance/chat/conversations", get(department::finance_conversations))
        .route("/api/dept/finance/chat/conversations/{id}", get(department::finance_history))
        .route("/api/dept/finance/config", get(department::finance_config_get))
        .route("/api/dept/finance/config", axum::routing::put(department::finance_config_update))
        .route("/api/dept/finance/events", get(department::finance_events))
        // Product
        .route("/api/dept/product/chat", post(department::product_chat))
        .route("/api/dept/product/chat/conversations", get(department::product_conversations))
        .route("/api/dept/product/chat/conversations/{id}", get(department::product_history))
        .route("/api/dept/product/config", get(department::product_config_get))
        .route("/api/dept/product/config", axum::routing::put(department::product_config_update))
        .route("/api/dept/product/events", get(department::product_events))
        // Growth
        .route("/api/dept/growth/chat", post(department::growth_chat))
        .route("/api/dept/growth/chat/conversations", get(department::growth_conversations))
        .route("/api/dept/growth/chat/conversations/{id}", get(department::growth_history))
        .route("/api/dept/growth/config", get(department::growth_config_get))
        .route("/api/dept/growth/config", axum::routing::put(department::growth_config_update))
        .route("/api/dept/growth/events", get(department::growth_events))
        // Distribution
        .route("/api/dept/distro/chat", post(department::distro_chat))
        .route("/api/dept/distro/chat/conversations", get(department::distro_conversations))
        .route("/api/dept/distro/chat/conversations/{id}", get(department::distro_history))
        .route("/api/dept/distro/config", get(department::distro_config_get))
        .route("/api/dept/distro/config", axum::routing::put(department::distro_config_update))
        .route("/api/dept/distro/events", get(department::distro_events))
        // Legal
        .route("/api/dept/legal/chat", post(department::legal_chat))
        .route("/api/dept/legal/chat/conversations", get(department::legal_conversations))
        .route("/api/dept/legal/chat/conversations/{id}", get(department::legal_history))
        .route("/api/dept/legal/config", get(department::legal_config_get))
        .route("/api/dept/legal/config", axum::routing::put(department::legal_config_update))
        .route("/api/dept/legal/events", get(department::legal_events))
        // Support
        .route("/api/dept/support/chat", post(department::support_chat))
        .route("/api/dept/support/chat/conversations", get(department::support_conversations))
        .route("/api/dept/support/chat/conversations/{id}", get(department::support_history))
        .route("/api/dept/support/config", get(department::support_config_get))
        .route("/api/dept/support/config", axum::routing::put(department::support_config_update))
        .route("/api/dept/support/events", get(department::support_events))
        // Infra
        .route("/api/dept/infra/chat", post(department::infra_chat))
        .route("/api/dept/infra/chat/conversations", get(department::infra_conversations))
        .route("/api/dept/infra/chat/conversations/{id}", get(department::infra_history))
        .route("/api/dept/infra/config", get(department::infra_config_get))
        .route("/api/dept/infra/config", axum::routing::put(department::infra_config_update))
        .route("/api/dept/infra/events", get(department::infra_events))
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
        // Analytics
        .route("/api/analytics", get(analytics::get_analytics))
        // Hooks CRUD
        .route("/api/hooks", get(hooks::list_hooks).post(hooks::create_hook))
        .route("/api/hooks/{id}", axum::routing::put(hooks::update_hook).delete(hooks::delete_hook))
        .route("/api/hooks/events", get(hooks::list_hook_events))
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

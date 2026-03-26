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
pub mod browser;
pub mod build_cmd;
pub mod capability;
pub mod chat;
pub mod config;
pub mod db_routes;
pub mod department;
pub mod engine_routes;
pub mod flow_routes;
pub mod help;
pub mod hook_dispatch;
pub mod hooks;
pub mod kits;
pub mod knowledge;
pub mod mcp_servers;
pub mod playbooks;
pub mod routes;
pub mod rules;
pub mod skills;
pub mod system;
pub mod terminal;
pub mod visual_report;
pub mod workflows;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

use code_engine::CodeEngine;
use content_engine::ContentEngine;
use flow_engine::FlowEngine;
use forge_engine::ForgeEngine;
use harvest_engine::HarvestEngine;
use rusvel_core::domain::UserProfile;
use rusvel_core::ports::{
    DeployPort, EmbeddingPort, EventPort, JobPort, MemoryPort, SessionPort, StoragePort,
    TerminalPort, ToolPort, VectorStorePort,
};
use rusvel_core::registry::DepartmentRegistry;
use rusvel_agent::AgentRuntime;
use rusvel_db::Database;

/// Shared application state injected into all handlers.
pub struct AppState {
    pub forge: Arc<ForgeEngine>,
    pub code_engine: Option<Arc<CodeEngine>>,
    pub content_engine: Option<Arc<ContentEngine>>,
    pub harvest_engine: Option<Arc<HarvestEngine>>,
    pub flow_engine: Option<Arc<FlowEngine>>,
    pub sessions: Arc<dyn SessionPort>,
    pub events: Arc<dyn EventPort>,
    /// Same SQLite job queue as [`StoragePort::jobs`] when using `rusvel_db::Database`.
    pub jobs: Arc<dyn JobPort>,
    /// Same database as `storage` when using `rusvel_db::Database` — for RusvelBase / schema API.
    pub database: Arc<Database>,
    pub storage: Arc<dyn StoragePort>,
    pub profile: Option<UserProfile>,
    pub registry: DepartmentRegistry,
    pub embedding: Option<Arc<dyn EmbeddingPort>>,
    pub vector_store: Option<Arc<dyn VectorStorePort>>,
    /// Session-scoped FTS memory (SQLite + FTS5); required for hybrid search.
    pub memory: Arc<dyn MemoryPort>,
    pub deploy: Option<Arc<dyn DeployPort>>,
    pub agent_runtime: Arc<AgentRuntime>,
    pub tools: Arc<dyn ToolPort>,
    pub terminal: Option<Arc<dyn TerminalPort>>,
    /// Chrome CDP client (passive capture + actions); `None` when not wired.
    pub cdp: Option<Arc<rusvel_cdp::CdpClient>>,
}

/// Build the Axum router with all routes, CORS, and tracing middleware.
///
/// If `frontend_dir` is provided and exists, the frontend SPA is served
/// at `/` with SPA fallback (all non-API routes serve `index.html`).
pub fn build_router(state: AppState) -> Router {
    build_router_with_frontend(state, None)
}

/// Build the router with optional frontend static file serving.
pub fn build_router_with_frontend(
    state: AppState,
    frontend_dir: Option<std::path::PathBuf>,
) -> Router {
    let shared = Arc::new(state);

    let api = Router::new()
        .route("/api/health", get(routes::health))
        .route("/api/brief", get(engine_routes::brief_get))
        .route("/api/brief/generate", post(engine_routes::brief_generate))
        .route("/api/sessions", get(routes::list_sessions))
        .route("/api/sessions", post(routes::create_session))
        .route("/api/sessions/{id}", get(routes::get_session))
        .route(
            "/api/sessions/{id}/mission/today",
            get(routes::mission_today),
        )
        .route("/api/sessions/{id}/mission/goals", get(routes::list_goals))
        .route(
            "/api/sessions/{id}/mission/goals",
            post(routes::create_goal),
        )
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
        // RusvelBase — DB browser (schema, rows, SQL)
        .route("/api/db/tables", get(db_routes::list_tables))
        .route(
            "/api/db/tables/{table}/schema",
            get(db_routes::get_table_schema),
        )
        .route(
            "/api/db/tables/{table}/rows",
            get(db_routes::get_table_rows),
        )
        .route("/api/db/sql", axum::routing::post(db_routes::post_sql))
        // Profile
        .route("/api/profile", get(department::get_profile))
        .route(
            "/api/profile",
            axum::routing::put(department::update_profile),
        )
        // Departments — 6 parameterized routes replace 72 hardcoded ones
        .route("/api/dept/{dept}/chat", post(department::dept_chat))
        .route(
            "/api/dept/{dept}/chat/conversations",
            get(department::dept_conversations),
        )
        .route(
            "/api/dept/{dept}/chat/conversations/{id}",
            get(department::dept_history),
        )
        .route("/api/dept/{dept}/config", get(department::dept_config_get))
        .route(
            "/api/dept/{dept}/config",
            axum::routing::put(department::dept_config_update),
        )
        .route("/api/dept/{dept}/events", get(department::dept_events))
        // Agents CRUD
        .route(
            "/api/agents",
            get(agents::list_agents).post(agents::create_agent),
        )
        .route(
            "/api/agents/{id}",
            get(agents::get_agent)
                .put(agents::update_agent)
                .delete(agents::delete_agent),
        )
        // Skills CRUD
        .route(
            "/api/skills",
            get(skills::list_skills).post(skills::create_skill),
        )
        .route(
            "/api/skills/{id}",
            get(skills::get_skill)
                .put(skills::update_skill)
                .delete(skills::delete_skill),
        )
        // Rules CRUD
        .route(
            "/api/rules",
            get(rules::list_rules).post(rules::create_rule),
        )
        .route(
            "/api/rules/{id}",
            get(rules::get_rule)
                .put(rules::update_rule)
                .delete(rules::delete_rule),
        )
        // MCP Servers CRUD
        .route(
            "/api/mcp-servers",
            get(mcp_servers::list_mcp_servers).post(mcp_servers::create_mcp_server),
        )
        .route(
            "/api/mcp-servers/{id}",
            axum::routing::put(mcp_servers::update_mcp_server)
                .delete(mcp_servers::delete_mcp_server),
        )
        // Workflows CRUD + execution
        .route(
            "/api/workflows",
            get(workflows::list_workflows).post(workflows::create_workflow),
        )
        .route(
            "/api/workflows/{id}",
            get(workflows::get_workflow)
                .put(workflows::update_workflow)
                .delete(workflows::delete_workflow),
        )
        .route("/api/workflows/{id}/run", post(workflows::run_workflow))
        // Engine-specific routes (Code, Content, Harvest)
        .route("/api/dept/code/analyze", post(engine_routes::code_analyze))
        .route("/api/dept/code/search", get(engine_routes::code_search))
        .route("/api/dept/content/draft", post(engine_routes::content_draft))
        .route(
            "/api/dept/content/from-code",
            post(engine_routes::content_from_code),
        )
        .route(
            "/api/dept/content/{id}/approve",
            axum::routing::patch(engine_routes::content_approve),
        )
        .route("/api/dept/content/publish", post(engine_routes::content_publish))
        .route("/api/dept/content/list", get(engine_routes::content_list))
        .route("/api/dept/harvest/score", post(engine_routes::harvest_score))
        .route("/api/dept/harvest/scan", post(engine_routes::harvest_scan))
        .route("/api/dept/harvest/proposal", post(engine_routes::harvest_proposal))
        .route("/api/dept/harvest/pipeline", get(engine_routes::harvest_pipeline))
        .route("/api/dept/harvest/list", get(engine_routes::harvest_list))
        // Flow Engine (DAG workflows)
        .route("/api/flows", get(flow_routes::list_flows).post(flow_routes::create_flow))
        .route(
            "/api/flows/{id}",
            get(flow_routes::get_flow)
                .put(flow_routes::update_flow)
                .delete(flow_routes::delete_flow),
        )
        .route("/api/flows/{id}/run", post(flow_routes::run_flow))
        .route("/api/flows/{id}/executions", get(flow_routes::list_executions))
        .route(
            "/api/flows/{id}/executions/{exec_id}/panes",
            get(flow_routes::list_flow_execution_panes),
        )
        .route("/api/flows/executions/{id}", get(flow_routes::get_execution))
        .route(
            "/api/flows/executions/{id}/resume",
            post(flow_routes::resume_flow),
        )
        .route(
            "/api/flows/executions/{id}/retry/{node_id}",
            post(flow_routes::retry_node),
        )
        .route(
            "/api/flows/executions/{id}/checkpoint",
            get(flow_routes::get_checkpoint),
        )
        .route("/api/flows/node-types", get(flow_routes::list_node_types))
        // Playbooks (multi-step pipelines)
        .route("/api/playbooks/runs", get(playbooks::list_runs))
        .route("/api/playbooks/runs/{run_id}", get(playbooks::get_run))
        .route("/api/playbooks", get(playbooks::list_playbooks).post(playbooks::create_playbook))
        .route("/api/playbooks/{id}", get(playbooks::get_playbook))
        .route("/api/playbooks/{id}/run", post(playbooks::run_playbook))
        // Starter kits
        .route("/api/kits", get(kits::list_kits))
        .route("/api/kits/{id}", get(kits::get_kit))
        .route("/api/kits/{id}/install", post(kits::install_kit))
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
        .route(
            "/api/hooks",
            get(hooks::list_hooks).post(hooks::create_hook),
        )
        .route(
            "/api/hooks/{id}",
            axum::routing::put(hooks::update_hook).delete(hooks::delete_hook),
        )
        .route("/api/hooks/events", get(hooks::list_hook_events))
        // Knowledge (RAG)
        .route("/api/knowledge", get(knowledge::list_knowledge))
        .route("/api/knowledge/ingest", post(knowledge::ingest_knowledge))
        .route("/api/knowledge/search", post(knowledge::search_knowledge))
        .route(
            "/api/knowledge/hybrid-search",
            post(knowledge::hybrid_search_knowledge),
        )
        .route("/api/knowledge/stats", get(knowledge::knowledge_stats))
        .route("/api/knowledge/related", get(knowledge::related_knowledge))
        .route(
            "/api/knowledge/{id}",
            axum::routing::delete(knowledge::delete_knowledge),
        )
        // System self-improvement
        .route("/api/system/test", post(system::run_tests))
        .route("/api/system/build", post(system::run_build))
        .route("/api/system/status", get(system::get_status))
        .route("/api/system/fix", post(system::self_fix))
        .route("/api/system/ingest-docs", post(system::ingest_docs))
        // Visual regression testing
        .route(
            "/api/system/visual-report",
            get(visual_report::get_reports).post(visual_report::store_report),
        )
        .route(
            "/api/system/visual-report/self-correct",
            post(visual_report::self_correct),
        )
        .route(
            "/api/system/visual-test",
            post(visual_report::run_visual_tests),
        )
        // Terminal: dept pane + WebSocket + run-scoped panes (delegation visibility)
        .route("/api/terminal/dept/{dept_id}", get(terminal::terminal_dept_pane))
        .route("/api/terminal/runs/{run_id}/panes", get(terminal::terminal_run_panes))
        .route("/api/terminal/ws", get(terminal::terminal_ws))
        // Browser (CDP)
        .route("/api/browser/status", get(browser::browser_status))
        .route("/api/browser/connect", axum::routing::post(browser::browser_connect))
        .route("/api/browser/tabs", get(browser::browser_tabs))
        .route(
            "/api/browser/observe/{tab}",
            axum::routing::post(browser::browser_observe),
        )
        .route("/api/browser/captures", get(browser::browser_captures))
        .route(
            "/api/browser/captures/stream",
            get(browser::browser_captures_stream),
        )
        .route("/api/browser/act", axum::routing::post(browser::browser_act))
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

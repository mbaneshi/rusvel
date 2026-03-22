//! # rusvel-api
//!
//! HTTP API surface for RUSVEL, built on Axum.
//!
//! Provides a JSON REST API for sessions, mission planning, goals,
//! and event queries. All domain logic is delegated to the Forge engine
//! and core port traits.

pub mod routes;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::routing::{get, post};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use forge_engine::ForgeEngine;
use rusvel_core::ports::{EventPort, SessionPort};

/// Shared application state injected into all handlers.
pub struct AppState {
    pub forge: Arc<ForgeEngine>,
    pub sessions: Arc<dyn SessionPort>,
    pub events: Arc<dyn EventPort>,
}

/// Build the Axum router with all routes, CORS, and tracing middleware.
pub fn build_router(state: AppState) -> Router {
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
        .with_state(shared);

    api.layer(
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

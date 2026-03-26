//! WebSocket bridge for the PTY-backed terminal.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::Json;
use futures::stream::StreamExt;
use serde::Deserialize;
use tokio::sync::broadcast;
use uuid::Uuid;

use rusvel_core::id::{PaneId, SessionId};
use rusvel_core::terminal::{PaneSize, PaneSource, WindowSource};

use crate::AppState;

fn dept_pane_cache() -> &'static Mutex<HashMap<(SessionId, String), PaneId>> {
    static CACHE: OnceLock<Mutex<HashMap<(SessionId, String), PaneId>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Deserialize)]
pub struct TerminalDeptQuery {
    pub session_id: String,
}

/// GET /api/terminal/dept/:dept_id?session_id=… — get or create a PTY pane for this department.
pub async fn terminal_dept_pane(
    Path(dept_id): Path<String>,
    Query(q): Query<TerminalDeptQuery>,
    State(state): State<std::sync::Arc<AppState>>,
) -> impl IntoResponse {
    let terminal = match state.terminal.as_ref() {
        Some(t) => t.clone(),
        None => {
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Terminal not configured" })),
            )
                .into_response();
        }
    };

    let session_uuid = match Uuid::parse_str(q.session_id.trim()) {
        Ok(u) => u,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": "invalid session_id" })),
            )
                .into_response();
        }
    };
    let session_id = SessionId::from_uuid(session_uuid);

    let key = (session_id, dept_id.clone());
    {
        let guard = dept_pane_cache().lock().unwrap();
        if let Some(pid) = guard.get(&key) {
            return Json(serde_json::json!({ "pane_id": pid.to_string() })).into_response();
        }
    }

    let window_id = match terminal
        .create_window(
            &session_id,
            &format!("dept-{dept_id}"),
            WindowSource::Department(dept_id.clone()),
        )
        .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create terminal window: {e}");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "failed to create window" })),
            )
                .into_response();
        }
    };

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let size = PaneSize { rows: 24, cols: 80 };

    let pane_id = match terminal
        .create_pane(&window_id, &shell, &cwd, size, PaneSource::Shell)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            tracing::error!("Failed to create terminal pane: {e}");
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "failed to create pane" })),
            )
                .into_response();
        }
    };

    let mut guard = dept_pane_cache().lock().unwrap();
    guard.insert(key, pane_id);

    Json(serde_json::json!({ "pane_id": pane_id.to_string() })).into_response()
}

#[derive(Debug, Deserialize)]
pub struct TerminalWsQuery {
    pub pane_id: Option<String>,
}

/// GET /api/terminal/ws — upgrade to WebSocket, spawn a PTY pane or attach to `pane_id`, bridge I/O.
pub async fn terminal_ws(
    ws: WebSocketUpgrade,
    Query(q): Query<TerminalWsQuery>,
    State(state): State<std::sync::Arc<AppState>>,
) -> impl IntoResponse {
    let pane_id = q.pane_id;
    ws.on_upgrade(move |socket| handle_ws(socket, state, pane_id))
}

async fn handle_ws(
    socket: WebSocket,
    state: std::sync::Arc<AppState>,
    existing_pane: Option<String>,
) {
    let owns_pane = existing_pane.is_none();
    let terminal = match state.terminal.as_ref() {
        Some(t) => t.clone(),
        None => {
            tracing::warn!("Terminal WebSocket requested but TerminalPort not configured");
            return;
        }
    };

    let pane_id = if let Some(pane_str) = existing_pane {
        let uuid = match Uuid::parse_str(pane_str.trim()) {
            Ok(u) => u,
            Err(_) => {
                tracing::warn!("Invalid pane_id in WebSocket query");
                return;
            }
        };
        PaneId::from_uuid(uuid)
    } else {
        // Create a session-scoped window + pane for this WebSocket connection.
        let session_id = SessionId::new();
        let window_id = match terminal
            .create_window(&session_id, "ws-terminal", WindowSource::Manual)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("Failed to create terminal window: {e}");
                return;
            }
        };

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
        let size = PaneSize { rows: 24, cols: 80 };

        match terminal
            .create_pane(&window_id, &shell, &cwd, size, PaneSource::Shell)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                tracing::error!("Failed to create terminal pane: {e}");
                return;
            }
        }
    };

    let mut rx: broadcast::Receiver<Vec<u8>> = match terminal.subscribe_pane(&pane_id).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to subscribe to pane output: {e}");
            return;
        }
    };

    let (mut ws_tx, mut ws_rx) = socket.split();

    // PTY output -> WebSocket
    let terminal_write = terminal.clone();
    let pane_for_close = pane_id;
    let pty_to_ws = tokio::spawn(async move {
        use futures::SinkExt;
        loop {
            match rx.recv().await {
                Ok(data) => {
                    if ws_tx.send(Message::Binary(data.into())).await.is_err() {
                        break;
                    }
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    tracing::debug!("Terminal WS lagged {n} messages");
                }
                Err(broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    // WebSocket input -> PTY
    let terminal_input = terminal.clone();
    let pane_for_input = pane_id;
    let ws_to_pty = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(text) => {
                    if let Err(e) = terminal_input
                        .write_pane(&pane_for_input, text.as_bytes())
                        .await
                    {
                        tracing::debug!("write_pane error: {e}");
                        break;
                    }
                }
                Message::Binary(data) => {
                    if let Err(e) = terminal_input
                        .write_pane(&pane_for_input, &data)
                        .await
                    {
                        tracing::debug!("write_pane error: {e}");
                        break;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either direction to finish, then clean up.
    tokio::select! {
        _ = pty_to_ws => {},
        _ = ws_to_pty => {},
    }

    if owns_pane {
        let _ = terminal_write.close_pane(&pane_for_close).await;
    }
    tracing::debug!(
        "Terminal WebSocket session closed (pane {pane_for_close}, owns_pane={owns_pane})"
    );
}

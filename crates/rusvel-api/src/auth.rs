//! Bearer token authentication middleware.
//!
//! Opt-in: when `RUSVEL_API_TOKEN` is unset, all requests pass through.
//! When set, every request (except exempt paths) must carry
//! `Authorization: Bearer <token>`.

use std::sync::Arc;

use axum::body::Body;
use axum::extract::State;
use axum::http::{Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::AppState;

/// Paths under `/api/*` that bypass bearer auth. Non-`/api/*` routes (embedded SPA) always bypass.
/// Webhook receive: [`webhook_receive_exempt`].
const EXEMPT_PATHS: &[&str] = &["/api/health"];

fn webhook_receive_exempt(path: &str, method: &Method) -> bool {
    if *method != Method::POST {
        return false;
    }
    let t = path.trim_end_matches('/');
    t.starts_with("/api/webhooks/") && t.len() > "/api/webhooks/".len()
}

/// Auth configuration resolved once at startup.
#[derive(Clone)]
pub struct AuthConfig {
    /// `None` means auth is disabled (opt-in).
    pub token: Option<String>,
}

impl AuthConfig {
    /// Build from the environment. Call once at router construction time.
    pub fn from_env() -> Self {
        Self {
            token: std::env::var("RUSVEL_API_TOKEN")
                .ok()
                .filter(|t| !t.is_empty()),
        }
    }
}

async fn bearer_check(config: &AuthConfig, req: Request<Body>, next: Next) -> Response {
    let Some(expected) = config.token.as_ref() else {
        return next.run(req).await;
    };

    let path = req.uri().path();
    // Non-API routes (embedded SPA, static assets) do not require a bearer token.
    if !path.starts_with("/api/") {
        return next.run(req).await;
    }
    if EXEMPT_PATHS.contains(&path) || webhook_receive_exempt(path, req.method()) {
        return next.run(req).await;
    }

    if let Some(header) = req.headers().get("authorization")
        && let Ok(value) = header.to_str()
        && let Some(token) = value.strip_prefix("Bearer ")
        && token == expected
    {
        return next.run(req).await;
    }

    unauthorized_response()
}

/// Tower-compatible middleware for `axum::middleware::from_fn_with_state` with [`AppState`].
pub async fn bearer_auth(
    State(state): State<Arc<AppState>>,
    req: Request<Body>,
    next: Next,
) -> Response {
    bearer_check(&state.auth, req, next).await
}

#[cfg(test)]
async fn bearer_auth_with_config(
    State(config): State<AuthConfig>,
    req: Request<Body>,
    next: Next,
) -> Response {
    bearer_check(&config, req, next).await
}

fn unauthorized_response() -> Response {
    let body = json!({
        "error": "Unauthorized",
        "message": "Invalid or missing Bearer token"
    });
    (
        StatusCode::UNAUTHORIZED,
        [("content-type", "application/json")],
        serde_json::to_string(&body).unwrap(),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::Router;
    use axum::body::to_bytes;
    use axum::routing::{get, post};
    use tower::ServiceExt;

    fn router_with(token: Option<&str>) -> Router {
        let config = AuthConfig {
            token: token.map(String::from),
        };
        Router::new()
            .route("/api/health", get(|| async { "ok" }))
            .route("/api/sessions", get(|| async { "sessions" }))
            .layer(axum::middleware::from_fn_with_state(
                config,
                bearer_auth_with_config,
            ))
    }

    fn req(uri: &str, token: Option<&str>) -> Request<Body> {
        let mut builder = Request::builder().uri(uri);
        if let Some(t) = token {
            builder = builder.header("authorization", format!("Bearer {t}"));
        }
        builder.body(Body::empty()).unwrap()
    }

    #[tokio::test]
    async fn no_token_configured_passes_all() {
        let app = router_with(None);
        let res = app.oneshot(req("/api/sessions", None)).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn valid_token_passes() {
        let app = router_with(Some("secret123"));
        let res = app
            .oneshot(req("/api/sessions", Some("secret123")))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn invalid_token_401() {
        let app = router_with(Some("secret123"));
        let res = app
            .oneshot(req("/api/sessions", Some("wrong")))
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(v["error"], "Unauthorized");
    }

    #[tokio::test]
    async fn missing_token_401() {
        let app = router_with(Some("secret123"));
        let res = app.oneshot(req("/api/sessions", None)).await.unwrap();
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn health_always_exempt() {
        let app = router_with(Some("secret123"));
        let res = app.oneshot(req("/api/health", None)).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn webhook_receive_post_exempt_without_bearer() {
        let config = AuthConfig {
            token: Some("secret123".into()),
        };
        let app = Router::new()
            .route("/api/webhooks/{id}", post(|| async { "ok" }))
            .layer(axum::middleware::from_fn_with_state(
                config,
                bearer_auth_with_config,
            ));
        let req = Request::builder()
            .method("POST")
            .uri("/api/webhooks/abc-id")
            .body(Body::empty())
            .unwrap();
        let res = app.oneshot(req).await.unwrap();
        assert_eq!(res.status(), StatusCode::OK);
    }
}

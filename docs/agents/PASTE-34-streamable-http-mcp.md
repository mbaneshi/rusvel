# Task #34: Streamable HTTP MCP

> Read this file, then do the task. Only modify files listed below.

## Goal

Upgrade MCP server from stdio-only to HTTP transport (2025-11-25 spec). Keep stdio as fallback.

## Files to Read First

- `crates/rusvel-mcp/src/lib.rs` — existing MCP server (stdio JSON-RPC)
- `crates/rusvel-app/src/main.rs` — how --mcp flag connects
- Search web for "MCP streamable HTTP transport specification 2025"

## What to Build

### 1. HTTP transport in `crates/rusvel-mcp/src/http.rs` (new)

Implement MCP over HTTP:
- `POST /mcp` — receive JSON-RPC request, return response
- `GET /mcp/sse` — SSE endpoint for server-initiated messages (notifications)
- Session management: track sessions via session ID header
- Keep-alive: periodic SSE heartbeat

### 2. Axum routes

```rust
pub fn mcp_http_routes() -> Router {
    Router::new()
        .route("/mcp", post(handle_mcp_request))
        .route("/mcp/sse", get(handle_mcp_sse))
}
```

### 3. Wire in main.rs

When `--mcp-http` flag is passed (or `--mcp http`), mount HTTP routes on the Axum server instead of using stdio. Keep `--mcp` as stdio mode.

### 4. OAuth stub

Add placeholder for OAuth authentication:
```rust
pub struct McpAuth {
    pub enabled: bool,
    pub token: Option<String>,
}
```

Middleware that checks `Authorization: Bearer <token>` header if auth is enabled.

## Files to Modify

- `crates/rusvel-mcp/src/http.rs` (new)
- `crates/rusvel-mcp/src/lib.rs` — add pub mod http, export routes
- `crates/rusvel-app/src/main.rs` — mount HTTP routes when flag set

## Verify

```bash
cargo check -p rusvel-mcp && cargo check --workspace
```

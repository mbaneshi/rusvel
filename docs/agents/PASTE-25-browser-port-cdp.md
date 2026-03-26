# Task #25: BrowserPort + rusvel-cdp

> Read this file, then do the task. Only modify files listed below.

## Goal

Add BrowserPort trait (port #20) and a CDP WebSocket adapter for passive browser observation.

## Files to Read First

- `crates/rusvel-core/src/ports.rs` — existing port traits pattern
- `crates/rusvel-core/src/domain.rs` — domain type patterns
- `docs/plans/cdp-browser-bridge.md` — full design doc (READ THIS THOROUGHLY)

## What to Build

### 1. BrowserPort trait in `crates/rusvel-core/src/ports.rs`

```rust
#[async_trait]
pub trait BrowserPort: Send + Sync {
    async fn connect(&self, endpoint: &str) -> Result<()>;
    async fn disconnect(&self) -> Result<()>;
    async fn tabs(&self) -> Result<Vec<TabInfo>>;
    async fn observe(&self, tab_id: &str) -> Result<tokio::sync::broadcast::Receiver<BrowserEvent>>;
    async fn evaluate_js(&self, tab_id: &str, script: &str) -> Result<serde_json::Value>;
    async fn navigate(&self, tab_id: &str, url: &str) -> Result<()>;
}
```

### 2. Browser types in `crates/rusvel-core/src/domain.rs`

```rust
pub struct TabInfo { pub id: String, pub url: String, pub title: String, pub platform: Option<String> }
pub enum BrowserEvent {
    DataCaptured { platform: String, kind: String, data: serde_json::Value, tab_id: String },
    Navigation { tab_id: String, url: String },
    TabChanged { tab_id: String, opened: bool },
}
pub enum BrowsingMode { Passive, Assisted, Autonomous, Vision }
```

### 3. New crate `crates/rusvel-cdp/`

Create a new crate (<1500 lines) with:
- `Cargo.toml` — deps: tokio, tokio-tungstenite, serde_json, rusvel-core
- `src/lib.rs` — `CdpClient` struct implementing `BrowserPort`
- `src/transport.rs` — WebSocket connection to `ws://localhost:9222`
- `src/network.rs` — `Network.enable`, intercept `Network.responseReceived`

For now, implement connect/disconnect/tabs. Network interception can return empty events. This is Phase 1 (Passive mode foundation).

### 4. Add to workspace

- Add `rusvel-cdp` to root `Cargo.toml` workspace members

## Files to Modify

- `crates/rusvel-core/src/ports.rs` — add BrowserPort trait
- `crates/rusvel-core/src/domain.rs` — add browser types
- `crates/rusvel-cdp/` (new crate)
- `Cargo.toml` — add workspace member

## Verify

```bash
cargo check -p rusvel-cdp && cargo check --workspace
```

## Depends On

- ADR-014 (done)
- #21 Tool permissions (done — browser tools will be `supervised` mode)

# Task #36: Wire Browser to Engines

> Read this file, then do the task. Only modify files listed below.

## Goal

Connect BrowserPort to harvest/content/GTM engines. Add Upwork extractor. Add browser flow nodes. Add /api/browser/* routes and CLI commands.

## Files to Read First

- `docs/plans/cdp-browser-bridge.md` — full design (Phases 2-4)
- `crates/rusvel-core/src/ports.rs` — BrowserPort trait (added by #25)
- `crates/harvest-engine/src/lib.rs` — HarvestEngine
- `crates/content-engine/src/lib.rs` — ContentEngine
- `crates/flow-engine/src/lib.rs` — FlowEngine, NodeHandler
- `crates/rusvel-cdp/src/lib.rs` — CdpClient (added by #25)

## What to Build

### 1. Upwork extractor in `crates/rusvel-cdp/src/platforms/upwork.rs`

- URL patterns: `*/api/v3/search/jobs*`, `*/api/graphql*`
- Parse job listing JSON → normalize to Opportunity domain type
- Parse client profile → normalize to Lead domain type

### 2. Harvest engine integration

In `crates/harvest-engine/src/lib.rs`:
- Add `browser: Option<Arc<dyn BrowserPort>>` field
- Add `pub fn with_browser(mut self, b: Arc<dyn BrowserPort>) -> Self`
- Add `pub async fn on_data_captured(&self, event: BrowserEvent) -> Result<()>` — normalizes, scores, creates Opportunity

### 3. Flow browser nodes

In `crates/flow-engine/src/lib.rs` or new file:
- `BrowserTriggerNode` — starts flow on `browser.data.captured` events
- `BrowserActionNode` — executes browser action as flow step (with approval gate)

### 4. API routes in `crates/rusvel-api/src/browser.rs` (new)

```
GET  /api/browser/status
POST /api/browser/connect
GET  /api/browser/tabs
POST /api/browser/observe/:tab
GET  /api/browser/captures
POST /api/browser/act
GET  /api/browser/captures/stream    (SSE)
```

### 5. CLI commands

In `crates/rusvel-cli/src/lib.rs`:
```
rusvel browser connect [endpoint]
rusvel browser status
rusvel browser captures [--platform]
```

### 6. Frontend dashboard

`frontend/src/routes/browser/+page.svelte` — connection status, tabs, live capture SSE feed.

## Files to Modify

- `crates/rusvel-cdp/src/platforms/upwork.rs` (new)
- `crates/harvest-engine/src/lib.rs` — add browser field
- `crates/flow-engine/src/lib.rs` — add browser node types
- `crates/rusvel-api/src/browser.rs` (new)
- `crates/rusvel-api/src/lib.rs` — add module + routes
- `crates/rusvel-cli/src/lib.rs` — add browser subcommand
- `frontend/src/routes/browser/+page.svelte` (new)

## Verify

```bash
cargo check --workspace
cd frontend && pnpm check
```

## Depends On

- #25 BrowserPort + CDP
- #30 Computer Use (for vision fallback)

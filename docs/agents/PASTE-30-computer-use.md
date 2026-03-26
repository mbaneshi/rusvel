# Task #30: Claude Computer Use + Browser Tools

> Read this file, then do the task. Only modify files listed below.

## Goal

Add Claude's computer use API support as a vision fallback mode alongside CDP. Register browser_observe/search/act built-in tools.

## Files to Read First

- `crates/rusvel-llm/src/claude.rs` — ClaudeProvider, to_claude_request, from_claude_response
- `crates/rusvel-core/src/ports.rs` — BrowserPort trait (added by #25)
- `crates/rusvel-core/src/domain.rs` — Part enum (Text, ToolCall, ToolResult), BrowserEvent, BrowsingMode
- `docs/plans/cdp-browser-bridge.md` — browser tool design

## What to Build

### 1. Computer use support in `crates/rusvel-llm/src/claude.rs`

Add to ClaudeProvider:
- Beta header: `"anthropic-beta": "computer-use-2025-01-24"` (only when computer use tools present)
- Support `computer_20250124` tool type format in request building:
  ```json
  { "type": "computer_20250124", "name": "computer", "display_width_px": 1024, "display_height_px": 768 }
  ```
- Parse computer use actions from responses: `screenshot`, `left_click`, `type`, `key`, `scroll`
- Support base64 image in tool results:
  ```json
  { "type": "image", "source": { "type": "base64", "media_type": "image/png", "data": "..." } }
  ```

### 2. Image support in `crates/rusvel-core/src/domain.rs`

Add to Part enum (if not already present):
```rust
Image { base64: String, media_type: String },
```

### 3. Browser tools in `crates/rusvel-builtin-tools/src/browser.rs` (new)

Three tools wrapping BrowserPort:
- `browser_observe` — start observing a tab. Params: `{ platform: string }`. Calls BrowserPort::observe.
- `browser_search` — search captured data. Params: `{ query: string, platform?: string }`. Returns matches.
- `browser_act` — execute browser action (requires approval). Params: `{ action: string, target?: string }`. Calls BrowserPort::navigate or evaluate_js. Returns "AWAITING_APPROVAL" for supervised mode.

All with `searchable: true`.

### 4. Register in `crates/rusvel-builtin-tools/src/lib.rs`

Add `pub mod browser;` and `pub async fn register_browser_tools(registry, browser_port: Arc<dyn BrowserPort>)`.

## Files to Modify

- `crates/rusvel-llm/src/claude.rs` — computer use beta header + tool format + image parsing
- `crates/rusvel-core/src/domain.rs` — Part::Image variant
- `crates/rusvel-builtin-tools/src/browser.rs` (new)
- `crates/rusvel-builtin-tools/src/lib.rs` — add module + register fn

## Verify

```bash
cargo check -p rusvel-llm && cargo check -p rusvel-builtin-tools && cargo check --workspace
```

## Depends On

- #25 BrowserPort + CDP
- #28 AG-UI (for streaming screenshots to frontend)

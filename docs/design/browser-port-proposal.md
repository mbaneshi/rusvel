# Browser Port — Attach to Running Browsers for Live Data Extraction

**Status:** Proposal
**Date:** 2026-03-23
**Author:** AI-assisted design

---

## Problem

RUSVEL's Harvest engine currently ingests data only via RSS feeds and manual input. To make the system truly aware of "what is going on in the world," we need infrastructure to:

- Attach to a **running browser** (Chrome, Edge, Arc, Brave — any Chromium-based)
- Extract data from **any platform** the user is logged into (LinkedIn, Twitter/X, GitHub, Gmail, Product Hunt, HackerNews, etc.)
- Feed that data into Harvest, Content, and GTM engines for opportunity discovery, market intelligence, and outreach

The user's existing sessions and cookies must be reused — no separate logins, no API keys for platforms that don't offer public APIs.

---

## Codebase Context

### Current Architecture (Hexagonal)

```
Engines (forge, code, harvest, content, gtm)
    ↓ depend only on traits
rusvel-core (10 port traits + ~40 domain types)
    ↓ implemented by
Adapters (llm, agent, db, event, memory, tool, jobs, auth, config)
    ↓ wired by
rusvel-app (composition root)
```

### Relevant Existing Pieces

| Component | What it does today | Relevance |
|-----------|-------------------|-----------|
| `HarvestSource` trait | `async fn scan() → Vec<RawOpportunity>` | New browser source implements this |
| `RssSource` | Fetches RSS/Atom feeds via `reqwest` | Pattern to follow for new sources |
| `RawOpportunity` | `title, description, url, budget, skills, source_data` | Target struct for extracted data |
| `OpportunityScorer` | Keyword + LLM scoring | Scores browser-extracted opportunities |
| `ObjectStore` | JSON CRUD in SQLite | Stores extracted entities |
| `EventPort` | Append-only event bus | Emits `harvest.browser.page_extracted` events |
| `AgentPort` | LLM orchestration | Parses unstructured page content into structured data |
| `JobPort` | Central async queue | Schedules periodic browser scans |

### Key Rules That Apply

1. **Engines never import adapter crates** — BrowserPort trait goes in `rusvel-core`
2. **Engines use AgentPort, not LlmPort** — page content parsing uses AgentPort
3. **All domain types carry `metadata: serde_json::Value`** — browser-extracted data stores extras in metadata
4. **Each crate < 2000 lines** — `rusvel-browser` must stay focused

---

## Technical Landscape: Rust Browser Automation (2025–2026)

### Crate Comparison

| Crate | Async/Tokio | Attach to Running Browser | CDP Coverage | Maintenance |
|-------|:-----------:|:-------------------------:|:------------:|:-----------:|
| **chromiumoxide** | Yes | Yes | Full (~60K generated lines) | Slowed (last: Nov 2025) |
| **chromey** (fork) | Yes | Yes | Full + fixes | Active (spider-rs team) |
| **headless_chrome** | **No** (threads) | Limited | Partial | Active |
| **fantoccini** | Yes | **No** (WebDriver) | N/A | Active |
| **thirtyfour** | Yes | **No** (WebDriver) | N/A | Active |

**Why WebDriver-based crates don't work:** WebDriver creates new sessions. It cannot attach to the user's existing browser with their logged-in cookies and sessions. Only CDP can do this.

### Chrome DevTools Protocol (CDP) — How It Works

```
User launches Chrome with: --remote-debugging-port=9222

GET http://localhost:9222/json/version  → browser WebSocket URL
GET http://localhost:9222/json          → list of all open tabs

Each tab has a WebSocket URL:
ws://localhost:9222/devtools/page/<guid>
    → Send JSON-RPC commands (DOM, Network, Runtime, Storage domains)
    → Receive real-time events (network requests, page loads, console)
```

**What CDP can extract:**

| Domain | Capabilities |
|--------|-------------|
| **DOM** | Full HTML, query selectors, attributes, text content |
| **Runtime** | Execute arbitrary JS — access localStorage, sessionStorage, any JS state |
| **Network** | Intercept all HTTP requests/responses, read cookies, headers, payloads |
| **Storage** | IndexedDB, Cache Storage, cookies |
| **Page** | Screenshots, PDF, navigation, lifecycle events |

### Alternative: Browser Extension + Native Messaging

A Chrome extension can communicate with a local Rust binary via Chrome's **Native Messaging API** (stdin/stdout JSON protocol). The user clicks a button (or it triggers on navigation), and the extension sends page content to RUSVEL.

**Pros:** No `--remote-debugging-port` needed, works with any Chromium browser out of the box, user-initiated.
**Cons:** Requires extension installation, less powerful than CDP (no network interception), extension store review process.

### Future: WebDriver BiDi

W3C standard combining WebDriver (cross-browser) + CDP (bidirectional events). Already implemented in Chrome and Firefox. No Rust client exists yet. Worth watching but not actionable today.

---

## Recommended Approach

### Phase 1: CDP Adapter via `chromey` (Primary)

**Why `chromey` over `chromiumoxide`:** Same API, actively maintained fork by spider-rs team, stays current with CDP protocol updates.

**Why CDP over alternatives:**
- Only option that attaches to running browsers with existing sessions
- Full network interception for monitoring API calls platforms make
- JS evaluation for accessing SPAs that render client-side
- Real-time events for monitoring page changes

### Phase 2: Browser Extension (Complement)

Lightweight Chrome extension for users who don't want to launch Chrome with debug flags. Sends page content to `localhost:3000/api/harvest/ingest` via Native Messaging or HTTP.

### Phase 3: Static HTTP Scraper (Already Possible)

For public pages that don't need JS rendering, `reqwest + scraper` crate is sufficient. This pattern already exists in `RssSource`.

---

## Proposed Architecture

### New Port Trait (`rusvel-core/src/ports.rs`)

```rust
/// BrowserPort — connect to and extract data from running browsers
#[async_trait]
pub trait BrowserPort: Send + Sync {
    /// List all open tabs/targets in the connected browser
    async fn list_tabs(&self) -> Result<Vec<BrowserTab>>;

    /// Get the full page content (HTML + metadata) of a tab
    async fn extract_page(&self, tab_id: &str) -> Result<PageContent>;

    /// Execute JavaScript in a tab and return the result
    async fn evaluate_js(&self, tab_id: &str, script: &str) -> Result<serde_json::Value>;

    /// Get all cookies for a tab's domain
    async fn get_cookies(&self, tab_id: &str) -> Result<Vec<BrowserCookie>>;

    /// Monitor network requests on a tab (returns recent captured requests)
    async fn capture_network(&self, tab_id: &str) -> Result<Vec<NetworkEntry>>;
}
```

### New Domain Types (`rusvel-core/src/domain.rs`)

```rust
pub struct BrowserTab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub tab_type: String,  // "page", "background_page", "service_worker"
    pub metadata: serde_json::Value,
}

pub struct PageContent {
    pub url: String,
    pub title: String,
    pub html: String,
    pub text: String,           // Extracted visible text
    pub links: Vec<String>,
    pub metadata: serde_json::Value,  // og:tags, meta description, structured data
}

pub struct BrowserCookie {
    pub name: String,
    pub domain: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub metadata: serde_json::Value,  // value intentionally NOT stored — security
}

pub struct NetworkEntry {
    pub url: String,
    pub method: String,
    pub status: u16,
    pub content_type: Option<String>,
    pub response_body: Option<String>,  // Only for JSON/text responses
    pub metadata: serde_json::Value,
}
```

### New Crate: `rusvel-browser`

```
crates/rusvel-browser/
├── Cargo.toml
├── src/
│   ├── lib.rs          # CdpBrowser implements BrowserPort
│   ├── connection.rs   # WebSocket connection to Chrome debug port
│   ├── tab.rs          # Tab enumeration and management
│   └── extractor.rs    # HTML → structured data extraction helpers
```

**Dependencies:**
```toml
[dependencies]
rusvel-core = { path = "../rusvel-core" }
chromey = "0.7"           # CDP client (chromiumoxide fork)
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
scraper = "0.22"          # HTML parsing for extracted pages
anyhow = "1"
async-trait = "0.1"
```

### Integration Points

```
                    ┌─────────────────────┐
                    │  Running Chrome      │
                    │  (--remote-debugging │
                    │   -port=9222)        │
                    └──────────┬──────────┘
                               │ CDP WebSocket
                    ┌──────────▼──────────┐
                    │  rusvel-browser      │
                    │  (CdpBrowser)        │
                    │  implements          │
                    │  BrowserPort         │
                    └──────────┬──────────┘
                               │
              ┌────────────────┼────────────────┐
              ▼                ▼                 ▼
     harvest-engine      content-engine     gtm-engine
     ┌──────────────┐   ┌──────────────┐   ┌────────────┐
     │ BrowserSource │   │ Trend scan   │   │ Contact    │
     │ implements    │   │ from open    │   │ enrichment │
     │ HarvestSource │   │ tabs         │   │ from       │
     │ → scan pages  │   │              │   │ LinkedIn   │
     │ → score opps  │   │              │   │ profiles   │
     └──────────────┘   └──────────────┘   └────────────┘
              │                │                 │
              ▼                ▼                 ▼
         EventPort        EventPort         EventPort
     "harvest.browser     "content.trend    "gtm.contact
      .page_extracted"     .detected"        .enriched"
```

### Harvest Engine Integration: `BrowserSource`

A new `HarvestSource` implementation in `harvest-engine`:

```rust
pub struct BrowserSource {
    browser: Arc<dyn BrowserPort>,
    agent: Arc<dyn AgentPort>,  // For parsing unstructured content
}

impl HarvestSource for BrowserSource {
    async fn scan(&self, config: &HarvestConfig) -> Result<Vec<RawOpportunity>> {
        let tabs = self.browser.list_tabs().await?;

        let mut opportunities = Vec::new();
        for tab in tabs.iter().filter(|t| matches_source_filter(t, config)) {
            let page = self.browser.extract_page(&tab.id).await?;

            // Use AgentPort to parse unstructured page into opportunities
            let parsed = self.agent.run(
                "harvest-parser",
                &format!("Extract freelance opportunities from this page:\n\n{}", page.text),
            ).await?;

            // Parse agent response into RawOpportunity structs
            opportunities.extend(parse_agent_response(&parsed)?);
        }
        opportunities
    }
}
```

### CLI & API Endpoints

```bash
# CLI
cargo run -- harvest scan-browser          # Scan all open tabs
cargo run -- harvest scan-browser --url-filter "linkedin.com"
cargo run -- browser list-tabs             # List open tabs
cargo run -- browser extract <tab-id>      # Extract single tab

# API
GET  /api/browser/tabs                     # List tabs
GET  /api/browser/tabs/{id}/content        # Extract tab content
POST /api/browser/tabs/{id}/extract        # Extract + run through agent
POST /api/harvest/scan-browser             # Full harvest scan via browser
```

### Composition Root Wiring (`rusvel-app/src/main.rs`)

```rust
// Only create browser adapter if --browser-port is provided
let browser: Option<Arc<dyn BrowserPort>> = match args.browser_port {
    Some(port) => Some(Arc::new(CdpBrowser::connect(port).await?)),
    None => None,
};

// Wire into harvest engine
let harvest = HarvestEngine::builder()
    .storage(storage.clone())
    .agent(agent.clone())
    .event(event.clone())
    .browser(browser.clone())  // Optional
    .build();
```

---

## Security Considerations

| Risk | Mitigation |
|------|-----------|
| CDP debug port exposes full browser control | Bind only to localhost (Chrome default). Document this clearly. |
| Extracted cookies could leak credentials | `BrowserCookie` struct intentionally omits `value` field. Only metadata is stored. |
| Network capture could contain auth tokens | Filter out `Authorization` headers and sensitive cookies before storage. |
| Page content may contain PII | Run extracted content through a sanitization step before persisting. |
| Anti-bot detection on platforms | This is YOUR browser with YOUR session — not a bot. No stealth needed. |

---

## User Experience

### Setup (One-time)

The user adds a Chrome launch flag. On macOS:

```bash
# Option A: Launch Chrome with debug port
open -a "Google Chrome" --args --remote-debugging-port=9222

# Option B: Create an alias
alias chrome-debug='open -a "Google Chrome" --args --remote-debugging-port=9222'

# Option C: Modify Chrome's default launch (persistent)
# Add to Chrome shortcut or launchd plist
```

### Daily Use

1. Browse normally — log into LinkedIn, Twitter, GitHub, etc.
2. RUSVEL connects in the background via CDP
3. Harvest engine periodically scans open tabs for opportunities
4. Content engine monitors trends from social feeds
5. GTM engine enriches contacts from LinkedIn profiles
6. Everything flows through the event bus → dashboard

---

## Implementation Plan

### Step 1 — Foundation (~200 lines)
- Add `BrowserPort` trait + domain types to `rusvel-core`
- Create `rusvel-browser` crate with CDP connection via `chromey`
- Implement `list_tabs()` and `extract_page()`

### Step 2 — Harvest Integration (~150 lines)
- Add `BrowserSource` to `harvest-engine`
- Wire into composition root with `--browser-port` CLI flag
- Add `cargo run -- browser list-tabs` command

### Step 3 — Agent-Powered Extraction (~100 lines)
- Use AgentPort to parse unstructured page content
- Create a `harvest-parser` agent persona for structured extraction
- Score extracted opportunities through existing pipeline

### Step 4 — API Endpoints (~100 lines)
- Add `/api/browser/*` routes to `rusvel-api`
- Connect to frontend dashboard

### Step 5 — Browser Extension (Future)
- Chrome extension with Native Messaging
- Push-based: user clicks to send page to RUSVEL
- Auto-mode: send on navigation to configured domains

---

## Decision Record

**Decision:** Use CDP via `chromey` crate as the primary browser attachment mechanism.

**Alternatives considered:**

| Option | Why not |
|--------|---------|
| WebDriver (fantoccini/thirtyfour) | Cannot attach to running browser sessions |
| headless_chrome | Not async/tokio — would need blocking wrappers everywhere |
| Playwright (via Node) | Adds Node.js dependency to a pure Rust stack |
| Browser extension only | Less powerful (no network interception), requires extension store |
| Direct HTTP scraping only | Cannot access JS-rendered SPAs or logged-in sessions |

**Why this fits RUSVEL's architecture:**
- Clean port/adapter separation — `BrowserPort` in core, `CdpBrowser` in adapter
- Engines remain unaware of CDP — they only see the trait
- Optional dependency — system works fine without a browser connected
- Composable — browser data feeds into existing scoring, event, and storage pipelines

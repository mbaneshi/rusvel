# CDP Browser Bridge — Passive Platform Mapping & Agent Actions

> Chrome DevTools Protocol integration for passive data capture from Upwork, LinkedIn, Freelancer + two-way agent communication + data persistence.

**Date:** 2026-03-25
**Status:** Proposed
**Roadmap:** Phase 2 (Revenue Engines) — see `docs/plans/roadmap-v2.md`
**Dependencies:** harvest-engine (wired), gtm-engine (stub), flow-engine (wired), rusvel-event, rusvel-jobs
**Related plans:**
- `docs/plans/flow-engine.md` — CDP events as flow triggers, browser actions as flow nodes
- `docs/plans/capability-engine.md` — Auto-discover platform extractors via `!build`
- `docs/plans/machine-awareness-fs-integration.md` — Sibling pattern: new port + adapter crate
- `docs/design/decisions.md` — ADR-003, ADR-005, ADR-007, ADR-008, ADR-009, ADR-010

---

## Motivation

RUSVEL's harvest-engine and gtm-engine need real-world data from freelance platforms. Instead of fragile scraping or unofficial APIs, we attach to the user's live Chrome session via CDP. This gives us:

- **Real-time data** — intercept API responses as the user browses
- **No credential management** — user is already logged in
- **Low detection risk** — we observe a real session, not a headless bot
- **Two-way actions** — agents can act through the same session (with approval gates)

### Roadmap Alignment

The roadmap (`roadmap-v2.md`) already calls for this:

- **Phase 2 (Revenue Engines):** "harvest-engine: CDP scraping (Upwork adapter), AI scoring, proposal generation"
- **Phase 3 (GoToMarket):** "Email + LinkedIn adapters"
- **Phase 5 (Ecosystem):** "Browser extension for passive harvesting" + "More platform adapters (LinkedIn, Freelancer)"

This plan implements Phase 2's CDP requirement and lays groundwork for Phase 3 and 5.

---

## Target Platforms

| Platform   | Passive Data                                  | Agent Actions                    |
|------------|-----------------------------------------------|----------------------------------|
| Upwork     | Job feeds, client profiles, proposal history  | Apply, message, withdraw         |
| LinkedIn   | Profile views, messages, job posts, network   | Connect, message, endorse        |
| Freelancer | Projects, bids, contest entries, messages     | Bid, message, accept             |

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│  Chrome (--remote-debugging-port=9222)              │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐            │
│  │ Upwork   │ │ LinkedIn │ │Freelancer│  ...tabs    │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘            │
└───────┼─────────────┼───────────┼───────────────────┘
        │ CDP WebSocket (ws://localhost:9222)
        ▼
┌─────────────────────────────────────────────────────┐
│  rusvel-cdp  (new crate, <1500 lines)               │
│                                                     │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────┐  │
│  │ TabManager  │  │NetworkSniffer│  │ DomReader  │  │
│  └─────────────┘  └──────────────┘  └───────────┘  │
│  ┌─────────────┐  ┌──────────────┐                  │
│  │ActionRunner │  │PlatformRouter│                  │
│  └─────────────┘  └──────────────┘                  │
└───────────────────────┬─────────────────────────────┘
                        │ impl BrowserPort (port #20)
                        ▼
┌─────────────────────────────────────────────────────┐
│  rusvel-core  (BrowserPort trait — alongside 19     │
│               existing ports in ports.rs)           │
└───────────────────────┬─────────────────────────────┘
                        │
        ┌───────────────┼───────────────┬───────────────┐
        ▼               ▼               ▼               ▼
  harvest-engine    gtm-engine     forge-engine     flow-engine
  (score opps)     (enrich leads)  (orchestrate)   (DAG triggers)
        │               │               │               │
        ▼               ▼               ▼               ▼
  ┌──────────┐   ┌──────────┐   ┌──────────┐   ┌──────────┐
  │rusvel-db │   │rusvel-vec│   │rusvel-mem│   │rusvel-evt│
  │ (SQLite) │   │ (LanceDB)│   │  (FTS5)  │   │(EventBus)│
  └──────────┘   └──────────┘   └──────────┘   └──────────┘
```

### Pattern: Same as Machine Awareness

This follows the identical integration pattern as `docs/plans/machine-awareness-fs-integration.md`:

| Machine Awareness | CDP Browser Bridge |
|---|---|
| `MachinePort` trait in rusvel-core | `BrowserPort` trait in rusvel-core |
| `rusvel-machine` adapter crate | `rusvel-cdp` adapter crate |
| CLI/HTTP backend to `fs` binary | WebSocket backend to Chrome CDP |
| Injected into code-engine, infra-engine | Injected into harvest-engine, gtm-engine |
| Wired in `rusvel-app/main.rs` | Wired in `rusvel-app/main.rs` |

---

## ADR Compliance

| ADR | How CDP Complies |
|-----|-----------------|
| **ADR-003** Single job queue | Browser actions enqueue as `JobKind::BrowserAction` in the central queue |
| **ADR-005** Event.kind as String | Browser events use string kinds: `browser.data.captured`, `browser.action.executed`, etc. |
| **ADR-006** Engine-internal ports | Platform extractors (Upwork/LinkedIn/Freelancer parsers) stay inside `rusvel-cdp`, not core ports |
| **ADR-007** metadata on all types | Captured data stored in `Opportunity.metadata` / `Lead.metadata` — no schema changes needed |
| **ADR-008** Human approval gates | All autonomous browser actions go through `JobStatus::AwaitingApproval` |
| **ADR-009** Engines use AgentPort only | Engines never call BrowserPort directly for AI decisions — agents use browser_* built-in tools |
| **ADR-010** Engines depend on traits only | harvest-engine and gtm-engine receive `Arc<dyn BrowserPort>`, never `rusvel-cdp` directly |

---

## New Crate: `rusvel-cdp`

**Responsibility:** CDP WebSocket client, tab lifecycle, network interception, JS evaluation, action execution.

**Dependencies:** `tokio-tungstenite`, `serde_json`, `tokio` (broadcast channels)

**Target:** < 1500 lines (per crate size rule)

### Core Types

```rust
pub struct CdpClient {
    endpoint: String,
    sessions: HashMap<String, TabSession>,
}

pub struct TabInfo {
    pub id: String,
    pub url: String,
    pub title: String,
    pub platform: Option<Platform>,
}

pub enum Platform {
    Upwork,
    LinkedIn,
    Freelancer,
    Unknown,
}

pub enum BrowserEvent {
    /// API response intercepted and parsed
    DataCaptured {
        platform: Platform,
        kind: String,          // "job_listing", "profile_view", "message", etc.
        data: serde_json::Value,
        tab_id: String,
        timestamp: DateTime<Utc>,
    },
    /// Page navigated
    Navigation {
        tab_id: String,
        url: String,
    },
    /// Tab opened/closed
    TabChanged {
        tab_id: String,
        event: TabEvent,
    },
}

pub enum BrowsingMode {
    /// Only observe — no actions taken
    Passive,
    /// Suggest actions, human confirms in browser
    Assisted,
    /// Agent acts directly (with approval gate)
    Autonomous,
}
```

### File Layout

```
rusvel-cdp/src/
├── lib.rs              # CdpClient, connect, tab management
├── transport.rs        # WebSocket send/recv, CDP message framing
├── network.rs          # Network.enable, intercept, parse responses
├── dom.rs              # Runtime.evaluate helpers
├── action.rs           # Click, type, navigate helpers (human-like delays)
└── platforms/
    ├── mod.rs          # PlatformExtractor trait, URL routing
    ├── upwork.rs       # URL patterns, API response parsers, action scripts
    ├── linkedin.rs     # URL patterns, API response parsers, action scripts
    └── freelancer.rs   # URL patterns, API response parsers, action scripts
```

---

## New Port Trait: `BrowserPort`

Added to `rusvel-core/src/ports.rs` (becomes port #20, alongside existing 19):

```rust
#[async_trait]
pub trait BrowserPort: Send + Sync {
    /// Connect to a running Chrome instance
    async fn connect(&self, endpoint: &str) -> Result<()>;

    /// List open tabs with detected platform
    async fn tabs(&self) -> Result<Vec<TabInfo>>;

    /// Subscribe to events from a tab (network intercepts, navigations)
    async fn observe(&self, tab_id: &str) -> Result<Receiver<BrowserEvent>>;

    /// Run JavaScript in a tab's context (for DOM extraction)
    async fn evaluate_js(&self, tab_id: &str, script: &str) -> Result<serde_json::Value>;

    /// Navigate a tab to a URL
    async fn navigate(&self, tab_id: &str, url: &str) -> Result<()>;

    /// Execute a platform action (proposal, message, etc.)
    async fn execute_action(&self, tab_id: &str, action: PlatformAction) -> Result<()>;
}
```

---

## Platform Extractors

### Extraction Strategy

**Primary: Network interception** — All three platforms use JSON APIs internally. We intercept `Network.responseReceived` events matching known URL patterns:

| Platform   | API Pattern Examples                                    |
|------------|---------------------------------------------------------|
| Upwork     | `*/api/v3/search/jobs*`, `*/api/graphql*`               |
| LinkedIn   | `*/voyager/api/*`, `*/li/track*`                        |
| Freelancer | `*/api/projects/*`, `*/api/messages/*`                  |

**Fallback: DOM extraction** — When API responses don't contain what we need, use `Runtime.evaluate()` to query the DOM. Each platform module defines CSS selectors for key data points.

### Data Normalization (reusing existing domain types)

Platform-specific data maps to existing RUSVEL domain types — no new types needed (ADR-007 metadata handles platform-specific fields):

```
Upwork Job Listing    → harvest-engine::Opportunity  (metadata: { platform: "upwork", ... })
LinkedIn Job Post     → harvest-engine::Opportunity  (metadata: { platform: "linkedin", ... })
Freelancer Project    → harvest-engine::Opportunity  (metadata: { platform: "freelancer", ... })

Upwork Client Profile → gtm-engine::Lead             (metadata: { platform: "upwork", ... })
LinkedIn Profile      → gtm-engine::Lead             (metadata: { platform: "linkedin", ... })
Freelancer Employer   → gtm-engine::Lead             (metadata: { platform: "freelancer", ... })

Upwork Message        → rusvel-core::Event (kind: "browser.message.upwork")
LinkedIn InMail       → rusvel-core::Event (kind: "browser.message.linkedin")
Freelancer Chat       → rusvel-core::Event (kind: "browser.message.freelancer")
```

### Capability Engine Integration

The Capability Engine (`docs/plans/capability-engine.md`) can auto-discover and install new platform extractors. Example:

```
User: "!build Add Indeed job board scraping to the browser bridge"

Capability Engine:
  1. Searches for Indeed's internal API patterns
  2. Generates a new platform extractor config
  3. Installs as a rule + skill bundle for the harvest department
```

This means platform coverage grows through conversation, not just code commits.

---

## Two-Way Agent Communication

### Passive → Agent (observation)

```
Chrome tab (user browsing Upwork)
  → CDP Network.responseReceived
  → rusvel-cdp parses job listing JSON
  → BrowserEvent::DataCaptured
  → rusvel-event bus: "browser.data.captured"
  → harvest-engine: scores opportunity, creates Proposal draft
  → Job queue: "review_opportunity" (pending human approval — ADR-008)
```

### Agent → Browser (action)

```
Agent decides: "This opportunity matches criteria, draft proposal"
  → forge-engine creates PlatformAction::SubmitProposal { ... }
  → Job queue: JobKind::BrowserAction (status: AwaitingApproval — ADR-008)
  → Human approves in UI
  → Job worker routes to rusvel-cdp.execute_action()
  → CDP: navigate to job page
  → CDP: Runtime.evaluate() fills form fields
  → CDP: clicks submit (or stops at preview for Assisted mode)
  → Event: "browser.action.completed"
```

### Built-in Tools for Agents

Add to `rusvel-builtin-tools` (currently has 9 tools: file_ops, shell, git — browser becomes #10-12):

```rust
// Tool: browser_observe
// Description: Start observing a platform tab for new data
// Input: { "platform": "upwork" | "linkedin" | "freelancer" }
// Output: { "tab_id": "...", "status": "observing" }

// Tool: browser_search
// Description: Search captured platform data
// Input: { "query": "rust developer remote", "platform": "upwork" }
// Output: [{ opportunity }, ...]

// Tool: browser_act
// Description: Execute an action in the browser (requires approval)
// Input: { "action": "apply", "opportunity_id": "...", "message": "..." }
// Output: { "job_id": "...", "status": "pending_approval" }
```

**ADR-009 compliance:** Engines call AgentPort, which has access to these tools. Engines never call BrowserPort directly for decisions — only agents use browser tools.

---

## Flow Engine Integration

The Flow Engine (`docs/plans/flow-engine.md`) already supports DAG workflows with typed nodes. CDP adds two new node types to the `NodeHandler` registry:

### New Flow Node Types

```rust
// 1. Browser Trigger Node — starts a flow when CDP captures data
pub struct BrowserTriggerNode;
impl NodeHandler for BrowserTriggerNode {
    fn node_type(&self) -> &str { "browser_trigger" }
    // Subscribes to EventBus "browser.data.captured" events
    // Fires flow when pattern matches (platform, data kind, score threshold)
}

// 2. Browser Action Node — executes a browser action as a flow step
pub struct BrowserActionNode;
impl NodeHandler for BrowserActionNode {
    fn node_type(&self) -> &str { "browser_action" }
    // Takes input from upstream nodes (opportunity data, draft message)
    // Queues a PlatformAction via BrowserPort
    // Respects ADR-008 approval gates
}
```

### Example Flow: Auto-Score and Draft Proposals

```
[Browser Trigger]  →  [Agent: Score]  →  [Condition: score >= 8?]
   (Upwork job)        (harvest agent)       │
                                        true ↓         false → [end]
                                    [Agent: Draft]  →  [Browser Action: Submit]
                                    (proposal writer)    (with approval gate)
```

This flow can be created visually on the existing `/flows` page (`frontend/src/routes/flows/+page.svelte`) once browser node types are registered.

### Frontend Flow Builder Update

The flow builder already shows node types from `GET /api/flows/node-types`. Adding `browser_trigger` and `browser_action` to the registry makes them immediately available in the node palette — no frontend changes needed for discovery.

---

## Data Persistence Layer

### SQLite (rusvel-db) — Structured Data via ObjectStore

Following the existing `ObjectStore` pattern (ADR-004), browser captures can use the existing object store rather than new tables. However, for query performance on high-volume captures, dedicated tables are better:

```sql
-- Captured platform data
CREATE TABLE browser_captures (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,          -- 'upwork', 'linkedin', 'freelancer'
    kind TEXT NOT NULL,              -- 'job_listing', 'profile', 'message'
    external_id TEXT,                -- Platform-specific ID
    data JSON NOT NULL,              -- Raw parsed payload
    normalized_type TEXT,            -- 'opportunity', 'lead', 'event'
    normalized_id TEXT,              -- FK to opportunities/leads/events
    captured_at TEXT NOT NULL,
    session_id TEXT NOT NULL,
    UNIQUE(platform, external_id)
);

-- Platform actions taken (integrates with JobStore for approval)
CREATE TABLE browser_actions (
    id TEXT PRIMARY KEY,
    platform TEXT NOT NULL,
    action_type TEXT NOT NULL,       -- 'apply', 'message', 'connect'
    target_id TEXT,                  -- browser_captures.id
    payload JSON NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, approved, executed, failed
    job_id TEXT,                     -- FK to jobs table (for approval tracking)
    created_at TEXT NOT NULL,
    executed_at TEXT,
    session_id TEXT NOT NULL
);

-- Active observation sessions
CREATE TABLE browser_sessions (
    id TEXT PRIMARY KEY,
    chrome_endpoint TEXT NOT NULL,
    mode TEXT NOT NULL DEFAULT 'passive', -- passive, assisted, autonomous
    started_at TEXT NOT NULL,
    last_heartbeat TEXT NOT NULL,
    tabs JSON NOT NULL DEFAULT '[]'
);
```

### LanceDB (rusvel-vector) — Semantic Search

Captured opportunities and profiles get embedded via `rusvel-embed` for semantic search:

- "Find opportunities similar to my best-performing proposals"
- "Find leads who match my ideal client profile"
- Cluster analysis across platforms

### FTS5 (rusvel-memory) — Full-Text Search

Session-scoped search over captured text data (existing FTS5 infrastructure):

- "What did that Upwork client say about timeline?"
- Quick filtering of job descriptions by keyword

---

## Wiring into RUSVEL

### main.rs Composition Root

Following the same pattern as all other adapters:

```rust
// In rusvel-app/main.rs composition root
let cdp_client = CdpClient::new();
let browser_port: Arc<dyn BrowserPort> = Arc::new(cdp_client);

// Pass to engines that need it (optional, like event_port pattern)
let harvest = HarvestEngine::new(agent_port, store_port)
    .with_browser(browser_port.clone());  // Optional builder method
let gtm = GtmEngine::new(agent_port, store_port)
    .with_browser(browser_port.clone());

// Register browser tools in tool registry
browser::register(&tool_registry, browser_port.clone()).await;

// Register browser flow nodes
flow_engine.register_node_handler(BrowserTriggerNode::new(browser_port.clone()));
flow_engine.register_node_handler(BrowserActionNode::new(browser_port.clone()));
```

### API Routes (rusvel-api)

New module: `rusvel-api/src/modules/browser.rs` (~300 lines, following pattern of other modules)

```
GET  /api/browser/status          # Connection status, active tabs
POST /api/browser/connect         # Connect to Chrome (endpoint URL)
POST /api/browser/disconnect      # Disconnect
GET  /api/browser/tabs            # List tabs with platform detection
POST /api/browser/observe/:tab    # Start observing a tab
DELETE /api/browser/observe/:tab  # Stop observing
GET  /api/browser/captures        # List captured data (filterable by platform, kind, date)
GET  /api/browser/captures/:id    # Single capture detail + normalized entity
POST /api/browser/act             # Queue a browser action (creates job with AwaitingApproval)
GET  /api/browser/actions         # List actions and their status
PUT  /api/browser/mode            # Set browsing mode (passive/assisted/autonomous)
GET  /api/browser/captures/stream # SSE: real-time capture feed (reuses chat SSE pattern)
```

### CLI Commands (rusvel-cli)

Following existing 3-tier CLI pattern:

```
# Tier 1: One-shot commands
rusvel browser connect [endpoint]     # Connect to Chrome
rusvel browser status                 # Show connection + tab info
rusvel browser captures [--platform]  # List captured data
rusvel browser mode <passive|assisted|autonomous>

# Tier 2: REPL integration
> use browser                         # Switch context in REPL shell
> status                              # Connection + tabs
> captures --platform upwork          # Filtered view
```

### Frontend Pages

```
frontend/src/routes/
└── browser/
    ├── +page.svelte             # Dashboard: connection, tabs, live capture feed (SSE)
    ├── captures/
    │   ├── +page.svelte         # Table of captures, filterable by platform/kind
    │   └── [id]/+page.svelte    # Detail: normalized data + raw payload + linked opportunity
    └── actions/
        └── +page.svelte         # Action queue with approval buttons (integrates with /api/approvals)
```

Follows existing Svelte 5 runes syntax, Tailwind 4, shadcn/ui oklch theming (ADR-012).

---

## Harvest Engine Integration (currently wired)

The harvest engine is already wired and functional. CDP extends it:

**Current state** (`crates/harvest-engine/src/lib.rs`):
```rust
pub struct HarvestEngine {
    storage: Arc<dyn StoragePort>,
    event_port: Option<Arc<dyn EventPort>>,
    agent: Option<Arc<dyn AgentPort>>,
    config: HarvestConfig,
}
```

**With CDP:**
```rust
pub struct HarvestEngine {
    storage: Arc<dyn StoragePort>,
    event_port: Option<Arc<dyn EventPort>>,
    agent: Option<Arc<dyn AgentPort>>,
    browser: Option<Arc<dyn BrowserPort>>,   // NEW — optional
    config: HarvestConfig,
}
```

New methods:
- `observe_platform(platform)` — start observing a tab for opportunities
- `on_data_captured(event)` — called on `browser.data.captured` events, normalizes + scores
- `auto_propose(opportunity)` — draft proposal for high-scoring opportunities (requires approval)

The existing `HarvestSource` trait in `source.rs` is the internal abstraction for data sources. Browser becomes a new `HarvestSource` implementation — ADR-006 says these stay engine-internal.

---

## GTM Engine Integration (currently stub)

The gtm engine has the structure but isn't wired. CDP gives it real data:

**Current state** (`crates/gtm-engine/src/lib.rs`):
```rust
pub struct GtmEngine {
    storage: Arc<dyn StoragePort>,
    events: Arc<dyn EventPort>,
    agent: Arc<dyn AgentPort>,
    jobs: Arc<dyn JobPort>,
    crm: CrmManager,
    outreach: OutreachManager,
    invoices: InvoiceManager,
}
```

CDP integration:
- `CrmManager` — auto-populate leads from browser-captured profiles
- `OutreachManager` — send messages through browser (with approval) via `browser_act` tool
- React to `browser.message.*` events — new message from a lead triggers CRM update

---

## Machine Awareness Synergy

When both CDP and Machine Awareness (`docs/plans/machine-awareness-fs-integration.md`) are wired, RUSVEL can reason holistically:

```
Harvest Engine scores opportunity:
  "Looking for Rust developer to build CLI tool"
  Score: 9/10

  ↓ Cross-reference with MachinePort

Machine Awareness finds:
  "You have 3 CLI projects: codeilus (maturity 72), rusvel (maturity 95), fs (maturity 80)"
  "You've written 12k lines of Rust CLI code"

  ↓ Agent combines both signals

Proposal draft:
  "I've built multiple Rust CLI tools including a 22k-line workspace..."
  Attached: relevant project stats from MachinePort
```

This cross-engine intelligence is Phase 4 in the roadmap but the data layer enables it from day one.

---

## Browsing Modes & Safety

### Passive (default)
- Network interception only — no DOM manipulation, no clicks
- All data flows one way: browser → RUSVEL
- No approval needed
- Zero risk of detection

### Assisted
- Agent suggests actions, displays them in RUSVEL UI
- Human manually executes in browser
- Agent can highlight elements (subtle overlay via CDP)
- Minimal detection risk

### Autonomous
- Agent executes actions directly via CDP
- **Every action goes through approval gate (ADR-008)**
- Rate-limited: configurable delays between actions
- Human-like interaction patterns (typing delays, scroll behavior)
- Higher detection risk — use sparingly
- **legal-engine compliance check** required before enabling per-platform

---

## Legal & Compliance Considerations

- **Passive observation of your own session** — Generally safe. You're reading your own browser data.
- **Automated actions** — LinkedIn ToS explicitly prohibits automation. Upwork is stricter on bots. Freelancer is more lenient.
- **Recommendation:** Default to Passive mode. Use Assisted for most actions. Reserve Autonomous for low-risk actions (saving/bookmarking) only.
- **legal-engine integration** — Before enabling Autonomous mode for a platform, run a compliance check via legal-engine (currently stub, but the gate is in place).

---

## Implementation Phases

### Phase 1: CDP Foundation (rusvel-cdp crate)
- [ ] `BrowserPort` trait in `rusvel-core/src/ports.rs` (port #20)
- [ ] `rusvel-cdp` crate: WebSocket connection to Chrome
- [ ] Tab discovery and lifecycle events
- [ ] Network interception (Network.enable, responseReceived)
- [ ] Basic tests with mock CDP server
- [ ] Wire in `rusvel-app/main.rs` composition root

### Phase 2: Upwork Extractor (first platform — matches roadmap Phase 2)
- [ ] URL pattern matching for Upwork API routes
- [ ] JSON response parsers for job listings, client profiles
- [ ] Normalize to `Opportunity` and `Lead` domain types (ADR-007 metadata)
- [ ] Store captures in `rusvel-db` (new migration)
- [ ] Wire to harvest-engine: `with_browser()` + `on_data_captured()`

### Phase 3: Agent Integration
- [ ] `browser_observe` + `browser_search` + `browser_act` built-in tools
- [ ] Event bus integration (`browser.data.captured` → harvest-engine scoring)
- [ ] API routes: `rusvel-api/src/modules/browser.rs` (12 routes)
- [ ] Frontend: `/browser` dashboard with SSE live feed
- [ ] CLI: `rusvel browser` subcommands

### Phase 4: Flow Engine Integration
- [ ] `BrowserTriggerNode` — flow starts on CDP events
- [ ] `BrowserActionNode` — flow step executes browser action
- [ ] Register both in flow-engine node handler registry
- [ ] Visual builder shows browser nodes in node palette
- [ ] Example flow: Upwork capture → score → draft proposal → submit (with approval)

### Phase 5: Two-Way Actions
- [ ] `PlatformAction` types (apply, message, connect, withdraw)
- [ ] `JobKind::BrowserAction` in job queue (ADR-003)
- [ ] Approval gate integration (ADR-008)
- [ ] CDP action execution: form fill, click, with human-like delays
- [ ] Assisted mode: highlight suggested actions via CDP overlay
- [ ] legal-engine compliance check before Autonomous mode

### Phase 6: LinkedIn + Freelancer (matches roadmap Phase 3 + 5)
- [ ] LinkedIn `voyager/api` response parsers
- [ ] Freelancer API response parsers
- [ ] Cross-platform deduplication (same opportunity on multiple platforms)
- [ ] Semantic search across all platforms via `rusvel-vector` embeddings
- [ ] GTM engine wiring: auto-populate CRM leads from browser captures

### Phase 7: Intelligence Layer (matches roadmap Phase 4)
- [ ] Pattern learning: which opportunities lead to wins (outcome tracking)
- [ ] Auto-tuning scoring criteria based on won/lost proposals
- [ ] Proactive alerts: "High-match opportunity just posted on Upwork"
- [ ] Cross-platform strategy: "This client is also on LinkedIn, connect there first"
- [ ] Machine Awareness cross-reference: match opportunities to local project portfolio

---

## File-by-File Changes

### New Files

| File | Lines (est.) | Purpose |
|---|---|---|
| `crates/rusvel-core/src/browser_types.rs` | ~80 | Domain types: TabInfo, BrowserEvent, Platform, BrowsingMode, PlatformAction |
| `crates/rusvel-cdp/Cargo.toml` | ~25 | New crate manifest |
| `crates/rusvel-cdp/src/lib.rs` | ~100 | CdpClient, BrowserPort impl, connect/disconnect |
| `crates/rusvel-cdp/src/transport.rs` | ~200 | WebSocket send/recv, CDP message framing |
| `crates/rusvel-cdp/src/network.rs` | ~200 | Network.enable, intercept, response body fetch |
| `crates/rusvel-cdp/src/dom.rs` | ~100 | Runtime.evaluate helpers |
| `crates/rusvel-cdp/src/action.rs` | ~150 | Click, type, navigate with human-like delays |
| `crates/rusvel-cdp/src/platforms/mod.rs` | ~50 | PlatformExtractor trait, URL router |
| `crates/rusvel-cdp/src/platforms/upwork.rs` | ~200 | Upwork URL patterns + parsers |
| `crates/rusvel-cdp/src/platforms/linkedin.rs` | ~200 | LinkedIn URL patterns + parsers |
| `crates/rusvel-cdp/src/platforms/freelancer.rs` | ~200 | Freelancer URL patterns + parsers |
| `crates/rusvel-builtin-tools/src/browser.rs` | ~150 | browser_observe, browser_search, browser_act tools |
| `crates/rusvel-api/src/modules/browser.rs` | ~300 | 12 API endpoints for browser data |
| `frontend/src/routes/browser/+page.svelte` | ~200 | Dashboard: connection, tabs, live feed |
| `frontend/src/routes/browser/captures/+page.svelte` | ~150 | Captures table with filters |
| `frontend/src/routes/browser/actions/+page.svelte` | ~150 | Action queue with approval UI |

### Modified Files

| File | Change |
|---|---|
| `crates/rusvel-core/src/ports.rs` | Add `BrowserPort` trait (#20) |
| `crates/rusvel-core/src/lib.rs` | Export `browser_types` module |
| `crates/rusvel-core/src/domain.rs` | Add `BrowserAction` to `JobKind` enum |
| `crates/harvest-engine/src/lib.rs` | Add `browser: Option<Arc<dyn BrowserPort>>` + `with_browser()` |
| `crates/gtm-engine/src/lib.rs` | Add `browser: Option<Arc<dyn BrowserPort>>` + `with_browser()` |
| `crates/flow-engine/src/lib.rs` | Register BrowserTrigger + BrowserAction node types |
| `crates/rusvel-builtin-tools/src/lib.rs` | Add `mod browser` + register in `register_all()` |
| `crates/rusvel-app/src/main.rs` | Construct + wire `CdpClient` as `BrowserPort` |
| `crates/rusvel-app/Cargo.toml` | Add `rusvel-cdp` dependency |
| `crates/rusvel-api/src/routes.rs` | Mount `/api/browser/*` routes |
| `crates/rusvel-cli/src/lib.rs` | Add `browser` subcommand (Tier 1 + Tier 2) |
| `crates/rusvel-db/src/lib.rs` | Add migration for browser_captures, browser_actions, browser_sessions tables |
| `Cargo.toml` | Add `rusvel-cdp` to workspace members |
| `frontend/src/lib/api.ts` | Add browser API client functions |
| `frontend/src/lib/components/nav/` | Add browser link to sidebar navigation |

---

## Dependencies

### Rust Crates (new)
- `tokio-tungstenite` — WebSocket client for CDP (pure Rust, no FFI — single binary stays single binary)
- `url` — URL pattern matching for platform detection

### Already in workspace
- `serde_json` — CDP message serialization
- `tokio` — broadcast channels for event distribution
- `chrono` — timestamps on captures

### External
- Chrome/Chromium launched with `--remote-debugging-port=9222`
- User must be logged into platforms in their browser

---

## Open Questions

1. **Chrome extension vs CDP?** — A companion Chrome extension could provide a more reliable bridge (survives Chrome restarts, permission prompts). CDP is simpler to start with but more fragile. Phase 5 in the roadmap calls for "Browser extension" — could be the upgrade path.
2. **Multiple Chrome profiles?** — Some users have separate profiles for different platforms. Support connecting to multiple CDP endpoints?
3. **Firefox/Arc/Brave support?** — All Chromium-based browsers support CDP. Firefox has its own protocol. Start with Chromium-only?
4. **Offline capture replay?** — Store raw CDP events for replay/reprocessing when extractors improve? Could integrate with `rusvel-memory` FTS5 for historical search.
5. **Capability Engine auto-discovery?** — Can the Capability Engine (`docs/plans/capability-engine.md`) generate new platform extractors from a natural language description? E.g., `!build Add Toptal job scraping`.

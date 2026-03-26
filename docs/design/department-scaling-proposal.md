> **SUPERSEDED** by ADR-014. See department-as-app.md.

# RUSVEL — Scalable Department Architecture Proposal

> Making each department a standalone, extensible powerhouse — ready to absorb
> enormous features from existing projects and future growth.

**Date:** 2026-03-24
**Status:** Proposal
**Author:** AI-assisted design session

---

## Executive Summary

RUSVEL's 12 departments currently share one binary, one process, one SQLite database.
This works for orchestration and lightweight operations, but cannot scale to:

- CDP browser scraping (Smart Harvester — Python + TypeScript)
- Multi-platform content publishing with 6 adapters (ContentForge — Rust)
- AI media generation: image, video, music, speech (Cinefilm — Python + Google ADK)
- Brand management and editorial calendars (content-mbaneshi)

**This proposal introduces the Department Gateway Pattern** — a standard protocol that
lets departments run as in-process Rust crates OR external sidecar services, while God
Agent orchestrates everything through one unified interface.

---

## 1. The Problem

### 1.1 Current State

```
rusvel-app (single binary)
  └── ForgeEngine (only engine wired)
      └── All 12 departments use generic agent chat
          └── No real domain logic executes
```

**What works:**
- DepartmentRegistry with 12 definitions (system prompts, capabilities, tabs)
- 6 parameterized API routes (`/api/dept/{dept}/*`)
- Event bus, hook dispatch, capability engine
- MCP server config per department

**What doesn't scale:**
- Departments are **labels**, not **services** — they share one agent runtime
- No way to run a Python process (harvester scraping) as a department
- No way to merge ContentForge's 11 Rust crates without crate explosion
- No standard contract for what a department must implement
- Media generation (video, image, music) requires GPU-heavy Python services

### 1.2 Existing Assets Not Yet Integrated

| Project | Lines | Stack | Domain | Status |
|---------|-------|-------|--------|--------|
| **ContentForge** | ~22k Rust | Rust, 11 crates, hexagonal | Content publishing | Working: 6 platform adapters, scheduler, MCP, job queue |
| **Smart Harvester** | ~15k Python+TS | FastAPI, Playwright, React | Job scraping + bidding | Working: CDP scraping, 4-agent bid pipeline, 11 MCP tools |
| **Cinefilm** | ~22k Python | FastAPI, Angular, Google ADK | Media production | Working: 9 ports, 12 agents, 31+ tools, Vertex AI / ComfyUI |
| **content-mbaneshi** | ~1k Markdown | Shell scripts, templates | Brand & editorial | Working: templates, calendar, publishing workflow |

**Total: ~60k lines of working code** across 4 repos that map directly to 3 RUSVEL departments.

---

## 2. Design: The Department Gateway Pattern

### 2.1 Core Idea

Every department implements a **DepartmentGateway** trait — whether it runs in-process
(Rust crate) or as an external service (Python/TypeScript over HTTP/MCP).

```
┌─────────────────────────────────────────────────────────┐
│  RUSVEL Core Binary                                     │
│                                                         │
│  ┌─────────┐  ┌──────────┐  ┌──────────────────────┐   │
│  │   God   │  │  Event   │  │  Department           │   │
│  │  Agent  │  │   Bus    │  │  Registry + Gateway   │   │
│  └────┬────┘  └────┬─────┘  └──────────┬───────────┘   │
│       │            │                   │               │
│  ┌────┴────────────┴───────────────────┴────────┐      │
│  │           DepartmentGateway trait             │      │
│  │                                               │      │
│  │  fn status()      → DeptStatus               │      │
│  │  fn execute()     → ActionResult             │      │
│  │  fn capabilities()→ Vec<Capability>          │      │
│  │  fn tools()       → Vec<ToolDefinition>      │      │
│  │  fn health()      → HealthCheck              │      │
│  └──┬─────────┬──────────┬──────────┬───────────┘      │
└─────┼─────────┼──────────┼──────────┼──────────────────┘
      │         │          │          │
 ┌────┴───┐ ┌──┴────┐ ┌───┴───┐ ┌───┴──────────┐
 │Finance │ │Legal  │ │Content│ │  Harvest     │
 │(Rust)  │ │(Rust) │ │(Rust) │ │  (Python)    │
 │in-proc │ │in-proc│ │+Forge │ │  sidecar     │
 └────────┘ └───────┘ └───┬───┘ └───┬──────────┘
                          │         │
                    ┌─────┴───┐ ┌───┴──────────┐
                    │Content- │ │Smart         │
                    │Forge    │ │Harvester     │
                    │(merged) │ │:8147         │
                    └─────────┘ └──────────────┘
```

### 2.2 The DepartmentGateway Trait

```rust
// In rusvel-core/src/ports.rs (new port)

#[async_trait]
pub trait DepartmentGateway: Send + Sync {
    /// Department identity
    fn id(&self) -> &str;
    fn kind(&self) -> EngineKind;

    /// What this department can do
    fn capabilities(&self) -> Vec<DeptCapability>;
    fn tools(&self) -> Vec<ToolDefinition>;

    /// Execute a department action
    async fn execute(&self, action: DeptAction) -> Result<DeptResult>;

    /// Department status summary
    async fn status(&self) -> Result<DeptStatus>;

    /// List domain objects (opportunities, content, invoices, etc.)
    async fn list(&self, kind: &str, filter: ObjectFilter) -> Result<Vec<serde_json::Value>>;

    /// Health check
    async fn health(&self) -> Result<HealthCheck>;

    /// Stream events from this department
    async fn events(&self, filter: EventFilter) -> Result<Vec<Event>>;
}

pub struct DeptAction {
    pub name: String,                    // "score_job", "publish_content", "generate_image"
    pub params: serde_json::Value,       // Action-specific parameters
    pub session_id: Option<SessionId>,
    pub approval: ApprovalStatus,
}

pub struct DeptResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub events: Vec<Event>,              // Side-effect events to emit
    pub jobs: Vec<NewJob>,               // Follow-up jobs to enqueue
}

pub struct DeptStatus {
    pub health: HealthStatus,
    pub summary: String,                 // "12 opportunities, 3 scored >70"
    pub metrics: serde_json::Value,      // Department-specific KPIs
    pub pending_approvals: u32,
}

pub struct DeptCapability {
    pub name: String,                    // "publish", "score", "generate_image"
    pub description: String,
    pub requires_approval: bool,
    pub category: String,                // "content", "media", "scraping"
}
```

### 2.3 Two Gateway Implementations

#### A. InProcessGateway — For Rust-native departments

```rust
// Wraps existing engine crates (finance-engine, legal-engine, etc.)
pub struct InProcessGateway<E: Engine> {
    engine: Arc<E>,
    events: Arc<dyn EventPort>,
    storage: Arc<dyn StoragePort>,
}
```

Departments that are lightweight or Rust-native use this. No network overhead.
The existing engine crates (forge-engine, code-engine, etc.) wrap into this.

#### B. ExternalGateway — For Python/TypeScript sidecar services

```rust
// Bridges to external services via HTTP or MCP
pub struct ExternalGateway {
    id: String,
    kind: EngineKind,
    /// HTTP endpoint for the external service
    base_url: String,
    /// MCP server config (stdio or HTTP)
    mcp_config: Option<McpServerConfig>,
    /// Health check interval
    health_interval: Duration,
    client: reqwest::Client,
}

#[async_trait]
impl DepartmentGateway for ExternalGateway {
    async fn execute(&self, action: DeptAction) -> Result<DeptResult> {
        // POST {base_url}/api/dept/execute
        let resp = self.client.post(&format!("{}/api/dept/execute", self.base_url))
            .json(&action)
            .send().await?;
        Ok(resp.json().await?)
    }

    async fn status(&self) -> Result<DeptStatus> {
        // GET {base_url}/api/dept/status
        let resp = self.client.get(&format!("{}/api/dept/status", self.base_url))
            .send().await?;
        Ok(resp.json().await?)
    }

    fn tools(&self) -> Vec<ToolDefinition> {
        // Cached from last health check or MCP tool listing
        self.cached_tools.read().clone()
    }
}
```

### 2.4 Department Protocol (for external services)

Any external service becomes a RUSVEL department by implementing 5 HTTP endpoints:

```
GET  /api/dept/status              → DeptStatus
POST /api/dept/execute             → DeptResult
GET  /api/dept/capabilities        → Vec<DeptCapability>
GET  /api/dept/tools               → Vec<ToolDefinition>
GET  /api/dept/health              → HealthCheck
GET  /api/dept/events?since=<ts>   → Vec<Event>
```

**Plus optional MCP server** for tool-level integration with Claude agents.

This is a thin wrapper — existing services keep their full API surface.
The department protocol is just the interface RUSVEL uses to orchestrate.

---

## 3. Integration Plan: Existing Repos

### 3.1 ContentForge → Content Department

**Strategy: Merge as Rust crates (same binary)**

ContentForge is already hexagonal Rust with the same architecture patterns.
This is the cleanest integration — merge its crates into RUSVEL's workspace.

```
crates/
├── content-engine/          # Existing RUSVEL crate (enhanced)
│   ├── src/
│   │   ├── lib.rs           # ContentEngine implements DepartmentGateway
│   │   ├── publisher.rs     # Publisher trait (from contentforge-publish)
│   │   ├── adapters/        # 6 platform adapters (devto, twitter, linkedin, etc.)
│   │   ├── scheduler.rs     # Cron + one-off scheduling
│   │   ├── pipeline.rs      # Draft → adapt → review → publish pipeline
│   │   └── agent.rs         # ContentGenerator, ContentAdapter, ThreadSplitter
```

**What merges:**
- `contentforge-core` types → extend `rusvel-core` domain types (ContentItem already exists)
- `contentforge-publish` → becomes content-engine internal module
- `contentforge-schedule` → integrates with RUSVEL's JobPort
- `contentforge-agent` → uses RUSVEL's AgentPort (Claude CLI already implemented)
- `contentforge-db` schema → new migration in rusvel-db
- `contentforge-mcp` tools → registered in RUSVEL's MCP server

**What's added to content-engine:**
- 6 Publisher adapters (DEV.to, Twitter/X, LinkedIn, Mastodon, Bluesky, Medium)
- Content pipeline: generate → adapt per platform → schedule → publish
- Platform account management (credentials via AuthPort)
- Engagement analytics
- Brand templates from content-mbaneshi

**Why merge, not sidecar:**
- Same language (Rust), same architecture (hexagonal)
- ContentForge's types align with rusvel-core's ContentItem
- No GPU or heavy runtime requirements
- Single binary philosophy preserved
- Shared SQLite database, shared event bus

**Migration path:**
1. Add contentforge-publish's Publisher trait + 6 adapters into content-engine
2. Add DB migration for content/adaptations/publications/schedule/analytics tables
3. Wire ContentEngine as InProcessGateway in main.rs
4. Import content-mbaneshi templates as seed data (brand, editorial calendar)
5. Register content MCP tools in RUSVEL's MCP server
6. Add API routes: `/api/dept/content/publish`, `/api/dept/content/schedule`, etc.

### 3.2 Smart Harvester → Harvest Department

**Strategy: External sidecar (Python + TypeScript, too different to merge)**

The harvester requires Playwright (browser automation), Chrome CDP, PostgreSQL,
and Google ADK — none of which belong in a Rust binary.

```
┌──────────────────────────────┐
│  RUSVEL Binary               │
│  ┌────────────────────────┐  │
│  │ ExternalGateway        │  │
│  │  id: "harvest"         │  │
│  │  url: localhost:8147   │  │
│  │  mcp: harvester-mcp   │  │
│  └───────────┬────────────┘  │
└──────────────┼───────────────┘
               │ HTTP + MCP
┌──────────────┼───────────────┐
│  Smart Harvester (Python)    │
│  ┌───────────┴────────────┐  │
│  │ Dept Protocol Wrapper  │  │ ← NEW: thin Flask/FastAPI router
│  │  GET /api/dept/status  │  │    wraps existing endpoints
│  │  POST /api/dept/execute│  │
│  └───────────┬────────────┘  │
│  ┌───────────┴────────────┐  │
│  │ Existing FastAPI :8147 │  │ ← UNCHANGED: 30+ existing endpoints
│  │ MCP Server (stdio)     │  │ ← UNCHANGED: 11 tools
│  │ CDP Scraper (TS)       │  │ ← UNCHANGED: Playwright + Chrome
│  │ 4-Agent Bid Pipeline   │  │ ← UNCHANGED: Analyst → Writer
│  └────────────────────────┘  │
└──────────────────────────────┘
```

**What changes in harvester (minimal):**
- Add 5 department protocol endpoints as a new FastAPI router (~100 lines)
- Map existing endpoints to DeptAction names:
  - `"search_jobs"` → `GET /api/jobs`
  - `"score_job"` → `POST /api/jobs/{id}/score`
  - `"generate_bid"` → `POST /api/jobs/{id}/bid`
  - `"approve_bid"` → `POST /api/jobs/{id}/bid/approve`
  - `"status"` → `GET /api/jobs/stats`

**What changes in RUSVEL:**
- Register harvester as ExternalGateway in departments.toml
- Connect harvester's MCP server for Claude agent tool access
- Harvest department chat gains real tools (score, bid, search)
- God Agent can orchestrate: "Find gigs" → harvest dept → "Draft proposals" → content dept

**Why sidecar, not merge:**
- Different language (Python + TypeScript)
- Requires Chrome CDP (browser process management)
- Requires PostgreSQL (not SQLite)
- Requires Google ADK for agent orchestration
- 118 existing tests depend on this stack
- Already works standalone — wrapping is cheaper than rewriting

### 3.3 Cinefilm → Media Factory (New Department or Content Sub-service)

**Strategy: External sidecar (Python + GPU, fundamentally different runtime)**

Cinefilm is the heaviest service — it runs AI models for image/video/music/speech
generation. It can use Vertex AI (cloud) or ComfyUI (local GPU).

**Two options:**

#### Option A: New "Media" Department (Recommended)

Add a 13th department: **Media** — dedicated to AI-powered content generation.

```
┌──────────────────────────────┐
│  RUSVEL Binary               │
│  ┌────────────────────────┐  │
│  │ ExternalGateway        │  │
│  │  id: "media"           │  │
│  │  url: localhost:8000   │  │
│  └───────────┬────────────┘  │
└──────────────┼───────────────┘
               │ HTTP
┌──────────────┼───────────────┐
│  Cinefilm (Python)           │
│  ┌───────────┴────────────┐  │
│  │ Dept Protocol Wrapper  │  │
│  └───────────┬────────────┘  │
│  ┌───────────┴────────────┐  │
│  │ 12-Agent Hierarchy     │  │
│  │ 31+ Tools              │  │
│  │ 9 Ports + Adapters     │  │
│  │ Vertex AI / ComfyUI    │  │
│  └────────────────────────┘  │
└──────────────────────────────┘
```

**Actions exposed:**
- `"generate_image"` → text-to-image via Imagen/Flux/SDXL
- `"generate_video"` → text-to-video via Veo/AnimateDiff
- `"generate_music"` → composition via Lyria/MusicGen
- `"generate_speech"` → TTS via Chirp/Piper
- `"edit_image"` → inpainting, outpainting, upscaling, virtual try-on
- `"run_workflow"` → character consistency, fashion, scene design

**Why new department, not sub-service of Content:**
- Content = writing, publishing, scheduling (text-centric)
- Media = generating visual/audio assets (compute-heavy, GPU-bound)
- Different scaling needs: Content is CPU-light, Media is GPU-heavy
- Different billing model: Media has per-generation costs
- Clean separation lets you run Media only when needed (save GPU resources)

#### Option B: Content Sub-service

Content department delegates media generation to Cinefilm via HTTP.
Content owns the publishing pipeline; Cinefilm just generates assets.
Simpler from God Agent's perspective (fewer departments to route to),
but conflates two very different operational profiles.

**Recommendation: Option A** — it's cleaner and matches how the industry works
(content teams and production teams are separate even in small agencies).

### 3.4 content-mbaneshi → Seed Data for Content Department

**Strategy: Import as configuration, not as a service**

This repo is content, not code. It becomes seed data:

```
rusvel seed content-brand     # Import brand identity (bio, links, voice)
rusvel seed content-templates  # Import content templates (launch-post, build-story, etc.)
rusvel seed content-calendar   # Import editorial calendar
rusvel seed content-projects   # Import project kits (Codeilus, GlassForge, etc.)
```

**Implementation:**
- Add a `seed_content_defaults()` function in rusvel-app (like existing `seed_defaults()`)
- Templates stored as ObjectStore items (kind: "content_template")
- Calendar entries become scheduled jobs via JobPort
- Brand identity becomes a system rule for Content department
- Project kits become session-scoped context

---

## 4. Architecture Changes Required

### 4.1 New Port: DepartmentGateway

Add to `rusvel-core/src/ports.rs` — the trait defined in Section 2.2.

### 4.2 New Crate: rusvel-gateway

```
crates/rusvel-gateway/
├── src/
│   ├── lib.rs              # Gateway registry + dispatch
│   ├── in_process.rs       # InProcessGateway<E> implementation
│   ├── external.rs         # ExternalGateway (HTTP client) implementation
│   └── protocol.rs         # Department protocol types (DeptAction, DeptResult, etc.)
```

### 4.3 Modified: DepartmentRegistry

Currently, DepartmentRegistry holds `DepartmentDef` (metadata only).
Extend it to also hold `Arc<dyn DepartmentGateway>` per department:

```rust
pub struct DepartmentEntry {
    pub def: DepartmentDef,
    pub gateway: Arc<dyn DepartmentGateway>,
}

pub struct DepartmentRegistry {
    pub departments: Vec<DepartmentEntry>,
}
```

### 4.4 Modified: Department Chat Handler

`rusvel-api/src/department.rs` currently streams all chats through Claude CLI.
With gateways, the handler can:

1. Check if the department has a gateway with real tools
2. Register those tools with the agent runtime
3. The agent can now call `score_job`, `publish_content`, `generate_image` as real tools
4. Results flow back through the gateway

### 4.5 Modified: AppState

```rust
pub struct AppState {
    pub forge: Arc<ForgeEngine>,
    pub sessions: Arc<dyn SessionPort>,
    pub events: Arc<dyn EventPort>,
    pub storage: Arc<dyn StoragePort>,
    pub profile: Option<UserProfile>,
    pub registry: DepartmentRegistry,        // Now holds gateways
    pub embedding: Option<Arc<dyn EmbeddingPort>>,
    pub vector_store: Option<Arc<dyn VectorStorePort>>,
}
```

### 4.6 New: departments.toml Enhanced

```toml
# External department: Smart Harvester
[departments.harvest]
gateway_type = "external"
base_url = "http://localhost:8147"
mcp_command = "python /path/to/harvester/backend/mcp_server.py"
health_interval_secs = 30
auto_start = true              # RUSVEL starts the process
startup_command = "cd /path/to/harvester && make backend"

# External department: Cinefilm Media Factory
[departments.media]
gateway_type = "external"
base_url = "http://localhost:8000"
health_interval_secs = 60
auto_start = false             # User starts manually (GPU-heavy)

# In-process department (default for all Rust engines)
[departments.finance]
gateway_type = "in_process"
# No additional config needed — uses engine crate
```

---

## 5. God Agent Integration

### 5.1 Unified Tool Surface

God Agent sees all department tools through one registry:

```
[forge tools]     — session, mission, goals
[content tools]   — draft, adapt, publish, schedule (from ContentForge)
[harvest tools]   — search_jobs, score, bid, approve (from Harvester MCP)
[media tools]     — generate_image, generate_video, generate_music (from Cinefilm)
[code tools]      — analyze, search, metrics
[finance tools]   — ledger, runway, tax
... (all departments)
```

### 5.2 Cross-Department Workflows

The real power: God Agent chains departments:

```
User: "Find Rust freelance gigs, draft proposals, and create LinkedIn posts about them"

God Agent plan:
  1. harvest.search_jobs(query="Rust", min_score=70)
  2. harvest.generate_bid(job_id=...) for top 3
  3. content.draft(type="proposal", context=bid_data)
  4. content.adapt(platform="linkedin", content_id=...)
  5. content.schedule(platform="linkedin", scheduled_at=tomorrow_9am)

Each step → DepartmentGateway.execute() → real action
```

### 5.3 Event-Driven Orchestration

Departments emit events; other departments react:

```
harvest.job.scored(score=85)
  → hook: content.draft(type="proposal", context=job)

content.published(platform="linkedin")
  → hook: growth.track_engagement(content_id=...)

media.image.generated(id=...)
  → hook: content.attach_media(content_id=..., media_id=...)
```

---

## 6. New Department: Media (13th)

### 6.1 Registration

```rust
// In DepartmentRegistry::defaults()
DepartmentDef {
    id: "media".into(),
    name: "Media".into(),
    title: "Media Production".into(),
    engine_kind: EngineKind::Media,     // New variant
    icon: "film".into(),
    color: "oklch(0.7 0.15 320)".into(),
    system_prompt: "You are the Media Production department. You generate \
        images, videos, music, and speech using AI models. You can create \
        character-consistent content, fashion workflows, and scene designs.".into(),
    capabilities: vec![
        "Image Generation", "Video Generation", "Music Composition",
        "Speech Synthesis", "Image Editing", "Virtual Try-On",
        "Character Consistency", "Scene Design"
    ],
    tabs: vec!["Gallery", "Workflows", "Models", "Usage"],
    quick_actions: vec![
        QuickAction { label: "Generate Image", action: "generate_image" },
        QuickAction { label: "Create Video", action: "generate_video" },
        QuickAction { label: "Compose Music", action: "generate_music" },
    ],
    default_config: LayeredConfig::default(),
}
```

### 6.2 EngineKind Extension

```rust
pub enum EngineKind {
    Forge, Code, Harvest, Content, GoToMarket,
    Finance, Product, Growth, Distribution, Legal, Support, Infra,
    Media,  // NEW
}
```

### 6.3 Frontend Route

The existing `/dept/[id]` dynamic route in SvelteKit automatically handles this.
Media department gets its own page at `/dept/media` with Gallery, Workflows,
Models, and Usage tabs.

---

## 7. Implementation Phases

### Phase 1: Foundation (1-2 weeks)

**Goal:** DepartmentGateway trait + registry integration

1. Add `DepartmentGateway` trait to `rusvel-core/src/ports.rs`
2. Create `rusvel-gateway` crate with `InProcessGateway` and `ExternalGateway`
3. Extend `DepartmentRegistry` to hold `DepartmentEntry` (def + gateway)
4. Modify `dept_chat` handler to query gateway tools
5. Add `EngineKind::Media` variant
6. Update `departments.toml` schema for gateway config
7. Tests: mock gateways, verify dispatch

### Phase 2: Content Department Power-Up (1-2 weeks)

**Goal:** Merge ContentForge capabilities into content-engine

1. Add Publisher trait + 6 platform adapters to content-engine
2. Add DB migration for content publishing tables
3. Wire ContentEngine as `InProcessGateway`
4. Register content tools in MCP server
5. Import content-mbaneshi templates as seed data
6. Add content-specific API routes (publish, schedule, platforms)
7. Tests: publish to mock platforms, verify scheduling

### Phase 3: Harvest Sidecar (1 week)

**Goal:** Connect Smart Harvester as external department

1. Add department protocol router to harvester FastAPI (~100 lines Python)
2. Configure `ExternalGateway` for harvest in departments.toml
3. Register harvester MCP server in RUSVEL
4. Verify God Agent can call harvest tools (search, score, bid)
5. Add cross-department hook: scored job → content draft
6. Tests: end-to-end job discovery → proposal generation

### Phase 4: Media Department (1-2 weeks)

**Goal:** Connect Cinefilm as Media department

1. Add department protocol router to Cinefilm FastAPI (~150 lines Python)
2. Register as `ExternalGateway` with `auto_start = false`
3. Expose key tools: generate_image, generate_video, generate_music, generate_speech
4. Add Media department to frontend (Gallery, Workflows tabs)
5. Cross-department: Media generates assets → Content publishes them
6. Tests: generate image, verify gallery display

### Phase 5: Cross-Department Orchestration (1 week)

**Goal:** God Agent workflows spanning multiple departments

1. Wire event-driven hooks between departments
2. Implement workflow templates: "Find gig → Bid → Post about it"
3. Add approval gates for publish and outreach actions
4. Dashboard: unified view of all department statuses
5. Tests: full cross-department workflow

---

## 8. Reasoning: Why This Approach

### 8.1 Why Gateway Pattern (not microservices, not monolith)

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| **Full monolith** (merge everything into Rust) | Single binary, simple deploy | Can't run Python/TS, rewrite cost enormous | Not viable for Python services |
| **Full microservices** (every dept is a service) | Max independence | Ops overhead for solo founder, defeats "one binary" | Over-engineered |
| **Gateway pattern** (in-process + sidecars) | Rust stays in-process, Python runs alongside, one protocol | Slightly more complex registry | Best of both worlds |

The Gateway Pattern preserves RUSVEL's core philosophy:
- **One binary** for Rust-native departments (no change)
- **Sidecars** only when the technology demands it (Python, GPU, browser automation)
- **One protocol** regardless of where the department runs
- **God Agent** doesn't know or care about the implementation

### 8.2 Why Not Rewrite Python Services in Rust

- Smart Harvester needs Playwright (no Rust equivalent for CDP at this level)
- Cinefilm needs Google ADK + Vertex AI SDK (Python-only)
- ComfyUI API is Python-native
- 118 + 197 = 315 existing tests would be lost
- ~37k lines of working Python code — months of rewrite for zero new capability

### 8.3 Why ContentForge Should Merge (Not Sidecar)

- Same language, same architecture, same patterns
- ContentForge's types map 1:1 to RUSVEL's domain types
- No heavy runtime requirements (no GPU, no browser, no PostgreSQL)
- Publisher adapters are just HTTP clients (reqwest) — trivial to port
- Merging avoids running two Rust binaries with two SQLite databases
- ContentForge's MCP tools become native RUSVEL tools

### 8.4 Why Add Media as a Separate Department

- Content writes; Media creates assets. Different skills, different costs.
- Media is GPU-bound — running it is expensive and optional
- Keeping them separate lets you skip starting Cinefilm when you only need text
- In a real agency, "copywriter" and "video editor" are different roles
- The 12→13 department expansion is trivial (registry is designed for it)

### 8.5 Why Department Protocol Over Raw HTTP

External services keep their full API. The protocol is a thin 5-endpoint wrapper
that RUSVEL uses for orchestration. Benefits:
- **Discoverability** — RUSVEL auto-discovers capabilities and tools
- **Health monitoring** — unified health dashboard
- **Event integration** — departments emit events into RUSVEL's event bus
- **Tool registration** — God Agent sees all tools from all departments
- **Consistency** — same UX whether department is Rust or Python

---

## 9. Migration Safety

### 9.1 Zero Breaking Changes

- Existing 12 departments continue to work exactly as they do today
- The gateway is **additive** — departments without a gateway fall back to current behavior
- No existing API routes change
- No existing tests break

### 9.2 Incremental Adoption

Departments can be upgraded one at a time:
1. Finance gets an InProcessGateway → its tools become real
2. Harvest gets an ExternalGateway → connects to Python scraper
3. Content gets enhanced → merges ContentForge
4. Media added → connects to Cinefilm

Departments not yet upgraded still work through generic agent chat.

### 9.3 Rollback

- External gateways can be disabled in departments.toml
- ContentForge merge is behind a feature flag until stable
- No data migration is destructive (new tables, no schema changes to existing)

---

## 10. Future Extensibility

Once the Gateway Pattern is in place, adding enormous features to any department
is just a matter of:

1. **New platform adapter** (e.g., TikTok, YouTube Shorts) → add to content-engine
2. **New scraping source** (e.g., LinkedIn jobs, GitHub issues) → add to harvester
3. **New media model** (e.g., Sora, Stability AI) → add adapter to Cinefilm
4. **New department entirely** (e.g., HR, Procurement) → one TOML entry + gateway
5. **Third-party plugins** → anyone can implement the 5-endpoint protocol

The system is ready to grow without architectural changes.

---

## 11. File Changes Summary

| File | Change |
|------|--------|
| `crates/rusvel-core/src/ports.rs` | Add DepartmentGateway trait |
| `crates/rusvel-core/src/domain.rs` | Add EngineKind::Media, DeptAction, DeptResult, etc. |
| `crates/rusvel-core/src/registry.rs` | DepartmentEntry (def + gateway), 13th dept |
| `crates/rusvel-gateway/` | **NEW CRATE**: InProcessGateway, ExternalGateway |
| `crates/content-engine/` | Enhanced: Publisher trait, 6 adapters, scheduler, pipeline |
| `crates/rusvel-db/src/migrations/` | New migration for content publishing tables |
| `crates/rusvel-api/src/department.rs` | Query gateway tools, enhanced execute |
| `crates/rusvel-api/src/lib.rs` | AppState with gateway registry |
| `crates/rusvel-app/src/main.rs` | Wire gateways in composition root |
| `departments.toml` | Gateway config for external services |
| `frontend/src/routes/dept/[id]/` | Media department tabs (Gallery, Workflows) |

---

## Appendix A: Repo Reference

| Repo | Path | Key Integration File |
|------|------|---------------------|
| ContentForge | `/Users/bm/contentforge` | `crates/contentforge-publish/src/lib.rs` (Publisher trait) |
| Smart Harvester | `/Users/bm/smart-standalone-harvestor` | `backend/mcp_server.py` (11 MCP tools) |
| Cinefilm (local-agentic-medium) | `/Users/bm/cod/in-progress/local-agentic-medium` | `backend/src/tools/registry.py` (tool registry) |
| Cinefilm (cine-combined) | `/Users/bm/cod/in-progress/cine-combined` | `backend/src/core/ports/` (9 port interfaces) |
| Content System | `/Users/bm/content-mbaneshi` | `templates/` (content templates) |

## Appendix B: Existing Test Counts

| Project | Tests | Notes |
|---------|-------|-------|
| RUSVEL | 197 | Rust (cargo test) |
| ContentForge | ~50 | Rust (cargo test) |
| Smart Harvester | 118 | Python + TypeScript |
| Cinefilm | ~11 | Python (pytest, minimal) |
| **Total preserved** | **~376** | Zero tests lost in this proposal |

# Autonomous Freelance Agency — Design & Implementation Plan

> **Vision:** RUSVEL becomes a fully autonomous freelance agency that discovers opportunities, scores them, scaffolds projects, deploys demos, generates proposals referencing live URLs, and manages client relationships — across multiple platforms, browsers, accounts, and machines.
>
> **Inspiration:** `/Users/bm/smart-standalone-harvestor` (working dual-platform scraper with 4-agent pipeline)
> **Date:** 2026-03-30 | **Status:** Design Phase
> **Depends on:** ADR-015 (Agent Intelligence Layer), ADR-014 (DepartmentApp)

---

## Table of Contents

1. [Current State Audit](#1-current-state-audit)
2. [Target Architecture](#2-target-architecture)
3. [The Full Pipeline](#3-the-full-pipeline)
4. [Phase 1: Bridge the Harvestor](#4-phase-1-bridge-the-harvestor)
5. [Phase 2: Multi-Browser Orchestration](#5-phase-2-multi-browser-orchestration)
6. [Phase 3: Cross-Department Pipeline](#6-phase-3-cross-department-pipeline)
7. [Phase 4: Auto-Capability Creation](#7-phase-4-auto-capability-creation)
8. [Phase 5: Cloud Sync & Multi-Machine](#8-phase-5-cloud-sync--multi-machine)
9. [Phase 6: Autonomous Agency Loop](#9-phase-6-autonomous-agency-loop)
10. [Data Model](#10-data-model)
11. [API Design](#11-api-design)
12. [File Impact Map](#12-file-impact-map)
13. [Sprint Breakdown](#13-sprint-breakdown)

---

## 1. Current State Audit

### Two Systems, Disconnected

#### smart-standalone-harvestor (Python/TS)

| Feature | Status | Detail |
|---------|--------|--------|
| Upwork scraping | **Production** | CDP `__NUXT__` interception + GraphQL + network intercept |
| Freelancer scraping | **Production** | Angular ngrx TransferState + DOM + REST API |
| Chrome Extension | **Production** | MV3, dual-platform, background worker extraction |
| Multi-browser | **Production** | CDP ports 9222-9231, profile isolation via `chrome-profiles.conf` |
| Job data model | **Production** | 58 columns, PostgreSQL, Drizzle ORM migrations |
| 5-component scoring | **Production** | skill_match(0.30) + budget(0.20) + client(0.20) + competition(0.15) + AI_leverage(0.15) |
| 4-agent pipeline | **Production** | Job Analyst → MVP Architect → Bid Strategist → Proposal Writer |
| 6 artifact types | **Production** | job-brief, scorecard, analysis, mvp-design, strategy, proposal |
| Auto-search scheduler | **Production** | Keyword rotation every N minutes via ADK browser agent |
| MCP server | **Production** | 11 tools via FastMCP (search, score, bid, artifacts, stats) |
| REST API | **Production** | 30 endpoints on port 8147 |
| React dashboard | **Production** | Job board, score cards, artifact viewer, settings |
| DNA scan mode | **Production** | All-traffic interception for hidden API discovery |
| Tests | **Production** | 118+ Python tests + 120+ TS tests |

#### RUSVEL Harvest Department (Rust)

| Feature | Status | Detail |
|---------|--------|--------|
| Opportunity scanning | **Mock only** | `MockSource` returns 3 fake opportunities |
| Scoring | **Basic** | LLM-based generic scoring (no weighted components) |
| Proposal generation | **Basic** | Single `harvest_propose` call (not multi-agent) |
| Pipeline tracking | **Working** | Cold→Contacted→Qualified→ProposalSent→Won/Lost |
| Browser capture | **Passive** | Single CDP instance, Upwork/LinkedIn URL detection |
| RAG for outcomes | **Working** | Vector similarity hints when vector store is wired |
| Cross-dept orchestration | **Working** | Flow engine + playbooks + god agent delegation |
| Auto-capability creation | **Working** | `!capability` generates skills/rules/MCP/agents from description |
| Cron scheduling | **Working** | Tick-based scheduler, not wired to harvest |
| Multi-browser | **Missing** | Single CDP connection only |
| Cloud sync | **Missing** | No remote coordination |
| Code scaffolding | **Missing** | Code engine is analysis-only |
| Infra deployment | **Missing** | Infra engine is CRUD skeleton |

### What the Harvestor Has That RUSVEL Doesn't

1. **Real Upwork data extraction** — `__NUXT__` SSR payload interception, the actual technique that works
2. **Freelancer support** — Angular ngrx state parsing
3. **Chrome Extension** — Passive capture while browsing
4. **58-column job model** — Rich client data (hire_rate, total_spent, payment_verified, country)
5. **5-component composite scoring** — Weighted, explainable, tunable
6. **4-agent proposal pipeline** — Each agent has a distinct role and model
7. **6 markdown artifact types** — Structured, cached, viewable
8. **Auto-search scheduler** — Keyword rotation + browser automation
9. **Multi-CDP management** — Profile isolation, port mapping
10. **DNA scan mode** — Discovery of hidden APIs through traffic analysis

### What RUSVEL Has That the Harvestor Doesn't

1. **14-department orchestration** — Code, Infra, Content, GTM, Finance, Legal, etc.
2. **Flow engine** — Petgraph DAG workflows with agent/code/condition nodes
3. **Playbooks** — Multi-step cross-department scenarios
4. **God agent** — Central orchestrator aware of all departments
5. **Capability engine** — Auto-create skills/rules/MCP/agents from natural language
6. **Single binary deployment** — Rust + embedded SvelteKit frontend
7. **Event bus** — Cross-department event propagation
8. **Job queue with approval gates** — Human oversight on outbound actions
9. **Knowledge base / RAG** — Vector-backed semantic search
10. **Tool registry with deferred loading** — 33 tools, searchable on demand

---

## 2. Target Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│                              CLOUD SYNC                                  │
│                    (Dashboard + State Coordination)                       │
│                                                                          │
│   Machine A (local Mac)        Machine B (VPS)        Machine C (VPS)   │
│   ┌──────────────────┐        ┌───────────────┐      ┌──────────────┐  │
│   │ Chrome Profile 1 │        │ Chrome Prof 3 │      │ Chrome Prof 5│  │
│   │ (main Upwork)    │        │ (alt Upwork)  │      │ (Freelancer) │  │
│   │ CDP :9222        │        │ CDP :9222     │      │ CDP :9222    │  │
│   ├──────────────────┤        ├───────────────┤      ├──────────────┤  │
│   │ Chrome Profile 2 │        │ Chrome Prof 4 │      │ Chrome Prof 6│  │
│   │ (LinkedIn)       │        │ (Toptal)      │      │ (alt Freelan)│  │
│   │ CDP :9223        │        │ CDP :9223     │      │ CDP :9223    │  │
│   └────────┬─────────┘        └───────┬───────┘      └──────┬───────┘  │
│            │                          │                      │          │
│            └──────────────────────────┴──────────────────────┘          │
│                                   │                                     │
│                        ┌──────────▼──────────┐                         │
│                        │   CDP POOL MANAGER   │                         │
│                        │   (multi-instance)    │                         │
│                        └──────────┬──────────┘                         │
│                                   │                                     │
│  ┌────────────────────────────────▼────────────────────────────────┐   │
│  │                         RUSVEL CORE                              │   │
│  │                                                                  │   │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │   │
│  │  │ HARVEST  │ │   CODE   │ │  INFRA   │ │ CONTENT  │          │   │
│  │  │          │ │          │ │          │ │          │          │   │
│  │  │ Scan     │→│ Scaffold │→│ Deploy   │→│ Portfolio│          │   │
│  │  │ Score    │ │ Build    │ │ Domain   │ │ Case     │          │   │
│  │  │ Propose  │ │ Test     │ │ SSL      │ │ Study    │          │   │
│  │  │ Track    │ │ CI       │ │ Monitor  │ │ Blog     │          │   │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │   │
│  │       │                                        │                │   │
│  │  ┌────▼─────┐                           ┌──────▼─────┐        │   │
│  │  │   GTM    │                           │   FORGE    │        │   │
│  │  │          │                           │            │        │   │
│  │  │ Outreach │                           │ Orchestrate│        │   │
│  │  │ Followup │                           │ Plan       │        │   │
│  │  │ Invoice  │                           │ Review     │        │   │
│  │  └──────────┘                           └────────────┘        │   │
│  │                                                                │   │
│  │  ┌─────────────────────────────────────────────────────────┐  │   │
│  │  │              CAPABILITY ENGINE                           │  │   │
│  │  │  Analyzes job requirements → auto-creates:               │  │   │
│  │  │  Skills, Rules, MCP servers, Agents, Workflows           │  │   │
│  │  │  per-client, per-job, on-the-fly                         │  │   │
│  │  └─────────────────────────────────────────────────────────┘  │   │
│  └────────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## 3. The Full Pipeline

### End-to-End Freelance Automation Flow

```
TRIGGER: Cron (every 2h) OR manual OR webhook
    │
    ▼
HARVEST DEPT: Scan
    │ CDP scraper extracts Upwork/Freelancer jobs
    │ Chrome Extension captures while browsing
    │ RSS feeds for broad monitoring
    │
    ▼
HARVEST DEPT: Score (5-component)
    │ skill_match (0.30) — does this match my stack?
    │ budget_score (0.20) — is the rate right?
    │ client_score (0.20) — verified? history? rating?
    │ competition  (0.15) — how many proposals? what quality?
    │ ai_leverage  (0.15) — can AI accelerate delivery?
    │
    ├── Score < 60 → Archive, log reason
    │
    ├── Score 60-80 → Queue for daily review
    │
    ▼ Score > 80 → Auto-trigger pipeline
    │
CAPABILITY ENGINE: Analyze & Equip
    │ Read job requirements
    │ Auto-create skills for the client's tech stack
    │ Install MCP servers for their APIs/tools
    │ Generate rules for their coding standards
    │ Wire workflows for their delivery preferences
    │
    ▼
CODE DEPT: Scaffold (if buildable)
    │ git init → template for job's tech stack
    │ Implement core feature (AI-assisted)
    │ Run tests → push to GitHub
    │
    ▼
INFRA DEPT: Deploy Demo
    │ Deploy to Fly.io / Railway / Vercel
    │ Set up domain: demo-{project}.rusvel.dev
    │ SSL + health check
    │ Get live URL
    │
    ▼
CONTENT DEPT: Portfolio Asset
    │ Generate case study from code analysis
    │ Create screenshots / architecture diagram
    │ Publish to portfolio site
    │
    ▼
HARVEST DEPT: Generate Proposal (4-agent pipeline)
    │ Agent 1 (Analyst): Extract real problem, hidden risks
    │ Agent 2 (Architect): Design solution, timeline, deliverables
    │ Agent 3 (Strategist): Price competitively, position strengths
    │ Agent 4 (Writer): Craft personalized proposal
    │   → References live demo URL
    │   → References portfolio case study
    │   → Cites relevant experience
    │   → Addresses specific job requirements
    │
    ▼
HUMAN APPROVAL GATE
    │ Review proposal in RUSVEL dashboard
    │ Edit if needed
    │ Approve → submit (or copy to clipboard)
    │ Reject → feedback → regenerate
    │
    ▼
GTM DEPT: Track & Follow Up
    │ Log proposal submission
    │ Schedule follow-up (3 days)
    │ If client responds → notify
    │ If hired → create project session
    │   → Code dept starts real work
    │   → Finance dept creates invoice
    │   → Legal dept generates contract
    │
    ▼
LEARNING LOOP
    │ Won/Lost outcome recorded
    │ Outcome vectors feed into scorer
    │ Proposal quality scores updated
    │ Capability Engine learns what skills/tools were useful
```

---

## 4. Phase 1: Bridge the Harvestor (2-3 days)

> Don't rebuild — connect. The harvestor works. RUSVEL orchestrates.

### 4.1 MCP Client Connection

RUSVEL already has `rusvel-mcp-client` for connecting to external MCP servers. Connect it to the harvestor's MCP server.

**Configuration:**

```toml
# ~/.rusvel/config.toml
[mcp.harvestor]
transport = "stdio"
command = "uv"
args = ["run", "--directory", "/Users/bm/smart-standalone-harvestor/backend", "python", "mcp_server.py"]
```

Or for a remote harvestor:

```toml
[mcp.harvestor]
transport = "http"
url = "http://vps-1.example.com:8147/mcp"
```

**Tools exposed (11):**
- `search_jobs` — Query jobs by keyword, platform, min_score
- `get_job` — Full job detail + score + bid
- `score_job` — Trigger 5-component scoring
- `generate_bid` — Run 4-agent pipeline
- `approve_bid` / `reject_bid` — Human gate
- `get_job_artifact` — Fetch markdown artifacts (6 types)
- `list_job_artifacts` — Available artifacts for a job
- `get_stats` — Pipeline statistics
- `get_settings` / `update_settings` — Configuration

### 4.2 Harvest Engine Adapter

Create a `HarvestSource` implementation that wraps the MCP client:

```rust
// crates/harvest-engine/src/source/mcp_bridge.rs

pub struct McpBridgeSource {
    mcp_client: Arc<dyn McpClient>,
    server_name: String,
}

#[async_trait]
impl HarvestSource for McpBridgeSource {
    async fn scan(&self, _session: &SessionId) -> Result<Vec<Opportunity>> {
        // Call harvestor's search_jobs tool via MCP
        let result = self.mcp_client
            .call_tool(&self.server_name, "search_jobs", json!({
                "min_score": 60,
                "status": "new",
                "limit": 50,
            }))
            .await?;

        // Map harvestor job format → RUSVEL Opportunity
        parse_harvestor_jobs(&result)
    }
}
```

### 4.3 Webhook Ingest Endpoint

For real-time ingestion when the harvestor discovers new jobs:

```rust
// POST /api/harvest/ingest
// Called by harvestor webhook or Chrome Extension
pub async fn harvest_ingest(
    State(state): State<Arc<AppState>>,
    Json(body): Json<IngestPayload>,
) -> Result<Json<IngestResponse>> {
    // body contains array of jobs in harvestor format
    // Map to RUSVEL Opportunity structs
    // Store in harvest engine pipeline
    // Trigger scoring if auto-score enabled
    // Emit harvest.opportunity.ingested event
}
```

### 4.4 Cron → Scan Wiring

Wire `rusvel-cron` to trigger harvest scans:

```rust
// In job worker, handle ScheduledCron for harvest
JobKind::ScheduledCron if payload.event_kind == "harvest.auto_scan" => {
    let keywords = payload.config.get("keywords").as_array();
    for keyword in keywords {
        harvest_engine.scan_with_query(&session_id, keyword).await?;
    }
}
```

**Cron config (stored in ObjectStore):**

```json
{
    "id": "harvest-auto-scan",
    "schedule": "0 */2 * * *",
    "event_kind": "harvest.auto_scan",
    "payload": {
        "keywords": ["ai developer", "rust engineer", "svelte developer", "llm engineer"],
        "platforms": ["upwork", "freelancer"],
        "min_score": 60
    },
    "enabled": true
}
```

---

## 5. Phase 2: Multi-Browser Orchestration (3-5 days)

### 5.1 CDP Pool Manager

Extend `rusvel-cdp` to manage multiple Chrome instances:

```rust
// crates/rusvel-cdp/src/pool.rs

pub struct CdpPool {
    instances: RwLock<HashMap<String, CdpInstance>>,
}

pub struct CdpInstance {
    pub profile_id: String,          // "baneshi-upwork", "alt-freelancer"
    pub platform: String,            // "upwork", "freelancer", "linkedin"
    pub endpoint: String,            // "ws://localhost:9222" or "ws://vps-1:9222"
    pub machine: String,             // "local", "vps-1", "vps-2"
    pub client: Arc<CdpClient>,
    pub status: InstanceStatus,
}

pub enum InstanceStatus {
    Connected,
    Disconnected,
    Scraping { since: DateTime<Utc> },
    Idle,
}

impl CdpPool {
    /// Connect to all configured Chrome instances
    pub async fn connect_all(configs: &[ChromeProfile]) -> Result<Self> { ... }

    /// Get an idle instance for a specific platform
    pub async fn acquire(&self, platform: &str) -> Result<&CdpInstance> { ... }

    /// Release an instance back to the pool
    pub async fn release(&self, profile_id: &str) { ... }

    /// Execute a scraping task on the best available instance
    pub async fn scrape(&self, platform: &str, query: &str) -> Result<Vec<Job>> { ... }
}
```

### 5.2 Chrome Profile Configuration

```toml
# ~/.rusvel/chrome-profiles.toml

[[profiles]]
id = "baneshi-upwork"
platform = "upwork"
machine = "local"
cdp_port = 9222
user_data_dir = "~/Library/Application Support/Google/Chrome/Profile 1"

[[profiles]]
id = "alt-upwork"
platform = "upwork"
machine = "vps-1"
cdp_endpoint = "ws://vps-1.example.com:9222"

[[profiles]]
id = "baneshi-freelancer"
platform = "freelancer"
machine = "local"
cdp_port = 9223
user_data_dir = "~/Library/Application Support/Google/Chrome/Profile 2"

[[profiles]]
id = "baneshi-linkedin"
platform = "linkedin"
machine = "local"
cdp_port = 9224
user_data_dir = "~/Library/Application Support/Google/Chrome/Profile 3"
```

### 5.3 Platform-Specific Extractors

Port extraction logic from the harvestor's TypeScript code into Rust adapters:

```rust
// crates/rusvel-cdp/src/extractors/upwork.rs

pub struct UpworkExtractor;

impl UpworkExtractor {
    /// Extract jobs from Upwork search page via __NUXT__ state
    pub async fn extract_search_results(client: &CdpClient) -> Result<Vec<RawJob>> {
        // Evaluate JS to extract __NUXT__ payload
        let nuxt_state = client.evaluate_js(
            "JSON.stringify(window.__NUXT__?.state?.['$s_results'] || {})"
        ).await?;

        // Parse and map to RawJob structs
        parse_nuxt_jobs(&nuxt_state)
    }

    /// Extract single job detail from Upwork job page
    pub async fn extract_job_detail(client: &CdpClient) -> Result<JobDetail> {
        let nuxt_state = client.evaluate_js(
            "JSON.stringify(window.__NUXT__?.state || {})"
        ).await?;

        parse_nuxt_job_detail(&nuxt_state)
    }

    /// Intercept GraphQL responses for richer data
    pub async fn setup_network_intercept(client: &CdpClient) -> Result<Receiver<RawJob>> {
        // Listen for /api/graphql/v1 responses
        // Parse job data from GraphQL payloads
    }
}
```

```rust
// crates/rusvel-cdp/src/extractors/freelancer.rs

pub struct FreelancerExtractor;

impl FreelancerExtractor {
    /// Extract from Angular ngrx TransferState
    pub async fn extract_search_results(client: &CdpClient) -> Result<Vec<RawJob>> {
        let ngrx_state = client.evaluate_js(
            "JSON.stringify(window['serverApp-state'] || {})"
        ).await?;

        parse_ngrx_projects(&ngrx_state)
    }
}
```

---

## 6. Phase 3: Cross-Department Pipeline (5-8 days)

### 6.1 Code Department: Project Scaffolding

Extend `code-engine` with project creation capabilities:

```rust
// crates/code-engine/src/scaffold.rs

pub struct ProjectScaffold {
    pub name: String,
    pub tech_stack: Vec<String>,     // ["rust", "axum", "svelte"]
    pub template: TemplateKind,
    pub git_remote: Option<String>,  // GitHub repo URL
}

pub enum TemplateKind {
    RustApi,           // Axum + SQLite
    SvelteApp,         // SvelteKit + Tailwind
    FullStack,         // Rust API + SvelteKit frontend
    PythonApi,         // FastAPI + SQLAlchemy
    StaticSite,        // HTML/CSS/JS
    Custom(String),    // Custom template repo URL
}

impl CodeEngine {
    /// Scaffold a new project from template
    pub async fn scaffold_project(&self, scaffold: ProjectScaffold) -> Result<ProjectInfo> {
        // 1. Create directory
        // 2. git init
        // 3. Apply template (cargo init, pnpm create, etc.)
        // 4. AI-generate initial implementation based on job requirements
        // 5. Run tests
        // 6. git commit + push to GitHub
        // 7. Return ProjectInfo with repo URL
    }
}
```

**New tool:** `code_scaffold`

```rust
ToolDefinition {
    name: "code_scaffold".into(),
    description: "Create a new project from template with AI-generated initial implementation.\n\n\
        WHEN TO USE: When a client job requires building a demo or MVP.\n\
        WHEN NOT TO USE: For analyzing existing code (use code_analyze).\n\n\
        TIPS:\n\
        - Choose template based on job tech stack requirements.\n\
        - AI generates initial code based on job description.\n\
        - Returns git repo URL for deployment.".into(),
    parameters: json!({
        "type": "object",
        "properties": {
            "name": { "type": "string", "description": "Project name (kebab-case)" },
            "tech_stack": { "type": "array", "items": { "type": "string" } },
            "template": { "type": "string", "enum": ["rust_api", "svelte_app", "full_stack", "python_api", "static_site"] },
            "job_description": { "type": "string", "description": "Job requirements to guide AI code generation" },
            "push_to_github": { "type": "boolean", "description": "Create GitHub repo and push" }
        },
        "required": ["name", "tech_stack", "template"]
    }),
}
```

### 6.2 Infra Department: Demo Deployment

Extend `infra-engine` with real deployment:

```rust
// crates/infra-engine/src/deploy.rs

pub struct DeployRequest {
    pub project_path: String,
    pub provider: DeployProvider,
    pub subdomain: Option<String>,  // demo-{name}.rusvel.dev
}

pub enum DeployProvider {
    FlyIo,
    Railway,
    Vercel,
    Cloudflare,   // Workers/Pages
}

pub struct DeployResult {
    pub url: String,              // https://demo-chatbot.rusvel.dev
    pub provider: DeployProvider,
    pub deploy_id: String,
    pub status: DeployStatus,
}

impl InfraEngine {
    pub async fn deploy(&self, req: DeployRequest) -> Result<DeployResult> {
        // 1. Detect project type (Dockerfile, package.json, Cargo.toml)
        // 2. Generate deploy config (fly.toml, railway.json, vercel.json)
        // 3. Execute deployment CLI (flyctl deploy, railway up, vercel deploy)
        // 4. Wait for health check
        // 5. Return live URL
    }
}
```

**New tool:** `infra_deploy`

### 6.3 Content Department: Portfolio Generation

```rust
// New playbook: "Demo Portfolio"
PlaybookDef {
    name: "Demo Portfolio",
    steps: vec![
        PlaybookStep::Agent {
            persona: "CodeAnalyst",
            prompt: "Analyze the project at {{project_path}}. Summarize architecture, \
                     tech stack, key features, and code quality metrics.",
            tools: vec!["code_analyze", "code_search"],
        },
        PlaybookStep::Agent {
            persona: "TechnicalWriter",
            prompt: "Write a case study based on the analysis:\n{{last_output}}\n\n\
                     Include: problem solved, approach, tech decisions, results.\n\
                     Format as a portfolio piece with sections.",
            tools: vec!["content_draft"],
        },
        PlaybookStep::Agent {
            persona: "Publisher",
            prompt: "Publish the case study to the portfolio. \
                     Deploy URL: {{deploy_url}}\n\nContent:\n{{last_output}}",
            tools: vec!["content_publish"],
        },
    ],
}
```

### 6.4 The Master Playbook: Freelance Pipeline

```rust
PlaybookDef {
    name: "Freelance Pipeline",
    description: "End-to-end: discover → score → scaffold → deploy → propose",
    steps: vec![
        // Step 1: Score the opportunity
        PlaybookStep::Agent {
            persona: "JobAnalyst",
            prompt: "Analyze this opportunity:\n{{opportunity_json}}\n\n\
                     Score on 5 components: skill_match, budget, client_quality, \
                     competition, ai_leverage. Return JSON scores.",
            tools: vec!["harvest_score"],
        },

        // Step 2: Decide if worth pursuing
        PlaybookStep::Condition {
            expression: "{{score}} >= 75",
            on_true: "continue",
            on_false: "archive",
        },

        // Step 3: Auto-equip capabilities
        PlaybookStep::Agent {
            persona: "CapabilityScout",
            prompt: "The job requires: {{job_skills}}\n\n\
                     Search for and install:\n\
                     - MCP servers for their tech stack\n\
                     - Skills for their coding patterns\n\
                     - Rules for their quality standards",
            tools: vec!["capability_build"],
        },

        // Step 4: Scaffold demo project
        PlaybookStep::Agent {
            persona: "CodeWriter",
            prompt: "Create a demo project for this job:\n{{job_description}}\n\n\
                     Tech stack: {{job_skills}}\n\
                     Build a minimal working demo showing the core feature.",
            tools: vec!["code_scaffold", "bash"],
        },

        // Step 5: Deploy demo
        PlaybookStep::Agent {
            persona: "InfraEngineer",
            prompt: "Deploy the project at {{project_path}} to get a live URL.\n\
                     Use Fly.io for backend or Vercel for frontend.",
            tools: vec!["infra_deploy"],
        },

        // Step 6: Generate portfolio piece
        PlaybookStep::Flow {
            flow_id: "demo-portfolio",
            input_mapping: json!({
                "project_path": "{{project_path}}",
                "deploy_url": "{{deploy_url}}"
            }),
        },

        // Step 7: Generate proposal (4-agent pipeline)
        PlaybookStep::Sequential {
            steps: vec![
                PlaybookStep::Agent {
                    persona: "JobAnalyst",
                    prompt: "Deep analysis of the job. Identify the real problem, \
                             hidden risks, core deliverables, and AI acceleration points.\n\n\
                             Job: {{job_description}}",
                },
                PlaybookStep::Agent {
                    persona: "MVPArchitect",
                    prompt: "Design the solution. Tech stack, deliverables, timeline.\n\n\
                             Analysis: {{last_output}}\n\
                             Demo URL: {{deploy_url}}",
                },
                PlaybookStep::Agent {
                    persona: "BidStrategist",
                    prompt: "Determine pricing. Consider the demo already built.\n\n\
                             Architecture: {{last_output}}\n\
                             Client budget: {{budget_range}}\n\
                             Competition: {{proposal_count}} proposals",
                },
                PlaybookStep::Agent {
                    persona: "ProposalWriter",
                    prompt: "Write the proposal. Reference:\n\
                             - Live demo: {{deploy_url}}\n\
                             - Portfolio: {{portfolio_url}}\n\
                             - Strategy: {{last_output}}\n\n\
                             Make it personal, specific, and compelling.\n\
                             Open with understanding their problem, not your resume.",
                },
            ],
        },

        // Step 8: Human approval
        PlaybookStep::Approval {
            message: "Review the proposal before submitting.\n\n\
                      Job: {{job_title}}\n\
                      Score: {{score}}/100\n\
                      Demo: {{deploy_url}}\n\n\
                      Proposal:\n{{proposal_markdown}}",
        },

        // Step 9: Track in GTM
        PlaybookStep::Agent {
            persona: "BizDev",
            prompt: "Log this proposal submission:\n\
                     Job: {{job_title}}\n\
                     Platform: {{platform}}\n\
                     Proposed rate: {{proposed_rate}}\n\
                     Schedule 3-day follow-up.",
            tools: vec!["gtm_create_deal", "cron_schedule"],
        },
    ],
}
```

---

## 7. Phase 4: Auto-Capability Creation (2-3 days)

### 7.1 Job-Aware Capability Engine

When a high-score job is discovered, automatically analyze its requirements and create capabilities:

```rust
// Triggered when harvest_score > 80
async fn auto_equip_for_job(
    capability_engine: &CapabilityEngine,
    job: &Opportunity,
    session_id: &SessionId,
) -> Result<CapabilityBundle> {
    let prompt = format!(
        "A client needs: {}\n\n\
         Required skills: {}\n\n\
         Create the following for this job:\n\
         1. An agent persona specialized in the client's domain\n\
         2. Skills (prompt templates) for their common tasks\n\
         3. Rules for their coding standards and quality requirements\n\
         4. MCP servers for any tools/APIs they mention\n\
         5. A workflow template for their delivery process",
        job.description,
        job.skills.join(", "),
    );

    capability_engine.build(&prompt, session_id).await
}
```

### 7.2 God Agent Integration

The God Agent, when it sees a high-score opportunity, should autonomously:

1. Call `!capability` with job requirements
2. Install created skills/rules/MCP to the harvest department
3. Use those capabilities in proposal generation
4. Log what was created for future jobs with similar requirements

```
God Agent sees: "New opportunity: AI chatbot for dental clinic (score: 87)"
    → Thinks: "I need dental industry knowledge and chatbot tools"
    → Calls: !capability "dental clinic AI chatbot with appointment booking"
    → Installs: dental-terms skill, appointment-api MCP, hipaa-compliance rule
    → Uses: these in proposal generation
    → Learns: saves for future dental/healthcare jobs
```

---

## 8. Phase 5: Cloud Sync & Multi-Machine (5-8 days)

### 8.1 Sync Protocol

Each RUSVEL instance (local or VPS) periodically syncs state:

```rust
// POST /api/sync/push — send local state to coordinator
pub struct SyncPush {
    pub machine_id: String,
    pub opportunities: Vec<Opportunity>,
    pub scores: Vec<Score>,
    pub proposals: Vec<Proposal>,
    pub events: Vec<Event>,
    pub timestamp: DateTime<Utc>,
}

// GET /api/sync/pull?since=<timestamp> — get updates from other machines
pub struct SyncPull {
    pub opportunities: Vec<Opportunity>,
    pub scores: Vec<Score>,
    pub proposals: Vec<Proposal>,
    pub events: Vec<Event>,
}
```

### 8.2 Cloud Dashboard

A central dashboard (can be the RUSVEL frontend deployed to a VPS) showing:

- All opportunities across all machines/platforms
- Score leaderboard
- Pipeline Kanban (Cold → Won)
- Active browser instances and their status
- Cross-machine event timeline
- Proposal queue with approval UI

### 8.3 Coordinator Architecture

```
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│  Machine A   │     │  Machine B   │     │  Machine C   │
│  (local Mac) │     │  (VPS EU)    │     │  (VPS US)    │
│              │     │              │     │              │
│  RUSVEL      │     │  RUSVEL      │     │  RUSVEL      │
│  + Chrome×2  │     │  + Chrome×2  │     │  + Chrome×2  │
│  + Harvestor │     │  + Harvestor │     │  + Harvestor │
└──────┬───────┘     └──────┬───────┘     └──────┬───────┘
       │                    │                    │
       └────────────────────┼────────────────────┘
                            │
                   ┌────────▼────────┐
                   │  COORDINATOR    │
                   │  (cloud VPS)    │
                   │                 │
                   │  PostgreSQL     │
                   │  Sync API       │
                   │  Dashboard      │
                   │  Approval Queue │
                   └─────────────────┘
```

---

## 9. Phase 6: Autonomous Agency Loop (ongoing)

### 9.1 Learning from Outcomes

```rust
// When a job is Won or Lost, record the outcome
pub struct OutcomeRecord {
    pub opportunity_id: String,
    pub outcome: Outcome,           // Won, Lost, Withdrawn
    pub proposal_used: String,      // Which proposal version won
    pub feedback: Option<String>,   // Client feedback if available
    pub actual_rate: Option<f64>,   // What they actually paid
    pub skills_used: Vec<String>,   // Which auto-created capabilities were useful
    pub time_to_win: Duration,      // How long from discovery to win
}

// Feed outcomes back into scoring weights
impl OpportunityScorer {
    pub fn update_weights(&mut self, outcomes: &[OutcomeRecord]) {
        // Bayesian update of scoring weights based on actual outcomes
        // Jobs we won → increase weight of matching features
        // Jobs we lost → analyze what was different
    }
}
```

### 9.2 Client Onboarding Flow

When a job is Won:

```
Won notification
    ↓
Forge: Create project session
    ↓
Code: Clone client repo / scaffold project
    ↓
Infra: Set up dev/staging environments
    ↓
Legal: Generate contract from template
    ↓
Finance: Create invoice schedule
    ↓
GTM: Set up milestone tracking
    ↓
Forge: Generate daily standup schedule
```

### 9.3 Multi-Platform Expansion

| Platform | Extraction Method | Priority |
|----------|------------------|----------|
| Upwork | `__NUXT__` + GraphQL (harvestor has this) | Done |
| Freelancer | ngrx + DOM + REST (harvestor has this) | Done |
| LinkedIn Jobs | CDP page extraction | Phase 2 |
| Toptal | CDP + API (invite-only) | Phase 3 |
| AngelList/Wellfound | DOM extraction | Phase 3 |
| GitHub Jobs | API (public) | Phase 2 |
| Remote.co | RSS feed | Phase 1 |
| We Work Remotely | RSS feed | Phase 1 |

---

## 10. Data Model

### Extended Opportunity (merging harvestor's 58 columns)

```rust
pub struct Opportunity {
    // ── Identity ──
    pub id: String,
    pub external_id: String,        // Platform-specific ID (ciphertext for Upwork)
    pub platform: Platform,         // Upwork, Freelancer, LinkedIn, etc.
    pub url: String,
    pub seo_slug: Option<String>,

    // ── Job Details ──
    pub title: String,
    pub description: String,
    pub skills: Vec<String>,
    pub experience_level: Option<String>,  // Entry, Intermediate, Expert
    pub project_type: Option<String>,      // One-time, Ongoing
    pub project_length: Option<String>,    // Less than 1 month, 1-3 months, etc.
    pub weekly_hours: Option<String>,      // Less than 30, 30+, etc.
    pub contract_to_hire: bool,

    // ── Budget ──
    pub budget_type: BudgetType,    // Hourly, Fixed
    pub budget_min: Option<f64>,
    pub budget_max: Option<f64>,
    pub hourly_range: Option<String>,

    // ── Client ──
    pub client_country: Option<String>,
    pub client_city: Option<String>,
    pub client_rating: Option<f64>,
    pub client_hire_rate: Option<f64>,
    pub client_total_spent: Option<f64>,
    pub client_payment_verified: bool,

    // ── Competition ──
    pub proposal_count: Option<u32>,
    pub interviewing: Option<u32>,

    // ── Pipeline ──
    pub stage: PipelineStage,
    pub score: Option<CompositeScore>,

    // ── Timestamps ──
    pub posted_at: Option<DateTime<Utc>>,
    pub first_seen_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,

    // ── Extensible ──
    pub raw_data: serde_json::Value,
    pub metadata: serde_json::Value,
}

pub struct CompositeScore {
    pub total: f64,               // 0-100
    pub skill_match: f64,         // 0-100 (weight: 0.30)
    pub budget_score: f64,        // 0-100 (weight: 0.20)
    pub client_score: f64,        // 0-100 (weight: 0.20)
    pub competition_score: f64,   // 0-100 (weight: 0.15)
    pub ai_leverage_score: f64,   // 0-100 (weight: 0.15)
    pub reasoning: String,        // LLM explanation
}
```

---

## 11. API Design

### New Endpoints

```
# Harvest ingest (from harvestor / extension / webhook)
POST   /api/harvest/ingest              Bulk job ingestion
POST   /api/harvest/ingest/single       Single job from browser

# Multi-browser management
GET    /api/browsers                    List connected Chrome instances
POST   /api/browsers/connect            Connect to CDP endpoint
DELETE /api/browsers/{id}               Disconnect
POST   /api/browsers/{id}/scrape        Trigger scrape on instance

# Code scaffolding
POST   /api/dept/code/scaffold          Create project from template
GET    /api/dept/code/templates         List available templates

# Infra deployment
POST   /api/dept/infra/deploy           Deploy project
GET    /api/dept/infra/deployments       List deployments
DELETE /api/dept/infra/deployments/{id}  Tear down

# Cloud sync
POST   /api/sync/push                   Push local state
GET    /api/sync/pull                    Pull remote state
GET    /api/sync/status                  Sync health

# Freelance pipeline
POST   /api/pipeline/run                Run full freelance pipeline for opportunity
GET    /api/pipeline/runs               List pipeline runs
GET    /api/pipeline/runs/{id}          Pipeline run detail + step results
```

---

## 12. File Impact Map

### New Files

```
crates/harvest-engine/src/
  source/mcp_bridge.rs           MCP bridge to harvestor
  source/rss.rs                  RSS feed source
  scorer/weighted.rs             5-component weighted scorer
  proposal/pipeline.rs           4-agent proposal pipeline

crates/rusvel-cdp/src/
  pool.rs                        Multi-instance CDP pool manager
  extractors/upwork.rs           Upwork __NUXT__ extraction
  extractors/freelancer.rs       Freelancer ngrx extraction
  extractors/linkedin.rs         LinkedIn page extraction

crates/code-engine/src/
  scaffold.rs                    Project scaffolding
  templates/                     Template definitions

crates/infra-engine/src/
  deploy.rs                      Real deployment (Fly.io, Railway, Vercel)
  providers/fly.rs               Fly.io adapter
  providers/railway.rs           Railway adapter
  providers/vercel.rs            Vercel adapter

crates/rusvel-api/src/
  harvest_ingest.rs              Ingest endpoint
  browsers.rs                    Multi-browser management
  sync.rs                        Cloud sync endpoints
  pipeline_runner.rs             (extend) Freelance pipeline
```

### Modified Files

```
crates/harvest-engine/src/lib.rs       Add weighted scorer, pipeline, new sources
crates/rusvel-cdp/src/lib.rs           Add pool manager, extractors
crates/code-engine/src/lib.rs          Add scaffold capabilities
crates/infra-engine/src/lib.rs         Add real deployment
crates/rusvel-core/src/domain.rs       Extend Opportunity struct
crates/rusvel-api/src/lib.rs           Register new routes
crates/rusvel-app/src/main.rs          Wire CDP pool, new engines
crates/dept-harvest/src/lib.rs         Register new tools, personas
crates/dept-code/src/lib.rs            Register scaffold tool
crates/dept-infra/src/lib.rs           Register deploy tool
```

---

## 13. Sprint Breakdown

### Sprint A: Bridge (2-3 days)

| # | Task | Effort |
|---|------|--------|
| A1 | MCP client → harvestor connection config | 2h |
| A2 | `McpBridgeSource` implementing `HarvestSource` | 4h |
| A3 | `/api/harvest/ingest` webhook endpoint | 3h |
| A4 | Cron → harvest_scan wiring with keyword rotation | 3h |
| A5 | Extended `Opportunity` struct (58 columns) | 2h |
| A6 | Test: MCP bridge scan + ingest round-trip | 2h |

### Sprint B: Scoring & Proposals (3-4 days)

| # | Task | Effort |
|---|------|--------|
| B1 | 5-component weighted scorer | 4h |
| B2 | Profile-aware scoring (inject UserProfile into prompt) | 2h |
| B3 | 4 proposal personas (Analyst, Architect, Strategist, Writer) | 3h |
| B4 | Proposal pipeline playbook (4-step sequential) | 4h |
| B5 | 6 artifact types (job-brief, scorecard, analysis, mvp-design, strategy, proposal) | 4h |
| B6 | Score + proposal API routes | 3h |
| B7 | Tests: scoring accuracy, proposal pipeline | 3h |

### Sprint C: Multi-Browser (3-5 days)

| # | Task | Effort |
|---|------|--------|
| C1 | `CdpPool` manager with connect/acquire/release | 4h |
| C2 | Chrome profile config (TOML) | 2h |
| C3 | Upwork extractor (`__NUXT__` + GraphQL) | 6h |
| C4 | Freelancer extractor (ngrx + DOM) | 4h |
| C5 | `/api/browsers` management endpoints | 3h |
| C6 | Auto-scrape cron (rotate across instances) | 3h |
| C7 | Tests: CDP pool, extraction parsing | 4h |

### Sprint D: Cross-Department Pipeline (5-8 days)

| # | Task | Effort |
|---|------|--------|
| D1 | Code engine: `scaffold_project` + `code_scaffold` tool | 6h |
| D2 | Project templates (rust_api, svelte_app, full_stack) | 4h |
| D3 | Infra engine: `deploy` + Fly.io provider | 6h |
| D4 | Content: portfolio generation playbook | 3h |
| D5 | Master "Freelance Pipeline" playbook | 4h |
| D6 | God agent: auto-trigger pipeline on score > 80 | 3h |
| D7 | Approval UI for proposals (review, edit, approve/reject) | 4h |
| D8 | Tests: end-to-end pipeline (mock LLM + mock deploy) | 4h |

### Sprint E: Capability Auto-Creation (2-3 days)

| # | Task | Effort |
|---|------|--------|
| E1 | Job-aware capability engine (analyze job → create bundle) | 4h |
| E2 | Auto-trigger on score > 80 | 2h |
| E3 | Capability learning (log what was useful per outcome) | 3h |
| E4 | God agent integration (autonomous equip → propose) | 3h |

### Sprint F: Cloud Sync (5-8 days)

| # | Task | Effort |
|---|------|--------|
| F1 | Sync push/pull API + conflict resolution | 6h |
| F2 | Coordinator service (PostgreSQL, central state) | 6h |
| F3 | Multi-machine CDP coordination | 4h |
| F4 | Cloud dashboard (unified view) | 6h |
| F5 | Remote Chrome connection (WS over internet) | 4h |

### Sprint G: Learning Loop (ongoing)

| # | Task | Effort |
|---|------|--------|
| G1 | Outcome recording + scorer weight update | 4h |
| G2 | Client onboarding flow (Won → session → project) | 4h |
| G3 | Multi-platform expansion (LinkedIn, Toptal) | 6h |
| G4 | Performance analytics (win rate, avg score, time-to-win) | 4h |

---

## Summary: Effort & Sequencing

```
Sprint A (bridge)        ███         2-3 days  ← START HERE
Sprint B (scoring)       ████        3-4 days  ← parallel with A
Sprint C (multi-browser) █████       3-5 days  ← after A
Sprint D (cross-dept)    ████████    5-8 days  ← after B
Sprint E (auto-cap)      ███         2-3 days  ← after D
Sprint F (cloud sync)    ████████    5-8 days  ← after C
Sprint G (learning)      ████        ongoing   ← after D

Total: ~25-35 days to full autonomous agency
Quick wins (A+B): ~5-7 days for real jobs + smart scoring + AI proposals
```

**Start with Sprint A (bridge the harvestor) + Sprint B (scoring & proposals) in parallel.** In one week you'll have real Upwork jobs flowing into RUSVEL with 5-component scoring and 4-agent proposals — using the harvestor you already have running.

# Chapter 3 — Engines & Departments

## DepartmentApp Trait

```rust
pub trait DepartmentApp: Send + Sync {
    fn manifest(&self) -> DepartmentManifest;   // Pure data, no side effects
    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;      // Default no-op
}
```

### DepartmentManifest Fields

```
Identity:      id, name, description
Visual:        icon, color
Chat:          system_prompt, capabilities, quick_actions
Contributions: routes, commands, tools, personas, skills, rules, jobs
UI:            UiContribution (frontend declarations)
Events:        events_produced, events_consumed
Dependencies:  requires_ports, depends_on (other dept IDs)
Config:        config_schema (JSON Schema), default_config (LayeredConfig)
```

### RegistrationContext

Ports available during `register()`:
- `agent`, `events`, `storage`, `jobs`, `memory`, `sessions`, `config`, `auth`
- Optional: `embedding`, `vector_store`

Registrars for contributions:
- `tools: ToolRegistrar` — register tool definitions + handlers
- `event_handlers: EventHandlerRegistrar` — subscribe to event kinds
- `job_handlers: JobHandlerRegistrar` — handle job kinds

---

## Engine-by-Engine Ground Truth

### 1. ForgeEngine — Agent Orchestration & Mission Planning

**Ports:** AgentPort, EventPort, MemoryPort, StoragePort, JobPort, SessionPort, ConfigPort (7 ports — most complex)

**Implemented:**
- **Mission module:** `mission_today()` generates prioritized daily plan via AI agent. `set_goal()`, `list_goals()`, `review()` — full goal lifecycle with period reviews (day/week/month/quarter).
- **Persona system:** 10 hardcoded personas (CodeWriter, SecurityAuditor, Tester, Reviewer, Architect, FullStack, DevOps, DataEngineer, MLEngineer, TechLead). `hire_persona()` spawns configured agents.
- **SafetyGuard:** Budget checks, circuit breaker pattern.
- **Events:** `mission.goal.created`, `mission.plan.generated`, `mission.review.completed`

**Verdict:** FULLY IMPLEMENTED. Core orchestration engine with real AI-backed planning.

---

### 2. CodeEngine — Code Intelligence

**Ports:** StoragePort, EventPort (2 ports — simplest)

**Implemented:**
- **Parser:** `parse_directory()` walks Rust files, extracts symbols (functions, structs, enums, traits, impls).
- **Graph:** `SymbolGraph::build()` constructs symbol reference graph.
- **Metrics:** Line counts, file metrics, project metrics. Real computation, not stubs.
- **BM25 Search:** `SearchIndex::build()` creates searchable index. `search(query, limit)` returns ranked hits.
- **Events:** `code.analyzed` emitted on analysis completion.

**Verdict:** FULLY IMPLEMENTED. Real code parsing (Rust-only v0), real search indexing.

---

### 3. ContentEngine — Content Creation & Publishing

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **Writer:** `draft()` invokes agent to write content. Real AI-backed generation.
- **Platform Adapters:** LinkedIn, Twitter, Dev.to with real API structures.
- **Adaptation:** `adapt()` transforms content for platform-specific constraints (length limits, formatting).
- **Calendar:** `schedule()` enqueues `JobKind::ContentPublish`.
- **Approval Gate:** ADR-008 — requires approval before publishing.
- **Code-to-Content:** `draft_blog_from_code_snapshot()` bridges code analysis → content.
- **Events:** `content.drafted`, `content.adapted`, `content.scheduled`, `content.published`, `content.reviewed`
- **ContentKind:** LongForm, Tweet, Thread, LinkedInPost, Blog, VideoScript, Email, Proposal

**Verdict:** FULLY IMPLEMENTED. Real drafting, multi-platform adaptation, approval workflow.

---

### 4. HarvestEngine — Opportunity Discovery

**Ports:** StoragePort, EventPort, AgentPort (optional), BrowserPort (optional) (4 ports)

**Implemented:**
- **Scanning:** Pluggable `HarvestSource` trait. `MockSource` provided. Real scanning pipeline.
- **Scoring:** `OpportunityScorer` uses agent-based evaluation (placeholder score if no agent).
- **Pipeline:** Stage progression (Cold → Lead → Qualified → Won).
- **CDP Integration:** `on_data_captured()` handles Upwork browser captures (JSON → Contact/Opportunity).
- **Proposals:** `ProposalGenerator` generates AI-backed proposals.
- **Events:** `harvest.opportunity.*`, `harvest.proposal.*`

**Verdict:** FULLY IMPLEMENTED. Real pipeline management, real browser-data normalization.

---

### 5. FlowEngine — DAG Workflow Automation

**Ports:** StoragePort, EventPort, AgentPort, TerminalPort (optional), BrowserPort (optional) (5 ports)

**Implemented:**
- **Node Registry:** Code, Condition, Agent, BrowserTrigger, BrowserAction node types.
- **Executor:** DAG execution with parallel branches, error handling.
- **Checkpointing:** Pause/resume flows at node boundaries. `resume_flow()`, `retry_node()`.
- **Expression evaluation:** Variable interpolation in node parameters.
- **Events:** `flow.execution.*`

**Verdict:** FULLY IMPLEMENTED. Real DAG executor with checkpointing. Not a stub.

---

### 6. GtmEngine — CRM, Outreach, Invoicing

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **CRM:** Contacts, deals, stage progression (Lead → Qualified → Negotiating → Won).
- **Outreach:** Multi-step sequences with templating, delay scheduling.
- **Invoicing:** Line items, payment tracking, `total_revenue()` (counted only when paid).
- **Events:** `gtm.outreach.sent`, `gtm.deal.updated`, `gtm.invoice.paid`

**Verdict:** FULLY IMPLEMENTED at engine level. Real deal pipeline, real revenue tracking.

---

### 7. FinanceEngine — Ledger, Tax, Runway

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **Ledger:** Income/Expense transactions, `balance()` sums income minus expenses.
- **Tax:** Categories (Income, SelfEmployment, CapitalGains), estimates, `total_liability()`.
- **Runway:** Structure present, detailed projection logic TBD.
- **Events:** `finance.income.recorded`, `finance.expense.recorded`, `finance.tax.estimated`

**Verdict:** SUBSTANTIALLY IMPLEMENTED. Ledger and tax working. Runway structure only.

---

### 8. ProductEngine — Roadmap, Pricing, Feedback

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **Roadmap:** Features with status (Draft → Planned → In Progress → Done), priority levels.
- **Pricing:** Tiers with monthly/annual pricing, feature lists.
- **Feedback:** Collection by kind and source.
- **Events:** `product.feature.created`, `product.milestone.reached`, `product.pricing.updated`

**Verdict:** FULLY IMPLEMENTED. Real CRUD operations for all three sub-domains.

---

### 9. GrowthEngine — Funnels, Cohorts, KPIs

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **Funnel:** Stages (Awareness → Consideration → Decision → Retention), user counts per stage.
- **Cohort:** User cohorts with analysis.
- **KPI:** Named KPIs (MRR, CAC, etc.) with values and units.
- **Events:** `growth.funnel.updated`, `growth.cohort.analyzed`, `growth.kpi.recorded`

**Verdict:** FULLY IMPLEMENTED. Real funnel progression, real KPI tracking.

---

### 10. DistroEngine — Marketplace, SEO, Affiliates

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **Marketplace:** Listings across platforms (crates.io, npm, etc.) with status tracking.
- **SEO:** Keywords with ranking data.
- **Affiliate:** Partners with commission tracking.
- **Events:** `distro.listing.published`, `distro.seo.ranked`, `distro.affiliate.joined`

**Verdict:** FULLY IMPLEMENTED. Real listing management, real SEO tracking.

---

### 11. LegalEngine — Contracts, Compliance, IP

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **Contracts:** Templates (standard_nda, etc.), status tracking lifecycle.
- **Compliance:** Checks by area (GDPR, CCPA, etc.).
- **IP:** Assets (patent, trademark, copyright) with filing tracking.
- **Events:** `legal.contract.created`, `legal.compliance.checked`, `legal.ip.filed`

**Verdict:** FULLY IMPLEMENTED. Real contract lifecycle, real compliance tracking.

---

### 12. SupportEngine — Tickets, Knowledge Base, NPS

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **Tickets:** Priority/status, SLA tracking.
- **Knowledge:** Articles with categories and search.
- **NPS:** Responses with score aggregation.
- **Events:** `support.ticket.created`, `support.ticket.resolved`, `support.article.published`, `support.nps.recorded`

**Verdict:** FULLY IMPLEMENTED. Real ticket management, real NPS calculation.

---

### 13. InfraEngine — Deployments, Monitoring, Incidents

**Ports:** StoragePort, EventPort, AgentPort, JobPort (4 ports)

**Implemented:**
- **Deploy:** Records with status (Pending → Running → Completed/Failed).
- **Monitor:** Health checks with status (Healthy, Degraded, Down).
- **Incidents:** Severity levels, open/resolve lifecycle.
- **Events:** `infra.deploy.completed`, `infra.alert.fired`, `infra.incident.opened/resolved`

**Verdict:** FULLY IMPLEMENTED. Real deployment tracking, real incident management.

---

## Department Wiring Status (ADR-014)

The critical distinction: **engines are implemented** but **departments may not expose them to agents**.

| Department | manifest() | register() | Tools Wired | Event Handlers | Job Handlers | Status |
|:-----------|:----------:|:----------:|:-----------:|:--------------:|:------------:|:------:|
| dept-forge | done | done | 5 | -- | -- | **COMPLETE** |
| dept-code | done | done | 2 | -- | -- | **COMPLETE** |
| dept-content | done | done | yes | 1 (code.analyzed) | 1 (content.publish) | **COMPLETE** |
| dept-harvest | done | done | -- | -- | -- | SKELETON |
| dept-flow | done | done | -- | -- | -- | SKELETON |
| dept-gtm | done | done | -- | -- | -- | SKELETON |
| dept-finance | done | done | -- | -- | -- | SKELETON |
| dept-product | done | done | -- | -- | -- | SKELETON |
| dept-growth | done | done | -- | -- | -- | SKELETON |
| dept-distro | done | done | -- | -- | -- | SKELETON |
| dept-legal | done | done | -- | -- | -- | SKELETON |
| dept-support | done | done | -- | -- | -- | SKELETON |
| dept-infra | done | done | -- | -- | -- | SKELETON |

### What "SKELETON" Means

- Engine is created and stored in `OnceLock` during `register()`
- Manifest returns valid data with correct port requirements, system prompts, capabilities
- **BUT no tools are registered** — agents cannot invoke engine methods
- The engine is accessible programmatically via the `engine()` accessor
- The department participates in chat (system prompt is set) but has no tool-backed actions

### What "COMPLETE" Means

- Tools registered in `RegistrationContext::tools` with async handlers
- Handlers call real engine methods (e.g., `forge.mission_today()`, `code.analyze()`)
- Cross-engine event reactions wired (content subscribes to `code.analyzed`)
- Job processing connected (content handles `content.publish` jobs)

### The Gap

10 departments have fully implemented engines but zero tool wiring. This means:
- The AI agent in these departments can chat but cannot take domain actions
- Engine methods are callable from API routes and CLI, but not from agent tool-use loops
- Wiring each department requires: define ToolDefinition, write handler closure, register in `register()`

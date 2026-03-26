> **PARTIALLY SUPERSEDED** — Mission/Ops/Connect engines removed.

# RUSVEL — Engine Specifications

> Detailed spec for each domain engine. What it does, what ports it uses, and what it produces.

---

## Engine Contract

Every engine implements:

```rust
#[async_trait]
pub trait Engine: Send + Sync {
    fn name(&self) -> &str;
    fn capabilities(&self) -> Vec<Capability>;
    async fn initialize(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
    async fn health(&self) -> Result<HealthStatus>;
}
```

Engines receive ports via constructor injection. They NEVER depend on concrete adapter types.

---

## 1. Forge Engine

**Purpose:** Orchestrate AI agents to do real work across all other engines.

**Ports used:** AgentPort, LlmPort, ToolPort, EventPort, MemoryPort, SessionPort, SchedulePort, AutomationPort

**Capabilities:**
- `agent.create` — Create an agent from a persona + config
- `agent.run` — Execute an agent with input, streaming output
- `agent.stop` — Stop a running agent
- `workflow.run` — Execute multi-agent workflows (Sequential, Parallel, Loop, Graph)
- `persona.list` — List available personas
- `persona.hire` — Activate a persona for the current session

**Domain types:**
```rust
Persona { id, name, division, system_prompt, tools, budget_limit }
AgentRun { id, agent_id, persona_id, input, output, events, cost, duration }
Workflow { id, name, steps: Vec<WorkflowStep>, error_strategy }
WorkflowStep::Agent { persona, input_mapping }
WorkflowStep::Parallel { agents: Vec<WorkflowStep> }
WorkflowStep::Conditional { condition, then, else_ }
WorkflowStep::Loop { agent, until_condition, max_iterations }
```

**Safety (built-in, not optional):**
- CircuitBreaker: open after N consecutive failures, half-open after cooldown
- RateLimiter: token bucket per LLM provider
- CostTracker: per-agent and total budget enforcement
- LoopDetector: detect and break infinite agent loops

**Tables:** `forge_personas`, `forge_runs`, `forge_workflows`, `forge_safety_state`

---

## 2. Code Engine

**Purpose:** Understand, analyze, transform, and learn from any codebase.

**Ports used:** StoragePort, EventPort, LlmPort, MemoryPort

**Capabilities:**
- `code.parse` — Parse a codebase into AST (tree-sitter)
- `code.graph` — Build dependency graph, detect communities
- `code.metrics` — Compute complexity, churn, coupling
- `code.analyze` — Detect anti-patterns, suggest improvements
- `code.search` — Full-text BM25 symbol search
- `code.narrate` — Generate human-readable explanations via LLM
- `code.learn` — Generate learning paths and quizzes
- `code.transpile` — Transform code (C→Rust, etc.)

**Domain types:**
```rust
Codebase { path, language_breakdown, file_count, total_lines }
Symbol { name, kind, file, line, references }
DependencyGraph { nodes: Vec<Module>, edges: Vec<Dependency> }
CodeMetrics { complexity, churn_rate, coupling_score, test_coverage }
AntiPattern { kind, location, severity, suggestion }
LearningPath { chapters: Vec<Chapter>, estimated_time }
```

**Tables:** `code_codebases`, `code_symbols`, `code_metrics`, `code_analyses`

---

## 3. Harvest Engine

**Purpose:** Find gigs, jobs, and opportunities from everywhere.

**Ports used:** HarvestPort, LlmPort, StoragePort, EventPort, SchedulePort, MemoryPort

**Capabilities:**
- `harvest.scan` — Scan a source for new opportunities
- `harvest.score` — AI-score opportunities by fit
- `harvest.propose` — Generate tailored proposal for an opportunity
- `harvest.pipeline` — View opportunity pipeline with stages
- `harvest.track` — Update opportunity status

**Domain types:**
```rust
HarvestSource { kind: SourceKind, config, last_scan, credentials_key }
SourceKind::Upwork | SourceKind::Freelancer | SourceKind::LinkedIn | SourceKind::GitHub | SourceKind::Custom
RawOpportunity { source, raw_data, extracted_at }
Opportunity { id, source, title, description, budget, skills, score, status, proposed_at, metadata }
Proposal { id, opportunity_id, body, tone, estimated_value, sent_at }
OpportunityStatus::Discovered | Scored | Qualified | Proposed | Won | Lost | Archived
```

**Pipeline flow:**
```
Source → Scan → RawOpportunity → Extract → Opportunity → Score → Qualify → Propose → Track
```

**Tables:** `harvest_sources`, `harvest_opportunities`, `harvest_proposals`, `harvest_scans`

---

## 4. Content Engine

**Purpose:** Write once, adapt per platform, schedule, publish, track.

**Ports used:** PublishPort, LlmPort, StoragePort, EventPort, SchedulePort, MemoryPort

**Capabilities:**
- `content.draft` — Create content from a topic/brief
- `content.adapt` — Adapt content for specific platforms
- `content.schedule` — Schedule content for future publishing
- `content.publish` — Publish to one or more platforms
- `content.metrics` — Get engagement metrics for published content
- `content.calendar` — View content calendar

**Domain types:**
```rust
ContentPiece { id, title, body, format, tags, status, variants, schedule, metrics }
ContentFormat::Markdown | Thread | ShortPost | LongPost | Video | Newsletter
PlatformVariant { platform, adapted_body, char_count, media }
Platform::Twitter | LinkedIn | DevTo | Medium | YouTube | Substack | Reddit | HackerNews
ContentSchedule { publish_at, platforms, recurrence }
PostMetrics { views, likes, comments, shares, clicks, fetched_at }
ContentStatus::Draft | Adapted | Scheduled | Published | Archived
```

**Pipeline flow:**
```
Topic → Draft → Adapt(per platform) → Review → Schedule → Publish → Track Metrics
```

**Tables:** `content_pieces`, `content_variants`, `content_schedules`, `content_metrics`

---

## 5. Ops Engine

**Purpose:** Run your solo business — CRM, pipeline, invoicing, knowledge.

**Ports used:** StoragePort, EventPort, LlmPort, MemoryPort, SchedulePort

**Capabilities:**
- `ops.contacts` — Manage contacts (clients, contractors, partners)
- `ops.pipeline` — Sales/deal pipeline with stages
- `ops.invoice` — Create and track invoices
- `ops.sop` — Standard operating procedures
- `ops.knowledge` — Knowledge base articles
- `ops.spend` — Track AI and tool spending

**Domain types:**
```rust
Contact { id, name, email, company, role, tags, notes, deals }
Deal { id, contact_id, title, value, stage, probability, close_date }
DealStage::Lead | Qualified | Proposal | Negotiation | Won | Lost
Invoice { id, contact_id, deal_id, items, total, status, due_date, paid_at }
InvoiceStatus::Draft | Sent | Paid | Overdue | Cancelled
SOP { id, title, steps, category, last_updated }
KnowledgeArticle { id, title, body, tags, category }
SpendRecord { id, provider, model, tokens, cost, timestamp }
```

**Tables:** `ops_contacts`, `ops_deals`, `ops_invoices`, `ops_sops`, `ops_knowledge`, `ops_spend`

---

## 6. Mission Engine

**Purpose:** Know what to do and track progress.

**Ports used:** LlmPort, MemoryPort, StoragePort, EventPort, SchedulePort

**Capabilities:**
- `mission.today` — Generate AI daily brief from goals + all engine states
- `mission.goals` — CRUD for goals (quarterly/monthly/weekly)
- `mission.review` — Periodic review (daily/weekly/monthly)
- `mission.decide` — Log a decision with context
- `mission.progress` — Track velocity and progress analytics

**Domain types:**
```rust
Goal { id, description, horizon, deadline, progress, sub_goals, status }
Horizon::Quarterly | Monthly | Weekly | Daily
DailyPlan { date, focus_areas, tasks, energy_level, notes }
Decision { id, title, context, options, chosen, outcome, decided_at }
Review { id, period, accomplishments, blockers, insights, next_actions, reviewed_at }
GoalStatus::Active | Completed | Abandoned | Deferred
```

**The `today` command pulls from ALL engines:**
```
Goals (mission) + Pipeline (ops) + Opportunities (harvest) +
Scheduled content (content) + Pending follow-ups (connect) →
LLM generates prioritized daily plan
```

**Tables:** `mission_goals`, `mission_plans`, `mission_decisions`, `mission_reviews`

---

## 7. Connect Engine

**Purpose:** Build and maintain professional relationships.

**Ports used:** StoragePort, EventPort, LlmPort, SchedulePort, MemoryPort, PublishPort

**Capabilities:**
- `connect.contacts` — Contact management with relationship scoring
- `connect.outreach` — Create and run outreach campaigns
- `connect.sequence` — Multi-step email/message sequences
- `connect.followup` — Schedule and track follow-ups
- `connect.templates` — Outreach templates with AI personalization

**Domain types:**
```rust
Relationship { contact_id, score, interactions, last_contact, next_followup }
OutreachCampaign { id, name, target_criteria, sequence, status, stats }
Sequence { steps: Vec<SequenceStep> }
SequenceStep { delay, channel, template, condition }
Channel::Email | LinkedIn | Twitter | Custom
FollowUp { id, contact_id, due_date, context, completed }
OutreachTemplate { id, name, subject, body, variables }
CampaignStatus::Draft | Active | Paused | Completed
```

**Tables:** `connect_relationships`, `connect_campaigns`, `connect_sequences`, `connect_followups`, `connect_templates`

---

## Cross-Engine Communication

Engines communicate via **EventPort**, not direct calls. Example flow:

```
1. harvest-engine emits: OpportunityScored { id, score: 0.9, skills: ["rust"] }
2. forge-engine subscribes, triggers workflow: "high-score opportunity"
3. forge-engine runs content-engine agent: draft proposal
4. content-engine emits: ContentDrafted { id, type: "proposal" }
5. forge-engine runs connect-engine agent: send proposal
6. connect-engine emits: OutreachSent { contact_id, content_id }
7. ops-engine subscribes, creates deal in pipeline
8. mission-engine subscribes, updates daily plan
```

No engine imports another engine's types. They share types from `rusvel-core`.

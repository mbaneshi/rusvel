# Next-Level Features — Inspired by TheGenAICircle.com

> Date: 2026-03-25
> Status: Proposed
> Inspiration: https://www.thegenaicircle.com/ — community-driven AI knowledge platform
> Related docs:
>   - `docs/design/department-as-app.md` — DepartmentApp trait + DepartmentManifest (ADR-014)
>   - `docs/plans/next-level-proposals.md` — P1-P12 enhancements
>   - `docs/plans/agent-orchestration.md` — delegate_agent, event triggers, workflow templates
>   - `docs/plans/agent-sdk-features.md` — hooks, compaction, memory tool, handoffs
>   - `docs/plans/cdp-browser-bridge.md` — Chrome DevTools Protocol for platform data
>   - `docs/plans/native-terminal-multiplexer.md` — built-in terminal with xterm.js + PTY
>   - `docs/plans/a2ui-department-apps.md` — AG-UI + A2UI generative UI, capability catalog, department app store
>   - `docs/plans/claude-ecosystem-integration.md` — Claude HUD observability, peer-to-peer agent messaging
>   - `docs/plans/sprints.md` — **Single source of truth for scheduling** (5 sprints, 28 tasks, ~13 weeks)
> In-progress code:
>   - `crates/rusvel-core/src/department/` — DepartmentApp, DepartmentManifest, RegistrationContext (implemented)
>   - `crates/dept-content/` — Content department as DepartmentApp (started)
>   - `crates/dept-forge/` — Forge department as DepartmentApp (started)
>   - `frontend/src/routes/flows/` — Flow builder UI (started)

## Context

TheGenAICircle succeeds with structured playbooks over raw chat, progression systems for engagement, curated bundles for fast onboarding, and cross-functional roundtable sessions. These patterns map directly onto RUSVEL's existing infrastructure — and even more powerfully onto the **in-progress work**:

| GenAICircle Pattern | RUSVEL Existing | In-Progress Enhancement |
|---------------------|-----------------|------------------------|
| Structured playbooks | FlowEngine DAG + Skills | P11 Playbooks, Durable Execution (P8) |
| Cross-functional braintrusts | 10 Forge personas, God Agent | `delegate_agent` (P9), Agent Orchestration doc |
| Curated tool stacks | `!build` + Capability Engine | Starter Kits, DepartmentManifest (ADR-014) |
| Leveling program | OnboardingChecklist, ProductTour | Event Bus milestones |
| Knowledge hub + case studies | Knowledge/RAG + Vector store | Hybrid RAG (P2), Self-improving KB |
| Anti-hype, outcome-led | — | Positioning for launch |

The key insight: **most GenAICircle-inspired features are product surfaces over infrastructure you're already building.** ADR-014 (Department-as-App) + P9 (delegate_agent) + P8 (Durable Execution) form the backbone. The features below are the user-facing skin.

---

## Feature 1: Playbooks — Named, Reusable Multi-Step Recipes

**What:** Surface Workflows + Skills + FlowEngine as user-facing "Playbooks" — versioned, shareable recipes that chain agents, skills, and workflows into named sequences.

**Examples:**
- "Content-from-Code Playbook" — analyze repo → extract insights → draft blog post → schedule
- "Weekly Growth Review Playbook" — pull metrics → compare cohorts → generate summary → notify
- "Opportunity Scout Playbook" — scan sources → score leads → generate proposals → queue for approval

**Integration with in-progress work:**
- **ADR-014 (Department-as-App):** Each department declares its playbooks in `DepartmentManifest.contributions.playbooks`. The host assembles a global playbook registry from all manifests at boot — no central "all playbooks" list needed.
- **P9 (delegate_agent + invoke_flow):** Playbook steps that span departments use `delegate_agent` to hand off to the right department's agent. Cross-department playbooks become first-class.
- **P8 (Durable Execution):** Playbooks run on FlowEngine DAGs with checkpoint/resume. A 10-step playbook that fails on step 7 resumes from step 7 after fix — not from scratch.
- **P5 (Self-Correction):** Each playbook step can optionally include a critique pass. "Content-from-Code" playbook auto-evaluates the draft before publishing.
- **Agent Orchestration doc:** Playbooks are the user-friendly name for what the orchestration doc calls "Workflow Templates" (predefined JSON pipelines loadable as FlowDefs).

**Implementation (revised):**

The `DepartmentManifest` (in `crates/rusvel-core/src/department/manifest.rs`) already has `skills: Vec<SkillContribution>` and works with the `RegistrationContext` pattern. Playbooks extend this:

1. Add `PlaybookContribution` to `DepartmentManifest`:
   ```rust
   // In manifest.rs, add to DepartmentManifest struct:
   pub playbooks: Vec<PlaybookContribution>,

   pub struct PlaybookContribution {
       pub id: String,          // "content-from-code"
       pub name: String,        // "Content from Code"
       pub description: String,
       pub flow_def_json: serde_json::Value,  // serialized FlowDef (petgraph DAG)
       pub parameters: Vec<ArgDef>,            // reuse existing ArgDef type
       pub departments: Vec<String>,           // involved departments
   }
   ```
2. CRUD API routes at `/api/playbooks` (list, get, run, status)
3. Frontend route `/playbooks` — card grid with run button, parameter form, execution history, live status via SSE
4. CLI: `rusvel playbook run content-from-code --topic "our new API"`
5. `dept-content` and `dept-forge` declare their playbooks in `manifest()` — host auto-discovers them
6. Cross-department playbooks use `delegate_agent` (from agent-orchestration.md) in their flow steps

**Maps to:** FlowEngine (petgraph DAG), Skills, P8 (Durable Execution), P9 (delegate_agent), ADR-014 manifests, A2UI (generative UI for playbook step rendering — `a2ui-department-apps.md`)

---

## Feature 2: Executive Brief — Daily Cross-Department Digest

**What:** Automated summary across all 12 departments — what happened, what needs attention, key metrics. The "CEO dashboard" for a solo builder.

**Examples:**
- "3 content drafts awaiting review, pipeline has 2 new opportunities scored >80, runway is 14 months, 1 approval pending"
- Daily summary card on `/dashboard`
- Enhanced `forge mission today` output

**Integration with in-progress work:**
- **ADR-014 (Department-as-App):** Each department declares a `status_summary()` method in `DepartmentApp` trait (or via manifest contribution). The brief aggregates all department summaries. New departments auto-appear in the brief without code changes.
- **P9 (delegate_agent):** The Forge God Agent uses `delegate_agent` to query each department's agent for its summary — a natural use of inline delegation (Mode 1 from agent-orchestration.md).
- **P3 (Batch API):** Daily brief generation is a perfect batch job — query 12 departments in one batch request at 50% cost discount. Non-interactive, async.
- **P4 (Approval UI):** Brief surfaces pending approvals with direct approve/reject buttons — making the approval UI a natural part of the daily digest rather than a separate page.
- **Agent SDK Features (Context Compaction):** Brief history stays compact — each day's brief is summarized, older briefs auto-compacted.

**Implementation (revised):**

The `DepartmentManifest` already has `UiContribution.dashboard_cards: Vec<DashboardCard>` — the brief extends this with live data:

1. Add `BriefContribution` to `DepartmentManifest`:
   ```rust
   // In manifest.rs, add to DepartmentManifest struct:
   pub brief: BriefContribution,

   #[derive(Debug, Clone, Serialize, Deserialize, Default)]
   pub struct BriefContribution {
       pub metrics: Vec<MetricDef>,          // KPIs this dept reports
       pub alert_conditions: Vec<String>,     // conditions that surface as "needs attention"
   }

   pub struct MetricDef {
       pub name: String,          // "pending_drafts"
       pub label: String,         // "Pending Drafts"
       pub query_kind: String,    // "count_by_status" — resolved by host
   }
   ```
2. Each `DepartmentApp::register()` registers a `brief_handler` callback via `RegistrationContext` that returns live metric values
3. API route `GET /api/brief/daily` — iterates all registered departments, calls their brief handlers, LLM summarizes
4. Frontend: `BriefCard.svelte` on `/dashboard` with expandable sections per department (reuses existing `DashboardCard` rendering)
5. CLI: `rusvel brief` (one-shot) or integrated into `forge mission today`
6. Job: `DailyBrief` job type, uses batch API (P3) for cost savings

**Maps to:** Forge Mission, Event Bus, Approvals API, ADR-014, P3 (Batch), P4 (Approval UI), P9 (delegate_agent), Claude HUD observability (`claude-ecosystem-integration.md`)

---

## Feature 3: Starter Kits — One-Click Department Bundles

**What:** Pre-built department configurations (agents + skills + rules + MCP servers + hooks) for common solo-builder personas. One-click install bootstraps a full working setup.

**Kits:**
| Kit | Departments | Target Persona |
|-----|-------------|----------------|
| Indie SaaS Founder | code, content, growth, finance | Building + marketing a product |
| Freelance Consultant | gtm, legal, support, content | Client work + business ops |
| Open Source Maintainer | code, infra, content, support | Repo management + community |
| Agency Owner | content, gtm, harvest, finance | Multi-client service delivery |
| Solopreneur | growth, distro, content, finance | Side project monetization |

**Integration with in-progress work:**
- **ADR-014 (Department-as-App):** Kits are a natural extension of `DepartmentManifest`. Each department declares its `default_kit_items` (agents, skills, rules, hooks, playbooks). A kit = a selection of departments + triggering their default seeding.
- **Agent SDK Features (Memory Tool):** Kit installation also seeds initial memory entries — "You are configured for an Indie SaaS founder building X" — giving agents immediate context.
- **CDP Browser Bridge:** Freelancer/Agency kits include MCP server configs for Upwork/LinkedIn CDP bridge — pre-wired platform connections.
- **Agent Orchestration (Workflow Templates):** Kits bundle workflow templates (agent pipelines) tailored to the persona — "Freelance Client Onboarding" pipeline pre-installed.

**Implementation (revised):**

Kits leverage what each department already declares in its `DepartmentManifest` — skills, personas, rules, tools, playbooks. A kit = selecting departments + activating their defaults + adding kit-specific overrides:

- Kit definitions embedded in binary (or in `data/kits/*.toml`):
  ```toml
  [kit]
  id = "indie-saas"
  name = "Indie SaaS Founder"
  departments = ["code", "content", "growth", "finance"]

  [[kit.playbooks]]
  id = "weekly-review"
  # references playbook from growth dept manifest

  [[kit.seed_agents]]
  dept = "content"
  name = "Blog Writer"
  persona = "content-writer"  # references PersonaContribution from dept-content manifest
  system_prompt = "You write technical blog posts for a SaaS product..."

  [[kit.browser_configs]]
  platform = "upwork"  # references cdp-browser-bridge.md
  mode = "passive"
  ```
- API: `GET /api/kits` (list), `POST /api/kits/:id/install` (seed), `GET /api/kits/:id/preview` (show what gets created)
- Frontend: Kit selection during onboarding OR at `/settings/kits`
- CLI: `rusvel kit install indie-saas`
- Kit install triggers `RegistrationContext` seeding — creates DB rows for agents, skills, rules per manifest declarations

**Maps to:** `!build` command, Capability Engine, ADR-014 manifests, Agent Orchestration templates

---

## Feature 4: Leveling / Progression System

**What:** Per-department proficiency tracker. As the user hits milestones, unlock levels and discover unused capabilities.

**Milestones (per department):**
- Level 1: First chat message sent
- Level 2: First agent created
- Level 3: First skill executed
- Level 4: First playbook run
- Level 5: First cross-department action triggered (delegate_agent)

**Integration with in-progress work:**
- **ADR-014:** Each department declares its milestones in `DepartmentManifest.contributions.milestones`. New departments auto-register their progression.
- **Event Bus:** Milestones tracked via event matching — `agent.created { dept: "content" }` → mark Level 2 for content.
- **Agent Orchestration (Event Triggers):** Level-up events can trigger celebratory agent messages or unlock suggestions — "You've reached Level 3 in Content! Try the Content-from-Code playbook."

**Implementation:**

Milestones declared per department in manifest, tracked via EventBus triggers (from agent-orchestration.md Phase 3):

1. Add `MilestoneContribution` to `DepartmentManifest`:
   ```rust
   pub milestones: Vec<MilestoneContribution>,

   pub struct MilestoneContribution {
       pub id: String,              // "first-agent-created"
       pub level: u8,               // 1-5
       pub label: String,           // "Created your first agent"
       pub event_pattern: String,   // "agent.created" — matches EventTrigger patterns
       pub next_suggestion: String, // "Try creating a skill next"
   }
   ```
2. Wire into `EventTrigger` system (from agent-orchestration.md Phase 3) — milestone events are just triggers with `TriggerAction::RecordMilestone`
3. `user_milestones` SQLite table (dept_id, milestone_id, achieved_at)
4. `GET /api/progress` — returns levels per department + next suggested action
5. Frontend: progress badges on department cards (alongside existing `DashboardCard` rendering), suggested "next step" prompts

**Maps to:** OnboardingChecklist, ProductTour, Event Bus, ADR-014, EventTrigger (agent-orchestration.md)

---

## Feature 5: Roundtable / Strategy Review UI

**What:** Enhanced God Agent multi-department collaboration displayed as a roundtable conversation, not flat chat. Each department persona contributes visibly with their avatar/role.

**Integration with in-progress work:**
- **P9 (delegate_agent):** Roundtable is literally the orchestrator pattern from agent-orchestration.md Mode 1 — God Agent delegates to department agents inline, collects responses, synthesizes.
- **AG-UI Protocol (P7):** Each persona's response streams via AG-UI events with agent identity metadata — frontend renders per-persona bubbles using `RUN_STARTED { agent_id, persona }`.
- **Agent SDK Features (Multi-Agent Handoffs):** Uses `handoff` pattern — God Agent hands off to Content Agent, gets response, hands off to Code Agent, etc.
- **P5 (Self-Correction):** Final synthesis step includes a critique pass — "Did we miss any department's perspective?"

**Implementation:**

This is essentially a specialized Workflow Template (from agent-orchestration.md Phase 4) with a visual roundtable UI:

1. Roundtable = predefined workflow template where each step is a different persona:
   ```json
   {
     "name": "strategy-roundtable",
     "steps": [
       { "id": "content-perspective", "persona": "ContentWriter", "prompt": "Analyze from content angle: {{input.topic}}" },
       { "id": "code-perspective", "persona": "Architect", "prompt": "Analyze from technical angle: {{input.topic}}" },
       { "id": "growth-perspective", "persona": "Researcher", "prompt": "Analyze from growth angle: {{input.topic}}" },
       { "id": "synthesis", "persona": "GodAgent", "prompt": "Synthesize: {{steps.*.output}}", "depends_on": ["content-perspective", "code-perspective", "growth-perspective"] }
     ]
   }
   ```
2. `Parallel` step execution — first 3 personas run concurrently via `delegate_agent` (agent-orchestration.md Mode 1)
3. AG-UI events (P7) carry persona identity: `RUN_STARTED { agent_id, persona }` → frontend renders per-persona bubbles
4. New API: `POST /api/roundtable` (topic, department_ids) → SSE stream with per-persona events
5. Frontend: `Roundtable.svelte` — persona avatars in a circle layout (reuses `PersonaPicker.svelte` from agent-orchestration.md frontend components), speech bubbles, synthesis panel
6. Final output: action items assigned to specific departments → auto-create jobs via `TriggerAction::EnqueueJob`

**Maps to:** Forge Engine (10 personas), P9 (delegate_agent), P7 (AG-UI), Agent Orchestration Workflow Templates, existing `PersonaContribution` in manifests

---

## Feature 6: Self-Improving Knowledge Base

**What:** Every analysis, draft, and discovery auto-indexes into the knowledge base. Cross-department insights surface automatically.

**Cross-pollination examples:**
- Code analysis finds new API endpoint → Content engine suggests blog post
- Harvest discovers competitor feature → Product engine adds to roadmap
- Support ticket pattern detected → Code engine suggests fix

**Integration with in-progress work:**
- **P2 (Hybrid RAG):** Cross-department search uses RRF fusion — BM25 (exact keyword) + LanceDB (semantic) merged. "Find content related to this code change" hits both stores.
- **Agent SDK Features (Memory Tool):** Agents auto-write discoveries to memory. Memory entries are vectorized and searchable across departments.
- **P9 (Event Triggers):** Event-driven indexing — `code.analysis.completed` event triggers auto-embed into vector store AND fires a cross-department query to find related content/harvest items.
- **ADR-014:** Each department declares its `knowledge_types` in manifest — what kinds of knowledge it produces and consumes. The platform routes cross-department suggestions based on these declarations.

**Implementation:**

Leverages Event Triggers (agent-orchestration.md), Hybrid RAG (P2), Memory Tool (agent-sdk-features.md), and CDP data (cdp-browser-bridge.md):

1. Post-action hooks via `EventTrigger` (agent-orchestration.md Phase 3):
   ```rust
   EventTrigger {
       pattern: EventPattern::Prefix { prefix: "job.completed".into() },
       action: TriggerAction::EnqueueJob {
           job_kind: "knowledge.auto_index".into(),
           payload: json!({ "source_event": "{{event}}" }),
       },
   }
   ```
2. `knowledge.auto_index` job: embeds job output into vector store with `dept_id`, `source_type`, `timestamp` metadata
3. CDP-captured data (cdp-browser-bridge.md) also flows through: `browser.data.captured` → auto-embed Upwork/LinkedIn opportunities
4. Cross-department query: `GET /api/knowledge/related?dept=code&id=xxx` → Hybrid RAG (P2) with RRF fusion across all departments
5. Agent Memory Tool (agent-sdk-features.md Tier 1.3) auto-writes discoveries — `memory_write` entries get vectorized alongside explicit knowledge
6. Suggestion system: `DailyBrief` job (Feature 2) includes cross-department link detection as part of its sweep
7. Frontend: "Related from other departments" section on knowledge items, surfaced via existing `DashboardCard` pattern

**Maps to:** Knowledge/RAG, Vector store, P2 (Hybrid RAG), P9 (Event Triggers), ADR-014, Memory Tool, CDP Bridge

---

## Feature 7: Outcome-Led Positioning

**What:** When ready for public launch, lead with outcomes not architecture.

**Messaging:**
- Hero: "One binary. 12 departments. You're the CEO, AI is the team."
- Social proof: "What a solo builder accomplished in one week with RUSVEL"
- Demo video: Show a real workflow from code commit → content published → leads captured

**Integration with in-progress work:**
- **CDP Browser Bridge:** Demo shows live Upwork data flowing into harvest pipeline — most impressive demo possible. User browses Upwork → RUSVEL auto-scores opportunities → drafts proposals → queues for approval.
- **Native Terminal Multiplexer:** Demo shows split panes (`/terminal` route with paneforge + xterm.js) with different departments running simultaneously — visual proof of the "virtual agency."
- **Playbooks:** "Watch this 5-step playbook execute autonomously" — the killer demo. Uses FlowEngine DAG with checkpoint/resume (P8).
- **Roundtable:** "Ask your AI team to debate your go-to-market strategy" — multi-persona roundtable with visible delegation tree.
- **Agent Orchestration (Workflow Templates):** The "autonomous-code-pipeline" template from agent-orchestration.md (Plan → Build → Test → Review → Report) is a perfect demo: 1 human message → 7 autonomous agent runs → complete feature with tests and docs.

**Not a code task** — marketing/positioning for when the product is ready. But the demo infrastructure depends on Phases 1-3 being complete.

---

## Sprint Mapping

> **`docs/plans/sprints.md` is the single source of truth for scheduling.**
> This section maps GenAICircle features to sprint tasks. Do not plan from here — plan from sprints.md.

| Feature | Sprint | Task # | Dependencies |
|---------|--------|--------|-------------|
| **Playbooks** | Sprint 4 | #23 | #16 (delegate_agent), #17 (invoke_flow), #20 (event triggers), #22 (durable execution) |
| **Executive Brief** | Sprint 5 | #25 | #16 (delegate_agent) |
| **Starter Kits** | Sprint 5 | #26 | Sprint 1 ADR-014 |
| **Self-Improving KB** | Sprint 5 | #27 | #14 (hybrid RAG) |
| **Leveling System** | Backlog | — | Sprint 1 ADR-014, #20 (event triggers) |
| **Roundtable UI** | Backlog | — | #16 (delegate_agent), #24 (AG-UI) |
| **Outcome-Led Positioning** | Backlog | — | Sprints 4-5 complete |

### Critical Path (from sprints.md)

```
Sprint 1                Sprint 2              Sprint 3              Sprint 4         Sprint 5
─────────               ─────────             ─────────             ─────────        ─────────
ADR-014 ──────────────→ Frontend align ──→ Hooks + Permissions
                                              │
Cost routing ────────→ Batch API              │
                                              ├──→ Self-correction
                                              │
                        Memory + RAG          │
                                              │
                        Compaction            delegate_agent ──────→ Playbooks ──→ Executive Brief
                                              │                         │
                                         Event triggers                 │
                                              │                    Durable Exec
                                         invoke_flow
                                                                  AG-UI ─────────→ (Roundtable)
```

**The two unlock points:**
1. **ADR-014** (Sprint 1) — unlocks hooks, permissions, dept-scoped everything, manifest-driven features
2. **delegate_agent** (Sprint 3) — unlocks playbooks, executive brief, roundtable, workforce dogfooding

## Cross-Document Reference Map

| This Feature | Depends On (doc → section) |
|---|---|
| Playbooks | `department-as-app.md` → DepartmentManifest, `agent-orchestration.md` → Workflow Templates + delegate_agent, `next-level-proposals.md` → P8 Durable Execution |
| Executive Brief | `department-as-app.md` → DepartmentApp.register(), `agent-orchestration.md` → delegate_agent (Mode 1), `next-level-proposals.md` → P3 Batch API, P4 Approval UI |
| Starter Kits | `department-as-app.md` → DepartmentManifest defaults, `agent-orchestration.md` → Workflow Templates, `cdp-browser-bridge.md` → platform configs |
| Leveling | `agent-orchestration.md` → Event Triggers (Phase 3), `department-as-app.md` → DepartmentManifest |
| Roundtable | `agent-orchestration.md` → delegate_agent + Workflow Templates, `next-level-proposals.md` → P7 AG-UI, `agent-sdk-features.md` → Multi-Agent Handoffs, `a2ui-department-apps.md` → A2UI roundtable rendering |
| Self-Improving KB | `next-level-proposals.md` → P2 Hybrid RAG, `agent-orchestration.md` → Event Triggers, `agent-sdk-features.md` → Memory Tool, `cdp-browser-bridge.md` → captured data |
| Starter Kits | `department-as-app.md` → DepartmentManifest defaults, `agent-orchestration.md` → Workflow Templates, `cdp-browser-bridge.md` → platform configs, `a2ui-department-apps.md` → capability catalog browsing |
| Outcome-Led Positioning | `cdp-browser-bridge.md` → Upwork demo, `native-terminal-multiplexer.md` → split pane demo, `agent-orchestration.md` → autonomous-code-pipeline template, `claude-ecosystem-integration.md` → HUD observability for demo polish |

## Manifest Extensions Summary

The current `DepartmentManifest` (in `crates/rusvel-core/src/department/manifest.rs`) needs these additions for GenAICircle-inspired features:

```rust
// Add to DepartmentManifest struct:
pub playbooks: Vec<PlaybookContribution>,    // Feature 1
pub brief: BriefContribution,                // Feature 2
pub milestones: Vec<MilestoneContribution>,  // Feature 4

// New contribution types:
pub struct PlaybookContribution { id, name, description, flow_def_json, parameters: Vec<ArgDef>, departments }
pub struct BriefContribution { metrics: Vec<MetricDef>, alert_conditions }
pub struct MetricDef { name, label, query_kind }
pub struct MilestoneContribution { id, level, label, event_pattern, next_suggestion }
```

All default to empty/default — zero-cost for departments that don't declare them. Existing manifests (dept-content, dept-forge) continue working unchanged.

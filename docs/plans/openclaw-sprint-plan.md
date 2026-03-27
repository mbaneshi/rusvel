# RUSVEL OpenClaw Integration: Complete Sprint Execution Plan

## Context

RUSVEL is a 50-crate Rust hexagonal architecture app — the solo builder's AI-powered virtual agency. After analyzing OpenClaw (a 296K-object, 84-extension, 24-channel personal AI gateway), we identified 15 workstreams of patterns worth integrating.

**Problem:** RUSVEL has 13 fully implemented engines but only 3 have agent tools wired. The app can't reach users beyond its web UI. No auth, no channels, no voice, no canvas.

**Goal:** Close the wiring gap, restructure the frontend, then progressively integrate OpenClaw patterns (channels, webhooks, cron, voice, canvas, plugin SDK) across 10 sprints / 20 weeks.

**Key discovery:** Job queue persistence is NOT a gap — SQLite Database already implements JobPort. Auth middleware IS required before exposing webhook endpoints.

**Source documents:**
- `docs/plans/openclaw-integration-proposal.md` — 15 workstreams, crate impact analysis
- `docs/state-report/` — 7-chapter codebase audit (52,560 LOC, 399 tests, 105 routes)
- `docs/design/ui-redesign-final.md` — 5-zone icon rail layout (supersedes two-level-nav-proposal)
- `docs/design/ui-design-principles.md` — 10 design principles (compliance checklist for ALL frontend PRs)
- `docs/design/ui-redesign-sprint-compliance.md` — story-by-story compliance analysis
- `docs/design/entity-hierarchy-proposal.md` — Playbook entity: composition root binding agents, skills, rules, workflows, hooks, MCP servers

---

## 0. DEFINITION OF DONE (Global)

Every story must satisfy ALL of the following before it can be marked done:

1. **Code compiles:** `cargo build` passes with zero warnings on `deny(warnings)`.
2. **Tests pass:** `cargo test --workspace` passes. New code has at least one test per public function.
3. **Clippy clean:** `cargo clippy --workspace --all-targets -- -D warnings` passes.
4. **Frontend checks (if touched):** `cd frontend && pnpm check && pnpm build` both pass.
5. **Documented:** New public items have doc-comments. New crates have module-level `//!` docs.
6. **Integration verified:** The story's acceptance criteria are all checked by manual run or automated test.
7. **Crate size:** No crate exceeds 2000 lines (convention from CLAUDE.md).
8. **No regressions:** Visual regression tests pass (`pnpm test:visual`) if frontend is touched.
9. **Conventional commit:** Commit message follows the `<scope>: <description>` pattern.
10. **UI Design Principles (if frontend touched):** Passes ALL 10 checks from `docs/design/ui-design-principles.md`:
    - No `{:else if activeTab}` chains — use component registry
    - No `dept === 'x'` checks in rendering — read from manifest
    - Components receive ≤ 4 props; no full `DepartmentDef` as prop
    - Zones don't import each other; communication via stores only
    - Colors via CSS `--dept-hsl` variable, not prop drilling
    - Only manifest-declared tabs rendered; sparse depts show fewer items
    - Sections don't know which zone renders them
    - No raw `fetch()` in components — use api.ts layer

---

## 1. EPIC BREAKDOWN

| Epic | Name | Workstream | Phase |
|------|------|-----------|-------|
| E0 | Auth Middleware | API security, Bearer token extractor | P0 |
| E1 | GTM Tool Wiring | Wire 5 GTM engine tools into dept-gtm | P0 |
| E2 | Finance Tool Wiring | Wire 4 finance-engine tools into dept-finance | P0 |
| E3 | Product Tool Wiring | Wire 4 product-engine tools into dept-product | P0 |
| E4 | Growth Tool Wiring | Wire 4 growth-engine tools into dept-growth | P0 |
| E5 | Support Tool Wiring | Wire 4 support-engine tools into dept-support | P0 |
| E6 | Distro Tool Wiring | Wire 4 distro-engine tools into dept-distro | P0 |
| E7 | Legal Tool Wiring | Wire 4 legal-engine tools into dept-legal | P0 |
| E8 | Infra Tool Wiring | Wire 4 infra-engine tools into dept-infra | P0 |
| E9 | Flow Tool Wiring | Wire 7 flow-engine tools into dept-flow | P0 |
| E10 | Frontend 5-Zone Layout | Icon rail + section sidebar + main + context panel + bottom panel | P1 |
| E11 | Revenue Engine Hardening | Harvest CDP, Content scheduling, approval UX | P2 |
| E12 | Webhook + Cron | rusvel-webhook and rusvel-cron crates | P3 |
| E13 | Channel System | rusvel-channel crate, messaging integration | P4 |
| E14 | Plugin SDK | rusvel-plugin-sdk crate for community engines | P5 |
| E15 | Voice Interface | rusvel-voice crate for speech I/O | P5 |
| E16 | Playbook Entity | Composition root binding agents/skills/rules/workflows/hooks/MCP per use case | P2 |

---

## 2. SPRINT PLAN + 3. USER STORIES

### ═══════════════════════════════════════════════════
### PHASE 0: SECURE THE PERIMETER + WIRE ALL DEPARTMENTS
### ═══════════════════════════════════════════════════

---

### SPRINT 1: "Lock & Load" (Weeks 1-2)
**Goal:** Secure API routes and wire the first batch of department tools (GTM, Finance, Product).

#### Stories

```
[S-001] As an operator, I want Bearer token auth on all API routes, so that the exposed API cannot be accessed without authorization.
Acceptance Criteria:
- [ ] New Axum `FromRequest` extractor `BearerAuth` in rusvel-api checks `Authorization: Bearer <token>`
- [ ] Token is validated against `RUSVEL_API_TOKEN` env var
- [ ] When env var is unset, all requests pass through (opt-in security)
- [ ] Returns 401 with JSON error body when token is invalid
- [ ] Health endpoint `/api/health` is exempt from auth
- [ ] Integration test covers auth-required and auth-bypass paths
Size: M
Files: crates/rusvel-api/src/lib.rs, crates/rusvel-api/src/auth.rs (new), crates/rusvel-api/src/routes.rs
Dependencies: none
```

```
[S-002] As an operator, I want auth middleware applied as an Axum layer, so that I do not need to add auth checks per-handler.
Acceptance Criteria:
- [ ] Auth is a tower middleware layer applied to the Router
- [ ] Configurable exempt paths list (default: ["/api/health"])
- [ ] Layer reads token from AppState or environment
- [ ] Tests verify layer integration with mock requests
Size: S
Files: crates/rusvel-api/src/auth.rs, crates/rusvel-api/src/lib.rs
Dependencies: S-001
```

```
[S-003] As an agent, I want GTM CRM tools (add_contact, list_contacts, add_deal), so that I can manage contacts and deals through the agent runtime.
Acceptance Criteria:
- [ ] `gtm.crm.add_contact` tool registered via ctx.tools.add in dept-gtm
- [ ] `gtm.crm.list_contacts` tool registered
- [ ] `gtm.crm.add_deal` tool registered
- [ ] Each tool invokes the corresponding GtmEngine::crm() method
- [ ] Tools return structured JSON with success/error handling matching harvest.rs pattern
- [ ] Unit test for department creation verifies tool count in manifest
Size: M
Files: crates/dept-gtm/src/lib.rs, crates/dept-gtm/src/manifest.rs
Dependencies: none
```

```
[S-004] As an agent, I want GTM outreach tools (create_sequence), so that I can create outreach sequences.
Acceptance Criteria:
- [ ] `gtm.outreach.create_sequence` tool registered
- [ ] Tool invokes GtmEngine::outreach().create_sequence()
- [ ] Parameters include target_contact_id, template, steps
- [ ] Error handling follows established ToolOutput pattern
Size: S
Files: crates/dept-gtm/src/lib.rs
Dependencies: S-003
```

```
[S-005] As an agent, I want a GTM invoice tool (create_invoice), so that I can generate invoices.
Acceptance Criteria:
- [ ] `gtm.invoices.create_invoice` tool registered
- [ ] Tool invokes GtmEngine::invoices().create_invoice()
- [ ] Parameters include client_name, line_items, due_date
- [ ] Returns invoice ID and status
Size: S
Files: crates/dept-gtm/src/lib.rs
Dependencies: S-003
```

```
[S-006] As an agent, I want finance ledger tools (record, balance), so that I can track income and expenses.
Acceptance Criteria:
- [ ] `finance.ledger.record` tool registered in dept-finance
- [ ] `finance.ledger.balance` tool registered
- [ ] Tools invoke FinanceEngine::ledger() methods
- [ ] Record tool accepts amount, category, description, date
- [ ] Balance tool returns current balance with breakdown
Size: M
Files: crates/dept-finance/src/lib.rs, crates/dept-finance/src/manifest.rs
Dependencies: none
```

```
[S-007] As an agent, I want finance tax tools (add_estimate, total_liability), so that I can track tax obligations.
Acceptance Criteria:
- [ ] `finance.tax.add_estimate` tool registered
- [ ] `finance.tax.total_liability` tool registered
- [ ] Tools invoke FinanceEngine::tax() methods
- [ ] Estimate tool accepts quarter, amount, jurisdiction
- [ ] Liability tool returns total with breakdown by jurisdiction
Size: S
Files: crates/dept-finance/src/lib.rs
Dependencies: S-006
```

```
[S-008] As an agent, I want product roadmap tools (add_feature, list_features), so that I can manage the product roadmap.
Acceptance Criteria:
- [ ] `product.roadmap.add_feature` tool registered in dept-product
- [ ] `product.roadmap.list_features` tool registered
- [ ] Tools invoke ProductEngine::roadmap() methods
- [ ] Add accepts title, description, priority, status
- [ ] List supports optional status filter
Size: M
Files: crates/dept-product/src/lib.rs, crates/dept-product/src/manifest.rs
Dependencies: none
```

```
[S-009] As an agent, I want product pricing and feedback tools, so that I can manage pricing tiers and record user feedback.
Acceptance Criteria:
- [ ] `product.pricing.create_tier` tool registered
- [ ] `product.feedback.record` tool registered
- [ ] Tools invoke ProductEngine pricing/feedback methods
- [ ] Pricing tier accepts name, price, features list
- [ ] Feedback accepts source, sentiment, content
Size: S
Files: crates/dept-product/src/lib.rs
Dependencies: S-008
```

**Sprint Acceptance Criteria:**
- `cargo test --workspace` passes with all new tool registration tests
- Auth middleware rejects unauthorized requests in integration tests
- GTM, Finance, Product departments each have 4-5 tools registered
- `cargo run` starts successfully with all tools visible in `/api/config/tools`

**Sprint Demo Scenario:**
1. Start server with `RUSVEL_API_TOKEN=secret cargo run`
2. `curl -H "Authorization: Bearer wrong" localhost:3000/api/departments` returns 401
3. `curl -H "Authorization: Bearer secret" localhost:3000/api/config/tools` shows new GTM/Finance/Product tools
4. Send chat to GTM department: "Add a contact named John Doe" and observe tool invocation

---

### SPRINT 2: "Full Arsenal" (Weeks 3-4)
**Goal:** Wire remaining 5 departments (Growth, Support, Distro, Legal, Infra) and Flow engine tools.

#### Stories

```
[S-010] As an agent, I want growth engine tools (funnel, cohort, KPI), so that I can analyze growth metrics.
Acceptance Criteria:
- [ ] `growth.funnel.add_stage` tool registered in dept-growth
- [ ] `growth.cohort.create_cohort` tool registered
- [ ] `growth.kpi.record_kpi` tool registered
- [ ] `growth.kpi.get_trend` tool registered
- [ ] All tools invoke corresponding GrowthEngine methods
Size: M
Files: crates/dept-growth/src/lib.rs, crates/dept-growth/src/manifest.rs
Dependencies: none
```

```
[S-011] As an agent, I want support engine tools (tickets, knowledge, NPS), so that I can manage support operations.
Acceptance Criteria:
- [ ] `support.tickets.create` tool registered in dept-support
- [ ] `support.tickets.list` tool registered
- [ ] `support.knowledge.search` tool registered
- [ ] `support.nps.calculate_score` tool registered
- [ ] Tools invoke SupportEngine::tickets(), knowledge(), nps() methods
Size: M
Files: crates/dept-support/src/lib.rs, crates/dept-support/src/manifest.rs
Dependencies: none
```

```
[S-012] As an agent, I want distro engine tools (SEO, marketplace, affiliates, analytics), so that I can manage distribution channels.
Acceptance Criteria:
- [ ] `distro.seo.analyze` tool registered in dept-distro
- [ ] `distro.marketplace.list` tool registered
- [ ] `distro.affiliate.create_link` tool registered
- [ ] `distro.analytics.report` tool registered
- [ ] All tools invoke corresponding DistroEngine methods
Size: M
Files: crates/dept-distro/src/lib.rs, crates/dept-distro/src/manifest.rs
Dependencies: none
```

```
[S-013] As an agent, I want legal engine tools (contracts, compliance, IP), so that I can manage legal operations.
Acceptance Criteria:
- [ ] `legal.contracts.create` tool registered in dept-legal
- [ ] `legal.compliance.check` tool registered
- [ ] `legal.ip.register` tool registered
- [ ] `legal.contracts.list` tool registered
- [ ] All tools invoke corresponding LegalEngine methods
Size: M
Files: crates/dept-legal/src/lib.rs, crates/dept-legal/src/manifest.rs
Dependencies: none
```

```
[S-014] As an agent, I want infra engine tools (deploy, monitor, incidents, health), so that I can manage infrastructure.
Acceptance Criteria:
- [ ] `infra.deploy.trigger` tool registered in dept-infra
- [ ] `infra.monitor.status` tool registered
- [ ] `infra.incidents.create` tool registered
- [ ] `infra.incidents.list` tool registered
- [ ] All tools invoke corresponding InfraEngine methods
Size: M
Files: crates/dept-infra/src/lib.rs, crates/dept-infra/src/manifest.rs
Dependencies: none
```

```
[S-015] As an agent, I want flow engine tools (CRUD + execution), so that I can create and run DAG workflows through chat.
Acceptance Criteria:
- [ ] `flow.save` tool registered in dept-flow
- [ ] `flow.get` tool registered
- [ ] `flow.list` tool registered
- [ ] `flow.run` tool registered
- [ ] `flow.resume` tool registered
- [ ] `flow.get_execution` tool registered
- [ ] `flow.list_executions` tool registered
- [ ] All tools invoke corresponding FlowEngine methods through existing flow_routes patterns
Size: L
Files: crates/dept-flow/src/lib.rs, crates/dept-flow/src/manifest.rs
Dependencies: none
```

```
[S-016] As a developer, I want a comprehensive integration test for all 13 departments, so that I can verify every department has tools registered.
Acceptance Criteria:
- [ ] Test in rusvel-app that calls boot_departments() and asserts tool count >= 60
- [ ] Test verifies each department ID has at least 3 tools
- [ ] Test verifies no tool name collisions across departments
Size: S
Files: crates/rusvel-app/src/boot.rs (test section)
Dependencies: S-003 through S-015
```

```
[S-017] As a developer, I want a ToolRegistrar test harness, so that each dept crate can unit-test tool registration without full boot.
Acceptance Criteria:
- [ ] Mock RegistrationContext factory function in rusvel-core test utils
- [ ] Each dept crate can call register() with mock context and assert tool count
- [ ] Pattern documented in dept-forge tests as reference
Size: S
Files: crates/rusvel-core/src/department/context.rs (test helpers)
Dependencies: none
```

**Sprint Acceptance Criteria:**
- All 13 departments have tools wired (60+ total tools)
- `cargo test --workspace` passes including new integration test
- Boot sequence logs tool registration count for each department

**Sprint Demo Scenario:**
1. `cargo run` and observe boot log: "Registered N tools across 13 departments"
2. `GET /api/config/tools` returns 60+ tools with department grouping
3. Chat with support department: "Create a ticket for login bug" and see tool call
4. Chat with flow department: "List all workflows" and see flow.list tool invoked

---

### ═══════════════════════════════════════════════════
### PHASE 1: FRONTEND RESTRUCTURING
### ═══════════════════════════════════════════════════

---

### SPRINT 3: "New Shell" (Weeks 5-6)
**Goal:** Restructure frontend to 5-zone icon rail layout per `docs/design/ui-redesign-final.md`.

Layout: Icon Rail (48px) | Section Sidebar (200px) | Main Content (flex) | Context Panel (320px) | Bottom Panel

#### Stories

```
[S-018] As a user, I want a 48px icon rail replacing the sidebar, so that all 13 departments + global pages fit without scrolling.
Acceptance Criteria:
- [ ] Rewrite +layout.svelte: remove PaneGroup sidebar, add fixed 48px icon rail
- [ ] Top section: Home, Chat, Approvals, DB, Flows icons (from AppState shared services)
- [ ] Divider
- [ ] Bottom section: 13 department icons from DepartmentRegistry (DeptIcon component reused)
- [ ] Bottom: Settings icon
- [ ] Active state: accent color pill behind active icon
- [ ] Approval badge count on Approvals icon
- [ ] Tooltip on hover showing full name
Size: M
Files: frontend/src/routes/+layout.svelte (rewrite), frontend/src/lib/components/IconRail.svelte (new)
Dependencies: none
```

```
[S-019] As a user, I want a manifest-driven section sidebar on /dept/* pages, so that I see department-specific sections.
Acceptance Criteria:
- [ ] New dept/[id]/+layout.svelte with 200px section sidebar
- [ ] Dept header: icon + title with department accent color
- [ ] Section links from tabsFromDepartment(dept) + 'chat' + 'settings'
- [ ] Active section highlighted with dept accent color
- [ ] Collapsible (icon-only mode)
- [ ] Session switcher at bottom of sidebar
- [ ] Sidebar hidden on non-dept pages (global pages get full width)
Size: M
Files: frontend/src/routes/dept/[id]/+layout.svelte (new), frontend/src/lib/stores.ts
Dependencies: S-018
```

```
[S-020] As a user, I want a collapsible context panel (320px right), so that AI chat and item properties appear alongside main content.
Acceptance Criteria:
- [ ] Context panel in dept/[id]/+layout.svelte, right side, 320px
- [ ] Three modes: (1) Quick AI chat with dept agent, (2) Selected item properties, (3) Execution output
- [ ] Toggle via Cmd+J or button
- [ ] Chat mode uses existing DepartmentChat component in compact mode
- [ ] Properties mode shows JSON detail of selected item
- [ ] New store: contextPanelOpen (boolean), contextPanelMode (chat|properties|execution)
Size: L
Files: frontend/src/routes/dept/[id]/+layout.svelte, frontend/src/lib/components/ContextPanel.svelte (new), frontend/src/lib/stores.ts
Dependencies: S-019
```

```
[S-021] As a user, I want a collapsible bottom panel for terminal/logs/events, so that these are accessible from any section.
Acceptance Criteria:
- [ ] Bottom panel in dept/[id]/+layout.svelte, ~200px, collapsible
- [ ] Toggle via Cmd+` or button
- [ ] Tabs: Terminal (DeptTerminal reused), Jobs (execution status from JobPort), Events (live event stream)
- [ ] New store: bottomPanelOpen (boolean)
- [ ] Terminal tab uses existing xterm.js WebSocket connection
Size: M
Files: frontend/src/routes/dept/[id]/+layout.svelte, frontend/src/lib/components/BottomPanel.svelte (new), frontend/src/lib/stores.ts
Dependencies: S-019
```

```
[S-022] As a user, I want 13 section page wrappers for department sub-pages, so that each section has a proper URL.
Acceptance Criteria:
- [ ] dept/[id]/actions/+page.svelte → ActionsTab
- [ ] dept/[id]/engine/+page.svelte → EngineTab
- [ ] dept/[id]/agents/+page.svelte → AgentsTab
- [ ] dept/[id]/skills/+page.svelte → SkillsTab
- [ ] dept/[id]/rules/+page.svelte → RulesTab
- [ ] dept/[id]/workflows/+page.svelte → WorkflowsTab
- [ ] dept/[id]/mcp/+page.svelte → McpTab
- [ ] dept/[id]/hooks/+page.svelte → HooksTab
- [ ] dept/[id]/terminal/+page.svelte → DeptTerminal
- [ ] dept/[id]/events/+page.svelte → EventsTab
- [ ] dept/[id]/dirs/+page.svelte → DirsTab
- [ ] dept/[id]/chat/+page.svelte → DepartmentChat (full width in main)
- [ ] dept/[id]/settings/+page.svelte → DepartmentConfig editor
- [ ] Each page ~15 lines: import tab component, derive dept from store, render
Size: M
Files: 13 new +page.svelte files under frontend/src/routes/dept/[id]/
Dependencies: S-019
```

```
[S-023] As a user, I want /dept/[id] to redirect to /dept/[id]/actions, so that there is always a default section.
Acceptance Criteria:
- [ ] dept/[id]/+page.svelte redirects to /dept/{id}/actions via $effect + goto()
- [ ] pendingCommand store: when set, also goto('/dept/{id}/chat') so chat picks it up
Size: S
Files: frontend/src/routes/dept/[id]/+page.svelte (modify)
Dependencies: S-022
```

```
[S-024] As a developer, I want DepartmentPanel.svelte retired, so that the section routing + icon rail replaces it.
Acceptance Criteria:
- [ ] DepartmentPanel.svelte removed or marked deprecated
- [ ] All references updated to use new section pages
- [ ] No broken imports or dead code
- [ ] Visual regression tests updated with new baselines
Size: M
Files: frontend/src/lib/components/department/DepartmentPanel.svelte, frontend/src/routes/dept/[id]/+page.svelte
Dependencies: S-022, S-023
```

```
[S-025] As a user, I want keyboard navigation for the icon rail, so that I can switch departments with Alt+1 through Alt+9.
Acceptance Criteria:
- [ ] Alt+1 through Alt+9 switches to department 1 through 9
- [ ] Cmd+J toggles context panel
- [ ] Cmd+` toggles bottom panel
- [ ] CommandPalette updated with department switching commands
Size: S
Files: frontend/src/lib/components/IconRail.svelte, frontend/src/lib/components/onboarding/CommandPalette.svelte
Dependencies: S-018
```

**Sprint Acceptance Criteria:**
- Frontend builds without errors: `pnpm build && pnpm check`
- 5-zone layout renders: icon rail (48px) + section sidebar (200px) + main + context panel (320px) + bottom panel
- All department pages accessible via icon rail + section sidebar
- Context panel shows quick AI chat alongside any section
- Bottom panel shows terminal, jobs, events
- Old DepartmentPanel no longer referenced
- **UI Design Principles compliance:** (per `docs/design/ui-design-principles.md`)
  - Zone A uses component registry (Principle 2: O/C) — no `{:else if}` chains
  - No `dept === 'x'` checks in any zone (Principle 3: LSP, Principle 5: DIP)
  - Colors via `--dept-hsl` CSS variable only (Principle 6: DRY)
  - Zones communicate via stores, not props between zones (Principle 7: Composition)
  - Tabs come from `tabsFromDepartment(manifest)` only (Principle 8: SSOT, Principle 10: Disclosure)
  - `grep -r "dept === '" src/lib/components/layout/` returns 0 results

**Sprint Demo Scenario:**
1. Open `localhost:5173` — see icon rail on left (global icons + 13 dept icons)
2. Click Forge icon — section sidebar appears with Actions, Engine, Skills, etc.
3. Click Skills in sidebar — main content shows full-width skills grid
4. Cmd+J — context panel opens with quick Forge chat
5. Cmd+` — bottom panel opens with terminal + events
6. Click Code icon in rail — sidebar updates for Code department
7. Navigate to `/dept/content/chat` — full-width chat in main area, context panel collapses
8. Alt+3 — switches to third department

---

### ═══════════════════════════════════════════════════
### PHASE 2: REVENUE ENGINE HARDENING
### ═══════════════════════════════════════════════════

---

### SPRINT 4: "Revenue Ready" (Weeks 7-8)
**Goal:** Harden harvest and content engines with real-world capabilities; approval UX.

#### Stories

```
[S-026] As a builder, I want harvest CDP scraping (browser-based source), so that I can discover real freelance opportunities beyond the mock source.
Acceptance Criteria:
- [ ] New `CdpSource` struct in harvest-engine implementing Source trait
- [ ] Uses BrowserPort (via rusvel-cdp) to navigate and extract listings
- [ ] Configurable target URL and extraction selectors
- [ ] Falls back to MockSource when CDP is unavailable
- [ ] Integration test with mock HTML
Size: L
Files: crates/harvest-engine/src/source.rs (or new cdp_source.rs), crates/harvest-engine/src/lib.rs
Dependencies: none
```

```
[S-027] As a builder, I want harvest AI scoring with real LLM evaluation, so that opportunities are scored by relevance to my skills.
Acceptance Criteria:
- [ ] score_opportunity() calls AgentPort with structured prompt including user skills
- [ ] Returns numeric score (0-100) plus reasoning text
- [ ] Score and reasoning persisted in opportunity metadata
- [ ] Test with mock AgentPort verifying prompt structure
Size: M
Files: crates/harvest-engine/src/scorer.rs, crates/harvest-engine/src/lib.rs
Dependencies: none
```

```
[S-028] As a builder, I want content scheduling with calendar persistence, so that drafted content is assigned to publication dates.
Acceptance Criteria:
- [ ] `ContentCalendar` struct in content-engine
- [ ] schedule_draft(draft_id, date, platform) persists via ObjectStore
- [ ] list_scheduled(date_range) returns upcoming publications
- [ ] Event `content.scheduled` emitted on schedule
Size: M
Files: crates/content-engine/src/calendar.rs (new), crates/content-engine/src/lib.rs
Dependencies: none
```

```
[S-029] As a builder, I want an approval queue UI with approve/reject buttons, so that I can review and approve content and proposals.
Acceptance Criteria:
- [ ] Approvals page shows pending items with full context (title, body, requester)
- [ ] Approve button calls POST /api/approvals/{id}/approve
- [ ] Reject button calls POST /api/approvals/{id}/reject with optional reason
- [ ] Real-time badge update on approval count change
- [ ] Toast notification on approval action
Size: M
Files: frontend/src/routes/approvals/+page.svelte (enhance existing)
Dependencies: none
```

```
[S-030] As a builder, I want the harvest pipeline UI to show real opportunity data with scores and stage, so that I can track my deal pipeline visually.
Acceptance Criteria:
- [ ] Pipeline page shows Kanban-style columns by OpportunityStage
- [ ] Each card shows title, score, budget, source
- [ ] Drag-and-drop between stages (calls API to update stage)
- [ ] Filter by score threshold
Size: L
Files: frontend/src/routes/dept/[id]/pipeline/+page.svelte
Dependencies: S-023
```

```
[S-031] As a builder, I want the content calendar UI, so that I can see upcoming publications on a timeline view.
Acceptance Criteria:
- [ ] Calendar view showing scheduled content by date
- [ ] Click to view draft details
- [ ] Color coding by platform (LinkedIn = blue, Twitter = cyan, DEV.to = green)
- [ ] Week/month view toggle
Size: M
Files: frontend/src/routes/dept/[id]/calendar/+page.svelte
Dependencies: S-023, S-028
```

```
[S-032] As a developer, I want an end-to-end test for the harvest-to-proposal flow, so that I can verify the full pipeline works.
Acceptance Criteria:
- [ ] Test creates session, runs scan, scores top opportunity, generates proposal
- [ ] Verifies proposal enters approval queue
- [ ] Uses mock source and mock agent for determinism
Size: M
Files: tests/harvest_e2e.rs (new)
Dependencies: S-026, S-027
```

**Sprint Acceptance Criteria:**
- Harvest engine can score opportunities using LLM
- Content engine supports scheduling
- Approval queue UI is functional
- Pipeline and calendar UIs render with real data

**Sprint Demo Scenario:**
1. `cargo run` with CDP configured; harvest scan discovers mock opportunities
2. Score top opportunity; see AI-generated score and reasoning
3. Draft a proposal; it appears in approval queue
4. Open approvals page; approve the proposal
5. View content calendar with scheduled posts

---

### SPRINT 4b: "Playbooks" (Weeks 7-8, parallel with Sprint 4)
**Goal:** Implement the Playbook entity as the composition root for agents/skills/rules/workflows/hooks/MCP per use case. See `docs/design/entity-hierarchy-proposal.md`.

#### Stories

```
[S-073] As a builder, I want a Playbook domain type and CRUD API, so that I can create named bundles of capabilities for specific use cases.
Acceptance Criteria:
- [ ] Playbook struct in rusvel-core/src/domain.rs with all fields from entity-hierarchy-proposal
- [ ] PlaybookId newtype in rusvel-core/src/id.rs
- [ ] CRUD API: GET/POST /api/playbooks, GET/PUT/DELETE /api/playbooks/{id}
- [ ] POST /api/playbooks/{id}/activate?session_id={id} — set active playbook
- [ ] GET /api/playbooks/active?dept={dept}&session_id={id} — get active playbook
- [ ] Stored in ObjectStore with kind="playbooks"
- [ ] All new fields on existing entities (Agent.default_skill_ids, Rule.scope, etc.) are optional with serde defaults — zero migration needed
Size: L
Files: crates/rusvel-core/src/domain.rs, crates/rusvel-core/src/id.rs, crates/rusvel-api/src/playbooks.rs (new), crates/rusvel-api/src/lib.rs
Dependencies: none
```

```
[S-074] As a builder, I want the chat handler to resolve capabilities from the active playbook, so that chat scope narrows to the playbook's agents/skills/rules.
Acceptance Criteria:
- [ ] Chat handler step 2: load active playbook for session+department
- [ ] Step 4: load rules filtered by playbook scope (Level 2) before agent defaults (Level 3)
- [ ] Step 5: inject capabilities filtered by playbook.skill_ids
- [ ] Step 7: apply playbook config overrides (model, effort, budget)
- [ ] If no active playbook: behavior identical to current (department-wide scope)
- [ ] Resolution chain: Step overrides > Agent defaults > Playbook bundle > Department scope > Global
Size: L
Files: crates/rusvel-api/src/chat.rs, crates/rusvel-agent/src/lib.rs
Dependencies: S-073
```

```
[S-075] As a builder, I want reverse-reference endpoints, so that I can see which playbooks/workflows use a given agent or skill.
Acceptance Criteria:
- [ ] GET /api/agents/{id}/used-in → list of playbooks + workflows referencing this agent
- [ ] GET /api/skills/{id}/used-in → list of playbooks + agents + workflow steps
- [ ] GET /api/rules/{id}/used-in → list of playbooks + agents
- [ ] Queries ObjectStore for playbooks containing the entity ID in their arrays
Size: M
Files: crates/rusvel-api/src/agents.rs, crates/rusvel-api/src/skills.rs, crates/rusvel-api/src/rules.rs
Dependencies: S-073
```

```
[S-076] As a builder, I want a PlaybookSelector in Zone A sidebar, so that I can activate a playbook and see filtered capabilities.
Acceptance Criteria:
- [ ] New "Playbooks" section at top of Zone A sidebar (per entity-hierarchy-proposal §6)
- [ ] Radio-select list of playbooks for current department
- [ ] Activating a playbook calls POST /api/playbooks/{id}/activate
- [ ] Agents/Skills/Rules counts in sidebar show "in playbook / total"
- [ ] Complies with design principles: manifest-driven, no dept === 'x' checks
Size: M
Files: frontend/src/lib/components/PlaybookSelector.svelte (new), frontend/src/routes/dept/[id]/+layout.svelte
Dependencies: S-073, S-019
```

```
[S-077] As a builder, I want "Used in" sections on entity pages, so that I can see which playbooks and workflows reference each agent/skill/rule.
Acceptance Criteria:
- [ ] Agent detail page shows "Used in: Playbook X, Workflow Y step 3"
- [ ] Skill detail page shows "Used in: Playbook X, Agent Y (default)"
- [ ] Rule detail page shows "Used in: Playbook X, Agent Y (default)"
- [ ] Uses reverse-reference API (S-075)
Size: M
Files: frontend/src/lib/components/department/AgentsTab.svelte, SkillsTab.svelte, RulesTab.svelte
Dependencies: S-075
```

```
[S-078] As a builder, I want bundled "Daily Planning" and "Daily Review" playbooks for Forge, so that the daily routine is immediately usable.
Acceptance Criteria:
- [ ] On first boot, seed two Forge playbooks via rusvel-app seed logic
- [ ] "Daily Planning": mission-planner agent, daily-brief skill, focus-on-revenue rule
- [ ] "Daily Review": reviewer agent, progress-summary skill, honest-assessment rule
- [ ] "Daily Planning" marked as is_default: true for Forge department
Size: S
Files: crates/rusvel-app/src/seed.rs (or main.rs seed section)
Dependencies: S-073
```

**Sprint 4b Acceptance Criteria:**
- Playbook CRUD API functional
- Chat handler respects active playbook scope
- Zone A sidebar shows PlaybookSelector with filtered entity counts
- Entity pages show "Used in" reverse references
- Forge department has 2 seeded playbooks
- No active playbook = behavior identical to current (zero regressions)

**Sprint 4b Demo Scenario:**
1. Open Forge department; see "Daily Planning" playbook active by default
2. Zone A sidebar shows filtered: 1/3 agents, 2/5 skills, 1/4 rules
3. Chat with Forge; agent scope limited to planning tools
4. Create a new "Code Sprint" playbook with specific agents/skills
5. Switch playbooks; see sidebar counts change
6. Open an agent; see "Used in: Daily Planning, Code Sprint"

---

### SPRINT 5: "Outreach Machine" (Weeks 9-10)
**Goal:** Complete GTM outreach sequences, email adapters, AI spend tracking.

#### Stories

```
[S-033] As a builder, I want outreach sequence execution with follow-up scheduling, so that sequences can run multi-step outreach automatically.
Acceptance Criteria:
- [ ] OutreachManager::execute_sequence() iterates steps with delays
- [ ] Each step creates a job with scheduled_after for follow-up
- [ ] Step results (sent, opened, replied) tracked per sequence
- [ ] Human approval gate before each send step (ADR-008)
Size: L
Files: crates/gtm-engine/src/outreach.rs, crates/gtm-engine/src/lib.rs
Dependencies: S-004
```

```
[S-034] As a builder, I want an email adapter for outreach sends, so that outreach sequences can send real emails.
Acceptance Criteria:
- [ ] EmailAdapter trait in gtm-engine
- [ ] SMTP implementation using lettre crate
- [ ] Configuration via RUSVEL_SMTP_* env vars
- [ ] Mock adapter for testing
- [ ] Sent emails logged as events
Size: L
Files: crates/gtm-engine/src/email.rs (new), crates/gtm-engine/Cargo.toml
Dependencies: S-033
```

```
[S-035] As a builder, I want AI spend tracking per department, so that I can see how much each department costs in LLM tokens.
Acceptance Criteria:
- [ ] CostTracker in rusvel-llm already exists; ensure per-department aggregation
- [ ] New API endpoint GET /api/analytics/spend?dept=X returns cost breakdown
- [ ] Spend visible in department config page
- [ ] Budget warning when spend approaches limit
Size: M
Files: crates/rusvel-api/src/analytics.rs, crates/rusvel-llm/src/cost.rs
Dependencies: none
```

```
[S-036] As a builder, I want a CRM contacts list UI with search and filtering, so that I can manage my contacts visually.
Acceptance Criteria:
- [ ] CRM page under dept/gtm shows contacts table
- [ ] Search by name/email
- [ ] Add contact form with fields from CRM model
- [ ] Link contacts to deals
Size: M
Files: frontend/src/routes/dept/[id]/contacts/+page.svelte (new)
Dependencies: S-003
```

```
[S-037] As a builder, I want a deals pipeline UI in the GTM department, so that I can track deal stages visually.
Acceptance Criteria:
- [ ] Kanban board with deal stages (Lead, Qualified, Proposal, Won, Lost)
- [ ] Cards show deal value, contact, last activity
- [ ] Drag-and-drop between stages
Size: M
Files: frontend/src/routes/dept/[id]/deals/+page.svelte (new)
Dependencies: S-003
```

```
[S-038] As a builder, I want an invoice management UI, so that I can create and track invoices.
Acceptance Criteria:
- [ ] Invoice list page showing all invoices with status
- [ ] Create invoice form with line items
- [ ] Invoice detail view with PDF-ready layout
- [ ] Status tracking (Draft, Sent, Paid, Overdue)
Size: M
Files: frontend/src/routes/dept/[id]/invoices/+page.svelte (new)
Dependencies: S-005
```

```
[S-039] As a developer, I want end-to-end test for outreach sequence, so that I can verify multi-step outreach works.
Acceptance Criteria:
- [ ] Test creates contact, creates sequence, executes first step
- [ ] Verifies approval job created
- [ ] Verifies follow-up job scheduled
- [ ] Uses mock email adapter
Size: M
Files: tests/outreach_e2e.rs (new)
Dependencies: S-033, S-034
```

**Sprint Acceptance Criteria:**
- Outreach sequences create approval-gated send jobs
- Email adapter sends via SMTP (or mock)
- AI spend tracking aggregates by department
- GTM department has contacts, deals, and invoice UIs

**Sprint Demo Scenario:**
1. Create a contact "Jane Smith" via GTM chat
2. Create an outreach sequence targeting Jane
3. Execute sequence; first step appears in approval queue
4. Check AI spend in analytics
5. Create an invoice for a won deal

---

### ═══════════════════════════════════════════════════
### PHASE 3: CROSS-ENGINE INTELLIGENCE
### ═══════════════════════════════════════════════════

---

### SPRINT 6: "Connective Tissue" (Weeks 11-12)
**Goal:** Build webhook and cron infrastructure; enable cross-engine workflow triggers.

#### Stories

```
[S-040] As a builder, I want a rusvel-webhook crate for incoming webhook handling, so that external services can trigger RUSVEL workflows.
Acceptance Criteria:
- [ ] New crate rusvel-webhook (~500 lines)
- [ ] WebhookReceiver struct that maps URL paths to event emissions
- [ ] Webhook secrets with HMAC-SHA256 validation
- [ ] API routes: POST /api/webhooks/{id} receives payload
- [ ] GET /api/webhooks lists registered webhooks
- [ ] POST /api/webhooks creates new webhook endpoint
Size: L
Files: crates/rusvel-webhook/src/lib.rs (new crate), crates/rusvel-webhook/Cargo.toml (new), Cargo.toml (workspace), crates/rusvel-api/src/lib.rs
Dependencies: S-001
```

```
[S-041] As a builder, I want a rusvel-cron crate for scheduled job execution, so that I can run recurring tasks.
Acceptance Criteria:
- [ ] New crate rusvel-cron (~400 lines)
- [ ] CronScheduler struct with tokio interval-based execution
- [ ] Cron expression parsing (simple: hourly, daily, weekly, or cron syntax)
- [ ] Registers scheduled jobs via JobPort
- [ ] API routes: CRUD for cron jobs at /api/cron/*
- [ ] Persists schedules in ObjectStore bucket "cron_schedules"
Size: L
Files: crates/rusvel-cron/src/lib.rs (new crate), crates/rusvel-cron/Cargo.toml (new), Cargo.toml (workspace)
Dependencies: none
```

```
[S-042] As a builder, I want cross-engine workflow orchestration, so that Forge can chain Harvest + Content + GTM into sequences.
Acceptance Criteria:
- [ ] New ForgeEngine method: orchestrate_pipeline(session_id, pipeline_def)
- [ ] Pipeline definition includes ordered steps: scan -> score -> propose -> draft_content
- [ ] Each step emits events that trigger the next
- [ ] Pipeline state tracked as a FlowExecution
- [ ] Failure in any step marks pipeline as failed with context
Size: XL
Files: crates/forge-engine/src/pipeline.rs (new), crates/forge-engine/src/lib.rs
Dependencies: S-015, S-027
```

```
[S-043] As a builder, I want daily autonomous briefings, so that I get a summary of all engine states each morning.
Acceptance Criteria:
- [ ] Cron-triggered job kind "forge.daily_briefing"
- [ ] Collects status from each engine: active deals, pipeline count, scheduled content, open tickets
- [ ] Generates a structured briefing via AgentPort
- [ ] Briefing stored as an event and displayed on dashboard
Size: L
Files: crates/forge-engine/src/briefing.rs (new), crates/rusvel-cron/src/lib.rs
Dependencies: S-041
```

```
[S-044] As a builder, I want learning from outcomes (won/lost proposals), so that the harvest scorer improves over time.
Acceptance Criteria:
- [ ] When a deal is marked Won/Lost, event triggers outcome recording
- [ ] Outcome data (opportunity features + result) stored in vector store
- [ ] Scorer retrieves similar past outcomes to inform new scores
- [ ] A/B comparison test showing score improvement with outcome data
Size: L
Files: crates/harvest-engine/src/scorer.rs, crates/harvest-engine/src/outcomes.rs (new)
Dependencies: S-027
```

```
[S-045] As a builder, I want context packs per session, so that agents have standardized context about the current session.
Acceptance Criteria:
- [ ] ContextPack struct with session summary, active goals, recent events, key metrics
- [ ] Built lazily and cached per session_id with TTL
- [ ] Injected into agent system prompt when running department chat
- [ ] Configurable per department which context sections to include
Size: M
Files: crates/rusvel-agent/src/context_pack.rs (new), crates/rusvel-agent/src/lib.rs
Dependencies: none
```

```
[S-046] As a developer, I want webhook and cron integration tests, so that I can verify scheduled and triggered workflows.
Acceptance Criteria:
- [ ] Test: create cron job -> verify it fires after interval
- [ ] Test: POST to webhook endpoint -> verify event emitted
- [ ] Test: webhook triggers flow execution
Size: M
Files: tests/webhook_cron_e2e.rs (new)
Dependencies: S-040, S-041
```

**Sprint Acceptance Criteria:**
- Webhook endpoints receive external payloads and emit events
- Cron scheduler runs recurring jobs
- Cross-engine pipeline orchestrates multi-department workflows
- Daily briefing generates comprehensive status summary

**Sprint Demo Scenario:**
1. Create a cron job for daily briefing at 9am
2. Trigger briefing manually; see summary across all departments
3. Create a webhook for GitHub; push triggers a flow
4. Run cross-engine pipeline: discover opportunity -> score -> draft proposal -> create content

---

### SPRINT 7: "Intelligence Layer" (Weeks 13-14)
**Goal:** Advanced agent workflows, dashboard, documentation artifacts.

#### Stories

```
[S-047] As a builder, I want a unified dashboard showing cross-department KPIs, so that I see my entire agency at a glance.
Acceptance Criteria:
- [ ] Dashboard page (/) shows cards per department with key metric
- [ ] Pipeline: total opportunities, conversion rate
- [ ] Content: scheduled posts this week, published count
- [ ] GTM: active deals value, contacts added this month
- [ ] Finance: current balance, monthly burn
- [ ] Support: open tickets, NPS score
- [ ] Auto-refreshes every 60 seconds
Size: L
Files: frontend/src/routes/+page.svelte
Dependencies: S-010 through S-014
```

```
[S-048] As a builder, I want graph-based agent workflows (best-of-N selection), so that agents can evaluate multiple approaches and pick the best.
Acceptance Criteria:
- [ ] New flow node type "parallel_evaluate" that runs N agents in parallel
- [ ] Evaluation node scores each output and selects best
- [ ] Integrated into FlowEngine node registry
- [ ] Test with 3 parallel content drafts selecting highest quality
Size: L
Files: crates/flow-engine/src/nodes/parallel_evaluate.rs (new), crates/flow-engine/src/lib.rs
Dependencies: S-015
```

```
[S-049] As a builder, I want agents to maintain documentation artifacts, so that mission.md and architecture.md are auto-updated per session.
Acceptance Criteria:
- [ ] DocArtifact struct with path, content, last_updated
- [ ] Agent tool "doc.update" that reads current doc, merges changes, writes back
- [ ] Forge mission.today includes doc maintenance step
- [ ] Artifacts stored in ObjectStore bucket "artifacts"
Size: M
Files: crates/forge-engine/src/artifacts.rs (new), crates/dept-forge/src/lib.rs
Dependencies: none
```

```
[S-050] As a builder, I want a TUI dashboard with per-department tabs, so that the terminal interface shows department-specific views.
Acceptance Criteria:
- [ ] Tab bar in TUI showing department names
- [ ] Tab key cycles through departments
- [ ] Each tab shows department-specific data (goals, pipeline, etc.)
- [ ] Existing 4-panel layout preserved as default "Overview" tab
Size: L
Files: crates/rusvel-tui/src/lib.rs, crates/rusvel-tui/src/tabs.rs (new)
Dependencies: none
```

```
[S-051] As a builder, I want an AI spend dashboard page, so that I can see cost breakdown by department and model tier.
Acceptance Criteria:
- [ ] Spend page under /settings or analytics showing charts
- [ ] Bar chart: spend per department
- [ ] Line chart: spend over time
- [ ] Table: top 10 most expensive operations
- [ ] Budget alerts configurable
Size: M
Files: frontend/src/routes/settings/spend/+page.svelte (new)
Dependencies: S-035
```

```
[S-052] As a developer, I want performance benchmarks for the boot sequence, so that startup time stays under 2 seconds.
Acceptance Criteria:
- [ ] Benchmark test measuring boot_departments() duration
- [ ] Benchmark for full AppState construction
- [ ] Baseline recorded; CI fails if boot exceeds 3 seconds
Size: S
Files: crates/rusvel-app/benches/boot.rs (new)
Dependencies: S-016
```

**Sprint Acceptance Criteria:**
- Dashboard shows live cross-department KPIs
- Graph agent workflows support parallel evaluation
- TUI has department tabs
- AI spend is visible and trackable

**Sprint Demo Scenario:**
1. Open dashboard; see pipeline stats, content schedule, deal pipeline
2. Run a best-of-3 content draft workflow; see parallel execution in flow view
3. Check AI spend page; see cost by department
4. Launch TUI with `--tui`; tab through departments

---

### ═══════════════════════════════════════════════════
### PHASE 4: CHANNEL SYSTEM
### ═══════════════════════════════════════════════════

---

### SPRINT 8: "Open Channels" (Weeks 15-16)
**Goal:** Build channel abstraction and messaging integration.

#### Stories

```
[S-053] As a builder, I want a rusvel-channel crate defining the Channel trait, so that messaging platforms can be integrated uniformly.
Acceptance Criteria:
- [ ] New crate rusvel-channel (~1200 lines)
- [ ] Channel trait: connect(), send_message(), receive_messages(), disconnect()
- [ ] ChannelMessage struct: id, channel_id, sender, content, timestamp, metadata
- [ ] ChannelRegistry for managing multiple active channels
- [ ] Event emission on message receive: "channel.message.received"
Size: L
Files: crates/rusvel-channel/src/lib.rs (new crate), crates/rusvel-channel/Cargo.toml, Cargo.toml
Dependencies: none
```

```
[S-054] As a builder, I want a dept-messaging crate, so that I can route channel messages to departments for processing.
Acceptance Criteria:
- [ ] New crate dept-messaging (~400 lines)
- [ ] Implements DepartmentApp trait
- [ ] Routes incoming messages to target department chat based on channel config
- [ ] Sends agent responses back through the originating channel
- [ ] Manifest registers messaging tools
Size: M
Files: crates/dept-messaging/src/lib.rs (new), crates/dept-messaging/src/manifest.rs (new), Cargo.toml
Dependencies: S-053
```

```
[S-055] As a builder, I want a Telegram channel adapter, so that I can interact with RUSVEL through Telegram.
Acceptance Criteria:
- [ ] TelegramChannel struct implementing Channel trait
- [ ] Uses Telegram Bot API via reqwest
- [ ] Configurable via RUSVEL_TELEGRAM_TOKEN env var
- [ ] Supports text messages and markdown formatting
- [ ] Webhook and long-poll modes
Size: L
Files: crates/rusvel-channel/src/telegram.rs (new)
Dependencies: S-053
```

```
[S-056] As a builder, I want a Discord channel adapter, so that I can interact with RUSVEL through Discord.
Acceptance Criteria:
- [ ] DiscordChannel struct implementing Channel trait
- [ ] Uses Discord API via reqwest (REST, not gateway for simplicity)
- [ ] Configurable via RUSVEL_DISCORD_TOKEN env var
- [ ] Supports text messages with markdown
- [ ] Webhook-based receiving
Size: L
Files: crates/rusvel-channel/src/discord.rs (new)
Dependencies: S-053
```

```
[S-057] As a builder, I want a channel management UI, so that I can configure and monitor connected channels.
Acceptance Criteria:
- [ ] Channel settings page under /settings/channels
- [ ] List connected channels with status (connected/disconnected/error)
- [ ] Add channel form with provider selection and config fields
- [ ] Test connection button
- [ ] Message log view per channel
Size: M
Files: frontend/src/routes/settings/channels/+page.svelte (new)
Dependencies: S-053, S-054
```

```
[S-058] As a developer, I want channel integration tests, so that I can verify message routing works end-to-end.
Acceptance Criteria:
- [ ] Test: mock channel sends message -> dept-messaging routes to correct department
- [ ] Test: department response routed back through channel
- [ ] Test: channel registry connect/disconnect lifecycle
Size: M
Files: tests/channel_e2e.rs (new)
Dependencies: S-053, S-054
```

**Sprint Acceptance Criteria:**
- Channel trait and registry are functional
- Telegram and Discord adapters compile and pass tests with mocks
- Message routing from channel to department to response works
- Channel management UI shows connected channels

**Sprint Demo Scenario:**
1. Configure Telegram channel via settings UI
2. Send message to bot from Telegram: "Check my pipeline"
3. RUSVEL routes to harvest department, generates response
4. Response appears in Telegram chat
5. View message log in channel management UI

---

### ═══════════════════════════════════════════════════
### PHASE 5: ECOSYSTEM (Plugin SDK + Voice)
### ═══════════════════════════════════════════════════

---

### SPRINT 9: "Plugin Foundation" (Weeks 17-18)
**Goal:** Build plugin SDK and first voice interface.

#### Stories

```
[S-059] As a plugin developer, I want a rusvel-plugin-sdk crate with a clear API, so that I can create community engines and adapters.
Acceptance Criteria:
- [ ] New crate rusvel-plugin-sdk (~600 lines)
- [ ] PluginManifest struct: id, name, version, author, capabilities, dependencies
- [ ] Plugin trait: init(), register(ctx), shutdown()
- [ ] Re-exports from rusvel-core: DepartmentApp, DepartmentManifest, ToolRegistrar types
- [ ] Example plugin in docs/examples/
Size: L
Files: crates/rusvel-plugin-sdk/src/lib.rs (new crate), crates/rusvel-plugin-sdk/Cargo.toml, Cargo.toml
Dependencies: none
```

```
[S-060] As an operator, I want dynamic plugin loading from shared libraries, so that I can install plugins without recompiling RUSVEL.
Acceptance Criteria:
- [ ] Plugin loader discovers .so/.dylib files in ~/.rusvel/plugins/
- [ ] Uses libloading to call plugin entry point
- [ ] Plugin registers itself via the same RegistrationContext as built-in departments
- [ ] Sandboxing: plugins cannot access ports they did not declare in manifest
- [ ] Error handling: bad plugins log errors but do not crash RUSVEL
Size: XL
Files: crates/rusvel-plugin-sdk/src/loader.rs (new), crates/rusvel-app/src/main.rs
Dependencies: S-059
```

```
[S-061] As a builder, I want a rusvel-voice crate for speech input/output, so that I can interact with RUSVEL by voice.
Acceptance Criteria:
- [ ] New crate rusvel-voice (~800 lines)
- [ ] VoicePort trait: listen(), speak(text), is_listening()
- [ ] Whisper-based STT adapter (via API or local)
- [ ] TTS adapter using system speech or API
- [ ] Voice channel implementing Channel trait from rusvel-channel
Size: L
Files: crates/rusvel-voice/src/lib.rs (new crate), crates/rusvel-voice/Cargo.toml, Cargo.toml
Dependencies: S-053
```

```
[S-062] As a builder, I want additional LLM adapters (Gemini, DeepSeek), so that I have more model provider options.
Acceptance Criteria:
- [ ] GeminiProvider implementing LlmPort
- [ ] DeepSeekProvider implementing LlmPort (OpenAI-compatible)
- [ ] Both registered in MultiProvider
- [ ] Configurable via RUSVEL_GEMINI_KEY and RUSVEL_DEEPSEEK_KEY env vars
- [ ] Tests with mock HTTP responses
Size: M
Files: crates/rusvel-llm/src/gemini.rs (new), crates/rusvel-llm/src/deepseek.rs (new)
Dependencies: none
```

```
[S-063] As a builder, I want additional platform adapters (Medium, YouTube, Substack), so that I can publish content to more platforms.
Acceptance Criteria:
- [ ] MediumAdapter in content-engine implementing PlatformAdapter
- [ ] SubstackAdapter implementing PlatformAdapter
- [ ] YouTubeAdapter for metadata/description (not video upload)
- [ ] Each adapter configurable via env vars
- [ ] Tests with mock API responses
Size: L
Files: crates/content-engine/src/platforms/ (new directory), crates/content-engine/src/lib.rs
Dependencies: none
```

```
[S-064] As a builder, I want a browser extension for passive harvesting, so that browsing activity can feed into the harvest pipeline.
Acceptance Criteria:
- [ ] Browser extension manifest (Chrome MV3)
- [ ] Content script extracts job posting data from configurable sites
- [ ] Sends extracted data to RUSVEL webhook endpoint
- [ ] Webhook triggers harvest scoring
- [ ] Extension popup shows recent captures
Size: L
Files: extensions/browser/ (new directory, separate from Rust workspace)
Dependencies: S-040
```

```
[S-065] As a plugin developer, I want plugin documentation and a starter template, so that I can build plugins quickly.
Acceptance Criteria:
- [ ] docs/plugins/getting-started.md with tutorial
- [ ] docs/plugins/api-reference.md with full SDK docs
- [ ] Template repository structure in docs/plugins/template/
- [ ] Example: custom department plugin (e.g., "research" department)
Size: M
Files: docs/plugins/ (new directory)
Dependencies: S-059
```

**Sprint Acceptance Criteria:**
- Plugin SDK compiles and passes tests
- Dynamic loading works with example plugin on at least one platform
- Voice crate compiles with STT/TTS adapters
- New LLM providers pass mock tests

**Sprint Demo Scenario:**
1. Build example plugin as .dylib
2. Place in ~/.rusvel/plugins/; restart RUSVEL
3. New "research" department appears in registry
4. Voice interface: speak "What's on my pipeline?" and hear response
5. Publish content to Medium via content engine

---

### SPRINT 10: "Polish & Ship" (Weeks 19-20)
**Goal:** Integration testing, documentation, performance, final polish.

#### Stories

```
[S-066] As a builder, I want A2A protocol for agent-to-agent communication, so that RUSVEL agents can collaborate with external agents.
Acceptance Criteria:
- [ ] A2A message struct: sender_agent_id, recipient_agent_id, intent, payload
- [ ] A2A endpoint: POST /api/a2a/message
- [ ] Agent runtime can dispatch to external A2A endpoints
- [ ] Authentication via shared secrets
Size: L
Files: crates/rusvel-api/src/a2a.rs (new), crates/rusvel-agent/src/a2a.rs (new)
Dependencies: S-001
```

```
[S-067] As a builder, I want a community persona marketplace page, so that I can browse and install pre-built agent personas.
Acceptance Criteria:
- [ ] Persona marketplace page in frontend
- [ ] Fetches persona list from configurable URL (or bundled default)
- [ ] Install persona creates AgentProfile in ObjectStore
- [ ] Preview shows persona capabilities and sample prompts
Size: M
Files: frontend/src/routes/settings/personas/+page.svelte (new)
Dependencies: none
```

```
[S-068] As a developer, I want comprehensive CI pipeline updates, so that all new crates are tested in CI.
Acceptance Criteria:
- [ ] CI workflow updated to include new crates in cargo test/clippy/build
- [ ] Frontend tests include new pages
- [ ] Docker build includes all new crates
- [ ] Release workflow includes new crates
Size: M
Files: .github/workflows/ci.yml, Dockerfile
Dependencies: all previous
```

```
[S-069] As a developer, I want a performance test suite for critical paths, so that regressions are caught early.
Acceptance Criteria:
- [ ] Benchmark: department chat round-trip (mock LLM) < 50ms
- [ ] Benchmark: tool registry lookup < 1ms for 100 tools
- [ ] Benchmark: boot sequence < 2 seconds
- [ ] Benchmark: flow execution (3-node DAG) < 100ms
- [ ] Results tracked in CI artifacts
Size: M
Files: crates/rusvel-app/benches/ (expand), crates/rusvel-tool/benches/ (new)
Dependencies: S-052
```

```
[S-070] As a builder, I want a "Getting Started" wizard that guides first-time setup, so that new users can configure RUSVEL in under 5 minutes.
Acceptance Criteria:
- [ ] Wizard detects first run (no sessions exist)
- [ ] Step 1: Configure LLM provider (Ollama/Claude/OpenAI)
- [ ] Step 2: Set up profile (name, skills)
- [ ] Step 3: Create first session
- [ ] Step 4: Run first mission.today
- [ ] Wizard can be skipped or re-triggered from settings
Size: M
Files: frontend/src/lib/components/onboarding/SetupWizard.svelte (new)
Dependencies: none
```

```
[S-071] As a developer, I want updated mdBook documentation site covering all features, so that the docs site reflects the current state.
Acceptance Criteria:
- [ ] docs-site chapters updated for: all 13 departments, tool list, channel system, plugins, webhooks, cron
- [ ] Architecture diagram updated with channel and plugin layers
- [ ] API reference auto-generated from route definitions
- [ ] Getting started guide references setup wizard
Size: L
Files: docs-site/src/ (multiple chapters)
Dependencies: all previous
```

```
[S-072] As a builder, I want the CHANGELOG updated for all sprint deliverables, so that release notes are ready.
Acceptance Criteria:
- [ ] CHANGELOG.md entries for all sprints in conventional format
- [ ] Grouped by: Features, Fixes, Breaking Changes, Internal
- [ ] Each entry references the epic it belongs to
Size: S
Files: CHANGELOG.md
Dependencies: all previous
```

**Sprint Acceptance Criteria:**
- All 72 stories pass their acceptance criteria
- Full test suite: `cargo test --workspace` + `pnpm test:visual` pass
- Documentation site builds and deploys
- Performance benchmarks establish baselines
- Docker image builds successfully

**Sprint Demo Scenario:**
1. Fresh Docker image start with setup wizard
2. Complete wizard, create session, run mission.today
3. Full demo: discover opportunity -> score -> propose -> approve -> invoice
4. Show channel integration via Telegram
5. Load community persona; show plugin in registry
6. Performance metrics dashboard shows all green

---

## 4. PARALLEL AGENT LAUNCH PLAYGROUND

### Sprint 1: Agent Assignments
```
├── Agent 1 (worktree: feature/auth-middleware): [S-001, S-002]
├── Agent 2 (worktree: feature/gtm-tools): [S-003, S-004, S-005]
├── Agent 3 (worktree: feature/finance-tools): [S-006, S-007]
└── Agent 4 (worktree: feature/product-tools): [S-008, S-009]
Merge order: Agent 1 → Agent 2 → Agent 3 → Agent 4
Conflict zones:
  - crates/rusvel-api/src/lib.rs (Agent 1 adds auth layer)
  - crates/rusvel-app/src/boot.rs (unlikely but possible if manifest changes)
```

### Sprint 2: Agent Assignments
```
├── Agent 1 (worktree: feature/growth-support-tools): [S-010, S-011]
├── Agent 2 (worktree: feature/distro-legal-tools): [S-012, S-013]
├── Agent 3 (worktree: feature/infra-tools): [S-014, S-017]
├── Agent 4 (worktree: feature/flow-tools): [S-015]
└── Agent 5 (worktree: feature/integration-tests): [S-016] (after Agents 1-4 merge)
Merge order: Agent 1 → Agent 2 → Agent 3 → Agent 4 → Agent 5
Conflict zones:
  - None significant: each agent touches distinct dept-* crates
  - S-016 must wait for all dept crates to merge first
```

### Sprint 3: Agent Assignments
```
├── Agent 1 (worktree: feature/icon-rail): [S-018, S-025]
│   Builds: IconRail.svelte + root +layout.svelte rewrite + keyboard shortcuts
├── Agent 2 (worktree: feature/dept-layout): [S-019, S-020, S-021]
│   Builds: dept/[id]/+layout.svelte with section sidebar + context panel + bottom panel
├── Agent 3 (worktree: feature/section-pages): [S-022, S-023]
│   Builds: 13 section page wrappers + default redirect
└── Agent 4 (worktree: feature/retire-deptpanel): [S-024] (after Agents 1-3 merge)
Merge order: Agent 1 → Agent 2 → Agent 3 → Agent 4
Conflict zones:
  - frontend/src/routes/+layout.svelte (Agent 1 rewrites it; Agent 2 reads it)
  - frontend/src/lib/stores.ts (Agent 2 adds contextPanelOpen, bottomPanelOpen)
  - Recommendation: Agent 1 completes +layout.svelte icon rail first; Agent 2 builds dept layout within it
```

### Sprint 4 + 4b: Agent Assignments
```
├── Agent 1 (worktree: feature/harvest-cdp): [S-026, S-027]
├── Agent 2 (worktree: feature/content-calendar): [S-028, S-031]
├── Agent 3 (worktree: feature/approval-ui): [S-029, S-030]
├── Agent 4 (worktree: feature/playbook-backend): [S-073, S-074, S-075, S-078]
│   Builds: Playbook domain type, CRUD API, chat resolution, reverse refs, seed data
├── Agent 5 (worktree: feature/playbook-frontend): [S-076, S-077] (after Agent 4 merges)
│   Builds: PlaybookSelector in Zone A, "Used in" sections on entity pages
└── Agent 6 (worktree: feature/harvest-e2e): [S-032] (after Agent 1 merges)
Merge order: Agent 2 → Agent 3 → Agent 1 → Agent 4 → Agent 5 → Agent 6
Conflict zones:
  - crates/rusvel-core/src/domain.rs (Agent 4 adds Playbook; Agent 1 may touch Opportunity)
  - crates/rusvel-api/src/lib.rs (Agent 4 adds playbook routes; Agent 3 modifies approval routes)
  - frontend/src/routes/dept/[id]/+layout.svelte (Agent 5 adds PlaybookSelector)
```

### Sprint 5: Agent Assignments
```
├── Agent 1 (worktree: feature/outreach-execution): [S-033, S-034]
├── Agent 2 (worktree: feature/ai-spend): [S-035, S-051 from Sprint 7, pulled forward if capacity allows]
├── Agent 3 (worktree: feature/gtm-ui): [S-036, S-037, S-038]
└── Agent 4 (worktree: feature/outreach-e2e): [S-039] (after Agent 1 merges)
Merge order: Agent 2 → Agent 3 → Agent 1 → Agent 4
Conflict zones:
  - crates/gtm-engine/src/outreach.rs (Agent 1 modifies, Agent 4 tests)
  - crates/gtm-engine/Cargo.toml (Agent 1 adds lettre dependency)
```

### Sprint 6: Agent Assignments
```
├── Agent 1 (worktree: feature/webhook-crate): [S-040]
├── Agent 2 (worktree: feature/cron-crate): [S-041, S-043]
├── Agent 3 (worktree: feature/cross-engine): [S-042, S-044]
├── Agent 4 (worktree: feature/context-packs): [S-045]
└── Agent 5 (worktree: feature/webhook-cron-tests): [S-046] (after Agents 1-2 merge)
Merge order: Agent 4 → Agent 1 → Agent 2 → Agent 3 → Agent 5
Conflict zones:
  - Cargo.toml workspace members (Agents 1, 2 add new crates)
  - crates/rusvel-api/src/lib.rs (Agent 1 adds webhook routes)
  - crates/forge-engine/src/lib.rs (Agent 3 adds pipeline methods)
```

### Sprint 7: Agent Assignments
```
├── Agent 1 (worktree: feature/dashboard-kpi): [S-047]
├── Agent 2 (worktree: feature/graph-agents): [S-048]
├── Agent 3 (worktree: feature/doc-artifacts-tui): [S-049, S-050]
├── Agent 4 (worktree: feature/spend-dashboard): [S-051]
└── Agent 5 (worktree: feature/boot-bench): [S-052]
Merge order: Any order (minimal conflicts)
Conflict zones:
  - crates/forge-engine/src/lib.rs (Agent 3 adds artifacts)
  - crates/flow-engine/src/lib.rs (Agent 2 adds parallel node type)
  - Mostly independent: frontend vs engine vs benchmarks
```

### Sprint 8: Agent Assignments
```
├── Agent 1 (worktree: feature/channel-crate): [S-053]
├── Agent 2 (worktree: feature/dept-messaging): [S-054] (after Agent 1)
├── Agent 3 (worktree: feature/telegram-adapter): [S-055] (after Agent 1)
├── Agent 4 (worktree: feature/discord-adapter): [S-056] (after Agent 1)
├── Agent 5 (worktree: feature/channel-ui): [S-057] (after Agent 2)
└── Agent 6 (worktree: feature/channel-tests): [S-058] (after Agents 2-4)
Merge order: Agent 1 → Agent 3 || Agent 4 → Agent 2 → Agent 5 → Agent 6
Conflict zones:
  - crates/rusvel-channel/src/lib.rs (Agents 3, 4 add to same crate but different files)
  - Cargo.toml workspace (Agent 1 creates crate; Agents 3-4 modify Cargo.toml in that crate)
```

### Sprint 9: Agent Assignments
```
├── Agent 1 (worktree: feature/plugin-sdk): [S-059, S-065]
├── Agent 2 (worktree: feature/plugin-loader): [S-060] (after Agent 1)
├── Agent 3 (worktree: feature/voice-crate): [S-061]
├── Agent 4 (worktree: feature/llm-adapters): [S-062]
├── Agent 5 (worktree: feature/platform-adapters): [S-063]
└── Agent 6 (worktree: feature/browser-extension): [S-064]
Merge order: Agent 1 → Agent 2; Agent 3 → Agent 4 → Agent 5 → Agent 6 (parallel chain)
Conflict zones:
  - Cargo.toml workspace (Agents 1, 3 add new crates)
  - crates/rusvel-llm/src/lib.rs (Agent 4 adds providers)
  - crates/content-engine/src/lib.rs (Agent 5 adds platforms)
```

### Sprint 10: Agent Assignments
```
├── Agent 1 (worktree: feature/a2a-protocol): [S-066]
├── Agent 2 (worktree: feature/persona-marketplace): [S-067]
├── Agent 3 (worktree: feature/ci-updates): [S-068]
├── Agent 4 (worktree: feature/perf-tests): [S-069]
├── Agent 5 (worktree: feature/setup-wizard): [S-070]
├── Agent 6 (worktree: feature/docs-update): [S-071]
└── Agent 7 (worktree: feature/changelog): [S-072] (last, after all merge)
Merge order: Agent 1 → Agent 2 → Agent 5 → Agent 3 → Agent 4 → Agent 6 → Agent 7
Conflict zones:
  - .github/workflows/ci.yml (Agent 3)
  - CHANGELOG.md (Agent 7, must be last)
```

---

## 5. CHANGELOG FORMAT (Pre-Draft)

### Sprint 1: v0.2.0
```
### Features
- api: add Bearer token authentication middleware with RUSVEL_API_TOKEN support
- dept-gtm: wire 5 CRM/outreach/invoice tools for agent runtime
- dept-finance: wire 4 ledger/tax tools for agent runtime
- dept-product: wire 4 roadmap/pricing/feedback tools for agent runtime
```

### Sprint 2: v0.3.0
```
### Features
- dept-growth: wire 4 funnel/cohort/KPI tools for agent runtime
- dept-support: wire 4 ticket/knowledge/NPS tools for agent runtime
- dept-distro: wire 4 SEO/marketplace/affiliate tools for agent runtime
- dept-legal: wire 4 contract/compliance/IP tools for agent runtime
- dept-infra: wire 4 deploy/monitor/incident tools for agent runtime
- dept-flow: wire 7 flow CRUD and execution tools for agent runtime
- core: add ToolRegistrar test harness for department unit testing

### Internal
- app: add integration test verifying 60+ tools across 13 departments
```

### Sprint 3: v0.4.0
```
### Features
- frontend: restructure to 5-zone layout (icon rail + section sidebar + main + context panel + bottom panel)
- frontend: add 13 department section page wrappers (chat, config, events, engine-specific)
- frontend: add Alt+N keyboard shortcuts for department switching

### Breaking Changes
- frontend: DepartmentPanel.svelte retired; replaced by section sidebar pattern
```

### Sprint 4: v0.5.0
```
### Features
- harvest: add CDP-based source for real opportunity discovery
- harvest: add AI-powered opportunity scoring with LLM evaluation
- content: add content scheduling with calendar persistence
- frontend: add approval queue UI with approve/reject workflow
- frontend: add harvest pipeline Kanban UI
- frontend: add content calendar view
```

### Sprint 5: v0.6.0
```
### Features
- gtm: add outreach sequence execution with follow-up scheduling
- gtm: add email adapter (SMTP via lettre) for outreach sends
- analytics: add AI spend tracking per department
- frontend: add CRM contacts, deals pipeline, and invoice management UIs
```

### Sprint 6: v0.7.0
```
### Features
- webhook: new rusvel-webhook crate for incoming webhook handling with HMAC-SHA256
- cron: new rusvel-cron crate for scheduled job execution
- forge: add cross-engine workflow orchestration (harvest + content + GTM pipelines)
- forge: add daily autonomous briefings from all engine states
- harvest: add outcome learning for improved scoring
- agent: add context packs for standardized session context
```

### Sprint 7: v0.8.0
```
### Features
- frontend: add unified dashboard with cross-department KPIs
- flow: add parallel_evaluate node type for best-of-N agent selection
- forge: add documentation artifact maintenance
- tui: add per-department tabs
- frontend: add AI spend dashboard
```

### Sprint 8: v0.9.0
```
### Features
- channel: new rusvel-channel crate with Channel trait and ChannelRegistry
- messaging: new dept-messaging crate for channel-to-department message routing
- channel: add Telegram adapter
- channel: add Discord adapter
- frontend: add channel management UI
```

### Sprint 9: v0.10.0
```
### Features
- plugin-sdk: new rusvel-plugin-sdk crate for community engines and adapters
- plugin: add dynamic plugin loading from ~/.rusvel/plugins/
- voice: new rusvel-voice crate with STT/TTS adapters
- llm: add Gemini and DeepSeek providers
- content: add Medium, Substack, and YouTube platform adapters
- extensions: add browser extension for passive harvesting
```

### Sprint 10: v1.0.0
```
### Features
- a2a: add agent-to-agent communication protocol
- frontend: add community persona marketplace
- frontend: add first-run setup wizard
- docs: comprehensive documentation update for all features

### Internal
- ci: update pipeline for all new crates
- perf: add benchmark suite for critical paths (boot, chat, tools, flows)
```

---

## 7. RISK REGISTER

| # | Phase | Risk | Likelihood | Impact | Mitigation |
|---|-------|------|-----------|--------|------------|
| R1 | P0 | Stub engines have incomplete methods; tool wiring reveals gaps | High | Medium | Write tools against existing method signatures. Where engine methods are truly unimplemented (return `todo!()`), implement minimal working versions first. Each dept crate is small (~86 lines now to ~200-250 lines after). |
| R2 | P0 | Auth middleware breaks existing frontend calls | Medium | High | Make auth opt-in (only when `RUSVEL_API_TOKEN` is set). Test with both enabled and disabled. Frontend dev server should work without token. |
| R3 | P1 | Frontend 5-zone icon rail restructure breaks existing routes | Medium | High | Keep old routes working as redirects for one sprint. Visual regression tests catch layout breaks. Incremental: icon rail first, then dept layout, then section pages. |
| R4 | P1 | Context panel + bottom panel add layout complexity | Medium | Medium | Both panels are collapsible with sane defaults (context closed, bottom closed). Start with basic implementation, enhance later. Use PaneForge for resizable splits. |
| R5 | P2 | CDP scraping is flaky and site-dependent | High | Medium | Keep MockSource as fallback. Design CdpSource with configurable selectors. Test with local HTML fixtures. Target a single well-known site first. |
| R6 | P2 | Human approval gates create UX friction | Medium | Medium | Make approval configurable per department (auto-approve for low-risk actions). Clear UI with one-click approve. Batch approvals for sequences. |
| R7 | P3 | Cross-engine pipeline execution is complex and brittle | Medium | High | Build on existing FlowEngine DAG executor rather than creating a new pipeline system. Reuse event-driven step chaining. Extensive integration tests. |
| R8 | P3 | Cron scheduling precision on different platforms | Low | Low | Use tokio intervals rather than OS-level cron. Test with short intervals in CI. Document that precision is best-effort (~1 second). |
| R9 | P4 | Channel adapters have breaking API changes from upstream providers | Medium | Medium | Abstract behind Channel trait. Version-pin API versions. Maintain mock adapters for testing. Monitor provider changelogs. |
| R10 | P4 | Message routing security (channel messages reaching wrong departments) | Medium | High | Channel configuration includes department allowlist. Auth on channel connections. Rate limiting. Audit log for all routed messages. |
| R11 | P5 | Dynamic plugin loading is unsafe / platform-dependent | High | High | Start with compile-time plugins (feature flags). Dynamic loading is opt-in and experimental. Sandbox via restricted RegistrationContext (no direct port access beyond declared requirements). |
| R12 | P5 | Voice STT accuracy affects usability | Medium | Low | Voice is optional addon. Support multiple STT backends. Allow correction before sending. |
| R13 | All | Crate size limit (2000 lines) exceeded by growing dept crates | Medium | Low | Each tool is ~30-40 lines. Even 7 tools (flow dept) is ~280 lines + boilerplate. Extracting tool modules into separate files if needed. |
| R14 | All | Merge conflicts from parallel agent development | High | Medium | Clear file ownership per agent assignment. Merge in specified order. Conflict zones documented per sprint. Small, focused stories minimize overlap. |
| R15 | All | Solo builder bandwidth across 72 stories in 20 weeks | High | High | Stories sized for parallel Claude agent execution. Critical path stories prioritized. P4-P5 can be deferred without blocking core value. MVP is sprints 1-5 (10 weeks). |

---

## 8. VERIFICATION MATRIX

| Epic | Verification Method | Commands / Steps | Pass Criteria |
|------|---------------------|------------------|---------------|
| E0: Auth | Integration test + manual curl | `cargo test -p rusvel-api auth` + `curl -H "Authorization: Bearer wrong" localhost:3000/api/departments` | 401 returned for invalid token; 200 for valid |
| E1-E9: Tool Wiring | Boot integration test | `cargo test -p rusvel-app -- boot` + `GET /api/config/tools` | 60+ tools registered; each dept has >= 3 tools |
| E1: GTM Tools | Department chat test | Chat with GTM dept: "Add contact John Doe" | Tool call `gtm.crm.add_contact` visible in response; contact persisted |
| E9: Flow Tools | Flow execution test | `POST /api/flows` (create) + `POST /api/flows/{id}/run` | Flow created, executed, execution status visible |
| E10: Frontend 5-Zone | Visual regression + manual test | `pnpm test:visual` + manual navigation | Icon rail shows 13 depts; section sidebar works; context panel toggles; bottom panel toggles; no layout regression |
| E11: Revenue | End-to-end pipeline test | Run: scan -> score -> propose -> approve | Opportunity scored, proposal in approval queue, approved successfully |
| E16: Playbooks | CRUD + resolution test | Create playbook, activate, chat with dept | Chat resolves only playbook-scoped agents/skills/rules; no playbook = current behavior |
| E12: Webhook+Cron | Trigger verification | Create webhook; POST to it; verify event. Create cron; wait; verify job. | Events emitted; jobs created on schedule |
| E13: Channel | Message roundtrip test | Configure mock channel; send message; verify response routing | Message routed to correct dept; response sent back through channel |
| E14: Plugin SDK | Example plugin load test | Build example .dylib; place in plugins dir; restart; verify dept registered | Plugin department appears in `/api/departments` |
| E15: Voice | Voice loop test | Speak command; verify STT -> agent -> TTS | Spoken text transcribed; response generated; audio played back |

**End-to-End Smoke Test (after Sprint 10):**
1. `RUSVEL_API_TOKEN=test cargo run`
2. Complete setup wizard in browser
3. Create session "Demo"
4. Chat with Forge: "Plan my day" -> mission.today generates plan
5. Chat with Harvest: "Find opportunities" -> scan runs, opportunities scored
6. Chat with Content: "Draft a blog post about Rust" -> draft created
7. View approvals page -> approve draft
8. Check dashboard -> KPIs from all departments visible
9. `GET /api/config/tools` returns 60+ tools
10. `GET /api/departments` returns 14 departments (13 original + messaging)
11. Webhook POST creates event in event log
12. All `cargo test --workspace` pass

---

### Critical Files for Implementation

- `/Users/bm/rusvel/crates/rusvel-core/src/department/context.rs` -- The ToolRegistrar, EventHandlerRegistrar, and RegistrationContext that all dept-* crates use to wire tools. Every wiring story (E1-E9) depends on this API.
- `/Users/bm/rusvel/crates/dept-gtm/src/lib.rs` -- Representative unwired dept crate (~86 lines). All 8 unwired departments follow this exact pattern and need the same expansion (adding `ctx.tools.add()` calls in `register()`).
- `/Users/bm/rusvel/crates/rusvel-engine-tools/src/harvest.rs` -- The reference implementation for how engine tools are registered with ToolRegistry. Shows the exact handler pattern (`Arc::new(move |args| { Box::pin(async move { ... }) })`) to replicate across all departments.
- `/Users/bm/rusvel/crates/rusvel-api/src/lib.rs` -- The 392-line router builder where auth middleware must be layered and where new route groups (webhooks, cron, A2A, channels) will be added.
- `/Users/bm/rusvel/frontend/src/routes/+layout.svelte` -- The 402-line root layout that must be restructured from vertical sidebar to 48px icon rail per `docs/design/ui-redesign-final.md`. This is the single most impactful frontend file.
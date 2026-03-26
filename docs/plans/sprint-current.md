> **SUPERSEDED** — See sprints.md for current plan.

# Sprint: Verified State & What's Left

> Verified from disk: 2026-03-23 | cargo build: 3 warnings, 0 errors | cargo test: 192 pass | svelte-check: 0 errors, 3 warnings

---

## Verified State

### Backend — 16 modules, 3,605 lines, 40 routes

All compiled and working:

| Module | Lines | Called From | Status |
|--------|-------|------------|--------|
| `department.rs` | 497 | Routes: 6 parameterized (`/api/dept/{dept}/*`) | @mention, rules, MCP config, !build — all wired |
| `build_cmd.rs` | 583 | `department.rs` calls `parse_build_command()` | Working |
| `capability.rs` | 422 | Route: `POST /api/capability/build` | Working |
| `workflows.rs` | 342 | Routes: CRUD + `/api/workflows/{id}/run` | Working |
| `chat.rs` | 275 | Routes: `/api/chat` + conversations | Working |
| `hook_dispatch.rs` | 174 | **NOT CALLED FROM ANYWHERE** | Implemented but dead code |
| `config.rs` | 162 | Routes: config + models + tools | Working |
| `skills.rs` | 157 | Routes: CRUD | Working |
| `routes.rs` | 150 | Routes: sessions, mission, goals, events | Working |
| `agents.rs` | 134 | Routes: CRUD | Working |
| `mcp_servers.rs` | 134 | Routes: CRUD | Working |
| `rules.rs` | 117 | Routes: CRUD + `load_rules_for_engine()` called in department.rs | Working |
| `help.rs` | 109 | Route: `POST /api/help` | Working |
| `hooks.rs` | 104 | Routes: CRUD + `list_hook_events()` | Working |
| `analytics.rs` | 82 | Route: `GET /api/analytics` | Working |
| `lib.rs` | 163 | Router + AppState + DepartmentRegistry | Working |

### New additions since last audit

| Component | Lines | Status |
|-----------|-------|--------|
| `DepartmentRegistry` in rusvel-core | — | Dynamic department definitions |
| `GET /api/departments` | — | Returns department list from registry |
| `GET/PUT /api/profile` | — | User profile management |
| `departments` store in frontend | — | Writable store loaded from API |
| `dept/[id]/+page.svelte` | 33 | Dynamic route — reads from departments store |
| `onboarding/CommandPalette.svelte` | 184 | Command palette component |
| `onboarding/OnboardingChecklist.svelte` | 100 | Checklist component |
| `onboarding/ProductTour.svelte` | 120 | Tour component |
| `onboarding/DeptHelpTooltip.svelte` | 51 | Help tooltip |
| `workflow/WorkflowBuilder.svelte` | 119 | Workflow builder UI |
| `workflow/AgentNode.svelte` | 17 | Agent node in workflow graph |
| `Separator.svelte` | — | New UI component |
| `getAnalytics()` in api.ts | — | Calls `/api/analytics` |
| Dashboard analytics cards | — | Shows agents, skills, rules, conversations counts |

### Frontend totals

- 61 source files (.svelte + .ts)
- 585 files checked by svelte-check
- 16 route pages (dashboard + chat + 12 departments + settings + dept/[id])
- 20 UI components
- 4 onboarding components
- 2 workflow components

### Build health

**Rust (3 warnings):**
1. `forge-engine` — fields `memory`, `jobs`, `session`, `config`, `safety` never read
2. `rusvel-api/chat.rs` — unused import `axum::response::IntoResponse`
3. `finance-engine/runway.rs` — unused import `rusvel_core::domain::ObjectFilter`

**Frontend (3 warnings):**
1. `DepartmentPanel.svelte:659` — div with mousedown handler needs ARIA role
2. `CommandPalette.svelte:119` — dialog role needs tabindex
3. `+layout.svelte:231` — div with mousedown handler needs ARIA role

---

## What's Actually Missing

### 1. `dispatch_hooks()` never called (~15 min fix)

`hook_dispatch.rs` (174 lines) is fully implemented — command, http, and prompt hook types. But `department.rs` never calls `dispatch_hooks()`. Need one line after the `StreamEvent::Done` branch stores the assistant message:

```rust
crate::hook_dispatch::dispatch_hooks(
    &format!("{}.chat.completed", engine),
    serde_json::json!({"conversation_id": conv_id, "cost_usd": cost}),
    engine,
    storage.clone(),
);
```

### 2. Dynamic Tailwind still partially broken (~30 min fix)

`DepartmentPanel.svelte:531` still uses `bg-${color}-900/30 text-${color}-400`. The shadcn migration fixed most components but this one line (and possibly others in DepartmentPanel) still interpolates.

### 3. Job queue worker loop (~3h)

No `dequeue` or worker loop in `main.rs`. `JobPort` infrastructure exists but nothing processes jobs.

### 4. No approval API/UI (~3h)

`ApprovalStatus` type exists in domain. No endpoints for listing pending approvals or approve/reject actions.

### 5. rust-embed (~1h)

Frontend served from filesystem, not embedded in binary.

### 6. Docs massively outdated (~3h)

CLAUDE.md says 20 crates/149 tests/5 engines. Reality: 27 crates/192 tests/12 departments. Architecture docs don't mention DepartmentRegistry, capability engine, hook dispatch, onboarding, workflow builder, dynamic routes, shadcn migration.

---

## Corrected Priorities

| # | Item | Effort | Type |
|---|------|--------|------|
| 1 | Wire `dispatch_hooks()` in department.rs | 15m | Fix |
| 2 | Fix 3 Rust warnings | 10m | Fix |
| 3 | Fix 3 a11y warnings | 10m | Fix |
| 4 | Fix remaining dynamic Tailwind in DepartmentPanel | 30m | Fix |
| 5 | Job queue worker loop | 3h | Feature |
| 6 | Approval API + UI | 3h | Feature |
| 7 | rust-embed frontend | 1h | Feature |
| 8 | Update CLAUDE.md | 1h | Docs |
| 9 | Rewrite current-state.md | 1h | Docs |
| 10 | Update architecture-v2.md + decisions.md | 1h | Docs |
| **Total** | | **~11h** |

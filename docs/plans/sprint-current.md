# Sprint Plan: Fix, Polish, and Ship

> Actual codebase audit as of 2026-03-23 (commit 3ae7020 + uncommitted).

---

## Codebase Reality

### What's Built (13 API modules, 60 routes, 2,938 lines in rusvel-api alone)

| Module | Lines | Status |
|--------|-------|--------|
| `department.rs` | 564 | Built: 5 departments, agent @mention, rules injection, MCP config injection, `!build` interceptor |
| `build_cmd.rs` | 592 | Built: Capability Engine — `!build agent/skill/rule/mcp/hook: description` |
| `workflows.rs` | 343 | Built: CRUD + `POST /api/workflows/{id}/run` with dependency resolution |
| `chat.rs` | 275 | Built: God agent streaming chat + conversation history |
| `lib.rs` | 184 | Built: 60 routes registered |
| `config.rs` | 162 | Built: model/effort/tools config |
| `routes.rs` | 150 | Built: sessions, mission, goals, events |
| `agents.rs` | 134 | Built: CRUD for AgentProfile |
| `mcp_servers.rs` | 134 | Built: CRUD + `build_mcp_config_for_engine()` |
| `rules.rs` | 117 | Built: CRUD + `load_rules_for_engine()` |
| `hooks.rs` | 104 | Built: CRUD + `list_hook_events()` |
| `skills.rs` | 97 | Built: CRUD |
| `analytics.rs` | 82 | Built: `GET /api/analytics` aggregate counts |

### What's Wired into Department Chat (`department.rs`)

| Feature | Line | Status |
|---------|------|--------|
| Agent @mention override | ~231 | DONE — loads agent, overrides config |
| Rules injection into prompt | ~249 | DONE — appends enabled rules to system prompt |
| MCP config injection | ~272 | DONE — `build_mcp_config_for_engine()` adds `--mcp-config` flag |
| `!build` interceptor | ~231 | DONE — routes to `build_cmd.rs` |

### Frontend DepartmentPanel.svelte — 8 tabs

| Tab | Status |
|-----|--------|
| Actions | DONE — quick actions per department |
| Agents | DONE — live CRUD from API |
| Skills | DONE — live CRUD, click-to-fill |
| Rules | DONE — live CRUD, enable/disable toggle |
| MCP | DONE — live CRUD |
| Hooks | DONE — live CRUD |
| Dirs | DONE — add/remove working directories |
| Events | DONE — filtered by department |

### What's Also Built

| Feature | Status |
|---------|--------|
| MCP `--mcp` flag in main.rs | DONE (line 302) |
| Workflows CRUD + run endpoint | DONE |
| Analytics endpoint | DONE |
| 5 department pages (Code/Content/Harvest/GTM/Forge) | DONE |
| Design system (14 components) | DONE |
| God agent chat | DONE |

---

## What's Broken

### Build Error (must fix first)

**`department.rs:398`** — type mismatch in `!build` interceptor's SSE stream.

The `!build` capability branch returns a different stream type than the normal chat branch. Both need to return `Sse<impl Stream<Item = Result<Event, Infallible>>>` but the `!build` path produces `StreamEvent` instead of `axum::response::sse::Event`.

**Fix:** The `!build` branch's stream `.map()` must convert `StreamEvent` → `axum::response::sse::Event`, matching the pattern in the normal chat branch (lines 280-310).

---

## What's Actually Left

### 1. Fix Build Error (~30 min)

Fix the type mismatch in `department.rs:398`. The `!build` interceptor needs the same `StreamEvent` → `sse::Event` mapping as the normal chat handler.

### 2. Wire Hooks into Chat Execution (~1 hour)

Hooks CRUD exists but hooks don't **fire** after chat completion. In `department.rs`, after `StreamEvent::Done` stores the assistant message, add:

```rust
let hooks = crate::hooks::load_hooks_for_event(&state_clone, "chat.completed").await;
for hook in hooks {
    if hook.hook_type == "command" { if let Some(cmd) = &hook.command {
        let cmd = cmd.clone();
        tokio::spawn(async move {
            let _ = tokio::process::Command::new("sh").arg("-c").arg(&cmd).output().await;
        });
    }}
}
```

Need to add `load_hooks_for_event()` to `hooks.rs` (same pattern as `load_rules_for_engine` in `rules.rs`).

### 3. Seed Data on First Boot (~1 hour)

ObjectStore starts empty. First boot should insert defaults so the UI isn't blank.

In `main.rs`, after DB init:
```rust
if storage.objects().list("agents", ObjectFilter::default()).await?.is_empty() {
    seed_defaults(&storage).await;
}
```

Insert: 5 agents, 5 skills, 3 rules, 2 MCP server suggestions, 1 sample workflow.

### 4. Update All Docs (~2 hours)

The audit found massive documentation drift:

| Doc | Issue |
|-----|-------|
| **CLAUDE.md** | Claims "7 endpoints" — actually 60 routes. Claims "149 tests". Missing: departments, agents/skills/rules CRUD, MCP servers, hooks, workflows, analytics, build commands |
| **current-state.md** | Claims "70% complete" — much more is done. harvest-engine claims 12 tests (actually 7). Missing: entire department abstraction |
| **gap-analysis.md** | Doesn't reflect that rules, agents, MCP, hooks, workflows, analytics are now built |
| **phase-0-foundation-v2.md** | Checkboxes stale. Many unchecked items are done |
| **modular-ui-blueprint.md** | Many modules marked "not built" that are built (model picker, tools toggle, agent gallery) |

### 5. Job Queue Worker Loop (~3 hours)

`JobPort` has enqueue/dequeue/approve/complete/fail. No worker loop processes jobs.

Add to `main.rs`:
```rust
let job_port = job_port.clone();
tokio::spawn(async move {
    loop {
        if let Ok(Some(job)) = job_port.dequeue(&[]).await {
            match job.kind {
                JobKind::ContentPublish => { /* call content-engine */ }
                JobKind::HarvestScan => { /* call harvest-engine */ }
                JobKind::OutreachSend => { /* call gtm-engine */ }
                _ => {}
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
});
```

### 6. Frontend Polish (~3 hours)

- **Dashboard** (`+page.svelte`): Shows goals + events but no department activity summary. Add: recent conversations count per dept, total agents/skills/rules counts (from `/api/analytics`)
- **Settings** (`settings/+page.svelte`): Still shows static status cards. Wire to real data: health check, analytics endpoint, global agents/skills/rules lists
- **Workflow UI**: Backend exists (`workflows.rs` with run endpoint) but no frontend page/tab for workflows yet. Add workflows tab to DepartmentPanel or Settings
- **Chat improvements**: Show `@agent-name` autocomplete suggestions in input. Show which rules are active in the config panel

### 7. Port Safety Patterns (~4 hours)

From `gap-analysis.md` — these exist in `old/forge-project` but not in RUSVEL:
- **Circuit Breaker** (3-state FSM: closed→open→half-open)
- **Rate Limiter** (token bucket)
- **Cost Tracker** (budget enforcement per engine)
- **Loop Detector** (detect repeating LLM outputs)
- **Context Pruner** (token-aware truncation)

SafetyGuard in forge-engine has basic versions but not the full implementations from old repos.

---

## Dependency Graph

```
1. Fix build error ←── BLOCKS EVERYTHING
   └→ 2. Wire hooks into chat
   └→ 3. Seed data
   └→ 5. Job queue worker
   └→ 6. Frontend polish
4. Update docs (independent)
7. Port safety patterns (independent)
```

---

## Effort Summary

| Step | Description | Effort | Priority |
|------|-------------|--------|----------|
| 1 | Fix build error in department.rs | 30m | BLOCKER |
| 2 | Wire hooks execution into chat | 1h | High |
| 3 | Seed data on first boot | 1h | High |
| 4 | Update all docs to match reality | 2h | High |
| 5 | Job queue worker loop | 3h | Medium |
| 6 | Frontend polish (dashboard, settings, workflows UI) | 3h | Medium |
| 7 | Port safety patterns from old repos | 4h | Medium |
| **Total** | | **~14.5 hours** |

At 3-4 hours/day → **4-5 days**.

---

## Definition of Done

- [ ] `cargo build` succeeds (build error fixed)
- [ ] `cargo test` — all tests pass
- [ ] `npm run check` — clean
- [ ] Hooks fire after chat completion
- [ ] First boot populates default agents/skills/rules
- [ ] CLAUDE.md reflects 60 routes, 154+ tests, department architecture
- [ ] current-state.md reflects what's actually built
- [ ] Job queue worker processes at least one job type
- [ ] Dashboard shows real activity data
- [ ] Settings page wired to analytics + health
- [ ] Workflows have frontend UI
- [ ] Safety patterns ported (circuit breaker, rate limiter)

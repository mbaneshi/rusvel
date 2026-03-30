# Sprint 6: Pattern Extraction — Flow Resilience, Multi-Channel, Self-Improvement

> **Date:** 2026-03-30 | **Status:** Proposed | **Est. total:** ~18 working days
> **ADRs:** [ADR-015](../design/decisions.md#adr-015), [ADR-016](../design/decisions.md#adr-016), [ADR-017](../design/decisions.md#adr-017)
> **Design:** [pattern-extraction-design.md](pattern-extraction-design.md)

---

## Theme

Extract hard-won patterns from reference repos (n8n, OpenClaw, everything-claude-code) and rebuild them inside RUSVEL's hexagonal architecture. Three features that plug production gaps.

---

## Note on Sprint 4 #25 Overlap

Sprint 4 task #25 ("Durable Execution — checkpoint/resume for FlowEngine, retry per node") **partially shipped already** — checkpoint persistence, resume from failure, and single-node retry all exist in `flow-engine`. Tasks 34-36 below cover the remaining gap: *automatic* retry with backoff and per-node timeouts. Mark Sprint 4 #25 as partially complete; remaining effort absorbed here.

---

## Tasks

### Phase A: Flow Resilience (ADR-015) — ~4 days

| # | Task | Effort | Status | Depends On |
|---|------|--------|--------|------------|
| 34 | **RetryPolicy domain type** — `RetryPolicy` struct + `retry_policy`/`timeout_secs` on `FlowNodeDef` + `Timeout` error variant | 0.5d | Not started | — |
| 35 | **execute_with_retry()** — timeout wrapper + exponential backoff loop in executor, replace bare `handler.execute()` | 2d | Not started | #34 |
| 36 | **Workflow export/import** — `export_flow`/`import_flow` on FlowEngine + `GET /api/flows/:id/export`, `POST /api/flows/import` | 1d | Not started | — |
| 37 | **Flow resilience tests** — retry counter handler, timeout handler, export/import round-trip, backward compat deser | 0.5d | Not started | #35, #36 |

**Parallelizable:** 34+36 parallel. 35 after 34. 37 after 35+36.

---

### Phase B: Multi-Channel Messaging (ADR-016) — ~6 days

| # | Task | Effort | Status | Depends On |
|---|------|--------|--------|------------|
| 38 | **ChannelMessage domain type** — `ChannelMessage`, `MessageDirection` in `rusvel-core` | 0.5d | Not started | — |
| 39 | **ChannelRegistry** — multi-channel router implementing `ChannelPort`, route-by-kind + broadcast | 1d | Not started | — |
| 40 | **SlackChannel adapter** — webhook-based, `from_env()`, `RUSVEL_SLACK_WEBHOOK_URL` | 0.5d | Not started | — |
| 41 | **DiscordChannel adapter** — webhook-based, `from_env()`, `RUSVEL_DISCORD_WEBHOOK_URL` | 0.5d | Not started | — |
| 42 | **Channel API routes** — `channel_routes.rs`: list, send, broadcast, inbox, inbound webhook | 2d | Not started | #38, #39 |
| 43 | **Composition root wiring** — replace single `TelegramChannel` with `ChannelRegistry` in main.rs | 0.5d | Not started | #39, #40, #41 |
| 44 | **Multi-channel tests** — adapter payload format, registry routing, inbound storage, integration round-trip | 1d | Not started | #42, #43 |

**Parallelizable:** 38+39+40+41 all parallel. 42 after 38+39. 43 after 39+40+41. 44 after 42+43.

---

### Phase C: Self-Improvement Loop (ADR-017) — ~8 days

| # | Task | Effort | Status | Depends On |
|---|------|--------|--------|------------|
| 45 | **Session/Build domain types** — `SessionContext`, `EntityRef`, `BuildRecord`, `BuildSuggestion`, `SuggestionStatus` in `rusvel-core` | 0.5d | Not started | — |
| 46 | **Session context persistence** — `session_context.rs`: `save_session_context()` (LLM extraction), `load_session_context()` | 2d | Not started | #45 |
| 47 | **Session restore in chat** — inject previous `SessionContext` into system prompt in `dept_chat` | 0.5d | Not started | #46 |
| 48 | **Pattern extraction** — `extract_patterns()`: LLM compares session against existing skills/rules, generates `BuildSuggestion` | 2d | Not started | #46 |
| 49 | **Build history tracking** — create `BuildRecord` on `!build` success, increment `usage_count` in `resolve_skill()` | 1d | Not started | #45 |
| 50 | **Session end endpoint** — `POST /api/dept/:dept/sessions/:id/end` triggers save + extract + event | 0.5d | Not started | #46, #48 |
| 51 | **Build suggestions API** — `GET/POST /api/build/suggestions`, accept/dismiss, `GET /api/build/history` | 1d | Not started | #48, #49 |
| 52 | **Self-improvement tests** — context save/load, pattern extraction output, usage tracking, session restore in prompt | 0.5d | Not started | #50, #51 |

**Parallelizable:** 45 first. Then 46+49 parallel. Then 47+48 parallel. Then 50+51 parallel. 52 last.

---

## Critical Path

```
Phase A (4d):  34 ──→ 35 ──→ 37
               36 ──────────↗

Phase B (6d):  38 ──────→ 42 ──→ 44
               39 ──→ 42, 43 ──↗
               40 ──→ 43
               41 ──→ 43

Phase C (8d):  45 ──→ 46 ──→ 47
                      46 ──→ 48 ──→ 50 ──→ 52
                 45 ──→ 49 ──→ 51 ──────↗
```

**Phases A, B, C are independent** — can run in parallel branches if desired. Sequential critical path: ~18 days. With full parallelization across phases: ~8 days (Phase C dominates).

---

## Files Created/Modified Summary

### New files (7)

| File | Phase | Lines |
|------|-------|-------|
| `crates/rusvel-channel/src/slack.rs` | B | ~80 |
| `crates/rusvel-channel/src/discord.rs` | B | ~80 |
| `crates/rusvel-channel/src/registry.rs` | B | ~100 |
| `crates/rusvel-api/src/channel_routes.rs` | B | ~200 |
| `crates/rusvel-api/src/session_context.rs` | C | ~200 |
| `crates/rusvel-api/src/build_suggestions.rs` | C | ~150 |
| `crates/flow-engine/tests/retry_timeout.rs` | A | ~100 |

### Modified files (10)

| File | Phase | Change |
|------|-------|--------|
| `crates/rusvel-core/src/domain.rs` | A,B,C | +95 lines (RetryPolicy, ChannelMessage, SessionContext, BuildRecord, BuildSuggestion) |
| `crates/rusvel-core/src/error.rs` | A | +3 lines (Timeout variant) |
| `crates/flow-engine/src/executor.rs` | A | +60 lines (execute_with_retry) |
| `crates/flow-engine/src/lib.rs` | A | +40 lines (export/import) |
| `crates/rusvel-api/src/flow_routes.rs` | A | +30 lines (export/import handlers) |
| `crates/rusvel-channel/src/lib.rs` | B | +6 lines (re-exports) |
| `crates/rusvel-api/src/lib.rs` | B,C | +20 lines (new modules + routes) |
| `crates/rusvel-app/src/main.rs` | B | ~15 lines (ChannelRegistry wiring) |
| `crates/rusvel-api/src/build_cmd.rs` | C | +20 lines (BuildRecord creation) |
| `crates/rusvel-api/src/department.rs` | C | +30 lines (session restore + end endpoint) |
| `crates/rusvel-api/src/skills.rs` | C | +5 lines (usage tracking) |

### Line budget check

| Crate | Current | Added | Total | Safe? |
|-------|---------|-------|-------|-------|
| flow-engine | ~1819 | +100 | ~1919 | Yes |
| rusvel-channel | ~112 | +266 | ~378 | Yes |
| rusvel-core | multi-file | +98 | — | Yes (per-file) |
| rusvel-api | multi-file | +455 across 3 new + 4 modified | — | Yes (per-file) |

---

## Verification Checklist

```bash
# After each phase
cargo check                        # Workspace compiles
cargo test -p flow-engine          # Phase A
cargo test -p rusvel-channel       # Phase B
cargo test -p rusvel-api           # Phases A, B, C
cargo test                         # Full suite — 554+ tests still pass

# Manual smoke tests
# Phase A: create flow with retry_policy, trigger with failing node, observe retries
# Phase B: set RUSVEL_SLACK_WEBHOOK_URL, POST /api/channels/send, verify Slack
# Phase C: chat, POST session end, GET /api/build/suggestions
```

---

## Backlog Items Unblocked by This Sprint

| Item | Unblocked By |
|------|-------------|
| WhatsApp/Signal adapters | #39 (ChannelRegistry pattern established) |
| Email channel adapter (SMTP) | #39 |
| Webhook-triggered flows with retry | #35 |
| Auto-generated workflow templates | #36 (export/import) |
| dept-messaging full implementation | #42, #43 |
| Self-healing agent pipelines | #48 (pattern extraction) + #35 (retry) |

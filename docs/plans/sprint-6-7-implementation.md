# Sprint 6–8 implementation matrix

Story → primary crate → route / surface → tests. Use as an execution checklist (aligned with [`openclaw-sprint-plan.md`](openclaw-sprint-plan.md)).

## Sprint 6 (close-out)

| Story | Crate / area | Route / surface | Tests / notes |
|-------|----------------|-----------------|---------------|
| S-040 Webhooks | `rusvel-webhook`, `rusvel-api` | `GET/POST /api/webhooks`, `POST /api/webhooks/{id}` | `webhook_cron_e2e.rs`, `webhooks_e2e.rs` |
| S-041 Cron | `rusvel-cron`, `rusvel-app` | `/api/cron*`, `POST /api/cron/tick` | `cron_api.rs`, `cron_api_smoke.rs` |
| S-042 Pipeline | `forge-engine`, `rusvel-api` | `POST /api/forge/pipeline` | `forge-engine/tests/pipeline_orchestration.rs`, `rusvel-api/tests/forge_pipeline_api.rs` |
| S-043 Daily brief | `forge-engine`, worker | Cron `forge.daily_briefing` → `generate_brief` | Events + `GET/POST /api/brief*` |
| S-044 Outcomes | `harvest-engine`, `rusvel-api` | `POST /api/dept/harvest/outcome`, `GET .../outcomes`, advance → record | `harvest_outcomes_smoke.rs` |
| S-045 Context pack | `rusvel-agent`, `rusvel-api` | Dept chat injects pack; `PUT /api/dept/{id}/config` `context_pack` flags; TTL cache in `AppState.context_pack_cache` | Manual / agent `context_pack` tests |
| S-046 E2E | `rusvel-api` | (webhooks + cron + worker) | `webhook_cron_e2e.rs` |

## Sprint 7 (intelligence layer)

| Story | Crate / area | Route / surface | Tests |
|-------|----------------|-----------------|-------|
| S-051 Spend UI | `frontend` | `/settings/spend`, `getAnalyticsSpend` | Visual / manual |
| S-047 Dashboard API | `rusvel-api` | `GET /api/analytics/dashboard` | Same filters as `/api/analytics/spend` |
| S-049 Artifacts | `forge-engine`, `rusvel-builtin-tools`, `rusvel-api` | Tool `forge_save_artifact`, `GET /api/forge/artifacts` | `cargo test -p forge-engine` |
| S-050 TUI tabs | `rusvel-tui` | `tabs.rs` panel labels → `widgets.rs` | — |
| S-048 `parallel_evaluate` | `flow-engine` | Flow node type `parallel_evaluate`, `parameters.branches[]` | Flow engine unit tests (extend as needed) |

## Sprint 8 (channels)

| Story | Crate / area | Notes |
|-------|----------------|-------|
| Channel port | `rusvel-channel` | `ChannelPort` trait — adapters implement next |
| Dept shell | `dept-messaging` | Placeholder crate; wire in `rusvel-app` last |

## Cross-cutting auth (phased)

| Phase | Location | Behavior |
|-------|----------|----------|
| 1 | `rusvel-api/src/auth.rs` | `RUSVEL_API_TOKEN` set → `/api/*` require `Authorization: Bearer` except non-`/api/*` routes (SPA), `/api/health`, `POST /api/webhooks/{id}` receive |
| 2 | TBD | Session-scoped API keys (out of band) |

## Quick API index (this roadmap)

- `POST /api/forge/pipeline` — cross-engine pipeline (S-042).
- `GET /api/analytics/dashboard` — counts + spend (S-047).
- `GET /api/forge/artifacts?session_id=` — Forge artifacts (S-049).
- `GET /api/brief/latest` — latest persisted brief (S-043 UI).

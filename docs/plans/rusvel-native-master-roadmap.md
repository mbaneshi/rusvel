# RUSVEL native app — master roadmap (code-mapped)

**Purpose:** Single place that maps product direction to this repository: crates, routes, and phased work. Complements [browser-fleet-and-capture-architecture.md](./browser-fleet-and-capture-architecture.md) and [autonomous-freelance-agency.md](./autonomous-freelance-agency.md).

**Principles:** One Rust binary + embedded SvelteKit UI; hexagonal ports in `rusvel-core`; engines depend on traits only; no second harvest application stack in production.

---

## 1. Native primitives

| Concept | Location | Role |
|--------|----------|------|
| Domain + ADR-007 metadata | `crates/rusvel-core/src/domain.rs` | `Opportunity`, `ApprovalPolicy`, `BrowserEvent`, `UserProfile`, jobs, sessions |
| Ports | `crates/rusvel-core/src/ports.rs` | `LlmPort`, `AgentPort`, `ToolPort`, `JobPort`, `BrowserPort`, … |
| Composition root | `crates/rusvel-app/src/main.rs` | DB, LLM, departments, job worker, cron → events |
| HTTP API | `crates/rusvel-api/src/lib.rs` + handlers | `/api/dept/*`, `/api/system/*`, chat, flows, knowledge |
| Harvest | `crates/harvest-engine/` | `HarvestEngine::scan`, `HarvestSource`, `scan_from_params` (unified scan entry) |
| CDP | `crates/rusvel-cdp/` | `BrowserPort` implementation |
| Agent loop | `crates/rusvel-agent/` | Tool loop, `ToolPermissionMode` |
| Engine tools | `crates/rusvel-engine-tools/` | `harvest_scan`, … |
| Forge pipeline | `crates/forge-engine/`, `rusvel-api/src/pipeline_runner.rs` | Harvest → content steps |
| Rusvel hooks | `crates/rusvel-api/src/hook_dispatch.rs` | ObjectStore hooks on `event_kind` (not Claude Code lifecycle hooks) |

---

## 2. External inspiration → native equivalent

| External | RUSVEL equivalent |
|----------|-------------------|
| Claude Cowork / delegated work | `JobPort` + approvals + flows + cron |
| Claude Code hooks | Rusvel hooks on domain events; optional `.claude/hooks` calling local API |
| Messages API / server tools | `ClaudeProvider` + agent loop; default Claude path may be `ClaudeCliProvider` |
| MCP (Claude Code) | `rusvel-mcp-client` — not the same as Claude.ai Connectors |
| CRM (e.g. Twenty) | `gtm-engine`; reference for UX/sync — keep stable `opportunity_id` in metadata |
| Research skills (e.g. multi-source digest) | Harvest + jobs + knowledge — same normalize → score shape |

---

## 3. Pillars

**A — Agency & capture (Phase 1–2)**  
Extend `Opportunity`, normalization/dedup/`content_hash`, always Rusvel re-score, `POST /api/dept/harvest/ingest`, CDP → `BrowserEvent::DataCaptured`, `CdpPool`. See autonomous-freelance-agency and browser-fleet docs.

**B — LLM / Anthropic**  
Tracks: product UX on jobs/flows; HTTP Claude streaming + server tools; OAuth-style connectors. See `rusvel-llm/src/claude.rs`, `claude_cli.rs`.

**C — Governance & cost**  
`ToolPermissionMode`, ADR-008 approvals, `CostEvent` / spend analytics (`rusvel-api/src/analytics.rs`, settings spend page).

**D — Orchestration**  
Forge pipeline, playbooks, flows, capability / `!build`, GTM ↔ harvest IDs.

---

## 4. Phased waves (summary)

| Wave | Focus |
|------|--------|
| **Unified scan** | Single `scan_from_params` for HTTP scan, `HarvestScan` jobs, forge pipeline scan step, `harvest_scan` tool (implemented in-tree). |
| **Domain + ingest** | A0 `Opportunity` + migration; A1b mapping; required ingest API. |
| **Continuous capture** | CDP network → normalization; cron/hooks for `harvest.auto_scan`. |
| **Multi-profile** | `CdpPool`, multi-endpoint scan. |
| **Optional** | `session-snapshot` for external hooks; playbook persistence; Anthropic HTTP parity. |

---

## 5. Related Cursor plans

- Autonomous Agency Phase 1 (A0/A1b, ingest, CDP depth) — `.cursor/plans/autonomous_agency_phase_1_*.plan.md`
- Anthropic vs Rusvel (tracks A/B/C) — `.cursor/plans/anthropic_vs_rusvel_map_*.plan.md`

---

*Last updated: 2026-03-30 — aligned with unified harvest scan execution in `harvest-engine::scan_from_params`.*

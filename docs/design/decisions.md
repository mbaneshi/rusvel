# RUSVEL — Architecture Decision Records

---

## ADR-001: Merge Ops + Connect into GoToMarket Engine

**Date:** 2026-03-21
**Status:** Accepted
**Context:** Original design had 7 engines including separate Ops (CRM, invoicing, SOPs) and Connect (outreach, sequences, follow-ups) engines. Perplexity review flagged that 7 is too many for a solo founder product.
**Decision:** Merge into single GoToMarket engine. CRM, outreach, pipeline, invoicing all live together.
**Consequence:** 5 engines instead of 7. GoToMarket engine is larger but cohesive around "find and close business."

---

## ADR-002: Fold Mission into Forge Engine

**Date:** 2026-03-21
**Status:** Accepted
**Context:** Mission engine (goals, daily planning, reviews) was a separate engine. Perplexity noted that mission is really "orchestrated agents" — goal-setting, daily planning, and reviews are all agent tasks.
**Decision:** Mission becomes a sub-module of Forge Engine. Goals, tasks, and reviews are managed by mission agents.
**Consequence:** `rusvel forge mission today` instead of `rusvel mission today`. Forge is the meta-engine.

---

## ADR-003: Single Central Job Queue (replaces AutomationPort + SchedulePort)

**Date:** 2026-03-21
**Status:** Accepted
**Context:** Original design had AutomationPort (workflows), SchedulePort (cron/triggers), plus Forge (agent orchestration) and per-engine scheduling. This created 4 overlapping workflow substrates.
**Decision:** Single JobPort with SQLite-backed job queue. All async work goes through one queue. Worker pool routes jobs to engines by JobKind.
**Consequence:** One place to see all pending/running/completed work. Simpler debugging. Inspired by Windmill's job queue design.

---

## ADR-004: StoragePort with 5 Canonical Stores (not generic key-value)

**Date:** 2026-03-21
**Status:** Accepted
**Context:** Original StoragePort was "persist anything" with put/get/delete/query. Too broad — would re-invent a typed repository layer.
**Decision:** StoragePort exposes 5 sub-stores: EventStore, ObjectStore, SessionStore, JobStore, MetricStore. Each has a focused API.
**Consequence:** Clear boundaries. Each store optimized for its access pattern (append-only events, CRUD objects, queue semantics for jobs, time-series for metrics).

---

## ADR-005: Event.kind as String, not Enum

**Date:** 2026-03-21
**Status:** Accepted
**Context:** Original design had EventKind as a giant enum (AgentCreated, OpportunityScored, ContentPublished, etc.). This forced rusvel-core to know about every possible event type.
**Decision:** Event.kind is a `String`. Engines define their own event kind constants (e.g., `forge::AGENT_CREATED`, `harvest::OPPORTUNITY_SCORED`).
**Consequence:** rusvel-core stays minimal. New engines can define new event kinds without modifying core. Trade-off: less compile-time safety on event matching.

---

## ADR-006: HarvestPort and PublishPort are engine-internal, not core ports

**Date:** 2026-03-21
**Status:** Accepted
**Context:** Original design had HarvestPort and PublishPort as core ports. But scraping and publishing are domain-specific to Harvest and Content engines, not cross-cutting concerns.
**Decision:** Move to engine-internal traits. Harvest engine defines its own source adapters. Content engine defines its own platform adapters.
**Consequence:** Core ports reduced from 13 to 10. Cleaner separation. Engines own their domain-specific abstractions.

---

## ADR-007: metadata: serde_json::Value on all domain types

**Date:** 2026-03-21
**Status:** Accepted
**Context:** Perplexity warned about schema evolution — "you will iterate on Opportunity/Content/Agent schemas a lot."
**Decision:** All domain types in rusvel-core include `metadata: serde_json::Value`. Engines can add fields without breaking older code or requiring migrations.
**Consequence:** Base columns + metadata JSON pattern. Slight runtime overhead for metadata access but huge flexibility for evolution.

---

## ADR-008: Human-in-the-loop approval model from day one

**Date:** 2026-03-21
**Status:** Accepted
**Context:** For a solo founder, the most valuable pattern is "agent proposes, human approves." Without this, the system either does too much (scary) or too little (useless).
**Decision:** ApprovalStatus and ApprovalPolicy are core domain types. Content publishing and outreach require approval by default. Agents can be auto-approved below cost thresholds.
**Consequence:** JobStatus includes `AwaitingApproval`. UI must show approval queue. Agents know to pause and present results for review.

---

## ADR-009: Engines never call LlmPort directly

**Date:** 2026-03-21
**Status:** Accepted
**Context:** Perplexity flagged unclear boundaries between LlmPort (raw model access) and AgentPort (orchestration). If engines call both, prompting/retries/tool selection logic gets scattered.
**Decision:** LlmPort is raw (generate, stream, embed). AgentPort wraps LlmPort + ToolPort + MemoryPort. Engines only use AgentPort.
**Consequence:** All prompt construction, tool selection, retries, and memory injection happen in one place (rusvel-agent). Engines express intent, agents handle execution.

---

## ADR-010: Engines depend only on rusvel-core traits

**Date:** 2026-03-21
**Status:** Accepted (from v1, reinforced)
**Context:** Hexagonal architecture rule. Engines must not import concrete adapter types.
**Decision:** Engines depend on rusvel-core. They receive port implementations via constructor injection. rusvel-app (composition root) wires concrete adapters to engines.
**Consequence:** Any adapter can be swapped without touching engine code. Engines are testable with mock ports.

---
title: Architecture Decisions
description: Summary of all Architecture Decision Records (ADRs) with rationale.
---

## Overview

RUSVEL's architecture is guided by 11 Architecture Decision Records (ADRs). These capture the "why" behind key design choices. All were informed by cross-validation with multiple AI systems and real-world constraints of a solo founder product.

---

## ADR-001: Merge Ops + Connect into GoToMarket Engine

**Status:** Accepted

**Context:** The original design had 7 engines, including separate Ops (CRM, invoicing, SOPs) and Connect (outreach, sequences, follow-ups) engines. Review flagged that 7 is too many for a solo founder product.

**Decision:** Merge into a single GoToMarket engine. CRM, outreach, pipeline, and invoicing all live together.

**Consequence:** 5 engines instead of 7. GoToMarket is larger but cohesive around "find and close business."

---

## ADR-002: Fold Mission into Forge Engine

**Status:** Accepted

**Context:** Mission (goals, daily planning, reviews) was a separate engine. Mission is really "orchestrated agents" -- goal-setting, planning, and reviews are all agent tasks.

**Decision:** Mission becomes a sub-module of the Forge engine.

**Consequence:** `rusvel forge mission today` instead of `rusvel mission today`. Forge is the meta-engine that orchestrates everything.

---

## ADR-003: Single Central Job Queue

**Status:** Accepted

**Context:** The original design had AutomationPort (workflows), SchedulePort (cron/triggers), plus Forge (agent orchestration) and per-engine scheduling. Four overlapping workflow substrates.

**Decision:** Single JobPort with SQLite-backed job queue. All async work goes through one queue. A worker pool routes jobs to engines by JobKind.

**Consequence:** One place to see all pending, running, and completed work. Simpler debugging. Inspired by Windmill's job queue design.

---

## ADR-004: StoragePort with 5 Canonical Stores

**Status:** Accepted

**Context:** The original StoragePort was "persist anything" with generic put/get/delete/query. Too broad -- would re-invent a typed repository layer.

**Decision:** StoragePort exposes 5 sub-stores: EventStore (append-only), ObjectStore (CRUD), SessionStore (hierarchy), JobStore (queue semantics), MetricStore (time-series).

**Consequence:** Clear boundaries. Each store optimized for its access pattern.

---

## ADR-005: Event.kind as String, Not Enum

**Status:** Accepted

**Context:** The original design had EventKind as a giant enum (AgentCreated, OpportunityScored, ContentPublished, etc.). This forced rusvel-core to know about every possible event type.

**Decision:** `Event.kind` is a `String`. Engines define their own event kind constants (e.g., `forge::AGENT_CREATED`).

**Consequence:** rusvel-core stays minimal. New engines can define new event kinds without modifying core. Trade-off: less compile-time safety on event matching.

---

## ADR-006: HarvestPort and PublishPort Are Engine-Internal

**Status:** Accepted

**Context:** The original design had HarvestPort and PublishPort as core ports. But scraping and publishing are domain-specific, not cross-cutting concerns.

**Decision:** Move to engine-internal traits. Harvest engine defines its own source adapters. Content engine defines its own platform adapters.

**Consequence:** Core ports reduced from 13 to 10. Cleaner separation.

---

## ADR-007: Metadata Field on All Domain Types

**Status:** Accepted

**Context:** Schema evolution is inevitable -- "you will iterate on Opportunity/Content/Agent schemas a lot." Migrations for every field change slow development.

**Decision:** All domain types include `metadata: serde_json::Value`. Engines can add fields without breaking older code or requiring migrations.

**Consequence:** Base columns plus metadata JSON pattern. Slight runtime overhead for metadata access but huge flexibility for evolution.

---

## ADR-008: Human-in-the-Loop Approval Model

**Status:** Accepted

**Context:** For a solo founder, the most valuable pattern is "agent proposes, human approves." Without this, the system either does too much (scary) or too little (useless).

**Decision:** `ApprovalStatus` and `ApprovalPolicy` are core domain types. Content publishing and outreach require approval by default. Agents can be auto-approved below cost thresholds.

**Consequence:** JobStatus includes `AwaitingApproval`. The UI shows an approval queue. Agents pause and present results for review.

---

## ADR-009: Engines Never Call LlmPort Directly

**Status:** Accepted

**Context:** Unclear boundaries between LlmPort (raw model access) and AgentPort (orchestration). If engines call both, prompting, retries, and tool selection logic gets scattered.

**Decision:** LlmPort is raw (generate, stream, embed). AgentPort wraps LlmPort + ToolPort + MemoryPort. Engines only use AgentPort.

**Consequence:** All prompt construction, tool selection, retries, and memory injection happen in one place (rusvel-agent). Engines express intent; agents handle execution.

---

## ADR-010: Engines Depend Only on rusvel-core Traits

**Status:** Accepted

**Context:** Hexagonal architecture rule. Engines must not import concrete adapter types.

**Decision:** Engines depend on rusvel-core. They receive port implementations via constructor injection. rusvel-app (composition root) wires concrete adapters to engines.

**Consequence:** Any adapter can be swapped without touching engine code. Engines are testable with mock ports.

---

## ADR-011: Config Hierarchy and Department Registry

**Status:** Proposed

**Context:** Four separate config systems (TomlConfig, ChatConfig, DepartmentConfig, UserProfile) with no hierarchy. Adding a department touched 7 files. The Settings page was non-functional.

**Decision:** Three-layer config cascade (Global > Department > Session) plus a declarative Department Registry defined in TOML. Dynamic API routes replace 72 hardcoded routes. Dynamic frontend routes replace 12 identical pages.

**Consequence:** Zero-code department addition. Single settings page. 90% boilerplate reduction. Config inheritance eliminates duplication.

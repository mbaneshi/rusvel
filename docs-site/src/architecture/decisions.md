
## Overview

RUSVEL's architecture is guided by 14 Architecture Decision Records (ADRs). These capture the "why" behind key design choices. All were informed by cross-validation with multiple AI systems and real-world constraints of a solo founder product.


## ADR-001: Merge Ops + Connect into GoToMarket Engine

**Status:** Accepted

**Context:** Original design had 7 engines including separate Ops (CRM, invoicing, SOPs) and Connect (outreach, sequences, follow-ups) engines. Perplexity review flagged that 7 is too many for a solo founder product.

**Decision:** Merge into single GoToMarket engine. CRM, outreach, pipeline, invoicing all live together.

**Consequence:** 5 engines instead of 7. GoToMarket engine is larger but cohesive around "find and close business."


## ADR-002: Fold Mission into Forge Engine

**Status:** Accepted

**Context:** Mission (goals, daily planning, reviews) was a separate engine. Mission is really "orchestrated agents" -- goal-setting, planning, and reviews are all agent tasks.

**Decision:** Mission becomes a sub-module of the Forge engine.

**Consequence:** `rusvel forge mission today` instead of `rusvel mission today`. Forge is the meta-engine that orchestrates everything.


## ADR-003: Single Central Job Queue

**Status:** Accepted

**Context:** Original design had AutomationPort (workflows), SchedulePort (cron/triggers), plus Forge (agent orchestration) and per-engine scheduling. This created 4 overlapping workflow substrates.

**Decision:** Single JobPort with SQLite-backed job queue. All async work goes through one queue. Worker pool routes jobs to engines by JobKind.

**Consequence:** One place to see all pending/running/completed work. Simpler debugging.


## ADR-004: StoragePort with 5 Canonical Stores

**Status:** Accepted

**Context:** The original StoragePort was "persist anything" with generic put/get/delete/query. Too broad -- would re-invent a typed repository layer.

**Decision:** StoragePort exposes 5 sub-stores: EventStore (append-only), ObjectStore (CRUD), SessionStore (hierarchy), JobStore (queue semantics), MetricStore (time-series).

**Consequence:** Clear boundaries. Each store optimized for its access pattern.


## ADR-005: Event.kind as String, not Enum

**Status:** Accepted

**Context:** The original EventKind was a giant enum forcing rusvel-core to know every event type.

**Decision:** Event.kind is a `String`. Engines define their own event kind constants (e.g., `forge::AGENT_CREATED`).

**Consequence:** rusvel-core stays minimal. New engines define new event kinds without modifying core.


## ADR-006: HarvestPort and PublishPort Are Engine-Internal

**Status:** Accepted

**Context:** The original design had HarvestPort and PublishPort as core ports. But scraping and publishing are domain-specific, not cross-cutting concerns.

**Decision:** Move to engine-internal traits. Harvest engine defines its own source adapters. Content engine defines its own platform adapters.

**Consequence:** Core ports reduced from 13 to 10 (later grew to 19 with new ports). Cleaner separation.


## ADR-007: metadata: serde_json::Value on All Domain Types

**Status:** Accepted

**Context:** Schema evolution concern -- domain types will iterate frequently.

**Decision:** All domain types include `metadata: serde_json::Value` for extensibility without migrations.

**Consequence:** Base columns + metadata JSON pattern. Huge flexibility for evolution.


## ADR-008: Human-in-the-Loop Approval Model

**Status:** Accepted

**Context:** For a solo founder, the most valuable pattern is "agent proposes, human approves." Without this, the system either does too much (scary) or too little (useless).

**Decision:** `ApprovalStatus` and `ApprovalPolicy` are core domain types. Content publishing and outreach require approval by default. Agents can be auto-approved below cost thresholds.

**Consequence:** JobStatus includes `AwaitingApproval`. The UI shows an approval queue. Agents pause and present results for review.


## ADR-009: Engines Never Call LlmPort Directly

**Status:** Accepted

**Context:** Unclear boundaries between LlmPort (raw model access) and AgentPort (orchestration). If engines call both, prompting/retries/tool selection logic gets scattered.

**Decision:** LlmPort is raw (generate, stream, embed). AgentPort wraps LlmPort + ToolPort + MemoryPort. Engines only use AgentPort.

**Consequence:** All prompt construction, tool selection, retries, and memory injection happen in one place (rusvel-agent).


## ADR-010: Engines Depend Only on rusvel-core Traits

**Status:** Accepted

**Context:** Hexagonal architecture rule. Engines must not import concrete adapter types.

**Decision:** Engines depend on rusvel-core. They receive port implementations via constructor injection. rusvel-app (composition root) wires concrete adapters to engines.

**Consequence:** Any adapter can be swapped without touching engine code. Engines are testable with mock ports.


## ADR-011: Department Registry — Dynamic Departments Replace Hardcoded Routing

**Status:** Accepted (superseded by ADR-014 for registration mechanism)

**Context:** Scaling from 5 engines to 12 departments with hardcoded routes meant 72+ routes and touching 7 files to add one department.

**Decision:** `DepartmentRegistry` holds `DepartmentDef` structs. 6 parameterized `/api/dept/{dept}/*` routes replace all per-department routes. Frontend uses a single dynamic `[dept]` route.

**Consequence:** Adding a department = adding a `DepartmentDef` entry. Zero route changes. Three-layer config cascade (Global -> Department -> Session).


## ADR-012: shadcn/ui Design Tokens — oklch Color System

**Status:** Accepted

**Context:** Frontend needed a consistent design system. Tailwind 4 supports oklch natively.

**Decision:** Adopt shadcn/ui design tokens with oklch color values. Each department gets a color token from the registry.

**Consequence:** Consistent theming across all 12 department UIs. Dark mode is a CSS variable swap.


## ADR-013: Capability Engine — AI-Driven Entity Creation

**Status:** Accepted

**Context:** Users need to extend the system with new agents, skills, rules, MCP servers, hooks, and workflows. Manually configuring each is tedious.

**Decision:** `POST /api/capability/build` accepts natural language, uses Claude with WebSearch/WebFetch to discover resources, generates a bundle of entities. Also available via `!build <description>` in chat.

**Consequence:** One-shot system extension. "Install a GitHub code review agent" creates agent, skills, hooks, and MCP config in one call.


## ADR-014: DepartmentApp Pattern — Departments as Self-Contained Crates

**Status:** Accepted

**Context:** The `EngineKind` enum forced `rusvel-core` to know about every department, violating the Open/Closed Principle. Adding a department touched 5+ files.

**Decision:** Introduce `DepartmentApp` trait and `DepartmentManifest` struct. Each department lives in its own `dept-*` crate. `EngineKind` enum is removed entirely; departments use string IDs. 13 `dept-*` crates created.

**Consequence:** Adding a department = adding a `dept-*` crate. Zero changes to `rusvel-core`. Each department declares its own routes, tools, capabilities, and system prompt via `DepartmentManifest`. Supersedes the registration mechanism of ADR-011.

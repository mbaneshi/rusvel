
## Overview

RUSVEL's architecture is guided by 11 Architecture Decision Records (ADRs). These capture the "why" behind key design choices. All were informed by cross-validation with multiple AI systems and real-world constraints of a solo founder product.


## ADR-002: Fold Mission into Forge Engine

**Status:** Accepted

**Context:** Mission (goals, daily planning, reviews) was a separate engine. Mission is really "orchestrated agents" -- goal-setting, planning, and reviews are all agent tasks.

**Decision:** Mission becomes a sub-module of the Forge engine.

**Consequence:** `rusvel forge mission today` instead of `rusvel mission today`. Forge is the meta-engine that orchestrates everything.


## ADR-004: StoragePort with 5 Canonical Stores

**Status:** Accepted

**Context:** The original StoragePort was "persist anything" with generic put/get/delete/query. Too broad -- would re-invent a typed repository layer.

**Decision:** StoragePort exposes 5 sub-stores: EventStore (append-only), ObjectStore (CRUD), SessionStore (hierarchy), JobStore (queue semantics), MetricStore (time-series).

**Consequence:** Clear boundaries. Each store optimized for its access pattern.


## ADR-006: HarvestPort and PublishPort Are Engine-Internal

**Status:** Accepted

**Context:** The original design had HarvestPort and PublishPort as core ports. But scraping and publishing are domain-specific, not cross-cutting concerns.

**Decision:** Move to engine-internal traits. Harvest engine defines its own source adapters. Content engine defines its own platform adapters.

**Consequence:** Core ports reduced from 13 to 10. Cleaner separation.


## ADR-008: Human-in-the-Loop Approval Model

**Status:** Accepted

**Context:** For a solo founder, the most valuable pattern is "agent proposes, human approves." Without this, the system either does too much (scary) or too little (useless).

**Decision:** `ApprovalStatus` and `ApprovalPolicy` are core domain types. Content publishing and outreach require approval by default. Agents can be auto-approved below cost thresholds.

**Consequence:** JobStatus includes `AwaitingApproval`. The UI shows an approval queue. Agents pause and present results for review.


## ADR-010: Engines Depend Only on rusvel-core Traits

**Status:** Accepted

**Context:** Hexagonal architecture rule. Engines must not import concrete adapter types.

**Decision:** Engines depend on rusvel-core. They receive port implementations via constructor injection. rusvel-app (composition root) wires concrete adapters to engines.

**Consequence:** Any adapter can be swapped without touching engine code. Engines are testable with mock ports.


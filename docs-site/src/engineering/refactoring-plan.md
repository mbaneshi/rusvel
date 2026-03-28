# Refactoring Plan

Structural refactoring for design pattern alignment, extensibility, and scalability. The full checklist lives in [`docs/plans/comprehensive-refactoring-plan.md`](https://github.com/mbaneshi/rusvel/blob/main/docs/plans/comprehensive-refactoring-plan.md).

## Strategic themes

1. **Type safety over stringly-typed** -- replace magic strings (event kinds, job kinds, tool names) with compile-time-safe constants and validated registries
2. **GoF patterns as Rust idioms** -- Builder (typestate), Strategy (generics), Observer (typed channels), Decorator (tower layers), Command (typed enums), Visitor (AST traversal)
3. **Scalability substrate** -- saga orchestration, association graphs, self-improvement loops, Tower Service composability

**What is NOT changing:** hexagonal port/adapter boundary, DepartmentApp trait, composition root, single-binary constraint, SQLite WAL, or any ADR decisions.

## Current patterns (verified)

| Pattern | Where | Quality |
|---------|-------|---------|
| Strategy | Port traits (`LlmPort`, `AgentPort`, etc.) | Excellent |
| Abstract Factory | Composition root (`rusvel-app`) | Excellent |
| Facade | `AgentRuntime` over LLM+Tool+Memory | Excellent |
| Adapter | Every `rusvel-*` crate | Excellent |
| Observer | `AgentEvent` over mpsc, `EventPort` broadcast | Good |
| Command | `JobKind` dispatch, `ToolHandler` closures | Good |
| Builder | `DepartmentManifest::new()` | Partial |
| Proxy | `ScopedToolRegistry` | Good |

## Gaps to address

| Pattern | Gap | Sprint |
|---------|-----|--------|
| Typestate Builder | No compile-time field enforcement | 2 |
| Decorator (Tower) | No composable middleware on ports | 3 |
| Visitor | code-engine has no visitor trait | 4 |
| Typed Observer | Event subscriptions use string matching | 4 |
| State (Typestate) | Job/flow status transitions unchecked | 4 |
| Composite | Flow nodes are flat (no nesting) | 8 |
| Mediator | No cross-department agent delegation | 9 |
| Saga | No compensation logic in multi-step flows | 6 |

## Sprint overview

| Sprint | Theme | Key Deliverables |
|--------|-------|-----------------|
| 1 | Type safety | Event kind constants, tool collision detection, job registry |
| 2 | Builders & factories | Typestate DepartmentManifest, PlatformFactory trait, TestFactory |
| 3 | Structural | AppState decomposition (25 fields -> 5 sub-states), Tower layers on LlmPort |
| 4 | Behavioral | TypedEvent system, JobCommand trait, SymbolVisitor, DelegateAgentTool |
| 5 | DDD | AggregateRoot trait, ObjectStore associations, value object enforcement |
| 6 | Events | Saga compensation, lightweight projections, event replay |
| 7 | Plugins | Manifest validation, port requirement checks, capability tokens |
| 8 | Workflows | Checkpoint/resume, SubFlow/Parallel/Loop nodes, per-node retry |
| 9 | Agents | Hierarchical delegation, supervisor pattern, blackboard cross-engine |
| 10 | Scalability | Magic number extraction, graceful shutdown, data-driven CLI |
| 11 | Frontend | Typed API client, command/query stores, error boundaries |
| 12 | Self-improvement | Failure reflection loop, skill accumulation, experience replay |

## Structural issues being fixed

| Issue | Severity | Sprint |
|-------|----------|--------|
| Monolithic AppState (~25 fields) | High | 3 |
| Tool name collisions silently overwrite | High | 1 |
| Event kinds are string literals | Medium | 1 |
| Job results stored in untyped metadata | Medium | 4 |
| No association graph between domain objects | Medium | 5 |
| Magic numbers scattered in code | Low | 10 |
| Job worker has no graceful shutdown | Medium | 10 |
| CLI department variants repeated 9x | Low | 10 |

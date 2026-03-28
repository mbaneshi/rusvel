# Design Patterns

How RUSVEL applies Gang of Four patterns, SOLID principles, and modern system design -- all adapted to Rust's type system. See also the [Architecture overview](../architecture/overview.md) and [ADRs](../architecture/decisions.md).

## GoF patterns in Rust

Rust has no inheritance. Each GoF pattern either translates via traits/generics, collapses into an enum, or becomes unnecessary because ownership solves the underlying problem.

### Creational

| Pattern | Rust Translation | RUSVEL Usage |
|---------|-----------------|-------------|
| **Builder** | Method chaining, typestate for required fields | `DepartmentManifest`, `LlmRequest`, `AgentConfig` |
| **Factory Method** | Trait with `create()` method | `DepartmentApp::register()` |
| **Abstract Factory** | Composition root returns trait objects | `rusvel-app/src/main.rs` wires all adapters |
| **Singleton** | `Arc<T>` passed at construction (no statics) | All ports are `Arc<dyn Port>` |
| **Prototype** | `#[derive(Clone)]` | `DepartmentManifest` |

### Structural

| Pattern | Rust Translation | RUSVEL Usage |
|---------|-----------------|-------------|
| **Adapter** | Struct wraps foreign type, implements local trait | Every `rusvel-*` adapter crate |
| **Decorator** | `tower::Layer` wraps `Service` | `CostTrackingLlm`, planned: TimeoutLayer, RetryLayer |
| **Facade** | Struct composes multiple ports | `AgentRuntime` over LLM + Tool + Memory (ADR-009) |
| **Proxy** | Same trait, forwarding + filtering | `ScopedToolRegistry` per-department filtering |
| **Composite** | Enum with recursive `Box<Self>` variants | Flow DAG nodes (planned: SubFlowNode) |
| **Bridge** | Trait separates abstraction from implementation | All port traits in `rusvel-core` |
| **Flyweight** | `Arc<T>` for shared immutable state | `Arc<dyn LlmPort>` shared across engines |

### Behavioral

| Pattern | Rust Translation | RUSVEL Usage |
|---------|-----------------|-------------|
| **Strategy** | Generic `<S: Strategy>` (static) or `dyn Strategy` (dynamic) | Port trait dispatch; `ModelTier` routing |
| **Observer** | `tokio::sync::broadcast` / `mpsc` channels | `AgentEvent` stream, `EventPort` broadcast |
| **Command** | Enum variants + match dispatch | `JobKind` in worker, `ToolHandler` closures |
| **Chain of Responsibility** | `tower::Layer` middleware chain | Axum middleware stack |
| **State** | Enum (runtime) or typestate (compile-time) | `JobStatus`, `FlowExecutionStatus` |
| **Template Method** | Trait with default method calling abstract methods | `LlmPort::stream()` defaults to `generate()` |
| **Visitor** | Trait with `visit_*` methods, separate traversal | Planned: `SymbolVisitor` in code-engine |
| **Mediator** | Central event bus or registry | `EventPort` as cross-department mediator |
| **Iterator** | `Iterator` trait (first-class in Rust) | Standard throughout |

### When NOT to use GoF in Rust

- **Singleton:** Never use `static Mutex<T>`. Pass `Arc<T>` via constructors.
- **Trait objects for closed sets:** If all variants are known, use an enum -- faster (no vtable), exhaustively matchable.
- **Observer with callback vectors:** Use async channels instead -- they compose with tokio's scheduler.
- **Visitor:** Unnecessary when `match` on exhaustive enums covers all cases.

## SOLID in Rust

### Single Responsibility

Each crate has one reason to change. The `<2000 lines per crate` rule enforces this. Crates that grow too large are split.

### Open/Closed

Traits are Rust's extension mechanism. Adding a new `LlmPort` implementation requires zero changes to `rusvel-core`. The `DepartmentApp` pattern (ADR-014) is the purest OCP expression -- adding a department means adding a `dept-*` crate, not modifying existing code.

### Liskov Substitution

Any `Arc<dyn LlmPort>` must behave according to the `LlmPort` contract. Rust enforces signature compatibility; integration tests validate semantic contracts.

### Interface Segregation

`StoragePort` splits into 5 focused sub-stores (ADR-004):

```
StoragePort
  |- EventStore    (append-only events)
  |- ObjectStore   (CRUD domain objects)
  |- SessionStore  (session lifecycle)
  |- JobStore      (job queue semantics)
  '- MetricStore   (time-series metrics)
```

Engines depend only on the sub-stores they use.

### Dependency Inversion

The hexagonal architecture IS dependency inversion. Engines depend on abstractions (`rusvel-core` traits), not on concrete adapters. `rusvel-app` is the only place that depends on concrete types. Verified via `just check-boundaries`.

## Rust-specific patterns

### Typestate

Encode operation ordering in the type system. The `build()` method is only available when required fields are set:

```rust
pub struct ManifestBuilder<Id, Prompt> {
    id: Id,
    prompt: Prompt,
    // ...
}

// Can't call build() until both id and system_prompt are set
impl ManifestBuilder<HasId, HasPrompt> {
    pub fn build(self) -> DepartmentManifest { /* ... */ }
}
```

Used for: `DepartmentManifest`, `AgentConfig`, `LlmRequest`. See [Typestate Pattern -- Cliffle](https://cliffle.com/blog/rust-typestate/).

### Newtype

Prevents mixing IDs at compile time:

```rust
pub struct SessionId(Uuid);  // Cannot be confused with...
pub struct AgentId(Uuid);    // ...this at the call site
```

Used throughout `rusvel-core` for all domain identifiers.

### Tower Service

The foundational middleware abstraction in the Rust async ecosystem. `poll_ready()` implements backpressure -- the caller must check readiness before sending:

```rust
let service = ServiceBuilder::new()
    .layer(TimeoutLayer::new(Duration::from_secs(30)))
    .layer(RateLimitLayer::new(10, Duration::from_secs(1)))
    .service(provider);
```

Used in: `rusvel-api` (Axum middleware). Planned for: `LlmPort`, `AgentPort`, `ToolPort`.

## Modern system design patterns

### DepartmentApp as Microkernel

RUSVEL's `DepartmentApp` pattern is a microkernel at the application level:

- **Kernel:** `rusvel-app` + `rusvel-core` port traits + composition root
- **Plugins:** Each `dept-*` crate is a self-contained service
- **Message passing:** `EventPort` (domain events), `JobPort` (async work)
- **Capability declaration:** `DepartmentManifest` declares what each plugin provides

### Event-Driven Architecture

Three EDA patterns in use:

1. **Event Notification** -- engines emit events, listeners react (hook dispatch)
2. **Central Job Queue** -- orchestration-style async work (ADR-003)
3. **Event Persistence** -- append-only `EventStore` for audit trail (ADR-005)

Planned additions:
- **Saga pattern** -- compensation edges in flow-engine DAGs
- **Projections** -- denormalized read views rebuilt from event streams

### Agent Orchestration Hierarchy

```
ForgeEngine (supervisor)
  |-- DelegateAgentTool --> ContentEngine agent
  |-- DelegateAgentTool --> HarvestEngine agent
  '-- DelegateAgentTool --> CodeEngine agent
```

Delegation is depth-limited (max 3 levels) with per-department tool scoping.

### Self-Improving System

The feedback loop substrate:

1. Agent runs --> emit success/failure events
2. Failure reflection agent analyzes transcript
3. Generates skill/rule suggestions --> stored as drafts
4. Human approves --> skill/rule promoted to active
5. Successful tool sequences mined as reusable templates

## References

- [GoF in Rust -- fadeevab/design-patterns-rust](https://github.com/fadeevab/design-patterns-rust)
- [Rust Design Patterns -- rust-unofficial](https://rust-unofficial.github.io/patterns/)
- [Hexagonal Architecture in Rust -- howtocodeit.com](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust)
- [Tower Service -- Tokio blog](https://tokio.rs/blog/2021-05-14-inventing-the-service-trait)
- [Typestate -- Cliffle](https://cliffle.com/blog/rust-typestate/)
- [CQRS in Rust -- doc.rust-cqrs.org](https://doc.rust-cqrs.org/)
- [Refactoring.guru -- Rust patterns](https://refactoring.guru/design-patterns/rust)

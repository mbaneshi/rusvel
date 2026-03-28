# Concept Hierarchy

The domain model spine -- the ontology that every crate, API route, and UI component maps onto. All refactoring decisions trace back to which level of this hierarchy they affect.

## Five levels

```
Level 0: Platform (RUSVEL)
Level 1: Session (workspace scope)
Level 2: Department (bounded context)
Level 3: Domain Entity (per-department)
Level 4: Cross-cutting Primitive (shared infrastructure)
```

## Level 0: Platform

```
RUSVEL
|- Identity: single binary, single human, infinite leverage
|- Constraint: SQLite WAL, tokio async, Rust + SvelteKit
'- Invariant: every action traceable to a Session
```

The platform is the composition root (`rusvel-app`), the port traits (`rusvel-core`), and the adapter crates. This level changes only when fundamental infrastructure changes.

## Level 1: Session

```
Session
|- Owns: Runs, Threads, Goals, Events, Jobs, Config overrides
|- Scopes: all state is session-namespaced
|- Lifecycle: create -> active -> archived
'- Future: Session becomes Workspace when multi-user
```

The session is the unit of isolation. All queries, all tool calls, all agent runs are scoped to a session. This is RUSVEL's equivalent of tenancy.

## Level 2: Department

```
Department (DepartmentApp)
|- Identity: string ID, manifest, icon, color
|- Owns: Engine, Tools, Skills, Rules, Hooks, Agents, Workflows
|- Communicates via: Events (pub/sub), Jobs (async work), ObjectStore
|- Never: imports another department's crate
'- Types:
    |- Wired (6): forge, code, harvest, content, gtm, flow
    |- Skeleton (7): finance, product, growth, distro, legal, support, infra
    '- Shell (1): messaging
```

Each department is a [bounded context](https://martinfowler.com/bliki/BoundedContext.html). Departments communicate only through cross-cutting primitives (Level 4), never by importing each other's types.

## Level 3: Domain entities

Each department owns its entities. Entities from different departments reference each other by ID, never by owned struct.

| Department | Entities |
|-----------|----------|
| **Forge** | Goal, Task, Plan, Review, Persona, AgentProfile |
| **Code** | Repository, SymbolGraph, Symbol, Metric, SearchResult |
| **Harvest** | Opportunity, Proposal, Pipeline, Source, Score |
| **Content** | ContentItem, CalendarEntry, PlatformAdapter, PublishResult |
| **GTM** | Contact, Deal, OutreachSequence, Step, Invoice |
| **Flow** | Workflow, Node, Edge, Execution, Checkpoint, NodeResult |
| **Finance** | Ledger, Transaction, TaxEstimate, RunwayForecast |
| **Product** | Roadmap, Feature, PricingTier, FeedbackItem |
| **Growth** | Funnel, Cohort, KPI, Experiment |
| **Distro** | Listing, SEOProfile, AffiliateProgram, Partnership |
| **Legal** | Contract, ComplianceCheck, IPRecord, LicenseAgreement |
| **Support** | Ticket, KBArticle, NPSSurvey, AutoTriageRule |
| **Infra** | Deployment, Monitor, Incident, Pipeline |

## Level 4: Cross-cutting primitives

Shared infrastructure that all departments use through port traits:

| Primitive | Purpose | Mutability |
|-----------|---------|-----------|
| **Event** | Immutable record of what happened | Append-only |
| **Job** | Async work item with state machine | Mutable (state transitions) |
| **Tool** | Registered capability with handler | Immutable after registration |
| **Skill** | Stored prompt template | Mutable (CRUD) |
| **Rule** | System prompt fragment | Mutable (CRUD) |
| **Hook** | Event-triggered automation | Mutable (CRUD) |
| **Agent** | LLM + tools + persona + memory | Stateful per run |
| **Approval** | Human gate on job or publishing | Mutable (approve/reject) |

## Level 5: Infrastructure ports

The port traits that connect departments to the outside world:

```
LlmPort         -> raw model access (generate, stream, embed)
AgentPort        -> orchestrated LLM (tool loop, memory, verification)
StoragePort      -> 5 sub-stores (events, objects, sessions, jobs, metrics)
EventPort        -> pub/sub + persistence
JobPort          -> central async work queue
ToolPort         -> tool registry + execution + permission
MemoryPort       -> session-scoped context + FTS5 search
ConfigPort       -> layered settings (global -> dept -> session)
AuthPort         -> opaque credential handles
EmbeddingPort    -> text -> dense vectors
VectorStorePort  -> similarity search
ChannelPort      -> outbound notifications
TerminalPort     -> PTY multiplexer
BrowserPort      -> Chrome DevTools Protocol
DeployPort       -> deployment operations
SessionPort      -> session lifecycle
```

## Design rules

1. **Concept ownership:** every concept belongs to exactly one level
2. **Cross-level references:** use IDs (newtypes), never owned structs
3. **A Department never owns a Session.** They reference each other via `SessionId` and department string ID
4. **Shared kernel:** types in `rusvel-core/src/domain.rs` are the small set all departments agree on
5. **Anti-corruption:** if a department needs another department's data, it goes through ObjectStore or Events -- never through direct type imports

## Visual map

```
                    ┌─────────────────────┐
                    │    RUSVEL (L0)       │
                    │  Single Binary       │
                    └──────────┬──────────┘
                               │
                    ┌──────────┴──────────┐
                    │    Session (L1)      │
                    │  Scope + Isolation   │
                    └──────────┬──────────┘
                               │
          ┌────────┬───────────┼───────────┬────────┐
          │        │           │           │        │
       ┌──┴──┐ ┌──┴──┐    ┌──┴──┐    ┌──┴──┐ ┌──┴──┐
       │Forge│ │Code │    │GTM  │    │Flow │ │ ... │
       │(L2) │ │(L2) │    │(L2) │    │(L2) │ │(L2) │
       └──┬──┘ └──┬──┘    └──┬──┘    └──┬──┘ └─────┘
          │       │           │          │
       Goals   Symbols    Contacts    Nodes    (L3)
       Tasks   Metrics    Deals      Edges
       Plans   Repos      Invoices   Checkpoints
          │       │           │          │
          └───────┴─────┬─────┴──────────┘
                        │
              ┌─────────┴─────────┐
              │  Cross-cutting    │
              │  Primitives (L4)  │
              │  Events, Jobs,    │
              │  Tools, Skills    │
              └─────────┬─────────┘
                        │
              ┌─────────┴─────────┐
              │   Port Traits     │
              │   (L5 infra)      │
              │   LLM, Storage,   │
              │   Agent, Event    │
              └───────────────────┘
```

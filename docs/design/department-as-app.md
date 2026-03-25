# ADR-014: Department-as-App Architecture

> Date: 2026-03-25
> Status: Proposed
> Supersedes: ADR-011 (Department Registry)
> Extends: ADR-005 (Event.kind as String), ADR-006 (Engine-internal traits), ADR-010 (Engines depend only on core)

---

## Context

RUSVEL has three rates of change:

| Layer | Cadence | Examples |
|-------|---------|---------|
| **Kernel** (rusvel-core) | Quarterly | Port traits, domain types, event bus, storage |
| **Platform** (agent runtime, LLM, tools) | Weekly | New providers, MCP spec updates, streaming formats, agent patterns |
| **Departments** (business domains) | Independently | Each department ships when its domain logic is ready |

Today, all three layers compile and deploy as one unit. A new LLM streaming format rebuilds 34 crates. Adding a field to ContentEngine touches the same binary as ForgeEngine. The `EngineKind` enum in `rusvel-core` forces core to know about every department — violating the Open/Closed Principle that ADR-005 already solved for events.

The AI/agent ecosystem (MCP, AG-UI, tool protocols, model routing, RAG strategies) changes weekly. RUSVEL must absorb these changes without cascading refactors into department code. Simultaneously, departments must be independently developable, testable, and — eventually — independently deployable.

### Prior Art

| System | Module Unit | Contract | Discovery | What Survives 10+ Years |
|--------|------------|----------|-----------|------------------------|
| **Django** | App (Python package) | `AppConfig` class + convention files (`models.py`, `urls.py`, `admin.py`) | `INSTALLED_APPS` list | The file naming convention + `AppConfig.ready()` hook |
| **Linux Kernel** | `.ko` module | `module_init()` / `module_exit()` + symbol exports | `depmod` offline scan → `modprobe` loads with deps | The init/exit contract + subsystem registration (`register_netdev()`) |
| **VSCode** | Extension folder | `package.json#contributes` (declarative) + `activate()` / `deactivate()` (imperative) | Directory scan; activation events for lazy loading | The manifest format + lazy activation model |
| **Django REST Framework** | Viewset + Router | `serializers.py`, `viewsets.py`, `permissions.py` | `DefaultRouter` auto-generates URLs from viewsets | The viewset contract (list/create/retrieve/update/destroy) |

### Cross-Cutting Principles Extracted

1. **Declarative manifest, imperative implementation.** Every successful system separates "what this module offers" (static, parseable without execution) from "how it works" (code). The manifest is the long-term stability surface.

2. **Two-function lifecycle.** `init()` + `teardown()`. Everything else is registration with subsystems during init.

3. **Lazy activation.** Load manifests eagerly, load code lazily. VSCode has 30,000+ extensions but only a handful activate per session.

4. **Subsystem registration, not central omniscience.** The kernel doesn't have a registry of "all module types." Each subsystem (networking, block devices, filesystems) defines its own registration API. Modules call `register_netdev()` during init.

5. **Explicit, ordered registration list.** Django's `INSTALLED_APPS` and VSCode's `extensionDependencies` show that explicit ordering beats auto-discovery for first-party modules.

6. **Convention-based file layout.** Django's `models.py`, `urls.py`, `admin.py` — the file names ARE the API.

7. **Per-module migrations.** Schema changes are scoped and independent.

8. **Cross-module communication via shared bus.** Not direct imports. Django uses signals, VSCode uses commands-as-events, Linux uses exported symbols.

---

## Decision

Adopt the **Department-as-App** pattern: each department is a self-contained crate that implements a stable contract (`DepartmentApp` trait), declares its contributions via a static manifest (`DepartmentManifest`), and registers with host subsystems during a lifecycle hook.

### Core Changes

1. **`EngineKind` enum is removed from `rusvel-core`.** Departments identify themselves by string ID (like `Event.kind` per ADR-005).
2. **`DepartmentRegistry` is generated from manifests at boot**, not defined in core.
3. **Each department crate owns its routes, CLI commands, tools, personas, skills, and UI declarations.**
4. **The composition root (`rusvel-app`) holds an ordered list of departments** (like `INSTALLED_APPS`).

---

## Architecture

### The Three Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                      DEPARTMENTS (Apps)                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │  Forge   │ │  Code    │ │ Content  │ │ Harvest  │  ...      │
│  │  dept    │ │  dept    │ │  dept    │ │  dept    │           │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘          │
│       │             │            │             │                │
│       └─────────────┴────────────┴─────────────┘                │
│                          │                                      │
│              DepartmentApp trait (the contract)                 │
├─────────────────────────────────────────────────────────────────┤
│                       PLATFORM                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ Agent    │ │ LLM      │ │ Tool     │ │ Memory   │          │
│  │ Runtime  │ │ Providers│ │ Registry │ │ + RAG    │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ Event    │ │ Job      │ │ MCP      │ │ Flow     │          │
│  │ Bus      │ │ Queue    │ │ Client   │ │ Engine   │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
├─────────────────────────────────────────────────────────────────┤
│                        KERNEL                                   │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  rusvel-core: 12 port traits + domain types + Content    │  │
│  │  DepartmentApp trait + DepartmentManifest struct          │  │
│  │  RegistrationContext + subsystem registrars               │  │
│  └──────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                       SURFACES                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ API      │ │ CLI      │ │ TUI      │ │ MCP      │          │
│  │ (Axum)   │ │ (Clap)   │ │(Ratatui) │ │ Server   │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Frontend Shell (SvelteKit) — renders UI contributions   │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Rate of Change Isolation

```
Kernel    ──▶  Changes yearly    (port traits, domain types)
Platform  ──▶  Changes weekly    (LLM providers, agent patterns, tool protocols)
Depts     ──▶  Change independently (each dept ships when ready)
Surfaces  ──▶  Change with platform (route assembly, CLI structure)
```

Departments depend **down** (on kernel). They never depend **sideways** (on each other) or **up** (on surfaces). Communication between departments flows through the event bus (ADR-005).

---

## The Contract

### `DepartmentApp` Trait

This is RUSVEL's most important API surface. It must be versioned and backward-compatible.

```rust
// In rusvel-core::department (new module)

use async_trait::async_trait;
use crate::error::Result;

/// The contract every department must implement.
///
/// Inspired by:
/// - Django's `AppConfig` (manifest + `ready()` hook)
/// - Linux kernel modules (`module_init` / `module_exit`)
/// - VSCode extensions (`package.json#contributes` + `activate()` / `deactivate()`)
///
/// ## Lifecycle
///
/// 1. Host reads `manifest()` — no side effects, fast, used for dependency
///    resolution and capability indexing before any department code runs.
/// 2. Host calls `register()` with a `RegistrationContext` — department
///    registers its routes, commands, tools, event handlers, and job handlers
///    with the host's subsystem registrars.
/// 3. Host calls `shutdown()` on graceful exit.
///
/// ## Rules
///
/// - Departments MUST NOT import other department crates (ADR-010 extended).
/// - Departments MUST NOT import adapter crates — only `rusvel-core` (ADR-010).
/// - Cross-department communication goes through `EventPort` (ADR-005).
/// - Departments use `AgentPort`, never `LlmPort` directly (ADR-009).
#[async_trait]
pub trait DepartmentApp: Send + Sync {
    /// Static manifest declaring what this department contributes.
    ///
    /// Called before `register()`. Must be side-effect-free.
    /// The host uses this for dependency resolution, capability
    /// indexing, and UI rendering — without executing department code.
    fn manifest(&self) -> DepartmentManifest;

    /// Register with host subsystems.
    ///
    /// Called once at boot, after dependency order is resolved.
    /// The department receives ports it needs and registrars for
    /// each surface it participates in.
    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()>;

    /// Graceful shutdown. Default is no-op.
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
```

### `DepartmentManifest` — The Stability Surface

Like VSCode's `package.json#contributes`, this is **declarative** and **parseable without executing department code**. This struct is the long-term contract — it outlasts any implementation detail.

```rust
// In rusvel-core::department

use semver::{Version, VersionReq};

/// Everything the host needs to know about a department
/// without executing any of its code.
///
/// This struct is RUSVEL's most important API surface.
/// Version it carefully. Never remove fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentManifest {
    // ── Identity ─────────────────────────────────────────────────
    /// Unique department identifier (e.g. "content", "forge").
    /// Replaces `EngineKind` enum variants.
    pub id: String,

    /// Human-readable name (e.g. "Content Department").
    pub name: String,

    /// Department's own semantic version.
    pub version: Version,

    /// Which core versions this department is compatible with.
    /// e.g. ">=0.5, <2.0" — the host refuses to load incompatible depts.
    pub core_compat: VersionReq,

    // ── Visual identity ──────────────────────────────────────────
    /// Icon character for CLI and UI (e.g. "#", "*", "§").
    pub icon: String,

    /// Color token for theming (e.g. "emerald", "purple").
    pub color: String,

    // ── Contributions (what this department adds to the host) ────
    /// API routes this department registers.
    pub routes: Vec<RouteContribution>,

    /// CLI subcommands this department adds.
    pub commands: Vec<CommandContribution>,

    /// Agent tools this department provides.
    pub tools: Vec<ToolContribution>,

    /// Agent personas this department defines.
    pub personas: Vec<PersonaContribution>,

    /// Reusable prompt templates (skills).
    pub skills: Vec<SkillContribution>,

    /// System prompt rules injected during agent runs.
    pub rules: Vec<RuleContribution>,

    /// Job types this department processes.
    pub jobs: Vec<JobContribution>,

    /// Frontend UI declarations.
    pub ui: UiContribution,

    // ── Events ───────────────────────────────────────────────────
    /// Event kinds this department emits (e.g. "content.drafted").
    pub events_produced: Vec<String>,

    /// Event kinds this department subscribes to (e.g. "code.analyzed").
    pub events_consumed: Vec<String>,

    // ── Dependencies ─────────────────────────────────────────────
    /// Which core ports this department requires.
    /// The host validates all required ports are available before
    /// calling `register()`.
    pub requires_ports: Vec<PortRequirement>,

    /// Other departments this one depends on (soft dependencies).
    /// The host ensures these are registered first.
    pub depends_on: Vec<String>,

    // ── Configuration ────────────────────────────────────────────
    /// JSON Schema for department-specific settings.
    /// Used by the settings UI to render config forms.
    pub config_schema: serde_json::Value,

    /// Default configuration values.
    pub default_config: serde_json::Value,

    // ── Chat personality ─────────────────────────────────────────
    /// System prompt for this department's chat personality.
    pub system_prompt: String,

    /// Capabilities this department advertises.
    pub capabilities: Vec<String>,

    /// Quick action buttons shown in the department panel.
    pub quick_actions: Vec<QuickAction>,
}
```

### Contribution Types

```rust
/// An API route this department contributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteContribution {
    /// HTTP method (GET, POST, PUT, PATCH, DELETE).
    pub method: String,
    /// Path pattern (e.g. "/api/dept/content/draft").
    pub path: String,
    /// Human-readable description for API docs.
    pub description: String,
}

/// A CLI subcommand this department contributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandContribution {
    /// Command name (e.g. "analyze" under "code" department).
    pub name: String,
    /// Description for --help.
    pub description: String,
    /// Argument definitions.
    pub args: Vec<ArgDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgDef {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

/// An agent tool this department provides.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContribution {
    /// Tool name (namespaced: "content.draft", "code.analyze").
    pub name: String,
    /// Description for tool selection.
    pub description: String,
    /// JSON Schema for tool parameters.
    pub parameters_schema: serde_json::Value,
}

/// An agent persona this department defines.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaContribution {
    /// Persona name (e.g. "rust-engineer", "content-strategist").
    pub name: String,
    /// Role description.
    pub role: String,
    /// Default model preference.
    pub default_model: String,
    /// Tools this persona has access to.
    pub allowed_tools: Vec<String>,
}

/// A reusable prompt template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillContribution {
    pub name: String,
    pub description: String,
    /// Template with `{{input}}` interpolation.
    pub prompt_template: String,
}

/// A system prompt rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleContribution {
    pub name: String,
    pub content: String,
    pub enabled: bool,
}

/// A job type this department processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobContribution {
    /// Job kind string (replaces `JobKind` enum variants).
    pub kind: String,
    /// Human-readable description.
    pub description: String,
    /// Whether this job type requires human approval by default.
    pub requires_approval: bool,
}

/// Frontend UI declarations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiContribution {
    /// Tab IDs shown in the department panel.
    /// Standard tabs: "actions", "engine", "agents", "skills",
    /// "rules", "workflows", "mcp", "hooks", "dirs", "events".
    pub tabs: Vec<String>,

    /// Dashboard cards for the home page.
    pub dashboard_cards: Vec<DashboardCard>,

    /// Whether this department has a custom settings section.
    pub has_settings: bool,

    /// Custom Svelte component paths (for departments needing
    /// UI beyond what the shell provides, e.g. Flow builder).
    pub custom_components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardCard {
    pub title: String,
    pub description: String,
    /// Card size: "small", "medium", "large".
    pub size: String,
}

/// Which port a department requires.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRequirement {
    /// Port name (e.g. "AgentPort", "EventPort", "StoragePort").
    pub port: String,
    /// Whether the department can function without this port.
    pub optional: bool,
}
```

### `RegistrationContext` — Subsystem Registration

Like Linux's `register_netdev()`, `register_chrdev()` — each surface defines its own registration API. The department plugs into whichever surfaces it participates in.

```rust
// In rusvel-core::department

use std::sync::Arc;
use crate::ports::*;

/// Passed to `DepartmentApp::register()`.
///
/// Contains the ports the department can use and the registrars
/// for each host subsystem. The department calls registrar methods
/// to contribute routes, commands, tools, event handlers, and jobs.
pub struct RegistrationContext {
    // ── Ports (what the department consumes) ──────────────────────
    pub agent: Arc<dyn AgentPort>,
    pub events: Arc<dyn EventPort>,
    pub storage: Arc<dyn StoragePort>,
    pub jobs: Arc<dyn JobPort>,
    pub memory: Arc<dyn MemoryPort>,
    pub sessions: Arc<dyn SessionPort>,
    pub config: Arc<dyn ConfigPort>,
    pub auth: Arc<dyn AuthPort>,
    pub embedding: Option<Arc<dyn EmbeddingPort>>,
    pub vector_store: Option<Arc<dyn VectorStorePort>>,

    // ── Registrars (what the department contributes to) ───────────
    pub routes: RouteRegistrar,
    pub commands: CommandRegistrar,
    pub tools: ToolRegistrar,
    pub event_handlers: EventRegistrar,
    pub job_handlers: JobRegistrar,
}
```

#### Registrar Interfaces

```rust
/// Collects API route handlers from departments.
pub struct RouteRegistrar {
    routes: Vec<(String, axum::Router)>,  // (prefix, router)
}

impl RouteRegistrar {
    /// Mount a sub-router under a path prefix.
    /// e.g. `registrar.mount("/api/dept/content", content_router)`
    pub fn mount(&mut self, prefix: &str, router: axum::Router) { ... }
}

/// Collects CLI subcommands from departments.
pub struct CommandRegistrar {
    commands: Vec<(String, Box<dyn CliHandler>)>,
}

impl CommandRegistrar {
    /// Register a CLI subcommand handler.
    /// e.g. `registrar.add("analyze", handler)` under "code" dept
    pub fn add(&mut self, name: &str, handler: Box<dyn CliHandler>) { ... }
}

/// Collects agent tools from departments.
pub struct ToolRegistrar {
    tools: Vec<ToolDefinition>,
}

impl ToolRegistrar {
    /// Register an agent tool.
    pub fn add(&mut self, tool: ToolDefinition) { ... }
}

/// Collects event subscriptions from departments.
pub struct EventRegistrar {
    handlers: Vec<(String, Box<dyn EventHandler>)>,
}

impl EventRegistrar {
    /// Subscribe to events matching a kind pattern.
    /// e.g. `registrar.on("code.analyzed", handler)` in content dept
    pub fn on(&mut self, event_kind: &str, handler: Box<dyn EventHandler>) { ... }
}

/// Collects job type handlers from departments.
pub struct JobRegistrar {
    handlers: HashMap<String, Box<dyn JobHandler>>,
}

impl JobRegistrar {
    /// Register a handler for a job kind.
    /// e.g. `registrar.handle("content.publish", handler)`
    pub fn handle(&mut self, kind: &str, handler: Box<dyn JobHandler>) { ... }
}
```

---

## Department Crate Layout (Convention Over Configuration)

Like Django's `models.py` + `urls.py` + `admin.py`, every department crate follows a predictable structure. A developer can navigate any department instantly.

```
crates/dept-content/                    # or crates/content-dept/
├── Cargo.toml                          # Depends ONLY on rusvel-core
├── src/
│   ├── lib.rs                          # impl DepartmentApp for ContentDepartment
│   ├── manifest.rs                     # fn manifest() → DepartmentManifest (static)
│   ├── engine.rs                       # ContentEngine — domain logic
│   ├── handlers.rs                     # Axum route handlers (POST /draft, etc.)
│   ├── commands.rs                     # CLI subcommand handlers
│   ├── tools.rs                        # Agent tools (content.draft, content.adapt)
│   ├── personas.rs                     # Agent personas (content-strategist, etc.)
│   ├── jobs.rs                         # Job handlers (content.publish, etc.)
│   ├── events.rs                       # Event constants + event listeners
│   └── platform/                       # Engine-internal adapters (ADR-006)
│       ├── mod.rs
│       ├── devto.rs                    # DevTo publishing adapter
│       ├── twitter.rs                  # Twitter/X adapter
│       └── linkedin.rs                # LinkedIn adapter
├── migrations/
│   ├── 001_content_tables.sql          # Department-scoped schema
│   └── 002_add_analytics.sql
├── ui/                                 # Frontend contribution
│   ├── manifest.json                   # Tabs, dashboard cards, settings
│   ├── EngineTab.svelte                # Custom engine-specific tab (optional)
│   └── components/                     # Department-specific Svelte components
│       └── ContentCalendar.svelte
├── seeds/                              # Default data for first-run
│   ├── agents.json
│   ├── skills.json
│   └── rules.json
└── tests/
    ├── integration.rs                  # Uses mock ports from rusvel-core
    └── engine_test.rs
```

### What Goes Where

| File | Purpose | Analogous To |
|------|---------|-------------|
| `lib.rs` | `DepartmentApp` impl, lifecycle | Django `apps.py` |
| `manifest.rs` | Static manifest, no side effects | VSCode `package.json#contributes` |
| `engine.rs` | Domain logic (the "business code") | Django `views.py` + `models.py` |
| `handlers.rs` | HTTP route handlers | Django `urls.py` + `views.py` |
| `commands.rs` | CLI subcommand handlers | Django `management/commands/` |
| `tools.rs` | Agent tool definitions + handlers | VSCode command contributions |
| `personas.rs` | Agent persona configs | — |
| `jobs.rs` | Background job handlers | Django Celery tasks |
| `events.rs` | Event kind constants + listeners | Django signals |
| `platform/` | Engine-internal adapters (ADR-006) | Django third-party backends |
| `migrations/` | Department-scoped SQL migrations | Django `migrations/` |
| `ui/manifest.json` | Frontend declarations | VSCode `contributes.views` |
| `ui/*.svelte` | Custom components (optional) | VSCode webview panels |
| `seeds/` | Default agents, skills, rules | Django fixtures |
| `tests/` | Integration tests with mock ports | Django `tests.py` |

---

## Boot Sequence

Like Django's startup: read all app configs → resolve dependencies → call `ready()`.

```rust
// In rusvel-app/src/main.rs

/// Explicit, ordered list of installed departments.
/// Like Django's INSTALLED_APPS — order matters for dependencies.
fn installed_departments() -> Vec<Box<dyn DepartmentApp>> {
    vec![
        // Core departments (no deps on other depts)
        Box::new(forge_dept::ForgeDepartment::new()),
        Box::new(code_dept::CodeDepartment::new()),

        // Departments with cross-dept event subscriptions
        Box::new(content_dept::ContentDepartment::new()),
        Box::new(harvest_dept::HarvestDepartment::new()),
        Box::new(flow_dept::FlowDepartment::new()),

        // Departments with minimal logic (progressive enhancement)
        Box::new(gtm_dept::GtmDepartment::new()),
        Box::new(finance_dept::FinanceDepartment::new()),
        Box::new(product_dept::ProductDepartment::new()),
        Box::new(growth_dept::GrowthDepartment::new()),
        Box::new(distro_dept::DistroDepartment::new()),
        Box::new(legal_dept::LegalDepartment::new()),
        Box::new(support_dept::SupportDepartment::new()),
        Box::new(infra_dept::InfraDepartment::new()),
    ]
}

/// Boot the application from installed departments.
async fn boot(depts: Vec<Box<dyn DepartmentApp>>) -> Result<App> {
    // ── Phase 1: Read all manifests (fast, no side effects) ──────
    let manifests: Vec<_> = depts.iter().map(|d| d.manifest()).collect();

    // Validate: no duplicate IDs
    validate_unique_ids(&manifests)?;

    // Validate: core compatibility
    let core_version = env!("CARGO_PKG_VERSION").parse::<Version>()?;
    for m in &manifests {
        if !m.core_compat.matches(&core_version) {
            anyhow::bail!(
                "Department '{}' v{} requires core {}, but core is v{}",
                m.id, m.version, m.core_compat, core_version
            );
        }
    }

    // Resolve dependency order (topological sort)
    let order = resolve_dependency_order(&manifests)?;

    // ── Phase 2: Create kernel ports (shared infrastructure) ─────
    let ports = create_ports(&config).await?;

    // ── Phase 3: Register departments in dependency order ────────
    let mut ctx = RegistrationContext::new(ports);
    for idx in &order {
        depts[*idx].register(&mut ctx).await?;
    }

    // ── Phase 4: Build surfaces from accumulated registrations ───
    let api_router = ctx.routes.build();       // Axum Router
    let cli_commands = ctx.commands.build();    // Clap Command tree
    let tool_registry = ctx.tools.build();      // ToolRegistry
    let event_dispatch = ctx.event_handlers.build(); // Event dispatcher
    let job_dispatch = ctx.job_handlers.build();     // Job dispatcher

    // ── Phase 5: Generate department registry for frontend ───────
    let registry = DepartmentRegistry::from_manifests(&manifests);

    Ok(App {
        router: api_router,
        cli: cli_commands,
        tools: tool_registry,
        events: event_dispatch,
        jobs: job_dispatch,
        registry,
        departments: depts,
    })
}
```

### Phase Diagram

```
  installed_departments()
          │
          ▼
  ┌─── Phase 1: Read Manifests ───┐
  │  • No code execution          │
  │  • Validate IDs, versions     │
  │  • Resolve dependency order   │
  └──────────────┬────────────────┘
                 │
                 ▼
  ┌─── Phase 2: Create Ports ─────┐
  │  • Database, LLM, EventBus    │
  │  • Memory, Jobs, Config       │
  │  • These are the "kernel"     │
  └──────────────┬────────────────┘
                 │
                 ▼
  ┌─── Phase 3: Register Depts ───┐
  │  for each dept (in dep order):│
  │    dept.register(&mut ctx)    │
  │    • Mount routes             │
  │    • Add CLI commands         │
  │    • Register tools           │
  │    • Subscribe to events      │
  │    • Register job handlers    │
  └──────────────┬────────────────┘
                 │
                 ▼
  ┌─── Phase 4: Build Surfaces ───┐
  │  • Assemble Axum router       │
  │  • Assemble Clap command tree │
  │  • Finalize tool registry     │
  │  • Wire event dispatch        │
  │  • Wire job dispatch          │
  └──────────────┬────────────────┘
                 │
                 ▼
  ┌─── Phase 5: Generate Registry─┐
  │  • Build DepartmentRegistry   │
  │    from manifests (for UI)    │
  │  • Serve via GET /api/depts   │
  └───────────────────────────────┘
```

---

## Example: Content Department

### `lib.rs` — DepartmentApp Implementation

```rust
use async_trait::async_trait;
use rusvel_core::department::*;
use rusvel_core::error::Result;

mod engine;
mod events;
mod handlers;
mod commands;
mod jobs;
mod manifest;
mod personas;
mod platform;
mod tools;

pub struct ContentDepartment {
    engine: OnceCell<ContentEngine>,
}

impl ContentDepartment {
    pub fn new() -> Self {
        Self { engine: OnceCell::new() }
    }
}

#[async_trait]
impl DepartmentApp for ContentDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::content_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        // Create engine with injected ports
        let engine = engine::ContentEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        );

        // Register platform adapters (engine-internal, ADR-006)
        engine.register_platform(Arc::new(platform::DevToAdapter::new(ctx.config.clone())));
        engine.register_platform(Arc::new(platform::TwitterAdapter::new(ctx.config.clone())));
        engine.register_platform(Arc::new(platform::LinkedInAdapter::new(ctx.config.clone())));

        let engine = Arc::new(engine);
        self.engine.set(engine.clone()).ok();

        // ── Routes ───────────────────────────────────────────
        ctx.routes.mount("/api/dept/content", handlers::router(engine.clone()));

        // ── CLI commands ─────────────────────────────────────
        ctx.commands.add("draft", Box::new(commands::DraftHandler::new(engine.clone())));

        // ── Agent tools ──────────────────────────────────────
        ctx.tools.add(tools::content_draft_tool(engine.clone()));
        ctx.tools.add(tools::content_adapt_tool(engine.clone()));

        // ── Event subscriptions ──────────────────────────────
        // Content department listens for code analysis results
        // to auto-generate technical blog posts (code-to-content pipeline)
        ctx.event_handlers.on(
            "code.analyzed",
            Box::new(events::OnCodeAnalyzed::new(engine.clone())),
        );

        // ── Job handlers ─────────────────────────────────────
        ctx.job_handlers.handle(
            "content.publish",
            Box::new(jobs::PublishHandler::new(engine.clone())),
        );

        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        if let Some(engine) = self.engine.get() {
            engine.shutdown().await?;
        }
        Ok(())
    }
}
```

### `manifest.rs` — Static Declaration

```rust
use rusvel_core::department::*;
use semver::{Version, VersionReq};

pub fn content_manifest() -> DepartmentManifest {
    DepartmentManifest {
        id: "content".into(),
        name: "Content Department".into(),
        version: Version::new(0, 1, 0),
        core_compat: VersionReq::parse(">=0.1, <1.0").unwrap(),

        icon: "*".into(),
        color: "purple".into(),

        routes: vec![
            RouteContribution {
                method: "POST".into(),
                path: "/api/dept/content/draft".into(),
                description: "Draft content from a topic".into(),
            },
            RouteContribution {
                method: "POST".into(),
                path: "/api/dept/content/from-code".into(),
                description: "Generate content from code analysis".into(),
            },
            RouteContribution {
                method: "PATCH".into(),
                path: "/api/dept/content/{id}/approve".into(),
                description: "Approve content for publishing".into(),
            },
            RouteContribution {
                method: "POST".into(),
                path: "/api/dept/content/publish".into(),
                description: "Publish approved content".into(),
            },
            RouteContribution {
                method: "GET".into(),
                path: "/api/dept/content/list".into(),
                description: "List all content items".into(),
            },
        ],

        commands: vec![
            CommandContribution {
                name: "draft".into(),
                description: "Draft content from a topic".into(),
                args: vec![ArgDef {
                    name: "topic".into(),
                    description: "Topic to write about".into(),
                    required: true,
                    default: None,
                }],
            },
        ],

        tools: vec![
            ToolContribution {
                name: "content.draft".into(),
                description: "Draft a blog post or article on a given topic".into(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "topic": { "type": "string" },
                        "audience": { "type": "string" },
                        "format": { "type": "string", "enum": ["blog", "thread", "article"] }
                    },
                    "required": ["topic"]
                }),
            },
            ToolContribution {
                name: "content.adapt".into(),
                description: "Adapt content for a specific platform".into(),
                parameters_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "content_id": { "type": "string" },
                        "platform": { "type": "string", "enum": ["twitter", "linkedin", "devto"] }
                    },
                    "required": ["content_id", "platform"]
                }),
            },
        ],

        personas: vec![
            PersonaContribution {
                name: "content-strategist".into(),
                role: "Content strategist and writer".into(),
                default_model: "sonnet".into(),
                allowed_tools: vec!["content.draft".into(), "content.adapt".into(), "web_search".into()],
            },
        ],

        skills: vec![
            SkillContribution {
                name: "Blog Draft".into(),
                description: "Draft a blog post from topic and key points".into(),
                prompt_template: "Write a blog post about: {{topic}}\n\nKey points:\n{{points}}\n\nAudience: {{audience}}".into(),
            },
        ],

        rules: vec![
            RuleContribution {
                name: "Human Approval Gate".into(),
                content: "All content must be approved before publishing. Never auto-publish.".into(),
                enabled: true,
            },
        ],

        jobs: vec![
            JobContribution {
                kind: "content.publish".into(),
                description: "Publish approved content to target platforms".into(),
                requires_approval: true,
            },
        ],

        ui: UiContribution {
            tabs: vec![
                "actions".into(), "engine".into(), "agents".into(),
                "skills".into(), "rules".into(), "events".into(),
            ],
            dashboard_cards: vec![
                DashboardCard {
                    title: "Content Pipeline".into(),
                    description: "Drafts, scheduled, and published content".into(),
                    size: "medium".into(),
                },
            ],
            has_settings: true,
            custom_components: vec![],
        },

        events_produced: vec![
            "content.drafted".into(),
            "content.adapted".into(),
            "content.scheduled".into(),
            "content.published".into(),
            "content.metrics_updated".into(),
        ],
        events_consumed: vec![
            "code.analyzed".into(),  // Code-to-content pipeline
        ],

        requires_ports: vec![
            PortRequirement { port: "AgentPort".into(), optional: false },
            PortRequirement { port: "EventPort".into(), optional: false },
            PortRequirement { port: "StoragePort".into(), optional: false },
            PortRequirement { port: "JobPort".into(), optional: false },
            PortRequirement { port: "ConfigPort".into(), optional: true },
        ],

        depends_on: vec![], // Content has no hard dept dependencies

        config_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "devto_api_key": { "type": "string", "description": "DEV.to API key" },
                "twitter_api_key": { "type": "string", "description": "Twitter/X API key" },
                "linkedin_api_key": { "type": "string", "description": "LinkedIn API key" },
                "default_format": { "type": "string", "enum": ["markdown", "html"] }
            }
        }),
        default_config: serde_json::json!({
            "default_format": "markdown"
        }),

        system_prompt: "You are the Content department of RUSVEL.\n\nFocus: content creation, platform adaptation, publishing strategy.\nDraft in Markdown. Adapt for LinkedIn, Twitter/X, DEV.to, Substack.".into(),
        capabilities: vec!["content_creation".into()],
        quick_actions: vec![
            QuickAction { label: "Draft blog post".into(), prompt: "Draft a blog post. Ask me for the topic, audience, and key points.".into() },
            QuickAction { label: "Adapt for Twitter".into(), prompt: "Adapt the latest content piece into a Twitter/X thread.".into() },
            QuickAction { label: "Content calendar".into(), prompt: "Show the content calendar for this week with scheduled and draft posts.".into() },
        ],
    }
}
```

---

## Cross-Department Communication

Departments MUST NOT import each other. All cross-department communication flows through the event bus, following ADR-005.

### Pattern: Code-to-Content Pipeline

```
Code Department                    Event Bus                    Content Department
     │                                │                              │
     │  code.analyzed                 │                              │
     │──────────────────────────────▶│                              │
     │                                │  code.analyzed               │
     │                                │─────────────────────────────▶│
     │                                │                              │
     │                                │                    OnCodeAnalyzed:
     │                                │                    extract insights,
     │                                │                    draft blog post
     │                                │                              │
     │                                │  content.drafted             │
     │                                │◀─────────────────────────────│
     │                                │                              │
```

### Pattern: Harvest-to-GTM Pipeline

```
Harvest Department                 Event Bus                    GTM Department
     │                                │                              │
     │  harvest.opportunity_won       │                              │
     │──────────────────────────────▶│                              │
     │                                │  harvest.opportunity_won     │
     │                                │─────────────────────────────▶│
     │                                │                              │
     │                                │                    OnOpportunityWon:
     │                                │                    create CRM contact,
     │                                │                    start onboarding
     │                                │                    sequence
     │                                │                              │
```

---

## Frontend Integration

### Shell Renders, Departments Declare

The SvelteKit frontend is the **shell**. It reads department manifests from `GET /api/departments` and renders generic UI. Departments declare what they contribute; the shell decides how to render it.

```
Frontend Shell                          Department Manifest
┌─────────────────────┐                ┌─────────────────────────┐
│ Sidebar             │◀───────────────│ id, name, icon, color   │
│ ├── Forge      =    │                └─────────────────────────┘
│ ├── Code       #    │                ┌─────────────────────────┐
│ ├── Content    *    │◀───────────────│ tabs: [actions, engine, │
│ ├── Harvest    $    │                │   agents, skills, ...]  │
│ └── ...             │                └─────────────────────────┘
│                     │                ┌─────────────────────────┐
│ Department Panel    │◀───────────────│ quick_actions: [...]    │
│ ┌─ Actions ──────┐  │                │ system_prompt: "..."    │
│ │ [Draft post]   │  │                └─────────────────────────┘
│ │ [Adapt]        │  │                ┌─────────────────────────┐
│ │ [Calendar]     │  │◀───────────────│ dashboard_cards: [...]  │
│ └────────────────┘  │                └─────────────────────────┘
│                     │
│ Dashboard           │
│ ┌── Content ──────┐ │
│ │ 3 drafts        │ │
│ │ 1 scheduled     │ │
│ └─────────────────┘ │
└─────────────────────┘
```

### Custom Components (Escape Hatch)

Most departments work fine with the shell's generic tabs (actions, agents, skills, rules, events). For departments needing specialized UI (Flow builder's DAG canvas, RusvelBase's SQL runner), the manifest declares `custom_components`:

```json
{
  "ui": {
    "tabs": ["actions", "engine", "agents"],
    "custom_components": ["FlowCanvas.svelte"],
    "dashboard_cards": [
      { "title": "Active Flows", "size": "large" }
    ]
  }
}
```

The shell lazy-loads custom components only when the department is active (VSCode's lazy activation pattern).

### Frontend File Ownership

```
frontend/src/
├── routes/
│   ├── +page.svelte              # Shell: home dashboard
│   ├── chat/+page.svelte         # Shell: god agent chat
│   ├── dept/[id]/+page.svelte    # Shell: renders dept from manifest
│   ├── database/                 # Shell: RusvelBase (not a dept)
│   ├── flows/                    # Shell: Flow builder (not a dept)
│   ├── knowledge/                # Shell: RAG (not a dept)
│   └── settings/                 # Shell: settings
├── lib/
│   ├── components/
│   │   ├── shell/                # Shell components (sidebar, layout)
│   │   ├── department/           # Generic dept rendering
│   │   │   ├── DepartmentPanel.svelte
│   │   │   ├── ActionsTab.svelte
│   │   │   ├── AgentsTab.svelte
│   │   │   └── ...
│   │   ├── chat/                 # Chat components
│   │   └── ui/                   # Design system primitives
│   └── stores/
│       └── departments.ts        # Fetches manifests, provides to components
```

Departments that need custom Svelte components place them in `crates/dept-*/ui/`. During build, these are copied/linked into the frontend's component tree. The shell imports them dynamically:

```svelte
{#if dept.ui.custom_components.includes('EngineTab.svelte')}
  {#await import(`$lib/dept/${dept.id}/EngineTab.svelte`) then mod}
    <mod.default {dept} />
  {/await}
{:else}
  <DefaultEngineTab {dept} />
{/if}
```

---

## The `EngineKind` Removal

### Before (ADR-011)

```rust
// In rusvel-core — must know about ALL departments
pub enum EngineKind {
    Forge, Code, Harvest, Content, GoToMarket,
    Finance, Product, Growth, Distribution,
    Legal, Support, Infra,
}
```

Adding a department = modifying `rusvel-core`. Violates Open/Closed.

### After (ADR-014)

```rust
// In rusvel-core — stable, never changes for new departments
pub type DepartmentId = String;

// Departments identify themselves in their manifest:
// manifest.id = "content"
//
// Events reference departments by string:
// Event { kind: "content.drafted", ... }
//
// Jobs reference departments by string:
// Job { kind: "content.publish", ... }
```

This is the same pattern as ADR-005 (`Event.kind` as String) applied to departments.

### Migration: `EngineKind` → `DepartmentId`

The `EngineKind` enum can be kept temporarily as a convenience alias that maps to string IDs, then deprecated:

```rust
// Transitional — remove in v0.3
impl EngineKind {
    pub fn as_department_id(&self) -> &'static str {
        match self {
            Self::Forge => "forge",
            Self::Code => "code",
            // ...
        }
    }
}
```

---

## Progressive Enhancement: Stub → Real

With Department-as-App, a "stub" department is not tech debt — it's a **real department with minimal domain logic**. The manifest is complete (identity, UI, personas, quick actions). The engine just delegates to the generic agent chat.

```rust
// A minimal department — fully valid, fully functional
#[async_trait]
impl DepartmentApp for FinanceDepartment {
    fn manifest(&self) -> DepartmentManifest {
        DepartmentManifest {
            id: "finance".into(),
            name: "Finance Department".into(),
            version: Version::new(0, 0, 1),  // Pre-release
            // ... full manifest with UI, personas, quick_actions
            routes: vec![],      // No engine-specific routes yet
            tools: vec![],       // No custom tools yet
            jobs: vec![],        // No background jobs yet
            events_produced: vec![],
            events_consumed: vec![],
            // ...
        }
    }

    async fn register(&self, _ctx: &mut RegistrationContext) -> Result<()> {
        // Nothing to register — chat works through generic agent
        Ok(())
    }
}
```

When finance domain logic is ready, the department adds routes, tools, and jobs in its `register()` — without touching core, other departments, or the frontend shell.

---

## Future: Runtime Loading

Today's architecture is **compile-time** (departments are crate dependencies). The contract is designed to support **runtime loading** in the future without changing the `DepartmentApp` trait:

### Phase 1 (Current): Compile-Time Crates

```toml
# rusvel-app/Cargo.toml
[dependencies]
dept-forge = { path = "../dept-forge" }
dept-code = { path = "../dept-code" }
dept-content = { path = "../dept-content" }
```

Single binary. Maximum performance. Full type safety.

### Phase 2 (Future): Feature Flags

```toml
[features]
default = ["forge", "code", "content", "harvest", "flow"]
forge = ["dep:dept-forge"]
code = ["dep:dept-code"]
content = ["dep:dept-content"]
# Opt-in departments
finance = ["dep:dept-finance"]
legal = ["dep:dept-legal"]
```

Smaller binaries for users who don't need all departments.

### Phase 3 (Future): WASM Plugins

```rust
// Load a department from a .wasm file at runtime
let wasm_dept = WasmDepartment::load("departments/custom-analytics.wasm")?;
installed.push(Box::new(wasm_dept));
```

Perfect sandboxing. Language-agnostic (departments written in any WASM-targeting language). The `DepartmentApp` trait maps naturally to WASM component model exports.

### Phase 4 (Future): Process-Level (MCP-like)

```rust
// Load a department running as a separate process
let remote_dept = RemoteDepartment::connect("http://localhost:9001")?;
installed.push(Box::new(remote_dept));
```

Complete isolation. Crash-safe. Independent deployment. The manifest is exchanged via JSON-RPC at startup.

---

## What This Architecture Enables

| Capability | How |
|-----------|-----|
| Add department without touching core | New crate + add to `installed_departments()` |
| Remove department cleanly | Remove from `installed_departments()` — no orphan code |
| Independent department testing | Each dept has its own `tests/` with mock ports |
| Department-scoped migrations | Each dept owns its `migrations/` directory |
| Department-scoped seeds | Each dept owns its `seeds/` directory |
| Cross-dept communication | Event bus (not imports) |
| Custom dept UI | `custom_components` in manifest, lazy-loaded by shell |
| Platform changes don't cascade | Depts call `AgentPort`, not `LlmPort` |
| Version compatibility checking | `core_compat` field in manifest |
| Observability per department | Events, jobs, and metrics are department-tagged |
| Third-party departments (future) | Same `DepartmentApp` trait via WASM or IPC |
| Feature-flagged builds (future) | Cargo features per department |

---

## Consequences

### Positive

1. **Separation of concerns by rate of change.** Kernel evolves quarterly, platform weekly, departments independently.
2. **Open/Closed Principle.** New departments don't modify core.
3. **Each department is testable in isolation.** Mock ports, no adapter dependencies.
4. **Stub departments are legitimate.** Manifest + chat works. Domain logic is progressive enhancement.
5. **The manifest becomes documentation.** Routes, tools, events, personas — all declared in one place.
6. **Future-proofed for runtime loading.** The trait contract works for compile-time, WASM, and process-level loading.
7. **Ecosystem absorption.** When LLM/agent patterns change, only the platform layer changes. Departments are insulated.

### Negative

1. **More ceremony per department.** Each department needs `manifest.rs`, `lib.rs`, `handlers.rs` even for minimal functionality.
2. **Manifest can drift from implementation.** The declared routes/tools must match what `register()` actually does. Mitigated by build-time validation.
3. **String-based department IDs lose some compile-time safety.** Mitigated by convention and tests (same trade-off as ADR-005 for events).
4. **Custom frontend components need a build pipeline.** Copying Svelte files from crate dirs into the frontend build. Mitigated by shell generics covering 80% of cases.

### Trade-offs

| Aspect | Before (ADR-011) | After (ADR-014) |
|--------|-------------------|-----------------|
| Adding a department | Modify `EngineKind` enum + registry + routes + CLI | New crate + `installed_departments()` |
| Compile-time safety | `EngineKind` variants checked | String IDs checked by convention/tests |
| Department coupling | Departments can import each other (not enforced) | Enforced: deps only on `rusvel-core` |
| Route ownership | Mixed: generic handlers + scattered engine routes | Each department owns its routes |
| Frontend rendering | Shell knows all tabs/actions | Shell reads from manifest |
| Testability | Engines testable, but wiring is in `main.rs` | Each dept testable independently |
| Ceremony | Low (just implement Engine trait) | Medium (manifest + registration + convention files) |

---

## Migration Path

This is not a rewrite. It's a refactor of the composition layer, done incrementally.

### Step 1: Define the contract (rusvel-core)

Add `department` module with `DepartmentApp`, `DepartmentManifest`, `RegistrationContext`, and all contribution types. This is **additive** — existing code continues to work.

### Step 2: Convert one real department (content-engine → dept-content)

Prove the contract works with the most complex engine that has routes, tools, jobs, event subscriptions, and platform adapters. The existing `content-engine` crate becomes `dept-content` with the new layout.

### Step 3: Convert forge-engine → dept-forge

Second department, validates the contract generalizes. Forge is the most port-heavy engine (7 ports injected).

### Step 4: Wire boot sequence in main.rs

Replace hardcoded engine instantiation with `installed_departments()` + `boot()`. Keep the old code behind a feature flag during transition.

### Step 5: Convert remaining departments

Code, Harvest, Flow, then the 8 stub departments. Each conversion is independent and compiles/tests in isolation.

### Step 6: Remove EngineKind enum

Once all departments use string IDs, deprecate and remove `EngineKind` from `rusvel-core`.

### Step 7: Align frontend

Update `GET /api/departments` to serve manifests. Update shell to read UI contributions from manifests instead of the old `DepartmentDef` struct.

Each step compiles and passes tests. No big bang.

---

## Relationship to Other ADRs

| ADR | Relationship |
|-----|-------------|
| ADR-005 (Event.kind as String) | Extended: same principle applied to department IDs |
| ADR-006 (Engine-internal traits) | Reinforced: platform adapters stay inside dept crate |
| ADR-007 (metadata: Value) | Unchanged: all domain types keep metadata |
| ADR-008 (Human approval) | Unchanged: approval is a JobPort concern, not dept-specific |
| ADR-009 (Engines use AgentPort) | Reinforced: departments call AgentPort, never LlmPort |
| ADR-010 (Engines depend on core) | Extended: departments depend ONLY on rusvel-core |
| ADR-011 (Department Registry) | **Superseded**: registry generated from manifests, not hardcoded |
| ADR-013 (Capability Engine) | Unchanged: capability engine generates entities per dept |

---

## Relationship to Proposals

This architecture clarifies where each proposal lives:

| Proposal | Layer | Why |
|----------|-------|-----|
| P1: Deferred tool loading | Platform (ToolPort) | Departments register tools; the platform decides loading strategy |
| P2: Hybrid RAG | Platform (MemoryPort, VectorStorePort) | Departments call `memory.search()`; platform handles RAG strategy |
| P3: Batch API | Platform (LlmPort) | Departments call `agent.run()`; platform routes to batch when async |
| P4: Approval UI | Shell (frontend) | Shell renders approval queue from `JobPort`; not dept-specific |
| P5: Self-correction | Platform (AgentPort) | Departments call `agent.run()`; platform adds critique loop |
| P6: Streamable HTTP MCP | Platform (MCP server) | Departments register tools; transport is platform concern |
| P7: AG-UI Protocol | Surface (API) | Departments emit events; surface formats as AG-UI |
| P8: Durable execution | Platform (FlowEngine, JobPort) | Departments declare workflows; platform handles durability |
| P9: delegate_agent | Platform (AgentPort) | Departments call `agent.run()`; platform handles delegation |
| P10: AI SDK frontend | Shell (frontend) | Shell renders agent UI; departments declare contributions |
| P11: Playbooks | Platform (FlowEngine + Skills) | Cross-department, not owned by any single dept |
| P12: Smart model routing | Platform (LlmPort) | Departments specify intent; platform selects model |

Every proposal is a **platform** or **shell** change. No proposal requires changing the `DepartmentApp` contract. This is the design goal: the contract absorbs ecosystem evolution without cascading to departments.

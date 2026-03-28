> **NOTE:** Phase 4 (remove EngineKind) completed via ADR-014.
>
> **2026-03-28:** Examples below may still mention `EngineKind` — treat as **historical pseudo-code**; runtime uses string department IDs (`DepartmentManifest.id`).

# ADR-011: Config Hierarchy & Department Registry

**Date:** 2026-03-23
**Status:** Proposed
**Author:** Mehdi + Claude

---

## Context

RUSVEL has 4 separate config systems that don't talk to each other:

| System | Storage | Wired? | UI? |
|--------|---------|--------|-----|
| `TomlConfig` (rusvel-config) | `~/.rusvel/config.toml` | No (dead code) | No |
| `ChatConfig` (god agent) | ObjectStore `chat_config/current` | Backend only | No |
| `DepartmentConfig` (x12) | ObjectStore `dept_config/{engine}` | Yes | Partial (chat panel) |
| `UserProfile` | `~/.rusvel/profile.toml` | Read-only at boot | No edit UI |

Plus 6 CRUD entity types (agents, skills, rules, MCP, hooks, workflows) scoped by loose `metadata.engine` string matching.

### SOLID Violations

- **S**: `department.rs` is 700+ lines doing config, streaming, history, events, rules, agent mentions, MCP, and storage — 8 responsibilities.
- **O**: Adding a department touches 7 files (core enum, workspace TOML, app TOML, main.rs, department.rs config + kind + macro, api routes, frontend route, frontend layout). Should be zero.
- **I**: DepartmentPanel has 9 tabs. Not every department needs all tabs.
- **D**: Hardcoded defaults in match arms, hardcoded model/tool lists.

---

## Decision

### 1. Three-Layer Config Cascade

```
GlobalConfig          (base — applies everywhere)
  └─ DepartmentConfig (overrides per engine)
       └─ SessionConfig (overrides per conversation)
```

**Resolution rule:** session overrides department overrides global. Unset fields inherit from parent layer.

```rust
/// Unified config — same shape at every layer.
/// None fields mean "inherit from parent".
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayeredConfig {
    pub model: Option<String>,
    pub effort: Option<String>,
    pub max_budget_usd: Option<f64>,
    pub permission_mode: Option<String>,
    pub allowed_tools: Option<Vec<String>>,
    pub disallowed_tools: Option<Vec<String>>,
    pub system_prompt: Option<String>,
    pub add_dirs: Option<Vec<String>>,
    pub max_turns: Option<u32>,
}

impl LayeredConfig {
    /// Merge: self overrides parent. None fields fall through.
    pub fn resolve(&self, parent: &LayeredConfig) -> ResolvedConfig {
        ResolvedConfig {
            model: self.model.clone().or(parent.model.clone()).unwrap_or("sonnet".into()),
            effort: self.effort.clone().or(parent.effort.clone()).unwrap_or("medium".into()),
            // ... same pattern for all fields
        }
    }
}
```

**Storage:**
- Global: `~/.rusvel/config.toml` (via TomlConfig, now actually wired)
- Department: ObjectStore `config/dept_{engine}`
- Session: ObjectStore `config/session_{id}_{engine}`

### 2. Department Registry

Replace hardcoded match arms, frontend routes, and nav items with a single declarative registry.

**File:** `~/.rusvel/departments.toml` (with built-in defaults)

```toml
# Each department is fully described here.
# Adding a department = adding a [[department]] block. Zero code changes.

[[department]]
id = "forge"
name = "Forge"
title = "Forge Department"
engine_kind = "Forge"
icon = "="
color = "indigo"
system_prompt = """
You are the Forge department of RUSVEL.
Focus: agent orchestration, goal planning, mission management.
"""
capabilities = ["planning", "orchestration"]
tabs = ["actions", "agents", "workflows", "rules", "events"]
quick_actions = [
  { label = "Daily plan", prompt = "Generate today's mission plan based on active goals." },
  { label = "Review progress", prompt = "Review progress on all active goals." },
]

[[department]]
id = "code"
name = "Code"
title = "Code Department"
engine_kind = "Code"
icon = "#"
color = "emerald"
default_config = { model = "sonnet", effort = "high", permission_mode = "default", add_dirs = ["."] }
system_prompt = """
You are the Code department of RUSVEL.
Full access to Claude Code tools. Focus: code intelligence, implementation, debugging.
"""
capabilities = ["code_analysis", "tool_use"]
tabs = ["actions", "agents", "skills", "rules", "mcp", "hooks", "dirs", "events"]
quick_actions = [
  { label = "Analyze codebase", prompt = "Analyze the codebase structure, dependencies, and code quality." },
  { label = "Run tests", prompt = "Run the full test suite and report results." },
]

[[department]]
id = "finance"
name = "Finance"
title = "Finance Department"
engine_kind = "Finance"
icon = "%"
color = "green"
system_prompt = """
You are the Finance department of RUSVEL.
Focus: revenue tracking, expense management, tax, runway forecasting, P&L, unit economics.
"""
capabilities = ["ledger", "tax", "runway"]
tabs = ["actions", "agents", "rules", "events"]
quick_actions = [
  { label = "Record income", prompt = "Record a new income transaction." },
  { label = "P&L report", prompt = "Generate a profit & loss report." },
  { label = "Calculate runway", prompt = "Calculate current runway." },
]

# ... same pattern for all 12 departments
```

### 3. Rust: DepartmentRegistry

```rust
// crates/rusvel-core/src/registry.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentDef {
    pub id: String,              // "finance"
    pub name: String,            // "Finance"
    pub title: String,           // "Finance Department"
    pub engine_kind: EngineKind,
    pub icon: String,            // "%"
    pub color: String,           // "green"
    pub system_prompt: String,
    pub capabilities: Vec<String>,
    pub tabs: Vec<String>,       // which tabs to show in panel
    pub quick_actions: Vec<QuickAction>,
    pub default_config: Option<LayeredConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickAction {
    pub label: String,
    pub prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentRegistry {
    pub departments: Vec<DepartmentDef>,
}

impl DepartmentRegistry {
    /// Load from TOML file, falling back to built-in defaults.
    pub fn load(path: &Path) -> Self { ... }

    pub fn get(&self, id: &str) -> Option<&DepartmentDef> {
        self.departments.iter().find(|d| d.id == id)
    }

    pub fn list(&self) -> &[DepartmentDef] {
        &self.departments
    }
}
```

### 4. API: Dynamic Routes from Registry

Instead of 72 hardcoded routes (6 per dept x 12), use a single parameterized router:

```rust
// Replace 72 lines of .route() with 6:
.route("/api/dept/{dept}/chat", post(department::dept_chat))
.route("/api/dept/{dept}/chat/conversations", get(department::dept_conversations))
.route("/api/dept/{dept}/chat/conversations/{id}", get(department::dept_history))
.route("/api/dept/{dept}/config", get(department::dept_config_get))
.route("/api/dept/{dept}/config", put(department::dept_config_update))
.route("/api/dept/{dept}/events", get(department::dept_events))

// Plus a registry endpoint:
.route("/api/departments", get(department::list_departments))
```

The handler validates `{dept}` against the registry:

```rust
pub async fn dept_chat(
    State(state): State<Arc<AppState>>,
    Path(dept): Path<String>,
    Json(body): Json<ChatRequest>,
) -> Result<..., (StatusCode, String)> {
    let def = state.registry.get(&dept)
        .ok_or((StatusCode::NOT_FOUND, format!("Unknown department: {dept}")))?;
    department_chat_handler(&dept, def.engine_kind, state, body).await
}
```

### 5. API: Unified Config Endpoint

```
GET  /api/config                    → GlobalConfig (resolved)
PUT  /api/config                    → Update GlobalConfig
GET  /api/config/layers             → { global, departments: {id: config}, sessions: {id: config} }
GET  /api/dept/{dept}/config        → Resolved (global + dept merged)
PUT  /api/dept/{dept}/config        → Update dept layer only
GET  /api/profile                   → UserProfile
PUT  /api/profile                   → Update UserProfile
GET  /api/departments               → DepartmentDef[] (from registry)
```

### 6. Frontend: Dynamic Department Pages

Replace 12 identical `+page.svelte` files with a single dynamic route:

```
frontend/src/routes/dept/[id]/+page.svelte
```

```svelte
<script lang="ts">
  import { page } from '$app/state';
  import { departments } from '$lib/stores';
  import DepartmentChat from '$lib/components/chat/DepartmentChat.svelte';
  import DepartmentPanel from '$lib/components/chat/DepartmentPanel.svelte';

  // Get department def from registry (loaded once at app init)
  let dept = $derived(departments.find(d => d.id === page.params.id));
</script>

{#if dept}
  <DepartmentPanel
    dept={dept.id}
    title={dept.title}
    icon={dept.icon}
    color={dept.color}
    quickActions={dept.quick_actions}
    tabs={dept.tabs}
  />
  <DepartmentChat dept={dept.id} title={dept.title} icon={dept.icon} />
{/if}
```

**Nav items generated from registry:**

```svelte
{#each departments as dept}
  <a href="/dept/{dept.id}">{dept.icon} {dept.name}</a>
{/each}
```

### 7. Frontend: Unified Settings Page

```
/settings
  ├── Profile         (edit UserProfile)
  ├── Global Config   (model, effort, budget, tools, permission mode)
  ├── Departments     (collapsible list, per-dept config overrides)
  │     ├── Code      (model override, system prompt, dirs, tabs enabled)
  │     ├── Finance   (model override, system prompt)
  │     └── ...
  └── Advanced        (data dir, API status, version)
```

### 8. Frontend: Stores Upgrade

```typescript
// Global state loaded once from /api/departments + /api/config
export const departments = writable<DepartmentDef[]>([]);
export const globalConfig = writable<LayeredConfig>({});
export const profile = writable<UserProfile | null>(null);
```

App loads these on mount via `GET /api/departments`, `GET /api/config`, `GET /api/profile`. All components read from stores instead of fetching independently.

---

## What Gets Deleted

| Before | After |
|--------|-------|
| 12 hardcoded `+page.svelte` files | 1 dynamic `[id]/+page.svelte` |
| 72 hardcoded API routes | 6 parameterized routes + 1 registry |
| 260-line `default_for()` match block | Registry TOML lookup |
| `dept_wrappers!()` macro + 12 invocations | Single generic handler |
| Separate `ChatConfig` struct | `LayeredConfig` at global level |
| Dead `TomlConfig` crate | Wired as global config backend |
| Hardcoded model/tool lists | Config-driven, loaded from registry |

## What Gets Added

| Component | Purpose |
|-----------|---------|
| `DepartmentRegistry` | Declarative department definitions |
| `LayeredConfig` | Unified config with inheritance |
| `GET /api/departments` | Frontend loads registry once |
| `GET/PUT /api/profile` | Edit UserProfile from UI |
| `/settings` page (real) | Single place for all config |
| `/dept/[id]` dynamic route | One page for all departments |
| `departments` store | Global state, no per-component fetch |

---

## Migration Path

### Phase 1: Registry + Dynamic Routes (backend)
1. Create `DepartmentRegistry` in rusvel-core
2. Add `departments.toml` with all 12 departments
3. Replace 72 routes with 6 parameterized + registry endpoint
4. Delete `dept_wrappers!()` macro and 260-line match block
5. Wire `TomlConfig` as global config source
6. Add `/api/profile` endpoint

### Phase 2: Frontend Unification
1. Add `departments` store, load from `/api/departments` on mount
2. Create `/dept/[id]/+page.svelte` dynamic route
3. Update DepartmentPanel to respect `tabs` from registry
4. Delete 12 static department pages
5. Generate nav items from registry

### Phase 3: Settings Page
1. Build unified `/settings` page with Profile, Global, Departments, Advanced sections
2. Wire `GET/PUT /api/config`, `GET/PUT /api/profile`
3. Per-department config override UI with inheritance visualization

### Phase 4: Cleanup
1. Remove `ChatConfig` (absorbed into `LayeredConfig` at global level)
2. Remove hardcoded model/tool lists (serve from registry or config)
3. Remove `EngineKind` variants from code — derive from registry at runtime
4. Update CLAUDE.md with new architecture

---

## Consequences

**Positive:**
- Zero-code department addition (just edit TOML)
- Single source of truth for all config
- Config inheritance eliminates duplication
- Settings page gives full visibility
- 90% reduction in boilerplate routes/pages
- DepartmentPanel shows only relevant tabs per department

**Negative:**
- Breaking change to API routes (`/api/dept/code/chat` → still works, just implemented differently)
- Registry TOML file is a new dependency (but has built-in defaults as fallback)
- Need migration for existing ObjectStore config data

**Risks:**
- Dynamic routing may complicate SvelteKit prerendering (mitigated: already using SPA fallback)
- TOML registry parsing errors at startup (mitigated: built-in Rust defaults as fallback)

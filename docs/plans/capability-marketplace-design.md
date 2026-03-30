# Capability Marketplace & Visual Builders — Design Doc

> **Goal:** Every artifact type (skill, workflow, playbook, rule, agent, MCP server, hook, plugin) can be **discovered from community, built with AI, edited visually, and used across departments on the fly.**
>
> **Inspired by:** n8n (community nodes, workflow templates, visual canvas), RUSVEL's existing Capability Engine
>
> **Date:** 2026-03-30

---

## 1. Current State Audit

### What EXISTS Today

| Artifact | CRUD API | AI Build (`!build`) | Web UI | Cross-Dept | Persistent | Community Discovery |
|----------|----------|---------------------|--------|------------|------------|---------------------|
| Skills | `GET/POST /api/skills` | `!build skill: ...` | Form + inline edit (SkillsTab) | via `metadata.engine` | ObjectStore | None |
| Rules | `GET/POST /api/rules` | `!build rule: ...` | Form + toggle (RulesTab) | via `metadata.engine` | ObjectStore | None |
| Agents | `GET/POST /api/agents` | `!build agent: ...` | Form (AgentsTab) | via `metadata.engine` | ObjectStore | None |
| Workflows | `GET/POST /api/workflows` | API only | Visual nodes (xyflow) + form | via `metadata.engine` | ObjectStore | None |
| Playbooks | `GET/POST /api/playbooks` | None | Partial | Not scoped | **In-memory only** | None |
| MCP Servers | `GET/POST /api/mcp-servers` | `!build mcp: ...` | Form (MCP tab) | via `metadata.engine` | ObjectStore | WebSearch (mcp.so) |
| Hooks | `GET/POST /api/hooks` | `!build hook: ...` | Form (HooksTab) | via `metadata.engine` | ObjectStore | None |
| Kits | `GET /api/kits` (3 built-in) | None | **No install UI** | All depts | Hardcoded | None |
| Flows | `GET/POST /api/flows` | None | **Raw JSON input** (see screenshot) | Global | ObjectStore | None |

### Architecture Strengths (to build on)

1. **Capability Engine** (`POST /api/capability/build`) — AI discovers MCP servers via WebSearch, generates bundles (agents + skills + rules + workflows), auto-installs to ObjectStore. This is the seed of marketplace.
2. **`!build` command** — Natural language to JSON entity, works for 5 artifact types.
3. **Flow Engine** (petgraph DAG) — Code/condition/agent node types, checkpoint/resume, parallel execution. Mature execution layer.
4. **ObjectStore + metadata** — All artifacts use `serde_json::Value` metadata with `engine` scoping. Clean extensibility.
5. **Department scoping** — Artifacts filter by department automatically. Cross-dept sharing is `metadata.engine = null`.
6. **WorkflowBuilder.svelte** — xyflow/svelte integration exists, just needs to be interactive.

### Critical Gaps

1. **No community registry** — Can't browse, publish, rate, or install shared artifacts
2. **Flows page = raw JSON** — No visual builder for the most powerful artifact type
3. **Playbooks not persisted** — Lost on restart (in-memory HashMap)
4. **Hooks not wired** — Schema exists, event dispatch doesn't fire them
5. **No visual builder for flows** — The existing WorkflowBuilder is for simple sequential workflows, not DAG flows
6. **No artifact versioning** — No rollback, no dependency tracking
7. **No credential vault** — rusvel-auth is env-var only, no per-integration encrypted storage
8. **Kit install UI missing** — 3 kits exist but no web UI to browse/install them

---

## 2. Design Vision

### The Experience We Want

```
┌─────────────────────────────────────────────────────────────────┐
│                        RUSVEL App                               │
│                                                                 │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ Discover │  │  Build   │  │  Manage  │  │   Use    │       │
│  │          │  │          │  │          │  │          │       │
│  │ Browse   │  │ AI Chat  │  │ Visual   │  │ Auto-    │       │
│  │ registry │  │ !build   │  │ editors  │  │ trigger  │       │
│  │ Search   │  │ !cap     │  │ CRUD     │  │ On-the-  │       │
│  │ Install  │  │ Template │  │ Version  │  │ fly      │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
│       │              │              │              │             │
│       └──────────────┴──────────────┴──────────────┘             │
│                          │                                       │
│              ┌───────────┴───────────┐                          │
│              │   Artifact Registry   │                          │
│              │   (ObjectStore +      │                          │
│              │    community index)   │                          │
│              └───────────────────────┘                          │
└─────────────────────────────────────────────────────────────────┘
```

### Four Pillars

1. **Discover** — Browse a marketplace of community-contributed artifacts. Search, filter by type/department/rating. One-click install.
2. **Build** — AI-powered builders for every artifact type. Natural language OR visual drag-and-drop. Generate from templates.
3. **Manage** — Visual editors for each artifact. Version history. Enable/disable per department. Test before deploy.
4. **Use** — Artifacts activate automatically. Hooks fire on events. Workflows trigger from webhooks. Skills available in chat. Rules inject into prompts.

---

## 3. Phased Implementation Plan

### Phase 1: Fix Foundations (1-2 days)

> Quick wins that unblock everything else.

#### 1.1 Persist Playbooks to ObjectStore
- **Current:** `playbooks.rs` uses `Arc<Mutex<HashMap<String, Playbook>>>` — lost on restart
- **Fix:** Use `ObjectStore` with namespace `"playbooks"`, same pattern as skills/rules/agents
- **Files:** `crates/rusvel-api/src/playbooks.rs`

#### 1.2 Wire Hook Event Dispatch
- **Current:** Hook CRUD exists, 16 event types defined, but hooks never fire
- **Fix:** In chat handler and job worker, after key events, query hooks by `event` field and execute (command/http/prompt)
- **Files:** `crates/rusvel-api/src/chat.rs`, `crates/rusvel-app/src/main.rs` (job worker)

#### 1.3 Add `!build` for Playbooks and Flows
- **Current:** `!build` supports agent/skill/rule/mcp/hook but not playbook/flow
- **Fix:** Add `"playbook"` and `"flow"` branches to `build_cmd.rs` parser
- **Files:** `crates/rusvel-api/src/build_cmd.rs`

---

### Phase 2: Visual Builders (3-5 days)

> Replace raw JSON with drag-and-drop visual editors. Inspired by n8n's canvas.

#### 2.1 Visual Flow Builder (Priority)
Replace the current "Create Flow (JSON)" textarea with an interactive canvas:

```
┌─────────────────────────────────────────────────────────────┐
│  Flow: my-pipeline                              [Save] [Run]│
│─────────────────────────────────────────────────────────────│
│                                                             │
│  ┌──────────┐     ┌──────────┐     ┌──────────┐           │
│  │ Webhook  │────▶│  Code    │────▶│  Agent   │           │
│  │ trigger  │     │ analyze  │     │ summarize│           │
│  └──────────┘     └──────────┘     └──────────┘           │
│                        │                                    │
│                   ┌────┴────┐                              │
│                   │Condition│                              │
│                   │score>80 │                              │
│                   └────┬────┘                              │
│              ┌─────────┼─────────┐                        │
│              ▼                   ▼                         │
│        ┌──────────┐       ┌──────────┐                    │
│        │ Publish  │       │  Queue   │                    │
│        │ content  │       │  review  │                    │
│        └──────────┘       └──────────┘                    │
│                                                             │
│─────────────────────────────────────────────────────────────│
│  Node Palette: [+ Trigger] [+ Code] [+ Agent] [+ Cond]    │
│  [+ Browser] [+ Department Action]                         │
└─────────────────────────────────────────────────────────────┘
```

**Implementation:**
- **Library:** `@xyflow/svelte` (already in deps, used by WorkflowBuilder)
- **Node types:** Map to existing FlowEngine node types: `code`, `condition`, `agent`, `browser_trigger`, `browser_action`
- **Interactions:** Drag from palette to canvas, connect handles, click to edit node config
- **Serialization:** Canvas state ↔ Flow JSON (nodes + connections arrays)
- **Files:** New `frontend/src/lib/components/flow/FlowCanvas.svelte`, update `frontend/src/routes/flows/+page.svelte`

#### 2.2 Unified Artifact Builder Modal
A single `<ArtifactBuilder>` component that adapts to any artifact type:

```svelte
<ArtifactBuilder type="skill" department={currentDept}>
  <!-- Renders appropriate form fields based on type -->
  <!-- "Generate with AI" button calls !build -->
  <!-- Preview pane shows what will be created -->
</ArtifactBuilder>
```

- **Types:** skill, rule, agent, mcp, hook, workflow, playbook
- **AI assist:** Each type has a "Generate with AI" button that calls the `!build` endpoint
- **Template picker:** "Start from template" shows relevant starter kit items
- **Files:** `frontend/src/lib/components/builders/ArtifactBuilder.svelte`

#### 2.3 Workflow Step Editor Enhancement
Upgrade the existing WorkflowBuilder from read-mostly to fully interactive:
- Drag to reorder steps
- Click node to edit agent/prompt inline
- Add conditional branches (when flow-engine nodes are used)
- Visual execution replay (show which steps ran, with timing)

---

### Phase 3: Marketplace & Discovery (5-8 days)

> The community layer. Browse, search, install, publish artifacts.

#### 3.1 Artifact Registry Design

**Local Registry (SQLite-backed):**
```sql
-- Extends ObjectStore with marketplace metadata
CREATE TABLE artifact_registry (
    id TEXT PRIMARY KEY,
    artifact_type TEXT NOT NULL,     -- skill, rule, agent, workflow, flow, playbook, mcp, hook
    name TEXT NOT NULL,
    description TEXT,
    author TEXT,                      -- "built-in", "ai-generated", "community:username"
    source TEXT,                      -- "local", "community", "capability-engine"
    version TEXT DEFAULT '1.0.0',
    tags TEXT,                        -- JSON array
    department_scope TEXT,            -- null = global, otherwise engine name
    install_count INTEGER DEFAULT 0,
    rating REAL,
    bundle_json TEXT NOT NULL,        -- The actual artifact definition
    created_at TEXT,
    updated_at TEXT
);
```

**Community Index (remote, future):**
- JSON index file hosted on GitHub/CDN (like Homebrew taps)
- `GET https://registry.rusvel.dev/v1/artifacts?type=workflow&q=lead+scoring`
- Start simple: curated JSON file, not a full server
- Phase 3 can use a GitHub repo as the registry (PRs = submissions)

#### 3.2 Marketplace UI — `/marketplace` Route

```
┌─────────────────────────────────────────────────────────────┐
│  Marketplace                                    [Publish]   │
│─────────────────────────────────────────────────────────────│
│  [All] [Skills] [Workflows] [Flows] [Agents] [MCP] [Kits]  │
│  Search: [________________________] [Filter ▾]              │
│─────────────────────────────────────────────────────────────│
│                                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐          │
│  │ 📋          │ │ 🔄          │ │ 🤖          │          │
│  │ Lead Scorer │ │ Content     │ │ Code Review │          │
│  │ Workflow    │ │ Pipeline    │ │ Agent       │          │
│  │             │ │             │ │             │          │
│  │ ★★★★☆ (23)│ │ ★★★★★ (45)│ │ ★★★★☆ (12)│          │
│  │ by: rusvel  │ │ by: community│ │ by: ai-gen │          │
│  │ [Install]   │ │ [Install]   │ │ [Install]   │          │
│  └─────────────┘ └─────────────┘ └─────────────┘          │
│                                                             │
│  Starter Kits                                               │
│  ┌─────────────────────────────────────────────┐           │
│  │ 🚀 Indie SaaS Kit — 12 artifacts            │           │
│  │ agents, skills, rules for SaaS founders     [Install]   │
│  ├─────────────────────────────────────────────┤           │
│  │ 💼 Freelancer Kit — 8 artifacts              │           │
│  │ invoicing, outreach, project management     [Install]   │
│  ├─────────────────────────────────────────────┤           │
│  │ 🔧 Open Source Kit — 10 artifacts            │           │
│  │ issue triage, release notes, community      [Install]   │
│  └─────────────────────────────────────────────┘           │
└─────────────────────────────────────────────────────────────┘
```

**Key features:**
- Tab filter by artifact type
- Search with full-text matching
- Source badges: built-in, ai-generated, community
- One-click install (writes to ObjectStore, scoped to chosen department)
- Kit installer (batch install with preview of what's included)

**API endpoints:**
```
GET  /api/marketplace                    — list all available artifacts (local + community index)
GET  /api/marketplace/search?q=...&type=...  — search with filters
POST /api/marketplace/install/{id}       — install artifact to department
POST /api/marketplace/publish            — export artifact for sharing
GET  /api/marketplace/kits               — list starter kits (replaces /api/kits)
POST /api/marketplace/kits/{id}/install  — install kit bundle
```

#### 3.3 AI-Powered Discovery Integration
Upgrade the existing Capability Engine to feed the marketplace:
- `!capability <description>` results appear in marketplace as "AI-discovered" items
- Discovered MCP servers from mcp.so get cached in registry
- "Suggest for me" button on marketplace page uses Capability Engine

#### 3.4 Artifact Export/Import
```
POST /api/artifacts/export   — export selected artifacts as JSON bundle
POST /api/artifacts/import   — import bundle, resolve conflicts
```
- Export generates a portable JSON bundle (same format as Kits)
- Import with conflict resolution: skip, overwrite, rename

---

### Phase 4: On-the-Fly Automation (3-5 days)

> Artifacts that activate and compose automatically.

#### 4.1 Wire Hooks to Real Events
Complete the hook dispatch system:
- **Chat events:** `PostToolUse`, `TaskCompleted` fire matching hooks
- **Job events:** `JobCompleted`, `JobFailed` fire hooks
- **Department events:** Department-scoped hooks trigger only for that department
- **Hook types:** `command` (shell exec), `http` (webhook POST), `prompt` (LLM call with context)

#### 4.2 Webhook → Flow Triggers
- Add `webhook_trigger` node type to FlowEngine
- `POST /api/flows/{id}/webhook` — external trigger endpoint
- Webhook node captures request body as flow input
- Auto-generate unique webhook URLs per flow

#### 4.3 Department Auto-Wiring
When an artifact is installed to a department:
- Skills immediately available in that department's chat (`/skill-name`)
- Rules auto-inject into system prompt (if enabled)
- Agents available via `@agent-name` mention
- Workflows appear in department's workflow tab
- MCP servers connect on next chat session

#### 4.4 Artifact Composition
- Skills can reference other skills: `{{skill:lead-scorer}}` in prompt templates
- Workflows can call other workflows as steps
- Playbooks can include flows as steps (already partially supported)
- Agents can have `tools: [skill:xyz, workflow:abc]` references

---

### Phase 5: Community & Ecosystem (future)

> Sharing with the world.

#### 5.1 Community Registry
- GitHub repo as registry (like Homebrew): `rusvel/marketplace`
- PRs for submissions, CI validates artifact schema
- JSON index auto-published to CDN
- RUSVEL app fetches index on marketplace page load

#### 5.2 Trust & Verification
- **Built-in:** Ships with RUSVEL, tested, maintained
- **AI-generated:** Created by Capability Engine, user-validated
- **Community:** Submitted by users, reviewed by maintainers
- **Verified:** Community artifacts that pass automated testing

#### 5.3 Versioning & Updates
- Semantic versioning for all artifacts
- "Update available" badge in marketplace
- Changelog per artifact
- Rollback to previous version

#### 5.4 Usage Analytics
- Track which artifacts are used, how often, in which departments
- Surface "popular in your industry" recommendations
- Department-level artifact usage dashboard

---

## 4. n8n Patterns to Adopt

| n8n Pattern | RUSVEL Adaptation |
|-------------|-------------------|
| NPM as node registry | GitHub repo as artifact registry (simpler, no auth needed) |
| Strapi CMS for verified list | JSON index with `verified: true` flag |
| `@vue-flow/core` canvas | `@xyflow/svelte` canvas (already in deps) |
| Workflow templates gallery | Marketplace with type filter |
| Credential type system | Per-integration credential vault in ObjectStore (encrypted) |
| Community node install/uninstall | `POST /api/marketplace/install` / `DELETE` |
| Node versioning | `version` field in artifact_registry |
| Webhook trigger node | `webhook_trigger` flow node type |
| Execution visualization | Flow run replay with node status badges |
| Pinned data for testing | "Dry run" mode with sample input |

---

## 5. File Impact Summary

### Backend (Rust)
| File | Change |
|------|--------|
| `crates/rusvel-api/src/playbooks.rs` | Migrate from HashMap to ObjectStore |
| `crates/rusvel-api/src/build_cmd.rs` | Add playbook + flow branches |
| `crates/rusvel-api/src/chat.rs` | Wire hook dispatch after events |
| `crates/rusvel-api/src/lib.rs` | Add `/api/marketplace/*` routes |
| `crates/rusvel-api/src/marketplace.rs` | **New** — marketplace handlers |
| `crates/rusvel-api/src/flow_routes.rs` | Add webhook trigger endpoint |
| `crates/rusvel-core/src/domain.rs` | Add `ArtifactMeta` struct (version, source, tags) |
| `crates/flow-engine/src/lib.rs` | Add `webhook_trigger` node type |

### Frontend (Svelte)
| File | Change |
|------|--------|
| `frontend/src/routes/marketplace/+page.svelte` | **New** — marketplace browse page |
| `frontend/src/routes/flows/+page.svelte` | Replace JSON textarea with FlowCanvas |
| `frontend/src/lib/components/flow/FlowCanvas.svelte` | **New** — interactive DAG editor |
| `frontend/src/lib/components/flow/NodePalette.svelte` | **New** — draggable node types |
| `frontend/src/lib/components/flow/NodeEditor.svelte` | **New** — node config panel |
| `frontend/src/lib/components/builders/ArtifactBuilder.svelte` | **New** — unified builder modal |
| `frontend/src/lib/components/marketplace/ArtifactCard.svelte` | **New** — marketplace item card |
| `frontend/src/lib/components/marketplace/KitInstaller.svelte` | **New** — kit preview + install |

### Navigation
| Change |
|--------|
| Add "Marketplace" to sidebar (between Approvals and Database) |
| Add marketplace icon + badge for available updates |

---

## 6. Priority Order

```
Phase 1 (foundations)     ███  1-2 days   — Unblocks everything
Phase 2 (visual builders) █████  3-5 days  — Biggest UX impact, replaces JSON input
Phase 3 (marketplace)    ████████  5-8 days  — Community discovery, the flagship feature
Phase 4 (automation)     █████  3-5 days  — On-the-fly activation, hook wiring
Phase 5 (ecosystem)      ──────  future   — Community registry, versioning, analytics
```

**Start with Phase 1** — it's quick wins that fix real bugs (playbook persistence, hook wiring) and unblock Phase 2-3.

**Phase 2 is the highest-impact work** — replacing the raw JSON flow editor with a visual canvas is the single biggest UX improvement. This is what transforms RUSVEL from "developer tool" to "visual automation platform."

**Phase 3 is the flagship** — the marketplace is what makes RUSVEL an ecosystem, not just an app. But it needs Phase 1-2 to have artifacts worth sharing.

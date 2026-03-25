# Native Terminal Multiplexer — Built on RUSVEL's Stack

> Date: 2026-03-25
> Status: Proposed
> Depends on: rusvel-core (Session, EventBus), rusvel-api (Axum), frontend (paneforge)
> Related:
> - `docs/design/department-as-app.md` — ADR-014, DepartmentApp contract, manifest contributions
> - `docs/design/agent-workforce.md` — 14 sub-agents, each gets visible panes
> - `docs/plans/agent-orchestration.md` — delegate_agent, event triggers, playbooks
> - `docs/plans/cdp-browser-bridge.md` — CDP browser tabs alongside terminal panes
> - `docs/plans/next-level-proposals.md` — P7 (AG-UI), P8 (durable execution), P9 (delegation)
> - `docs/plans/next-level-inspiration-2026-03-25.md` — Playbooks, Executive Brief, Roundtable

---

## Vision

Every department, project, and session maps to a window, tab, and pane — with full visibility in the web UI. No external dependency (no zellij, no tmux). Built entirely on our Rust + SvelteKit stack, reusing the patterns already battle-tested in the codebase.

The terminal is not a department — it's a **platform service** (like EventBus, JobQueue, MemoryPort). Every department can use it. Agent runs, playbook steps, flow nodes, CDP browser sessions, and manual shells all converge into a single observable workspace.

---

## What We Already Have (Reuse)

| Existing Asset | Location | Reuse For |
|---|---|---|
| `axum` with `features = ["ws"]` | Cargo.toml workspace | WebSocket per pane |
| `tokio::process::Command` | rusvel-builtin-tools/shell.rs | Child process spawning pattern |
| `mpsc::channel(64)` | rusvel-agent/lib.rs:142 | Process output streaming |
| `broadcast::Sender` | rusvel-event/lib.rs | Multi-subscriber pane output |
| `crossterm` | rusvel-tui | Terminal input/ANSI handling |
| `ratatui` | rusvel-tui | TUI surface (`--tui` mode) |
| `paneforge` | frontend/package.json | Resizable pane splits (already installed) |
| `Session.metadata: Value` | rusvel-core/domain.rs | Persist window/pane layouts (ADR-007) |
| `DepartmentManifest.ui` | rusvel-core/department/manifest.rs | Departments declare "terminal" tab |
| `EventBus` | rusvel-event | Pane lifecycle events |
| `DepartmentApp::register()` | rusvel-core/department/app.rs | Departments register default pane commands |
| SSE streaming pattern | rusvel-api/chat.rs | Proven server→client streaming |
| `AgentRuntime::run_streaming` | rusvel-agent/lib.rs | mpsc→consumer pattern |
| `RegistrationContext` | rusvel-core/department/context.rs | Terminal registrar slot |
| `@xyflow/svelte` | frontend/package.json | Workflow builder (pane layout inspiration) |

## What We Add

| New Dependency | Purpose |
|---|---|
| `portable-pty` (Rust crate) | Unix/macOS PTY allocation (pseudoterminal pairs) |
| `@xterm/xterm` + `@xterm/addon-fit` (npm) | Browser-side terminal emulator |
| `rusvel-terminal` (new crate) | Multiplexer core: domain types, PTY manager, port trait |

---

## Architecture: Terminal as Platform Service

```
┌─────────────────────────────────────────────────────────────────┐
│                      DEPARTMENTS (Apps)                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │  Forge   │ │  Code    │ │ Content  │ │ Harvest  │  ...      │
│  │  dept    │ │  dept    │ │  dept    │ │  dept    │           │
│  └────┬─────┘ └────┬─────┘ └────┬─────┘ └────┬─────┘          │
│       └─────────────┴────────────┴─────────────┘                │
│              DepartmentApp trait (ADR-014)                      │
├─────────────────────────────────────────────────────────────────┤
│                       PLATFORM                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ Agent    │ │ Terminal  │ │ Browser  │ │ Flow     │          │
│  │ Runtime  │ │ Manager  │ │ (CDP)    │ │ Engine   │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ Event    │ │ Job      │ │ Memory   │ │ Tool     │          │
│  │ Bus      │ │ Queue    │ │ + RAG    │ │ Registry │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
├─────────────────────────────────────────────────────────────────┤
│  KERNEL: rusvel-core ports + DepartmentApp + TerminalPort       │
├─────────────────────────────────────────────────────────────────┤
│                       SURFACES                                  │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ API      │ │ CLI      │ │ TUI      │ │ MCP      │          │
│  │ (Axum)   │ │ (Clap)   │ │(Ratatui) │ │ Server   │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  Frontend Shell (SvelteKit) — terminal + browser tabs    │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

**Terminal sits in the Platform layer** — same level as AgentRuntime, EventBus, FlowEngine. Departments consume it through `TerminalPort`, never manage PTYs directly.

---

## Domain Model

### Hierarchy

```
Session (existing)
  └── Window (new — one per department, agent chain, or custom workspace)
        └── Pane (new — one per shell, agent run, flow node, or CDP tab)
```

### Pane Sources (what can create a pane)

| Source | Pane Type | Auto-close | Example |
|--------|-----------|------------|---------|
| User (manual shell) | `Shell` | No | Click "Terminal" tab in dept page |
| Agent `bash` tool call | `AgentTool` | Configurable | `cargo test -p rusvel-api` during agent run |
| `delegate_agent` (P9) | `Delegation` | On completion | Sub-agent gets own pane in parent's window |
| Flow node execution (P8) | `FlowNode` | On node completion | Code node running `rustfmt` |
| Playbook step | `PlaybookStep` | On step completion | "Content-from-Code" playbook step 3 |
| CDP browser tab (future) | `Browser` | No | Upwork tab being observed |
| Builder agent (workforce) | `Builder` | On PR merged | Agent B2 migrating dept-content in worktree |

### Types (in `rusvel-terminal`)

```rust
pub struct Window {
    pub id: WindowId,
    pub session_id: SessionId,
    pub name: String,              // "Finance", "Code", "Pipeline: content-from-code"
    pub dept_id: Option<String>,   // links to DepartmentManifest.id
    pub source: WindowSource,      // Manual, Playbook, DelegationChain
    pub panes: Vec<PaneId>,
    pub layout: Layout,
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

pub enum WindowSource {
    Manual,                        // user-created
    Department(String),            // auto-created for dept
    Playbook(String),              // playbook execution window
    DelegationChain(RunId),        // orchestrator agent's workspace
}

pub struct Pane {
    pub id: PaneId,
    pub window_id: WindowId,
    pub title: String,
    pub command: String,           // "/bin/zsh", "cargo test -p X", etc.
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub size: PaneSize,
    pub status: PaneStatus,        // Running, Exited(i32), Suspended
    pub source: PaneSource,        // what created this pane
    pub run_id: Option<RunId>,     // links to AgentRuntime run (for delegation tree)
    pub node_id: Option<String>,   // links to FlowEngine node (for DAG visibility)
    pub created_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

pub enum PaneSource {
    Shell,                         // interactive user shell
    AgentTool { run_id: RunId },   // agent's bash tool call
    Delegation { parent_run_id: RunId, persona: String },
    FlowNode { execution_id: Uuid, node_id: String },
    PlaybookStep { playbook_id: String, step_id: String },
    Browser { tab_id: String },    // CDP browser tab (future)
    Builder { agent_id: String },  // workforce builder agent
}

pub struct PaneSize {
    pub rows: u16,
    pub cols: u16,
}

pub enum PaneStatus {
    Running,
    Exited(i32),
    Suspended,
}

pub enum Layout {
    Single,
    HSplit(Vec<f32>),              // horizontal split with ratios [0.5, 0.5]
    VSplit(Vec<f32>),              // vertical split with ratios
    Grid(u16, u16),               // rows x cols
    Custom(serde_json::Value),    // arbitrary layout for future evolution
}
```

### IDs (follow existing newtype pattern from rusvel-core/id.rs)

```rust
rusvel_id!(WindowId);
rusvel_id!(PaneId);
```

---

## Port Trait (Hexagonal)

Lives in `rusvel-core/ports.rs` alongside existing port traits.

```rust
#[async_trait]
pub trait TerminalPort: Send + Sync {
    // Window management
    async fn create_window(&self, session_id: &SessionId, name: &str, source: WindowSource) -> Result<WindowId>;
    async fn list_windows(&self, session_id: &SessionId) -> Result<Vec<Window>>;
    async fn close_window(&self, window_id: &WindowId) -> Result<()>;

    // Pane management
    async fn create_pane(&self, window_id: &WindowId, cmd: &str, cwd: &Path, size: PaneSize, source: PaneSource) -> Result<PaneId>;
    async fn write_pane(&self, pane_id: &PaneId, data: &[u8]) -> Result<()>;
    async fn resize_pane(&self, pane_id: &PaneId, size: PaneSize) -> Result<()>;
    async fn close_pane(&self, pane_id: &PaneId) -> Result<()>;

    // Streaming
    async fn subscribe_pane(&self, pane_id: &PaneId) -> Result<broadcast::Receiver<Vec<u8>>>;

    // Layout
    async fn get_layout(&self, window_id: &WindowId) -> Result<Layout>;
    async fn set_layout(&self, window_id: &WindowId, layout: Layout) -> Result<()>;

    // Query
    async fn panes_for_run(&self, run_id: &RunId) -> Result<Vec<Pane>>;
    async fn panes_for_flow(&self, execution_id: &Uuid) -> Result<Vec<Pane>>;
}
```

---

## Integration with ADR-014 (Department-as-App)

### DepartmentManifest declares terminal needs

The `UiContribution` in `DepartmentManifest` already has a `tabs: Vec<String>` field. Departments that want a terminal tab add `"terminal"` to their tabs list:

```rust
// In dept-code/src/manifest.rs
ui: UiContribution {
    tabs: vec![
        "actions".into(), "engine".into(), "agents".into(),
        "skills".into(), "rules".into(), "terminal".into(),  // ← terminal tab
        "events".into(),
    ],
    ..Default::default()
}
```

### TerminalContribution (new, added to manifest)

Departments declare default terminal commands and layout preferences:

```rust
/// Terminal configuration this department contributes.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TerminalContribution {
    /// Default shell commands to run when dept terminal opens.
    /// e.g., ["cargo watch -x 'test -p code-engine'"]
    pub default_commands: Vec<String>,

    /// Preferred pane layout for this department.
    pub default_layout: Option<Layout>,

    /// Environment variables set in this dept's terminal panes.
    pub env: HashMap<String, String>,

    /// Working directory override (default: project root).
    pub cwd: Option<String>,
}
```

Added to `DepartmentManifest` as `pub terminal: TerminalContribution`.

### Registration context gets TerminalPort

During boot, `RegistrationContext` provides `TerminalPort` to departments:

```rust
// In rusvel-core/department/context.rs (extend existing)
pub struct RegistrationContext {
    pub events: Arc<dyn EventPort>,
    pub storage: Arc<dyn StoragePort>,
    pub agent: Arc<dyn AgentPort>,
    pub tools: Arc<dyn ToolPort>,
    pub terminal: Arc<dyn TerminalPort>,   // ← NEW
    // ... other ports
}
```

### Departments can spawn panes in register()

```rust
// In dept-code/src/lib.rs
async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
    // Register routes, tools, etc. (existing pattern)
    // ...

    // Register terminal defaults — the platform creates panes lazily
    // (no PTY spawned until user opens the terminal tab)
    Ok(())
}
```

---

## Integration with Agent Orchestration

### delegate_agent → visible delegation tree

When an orchestrator agent calls `delegate_agent` (P9), each sub-agent gets its own pane in a delegation window:

```
Orchestrator calls delegate_agent("CodeWriter", "implement rate limiting")
  → TerminalManager creates pane with source: PaneSource::Delegation {
      parent_run_id: orchestrator.run_id,
      persona: "CodeWriter"
    }
  → CodeWriter's bash tool calls are visible in this pane
  → On completion, pane shows exit status + stays open for review

Orchestrator calls delegate_agent("Tester", "run tests")
  → New pane created in same delegation window
  → Split layout auto-adjusts: VSplit([0.5, 0.5])
```

**Frontend: DelegationTree.svelte** (from agent-orchestration.md) renders alongside terminal panes — tree view on left, live terminals on right:

```
┌──────────────────────────────────────────────┐
│ Delegation Tree     │ Terminal Panes          │
│                     │                         │
│ ▼ Orchestrator      │ ┌─────────────────────┐ │
│   ├─ CodeWriter ●   │ │ CodeWriter (running) │ │
│   ├─ Tester ○       │ │ $ cargo test...      │ │
│   └─ Reviewer ○     │ ├─────────────────────┤ │
│                     │ │ Tester (waiting)     │ │
│                     │ │                      │ │
│                     │ └─────────────────────┘ │
└──────────────────────────────────────────────┘
```

### invoke_flow → node-per-pane

When FlowEngine executes a DAG (P8 durable execution), each node gets a terminal pane:

```
Flow: plan → build → test → review
  → Window created: source = WindowSource::Playbook("code-pipeline")
  → As each node starts, a pane spawns: PaneSource::FlowNode { execution_id, node_id: "build" }
  → Code nodes show command output live
  → Agent nodes show LLM streaming + tool calls
  → Condition nodes show evaluation result
  → Layout auto-splits: Grid(2, 2) for 4 nodes
```

With durable execution (P8), if the flow crashes at node 3:
1. Panes 1-2 show "Completed (exit 0)" with scrollback preserved
2. Pane 3 shows "Interrupted — resume from checkpoint"
3. `POST /api/flows/{id}/executions/{exec_id}/resume` restarts node 3's pane

### PreToolUse hooks → pane approval

When a sub-agent calls a dangerous tool (SDK Feature 1), the PreToolUse hook can pause execution. The terminal pane shows:

```
$ agent requesting: rm -rf build/
⚠ APPROVAL REQUIRED — approve in RUSVEL UI
[Waiting for approval...]
```

The approval card (P4) appears in chat AND in the terminal pane header.

---

## Integration with CDP Browser Bridge

When `rusvel-cdp` connects to Chrome (cdp-browser-bridge.md), browser tabs appear as panes alongside terminal panes in the unified workspace:

```
┌──────────────────────────────────────────────┐
│ Harvest Department                            │
│ ┌─────────┬──────────┬──────────┐            │
│ │Terminal  │Browser   │Browser   │            │
│ │(shell)   │(Upwork)  │(LinkedIn)│            │
│ │$ rusvel  │ 🟢 obs   │ 🟢 obs   │            │
│ │harvest   │ 3 jobs   │ 2 leads  │            │
│ │pipeline  │ captured │ captured │            │
│ └─────────┴──────────┴──────────┘            │
└──────────────────────────────────────────────┘
```

Browser panes use `PaneSource::Browser { tab_id }` and show a read-only status view (not the full browser — that stays in Chrome). When CDP captures data, the terminal pane shows a live log:

```
[14:23:01] captured: Upwork job "Rust API developer" (score: 87)
[14:23:15] captured: Upwork job "Backend engineer" (score: 72)
[14:23:22] → auto-queued: HarvestScan job for 2 new opportunities
```

---

## Integration with Agent Workforce

The 14 builder agents (agent-workforce.md) each get isolated panes. When running a sprint:

```
Sprint: "Migrate 10 departments to ADR-014"
  → Window: "Sprint 2026-03-25" (source: Playbook)
  → Pane B2: dept-content migration (worktree: /tmp/rusvel-b2)
  → Pane B3: dept-forge migration (worktree: /tmp/rusvel-b3)
  → Pane B4a: dept-code migration
  → Pane B4b: dept-harvest migration (parallel)
  → ...

All visible simultaneously in a Grid(3, 4) layout.
Each pane shows real-time cargo test output from its worktree.
```

This is the literal dogfooding of the agent orchestration plan — builder agents are playbook templates, each step runs in a visible pane.

---

## Integration with Playbooks & Executive Brief

### Playbook execution → live terminal dashboard

When a playbook runs (next-level-inspiration Priority 1, agent-orchestration Phase 5):

```json
{
    "name": "content-from-code",
    "steps": [
        { "id": "analyze", "persona": "CodeWriter", "department": "code" },
        { "id": "draft", "persona": "ContentWriter", "department": "content" },
        { "id": "schedule", "persona": "ContentWriter", "department": "content" }
    ]
}
```

Each step creates a pane. The user watches the full pipeline execute:

```
┌──────────────────────────────────────────────┐
│ Playbook: Content-from-Code                   │
│ ┌──────────────┬──────────────┬─────────────┐│
│ │ analyze ✓    │ draft ●      │ schedule ○  ││
│ │ code dept    │ content dept │ content dept││
│ │ exit 0       │ Drafting...  │ (waiting)   ││
│ │ Found 3 new  │ ## Blog Post │             ││
│ │ API endpoints│ "RUSVEL now  │             ││
│ │              │ supports..." │             ││
│ └──────────────┴──────────────┴─────────────┘│
└──────────────────────────────────────────────┘
```

### Executive Brief → compact pane view

The daily brief (inspiration Priority 2) queries all 12 departments. With terminal integration, each department's status check runs in a temporary pane, and the brief aggregates:

```
[06:00] Executive Brief — querying 12 departments...
  ✓ code:    2 PRs merged, test coverage 91%
  ✓ content: 3 drafts awaiting review
  ✓ harvest: 5 new opportunities, 2 scored >80
  ✓ finance: runway 14 months
  ...
```

---

## Integration with AG-UI Protocol (P7)

Terminal pane events emit AG-UI events for frontend consumption:

```
LIFECYCLE:  terminal.pane.created → RUN_STARTED (when source is AgentTool/Delegation)
TEXT:       PTY output bytes → TEXT_MESSAGE_CONTENT (for agent tool panes)
TOOL:      bash tool call → TOOL_CALL_START + TOOL_CALL_END
STATE:     pane resize/focus → STATE_DELTA
CUSTOM:    terminal.pane.exited → CUSTOM { type: "pane_exited", exit_code }
```

This means third-party AG-UI clients can observe agent terminal activity — not just chat messages but actual command execution.

---

## Backend Implementation

### `rusvel-terminal` Crate Structure

```
crates/rusvel-terminal/
├── Cargo.toml
└── src/
    ├── lib.rs          # Domain types, re-exports (~200 lines)
    ├── manager.rs      # TerminalManager: PTY lifecycle, PaneHandle map (~450 lines)
    └── persistence.rs  # Save/restore window+pane layouts to SQLite (~150 lines)
```

### PaneHandle (internal, not exposed)

```rust
struct PaneHandle {
    id: PaneId,
    pty_pair: portable_pty::PtyPair,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    output_tx: broadcast::Sender<Vec<u8>>,
    child: Box<dyn portable_pty::Child + Send>,
    source: PaneSource,
    _reader_task: JoinHandle<()>,   // tokio task reading PTY master → broadcast
}
```

### TerminalManager

```rust
pub struct TerminalManager {
    panes: RwLock<HashMap<PaneId, PaneHandle>>,
    windows: RwLock<HashMap<WindowId, Window>>,
    events: Arc<dyn EventPort>,     // emit pane.created, pane.exited, etc.
    store: Arc<dyn StoragePort>,    // persist layouts
}
```

Key methods:

- `create_pane()` — allocates PTY pair via `portable-pty`, spawns child process on slave, starts tokio task reading master into `broadcast::Sender<Vec<u8>>`
- `write_pane()` — writes bytes to PTY master (user keystrokes)
- `subscribe_pane()` — returns `broadcast::Receiver<Vec<u8>>` (terminal output bytes)
- `resize_pane()` — calls `pty_pair.master.resize()` + sends SIGWINCH
- `panes_for_run()` — query panes by RunId (for delegation tree view)
- `panes_for_flow()` — query panes by FlowExecution (for playbook view)

### PTY Read Loop (per pane)

```rust
// Spawned as tokio::spawn_blocking (PTY read is blocking I/O)
fn spawn_reader(
    master: Box<dyn Read + Send>,
    tx: broadcast::Sender<Vec<u8>>,
    events: Arc<dyn EventPort>,
    pane_id: PaneId,
) -> JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        let mut buf = [0u8; 4096];
        loop {
            match master.read(&mut buf) {
                Ok(0) => {
                    // EOF — process exited
                    let _ = events.emit(Event {
                        kind: "terminal.pane.exited".into(),
                        payload: json!({ "pane_id": pane_id }),
                        ..Default::default()
                    });
                    break;
                }
                Ok(n) => { let _ = tx.send(buf[..n].to_vec()); }
                Err(_) => break,
            }
        }
    })
}
```

---

## API Routes (8 endpoints, new module `terminal_routes.rs`)

```
GET    /api/terminal/windows                  → list windows for active session
POST   /api/terminal/windows                  → create window { name, dept_id?, source? }
DELETE /api/terminal/windows/:id              → close window + all panes

POST   /api/terminal/panes                    → create pane { window_id, command?, cwd?, source? }
DELETE /api/terminal/panes/:id                → close pane (kill process)
POST   /api/terminal/panes/:id/resize         → resize { rows, cols }

GET    /api/terminal/runs/:run_id/panes       → panes for agent delegation chain
GET    /api/terminal/flows/:exec_id/panes     → panes for flow execution

WS     /api/terminal/ws/:pane_id              → bidirectional WebSocket
```

### WebSocket Handler

```rust
async fn ws_pane_handler(
    ws: WebSocketUpgrade,
    Path(pane_id): Path<PaneId>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| async move {
        let (mut ws_tx, mut ws_rx) = socket.split();
        let mut pty_rx = state.terminal.subscribe_pane(&pane_id).await.unwrap();
        let writer = state.terminal.get_writer(&pane_id).unwrap();

        loop {
            tokio::select! {
                // PTY output → browser
                Ok(bytes) = pty_rx.recv() => {
                    if ws_tx.send(Message::Binary(bytes)).await.is_err() { break; }
                }
                // Browser input → PTY
                Some(Ok(msg)) = ws_rx.next() => {
                    match msg {
                        Message::Binary(data) => {
                            writer.lock().await.write_all(&data).ok();
                        }
                        Message::Text(text) => {
                            if let Ok(ctrl) = serde_json::from_str::<ControlMsg>(&text) {
                                match ctrl {
                                    ControlMsg::Resize { rows, cols } => {
                                        state.terminal.resize_pane(&pane_id, PaneSize { rows, cols }).await.ok();
                                    }
                                }
                            }
                        }
                        Message::Close(_) => break,
                        _ => {}
                    }
                }
                else => break,
            }
        }
    })
}
```

---

## Frontend Implementation

### New Dependencies

```bash
cd frontend && pnpm add @xterm/xterm @xterm/addon-fit @xterm/addon-web-links
```

### Component: `TerminalPane.svelte`

```svelte
<script lang="ts">
  import { Terminal } from '@xterm/xterm';
  import { FitAddon } from '@xterm/addon-fit';
  import { WebLinksAddon } from '@xterm/addon-web-links';
  import { onMount } from 'svelte';
  import '@xterm/xterm/css/xterm.css';

  let { paneId, readOnly = false }: { paneId: string; readOnly?: boolean } = $props();
  let termEl: HTMLDivElement;

  onMount(() => {
    const term = new Terminal({
      cursorBlink: !readOnly,
      disableStdin: readOnly,
      fontSize: 13,
      fontFamily: 'JetBrains Mono, monospace',
      theme: {
        background: 'oklch(0.145 0 0)',           // matches --background token
        foreground: 'oklch(0.985 0 0)',           // matches --foreground token
      }
    });

    const fit = new FitAddon();
    term.loadAddon(fit);
    term.loadAddon(new WebLinksAddon());
    term.open(termEl);
    fit.fit();

    // Bidirectional WebSocket
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
    const base = import.meta.env.DEV ? 'localhost:3000' : location.host;
    const ws = new WebSocket(`${proto}//${base}/api/terminal/ws/${paneId}`);
    ws.binaryType = 'arraybuffer';

    ws.onmessage = (e) => term.write(new Uint8Array(e.data));
    if (!readOnly) {
      term.onData((data) => ws.send(new TextEncoder().encode(data)));
    }

    const ro = new ResizeObserver(() => {
      fit.fit();
      if (ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify({ type: 'resize', rows: term.rows, cols: term.cols }));
      }
    });
    ro.observe(termEl);

    return () => { ws.close(); term.dispose(); ro.disconnect(); };
  });
</script>

<div bind:this={termEl} class="h-full w-full bg-background"></div>
```

### Route: `/terminal/+page.svelte`

Full terminal workspace — shows all windows as tabs, panes in active window:

```svelte
<script lang="ts">
  import { PaneGroup, Pane, PaneResizer } from 'paneforge';
  import TerminalPane from '$lib/components/terminal/TerminalPane.svelte';
  import { getTerminalWindows, createPane, createWindow } from '$lib/api';

  let windows = $state([]);
  let activeWindow = $state(null);
  let panes = $state([]);
</script>

<!-- Tab bar: one tab per window -->
<div class="flex border-b border-border">
  {#each windows as win}
    <button onclick={() => activeWindow = win}
      class="px-3 py-1.5 text-xs" class:bg-muted={activeWindow?.id === win.id}>
      {win.name}
    </button>
  {/each}
  <button onclick={addWindow} class="px-2 text-muted-foreground">+</button>
</div>

<!-- Pane area: resizable splits via paneforge -->
<PaneGroup direction="horizontal" class="flex-1">
  {#each panes as pane, i}
    {#if i > 0}<PaneResizer class="w-1 bg-border hover:bg-primary" />{/if}
    <Pane>
      <TerminalPane paneId={pane.id} />
    </Pane>
  {/each}
</PaneGroup>
```

### Department Panel integration

Each department page gets a "Terminal" tab. Lazy — no PTY spawned until opened:

```svelte
<!-- In DepartmentPanel.svelte, alongside: actions, agents, skills, rules, ... -->
{:else if activeTab === 'terminal'}
  <TerminalPane paneId={deptPaneId} />
```

### Delegation Tree view (extends agent-orchestration frontend)

```svelte
<!-- DelegationTerminal.svelte — shows alongside DelegationTree.svelte -->
<script lang="ts">
  import { PaneGroup, Pane, PaneResizer } from 'paneforge';
  import TerminalPane from '$lib/components/terminal/TerminalPane.svelte';
  import DelegationTree from '$lib/components/orchestration/DelegationTree.svelte';

  let { runId }: { runId: string } = $props();
  let panes = $state([]);

  // Fetch panes for this delegation chain
  $effect(() => {
    fetch(`/api/terminal/runs/${runId}/panes`).then(r => r.json()).then(p => panes = p);
  });
</script>

<PaneGroup direction="horizontal">
  <Pane defaultSize={30}>
    <DelegationTree {runId} />
  </Pane>
  <PaneResizer />
  <Pane defaultSize={70}>
    <PaneGroup direction="vertical">
      {#each panes as pane, i}
        {#if i > 0}<PaneResizer class="h-1 bg-border" />{/if}
        <Pane>
          <div class="flex items-center gap-2 border-b border-border px-2 py-1 text-xs text-muted-foreground">
            <span class="font-medium">{pane.title}</span>
            <span class="text-xs">{pane.source.persona ?? pane.command}</span>
          </div>
          <TerminalPane paneId={pane.id} readOnly={pane.status !== 'Running'} />
        </Pane>
      {/each}
    </PaneGroup>
  </Pane>
</PaneGroup>
```

---

## Session ↔ Department ↔ Pane Lifecycle

### Auto-creation (lazy, per ADR-014)

1. User visits `/dept/code` and clicks "Terminal" tab
2. Frontend calls `POST /api/terminal/windows { name: "Code", dept_id: "code", source: "department" }`
3. Backend reads `DepartmentManifest.terminal.default_commands` from dept-code manifest
4. Creates window + pane with dept's default shell/command
5. PTY spawned, WebSocket opened, xterm.js renders live shell
6. Layout saved in `Session.metadata.terminal_layout`

### Agent delegation visibility

When `delegate_agent` (agent-orchestration.md Phase 2) fires:

1. Parent agent calls `delegate_agent("CodeWriter", "implement X")`
2. `delegate.rs` handler checks: does parent have a terminal window?
3. If yes → create pane in parent's window with `PaneSource::Delegation`
4. Sub-agent's `bash` tool calls execute in this visible pane
5. User watches in real-time; can read-only subscribe
6. On completion → pane header shows "✓ Completed" with cost estimate

### Flow/playbook execution (P8 durable)

1. `POST /api/playbooks/{id}/run` triggers flow execution
2. FlowEngine creates terminal window: `WindowSource::Playbook("content-from-code")`
3. As each node starts → pane created with `PaneSource::FlowNode`
4. Checkpoints (P8) saved after each node
5. If crash → resume from checkpoint → pane shows "Resumed from checkpoint"

### Session restore

On session switch (`rusvel session switch <id>`):
1. Load `Session.metadata.terminal_layout` (window/pane definitions)
2. Re-create windows + panes from saved state
3. New shells spawned in same cwd, scroll-back cleared
4. Agent/flow panes marked as "historical" (read-only with preserved output)

---

## Event Integration

Pane lifecycle events flow through existing EventBus (ADR-005 — kind is String):

```rust
// Terminal events — consumed by departments via events_consumed in manifest
"terminal.pane.created"    // payload: { pane_id, window_id, command, source }
"terminal.pane.exited"     // payload: { pane_id, exit_code, duration_ms }
"terminal.pane.resized"    // payload: { pane_id, rows, cols }
"terminal.window.created"  // payload: { window_id, session_id, name, source }
"terminal.window.closed"   // payload: { window_id }

// Cross-integration events
"terminal.agent.started"   // payload: { pane_id, run_id, persona }
"terminal.agent.completed" // payload: { pane_id, run_id, exit_code, cost_usd }
"terminal.flow.node.started"   // payload: { pane_id, execution_id, node_id }
"terminal.flow.node.completed" // payload: { pane_id, execution_id, node_id, status }
```

Departments can subscribe to terminal events in their manifest:

```rust
// In dept-code manifest
events_consumed: vec![
    "terminal.pane.exited".into(),  // track test execution results
]
```

**Event triggers (agent-orchestration.md Phase 4)** can react to terminal events:

```rust
EventTrigger {
    pattern: EventPattern::Filter {
        kind: "terminal.pane.exited".into(),
        field: "exit_code".into(),
        value: json!(1),  // non-zero exit
    },
    action: TriggerAction::StartAgent {
        persona: "Debugger".into(),
        prompt_template: "A command failed with exit code {{payload.exit_code}}. Investigate.".into(),
        department: None,
    },
}
```

---

## TUI Surface (`--tui` mode)

Extend `rusvel-tui` to show terminal panes alongside dashboard widgets:

- New layout mode: split screen — dashboard top, terminal bottom
- Or full-screen terminal mode (pressing `F11`)
- Reuses same `TerminalManager` — ratatui renders raw ANSI from the broadcast channel
- Key binding: `Ctrl+T` toggles terminal panel
- Department tabs map to terminal windows

---

## Built-in Agent Tools

Two new tools in `rusvel-builtin-tools`:

```rust
// terminal_open — open a visible terminal pane
ToolDefinition {
    name: "terminal_open",
    description: "Open a visible terminal pane in the current department's window. \
                  The user can watch the command execute in real-time.",
    parameters: json!({
        "type": "object",
        "properties": {
            "command": { "type": "string", "description": "Command to run" },
            "title": { "type": "string", "description": "Pane title" },
            "wait": { "type": "boolean", "description": "Wait for exit (default: true)" }
        },
        "required": ["command"]
    }),
    // Mark as searchable: true (P1 deferred tool loading)
    // Only discovered when agent needs terminal visibility
}

// terminal_watch — subscribe to a running pane's output
ToolDefinition {
    name: "terminal_watch",
    description: "Read the current output of a terminal pane.",
    parameters: json!({
        "type": "object",
        "properties": {
            "pane_id": { "type": "string" }
        },
        "required": ["pane_id"]
    }),
}
```

These are `searchable: true` (P1 deferred tool loading) — not injected into every prompt, discovered via `tool_search` when an agent needs terminal visibility.

---

## Crate Budget

| File | Est. Lines | Purpose |
|---|---|---|
| `rusvel-terminal/src/lib.rs` | ~250 | Domain types, PaneSource, WindowSource, Layout |
| `rusvel-terminal/src/manager.rs` | ~500 | TerminalManager, PaneHandle, PTY lifecycle, query methods |
| `rusvel-terminal/src/persistence.rs` | ~150 | Save/restore layouts to StoragePort |
| `rusvel-api/src/terminal_routes.rs` | ~300 | 8 HTTP + 1 WebSocket route |
| `rusvel-builtin-tools/src/terminal.rs` | ~100 | terminal_open + terminal_watch tools |
| `frontend TerminalPane.svelte` | ~80 | xterm.js + WebSocket component |
| `frontend terminal/+page.svelte` | ~120 | Tab bar + paneforge layout |
| `frontend DelegationTerminal.svelte` | ~80 | Delegation tree + terminal split |
| **Total** | **~1580** | Under 2000-line crate limit |

---

## Implementation Order

### Phase 1: Core PTY + Port (foundation)

1. Add `TerminalPort` trait to `rusvel-core/ports.rs`
2. Add `TerminalContribution` to `DepartmentManifest`
3. Create `rusvel-terminal` crate with domain types + `TerminalManager`
4. Wire `TerminalManager` into `main.rs` composition root
5. Add `terminal` to `RegistrationContext`

**Depends on:** Nothing. Can start now.
**Unblocks:** Everything below.

### Phase 2: Web Bridge

6. Add WebSocket route in `rusvel-api/src/terminal_routes.rs`
7. `pnpm add @xterm/xterm @xterm/addon-fit @xterm/addon-web-links`
8. Build `TerminalPane.svelte` component
9. Add `/terminal` route with paneforge layout
10. Add nav item in `+layout.svelte`

**Depends on:** Phase 1.
**Unblocks:** Department integration, agent visibility.

### Phase 3: Department Integration (ADR-014)

11. Add "terminal" tab to `UiContribution.tabs` in dept manifests
12. Lazy pane creation on first dept terminal tab open
13. Default commands from `DepartmentManifest.terminal`
14. Persist layouts in `Session.metadata`

**Depends on:** Phase 2 + ADR-014 migration (dept-* crates in progress).
**Coordinates with:** Agent B2-B4 department migrations.

### Phase 4: Agent Visibility (orchestration)

15. `PaneSource::Delegation` + `PaneSource::AgentTool` creation in `delegate_agent` handler
16. `terminal_open` + `terminal_watch` built-in tools (searchable: true)
17. `DelegationTerminal.svelte` component
18. `/api/terminal/runs/:run_id/panes` query endpoint

**Depends on:** Phase 2 + P9 delegate_agent (agent-orchestration.md Phase 2).
**Coordinates with:** Agent F6 (delegate_agent implementation).

### Phase 5: Flow/Playbook Visibility (durable execution)

19. `PaneSource::FlowNode` + `PaneSource::PlaybookStep` creation in FlowEngine executor
20. `/api/terminal/flows/:exec_id/panes` query endpoint
21. Playbook terminal dashboard view
22. Checkpoint resume shows in terminal UI

**Depends on:** Phase 2 + P8 durable execution + P9 playbook templates.
**Coordinates with:** Agent F9 (durable execution), flow-engine.

### Phase 6: CDP Browser Panes (future)

23. `PaneSource::Browser` for CDP-observed tabs
24. Read-only pane showing capture log
25. Unified tab bar: terminal + browser panes

**Depends on:** Phase 2 + cdp-browser-bridge.md Phase 1.

### Phase 7: TUI Surface

26. Extend `rusvel-tui` with terminal panel
27. Raw ANSI rendering from broadcast channel
28. Keyboard shortcut for terminal toggle

**Depends on:** Phase 1 (uses same TerminalManager).

---

## Cross-Reference: How Terminal Connects to Everything

| This Plan | Related Doc | Integration Point |
|-----------|-------------|-------------------|
| TerminalPort as platform service | ADR-014 (department-as-app) | Platform layer, RegistrationContext |
| TerminalContribution in manifest | ADR-014 manifest.rs | Departments declare terminal defaults |
| Delegation panes | Agent Orchestration Phase 2 | `delegate_agent` spawns visible panes |
| Flow node panes | Agent Orchestration Phase 3 | `invoke_flow` nodes get panes |
| Playbook dashboard | Inspiration (Priority 1) | Playbook steps visible as terminal panes |
| Event triggers on pane exit | Agent Orchestration Phase 4 | `terminal.pane.exited` triggers agents |
| Builder agent panes | Agent Workforce (B2-B4) | Each builder gets visible worktree pane |
| Browser tab panes | CDP Browser Bridge | Unified tab bar: terminal + browser |
| Deferred tool loading | P1 | `terminal_open/watch` are searchable tools |
| AG-UI events | P7 | Terminal events emit AG-UI lifecycle events |
| Durable execution | P8 | Flow panes show checkpoint/resume state |
| Self-correction | P5 | Critique agent sees terminal output for evaluation |
| Approval UI | P4 | PreToolUse approval shown in pane header |
| Smart routing | P12 | Terminal panes show model tier + cost per agent |
| Executive Brief | Inspiration (Priority 2) | Brief queries run as temporary panes |
| Roundtable UI | Inspiration (Priority 5) | Each persona gets a terminal pane in roundtable |

---

## Why Native > Wrapping Zellij

| | Native | Zellij Wrapper |
|---|---|---|
| Process count | 1 (rusvel binary) | 2+ (rusvel + zellij) |
| IPC overhead | Zero (in-process) | CLI subprocess calls |
| Auth | Same middleware | Separate token system |
| Events | Same EventBus | Bridge layer needed |
| Layouts | Session.metadata | Separate config files |
| Styling | Same design tokens | Zellij's own theme |
| Crash recovery | Single restart | Orphaned zellij sessions |
| Binary size | +portable-pty (~small) | +zellij binary (~15MB) |
| Observability | Unified event stream | Two log streams |
| Agent panes | Native (PaneSource enum) | Custom bridging code |
| Flow visibility | Native (FlowNode source) | Not possible |
| CDP integration | Unified tab bar | Two separate UIs |
| ADR-014 compat | TerminalPort in context | External process |

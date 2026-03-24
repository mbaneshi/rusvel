# Machine Awareness: RUSVEL + fs Integration Plan

> RUSVEL gains eyes through `fs`, then thinks with AI.
> One tool indexes the machine, the other reasons about what it finds.

**Date:** 2026-03-24
**Status:** Proposed
**Dependencies:** `fs` binary installed at `/Users/bm/.local/bin/fs`

---

## Problem

RUSVEL is a virtual agency that can reason, plan, and execute — but it's blind. It has no awareness of:

- What projects exist on the machine
- Their size, language, maturity, or activity
- System health (CPU, memory, disk)
- Duplicate files, wasted space, cleanup opportunities
- Which of ~60+ projects in `~/` are worth shipping

Meanwhile, `fs` (free-storage-app) already solves all of this — 20 commands, 12 HTTP endpoints, SQLite FTS5 indexing, 83 tests — but it has no AI brain to reason about what it finds.

**The integration:** `fs` becomes RUSVEL's sensory system, RUSVEL becomes `fs`'s intelligence layer.

---

## What `fs` Already Provides

| Capability | CLI Command | HTTP Endpoint | Output |
|---|---|---|---|
| File search | `fs find <pattern> [path]` | `GET /api/find` | File paths, metadata |
| Content search | `fs grep <pattern> [path]` | `GET /api/grep` | Matching lines |
| Disk usage | `fs usage [path]` | `GET /api/usage` | Size tree, treemap |
| Cleanup recs | `fs clean [path]` | `GET /api/cleanup/recommend` | Large/old/cache files |
| Duplicate detection | `fs dupes [path]` | `GET /api/cleanup/dupes` | BLAKE3-hashed groups |
| Code statistics | `fs stats [path]` | `GET /api/stats` | Lines, languages, files |
| System monitor | `fs top` | `GET /api/system` | CPU, memory, network |
| Process list | `fs ps` | `GET /api/ps` | Processes, tree view |
| Disk info | `fs disks` | `GET /api/disks` | Mount points, usage |
| Directory listing | `fs ls [path]` | `GET /api/ls` | Files with git status |
| File preview | `fs cat <file>` | `GET /api/cat` | Syntax-highlighted content |
| Index + FTS search | `fs index`, `fs search` | — | SQLite FTS5 results |

All commands support `--json` flag for structured output.

**Database:** `~/.local/share/freestorage/index.db` (SQLite WAL, FTS5)

**Server:** `fs serve --port 3141` (Axum, localhost only)

---

## Architecture

### New Components

```
crates/
├── rusvel-core/
│   └── src/ports.rs            # Add MachinePort trait
├── rusvel-machine/             # NEW: fs adapter crate
│   ├── src/lib.rs              # MachinePort implementation
│   ├── src/cli.rs              # fs CLI wrapper (--json parsing)
│   ├── src/http.rs             # fs HTTP client (port 3141)
│   └── src/project_scanner.rs  # Project discovery logic
├── code-engine/
│   └── (enhanced)              # Multi-project awareness via MachinePort
├── rusvel-api/
│   └── src/machine.rs          # NEW: Machine API endpoints
└── rusvel-app/
    └── src/main.rs             # Wire MachinePort into composition root
```

### Port Trait

```rust
// In rusvel-core/src/ports.rs

#[async_trait]
pub trait MachinePort: Send + Sync {
    /// Discover all projects under a root path
    async fn scan_projects(&self, root: &Path) -> Result<Vec<ProjectInfo>>;

    /// Get system stats (CPU, memory, disk)
    async fn system_stats(&self) -> Result<SystemStats>;

    /// Get cleanup recommendations for a path
    async fn cleanup_recommendations(&self, path: &Path) -> Result<CleanupReport>;

    /// Find duplicate files under a path
    async fn find_duplicates(&self, path: &Path) -> Result<Vec<DuplicateGroup>>;

    /// Get code statistics for a project
    async fn code_stats(&self, path: &Path) -> Result<CodeStats>;

    /// Search indexed files by query
    async fn search_files(&self, query: &str) -> Result<Vec<FileEntry>>;

    /// Get disk usage breakdown for a path
    async fn disk_usage(&self, path: &Path) -> Result<DiskUsage>;

    /// Find files matching a pattern
    async fn find_files(&self, pattern: &str, path: &Path) -> Result<Vec<FileEntry>>;

    /// Search file contents (grep)
    async fn grep(&self, pattern: &str, path: &Path) -> Result<Vec<GrepMatch>>;
}
```

### Domain Types

```rust
// In rusvel-core/src/types.rs

pub struct ProjectInfo {
    pub path: PathBuf,
    pub name: String,
    pub project_type: ProjectType,       // Rust, Node, Python, Go, etc.
    pub size_bytes: u64,
    pub file_count: u64,
    pub last_modified: i64,              // Unix timestamp
    pub git_status: Option<GitStatus>,   // Clean, dirty, ahead/behind
    pub languages: Vec<LanguageStats>,   // From fs stats
    pub has_tests: bool,
    pub has_ci: bool,
    pub metadata: serde_json::Value,
}

pub enum ProjectType {
    Rust,       // Cargo.toml
    Node,       // package.json
    Python,     // pyproject.toml / setup.py / requirements.txt
    Go,         // go.mod
    Swift,      // Package.swift / *.xcodeproj
    Unknown,
}

pub struct GitStatus {
    pub branch: String,
    pub is_dirty: bool,
    pub unpushed_commits: u32,
    pub last_commit_date: i64,
    pub remote_url: Option<String>,
}

pub struct CleanupReport {
    pub duplicates: Vec<DuplicateGroup>,
    pub large_files: Vec<FileEntry>,
    pub old_files: Vec<FileEntry>,
    pub cache_dirs: Vec<DirEntry>,
    pub total_recoverable_bytes: u64,
}

pub struct SystemStats {
    pub cpu_usage_percent: f64,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub disk_used_bytes: u64,
    pub disk_total_bytes: u64,
    pub process_count: u32,
}

pub struct CodeStats {
    pub total_files: u64,
    pub total_lines: u64,
    pub code_lines: u64,
    pub comment_lines: u64,
    pub blank_lines: u64,
    pub languages: Vec<LanguageStats>,
}

pub struct LanguageStats {
    pub language: String,
    pub files: u64,
    pub lines: u64,
    pub code: u64,
    pub comments: u64,
    pub blanks: u64,
}

pub struct DuplicateGroup {
    pub hash: String,
    pub file_size: u64,
    pub file_count: u32,
    pub wasted_bytes: u64,
    pub members: Vec<PathBuf>,
}
```

---

## Implementation Phases

### Phase 1: MachinePort + fs Adapter (Foundation)

**Goal:** RUSVEL can call `fs` and get structured data back.

1. **Add `MachinePort` trait** to `rusvel-core/src/ports.rs`
2. **Add domain types** (`ProjectInfo`, `SystemStats`, etc.) to `rusvel-core/src/types.rs`
3. **Create `rusvel-machine` crate** with two backends:
   - `CliBackend` — runs `fs --json <command>`, parses JSON stdout
   - `HttpBackend` — calls `http://127.0.0.1:3141/api/*` (when `fs serve` is running)
4. **Auto-detect backend** — try HTTP first, fall back to CLI
5. **Wire into `rusvel-app/src/main.rs`** — construct and inject `MachinePort`

**Tests:** Mock `fs` JSON output, verify parsing of all 12 response types.

**Estimate:** ~600 lines of Rust across 3 files.

### Phase 2: Project Scanner (Discovery)

**Goal:** RUSVEL knows every project on the machine.

1. **Project detection heuristics:**
   - Walk top-level directories under `~/`
   - Detect project type by marker files:
     - `Cargo.toml` → Rust
     - `package.json` → Node
     - `pyproject.toml` / `setup.py` → Python
     - `go.mod` → Go
     - `*.xcodeproj` / `Package.swift` → Swift
   - Skip: `Library`, `Applications`, `.Trash`, dotfiles
2. **Enrich each project** with data from `fs`:
   - `fs stats --json <path>` → language breakdown, line counts
   - `fs usage --json <path>` → size on disk
   - `git log --oneline -1` → last commit date
   - `git status --porcelain` → dirty/clean
   - `git remote -v` → remote URL
3. **Store in RUSVEL's SQLite** via `StoragePort` as `machine.project.<name>` objects
4. **FTS index** project names, paths, languages for natural language search
5. **Periodic rescan** via job queue — detect new/removed projects

**Output:** `GET /api/machine/projects` returns all known projects with metadata.

### Phase 3: Machine Dashboard (Visibility)

**Goal:** See everything from the frontend.

#### API Endpoints (in `rusvel-api/src/machine.rs`)

```
GET  /api/machine/projects                # All indexed projects
GET  /api/machine/projects/:name          # Single project details
GET  /api/machine/projects/:name/stats    # Code stats for a project
GET  /api/machine/system                  # Live system stats
GET  /api/machine/cleanup                 # Cleanup recommendations
GET  /api/machine/cleanup/dupes           # Duplicate files
GET  /api/machine/disk                    # Disk usage overview
POST /api/machine/scan                    # Trigger full rescan
POST /api/machine/cleanup/execute         # Execute cleanup (with approval gate)
```

#### Frontend Pages

- `/machine` — Overview: system stats, disk usage, project count
- `/machine/projects` — Project list: sortable by size, activity, language
- `/machine/projects/:name` — Project detail: stats, git status, files, dependencies
- `/machine/cleanup` — Cleanup recommendations with one-click actions
- `/machine/system` — Live system monitor (CPU, memory, processes)

### Phase 4: AI Reasoning (The Brain)

**Goal:** RUSVEL thinks about what it sees.

1. **God Agent machine context** — inject project registry summary into God agent's system prompt:
   ```
   You have access to 47 projects on this machine.
   Top by size: all-in-one-rusvel (22k lines), free-storage-app (9.8 MB), ...
   Recently active: codeilus (2 days ago), contentforge (5 days ago), ...
   Stale (>30 days): hackaton, ton-hackaton, robotic, ...
   ```

2. **Natural language queries** via chat:
   - "What are my biggest projects?" → queries MachinePort, formats response
   - "Which projects have uncommitted changes?" → scans git status
   - "How much space can I free up?" → cleanup recommendations
   - "Show me all Rust projects" → filters by ProjectType
   - "What's using the most disk?" → `fs usage --json ~`

3. **Machine-aware skills** — auto-generated agent skills:
   - `scan_projects` — trigger project index refresh
   - `system_health` — get current system stats
   - `cleanup_suggest` — get cleanup recommendations
   - `project_summary` — detailed analysis of a specific project

4. **Cross-project analysis** via Harvest Engine:
   - Score projects by shipping readiness (tests, CI, docs, git activity)
   - Estimate effort-to-launch based on code complexity + completeness
   - Match against market opportunities (harvest-engine scoring)
   - Generate "top 5 projects to ship this month" recommendations

### Phase 5: Money Analysis (The Strategy)

**Goal:** RUSVEL tells you what to ship and why.

1. **Project Maturity Score** (0-100):
   - Has tests? (+20)
   - Has CI/CD? (+15)
   - Has README/docs? (+10)
   - Active in last 30 days? (+15)
   - Has frontend? (+10)
   - Binary/package published? (+15)
   - Has users/stars? (+15)

2. **Market Fit Score** (via Harvest Engine):
   - Does a competing product exist? (validates market)
   - What's the pricing landscape?
   - Is there search volume / demand signal?

3. **Effort Estimate**:
   - Lines of code remaining (from TODOs, stubs, unimplemented)
   - Test coverage gaps
   - Missing features vs. MVP checklist

4. **Money Potential Ranking**:
   ```
   Priority = (Maturity × 0.3) + (Market_Fit × 0.4) + (1/Effort × 0.3)
   ```
   Output: Ranked list of projects with actionable next steps.

5. **Weekly digest** (via God agent + job queue):
   - "This week: focus on Codeilus (maturity 72, market fit 85, ~3 days to MVP)"
   - "Cleanup opportunity: 12 GB recoverable across 4 stale projects"
   - "New: contentforge had 3 commits, consider publishing beta"

---

## Integration with Existing RUSVEL Architecture

### Hexagonal Compliance

- `MachinePort` is a **port trait** in `rusvel-core` — engines depend on the trait, not `fs`
- `rusvel-machine` is an **adapter** — implements `MachinePort` by calling `fs`
- Engines (code-engine, infra-engine, harvest-engine) consume `MachinePort` via injection
- If `fs` is replaced later, only the adapter changes

### Department Mapping

- **Code Department** — enhanced with multi-project analysis via MachinePort
- **Infra Department** — real system monitoring via MachinePort.system_stats()
- **Harvest Department** — project scoring uses MachinePort.scan_projects() for input
- **God Agent** — machine context injected into system prompt for holistic reasoning

### Event Flow

```
fs index ~/                          # fs indexes the filesystem
  → rusvel-machine polls/watches     # adapter detects changes
    → "machine.projects.updated"     # event emitted
      → God agent notified           # "3 new projects detected"
      → Harvest engine re-scores     # money rankings updated
      → Frontend dashboard refreshes # live project list
```

### ADR Compliance

- **ADR-009:** Engines use AgentPort, not LlmPort directly — machine analysis goes through agent
- **ADR-007:** All ProjectInfo has `metadata: serde_json::Value` for extensibility
- **ADR-005:** Event kinds are strings: `machine.scan.completed`, `machine.project.discovered`
- **ADR-003:** Periodic scans use the single job queue
- **ADR-008:** Cleanup execution requires human approval gate

---

## File-by-File Changes

### New Files

| File | Lines (est.) | Purpose |
|---|---|---|
| `crates/rusvel-core/src/machine_types.rs` | ~120 | Domain types: ProjectInfo, SystemStats, etc. |
| `crates/rusvel-machine/Cargo.toml` | ~25 | New crate manifest |
| `crates/rusvel-machine/src/lib.rs` | ~80 | MachinePort implementation + backend selection |
| `crates/rusvel-machine/src/cli.rs` | ~200 | CLI backend: runs `fs --json`, parses output |
| `crates/rusvel-machine/src/http.rs` | ~150 | HTTP backend: calls `fs serve` API |
| `crates/rusvel-machine/src/scanner.rs` | ~200 | Project discovery + enrichment logic |
| `crates/rusvel-api/src/machine.rs` | ~250 | 9 API endpoints for machine data |
| `frontend/src/routes/machine/` | ~400 | Dashboard pages (overview, projects, cleanup, system) |

### Modified Files

| File | Change |
|---|---|
| `crates/rusvel-core/src/ports.rs` | Add `MachinePort` trait |
| `crates/rusvel-core/src/lib.rs` | Export machine_types module |
| `crates/rusvel-app/src/main.rs` | Construct + wire `rusvel-machine` adapter |
| `crates/rusvel-app/Cargo.toml` | Add `rusvel-machine` dependency |
| `crates/rusvel-api/src/routes.rs` | Mount `/api/machine/*` routes |
| `crates/rusvel-api/Cargo.toml` | Add machine module |
| `crates/code-engine/src/lib.rs` | Accept `MachinePort` for multi-project analysis |
| `crates/infra-engine/src/monitor.rs` | Use `MachinePort` for real system checks |
| `Cargo.toml` | Add `rusvel-machine` to workspace members |

---

## What This Unlocks

Once wired, you can:

```
# Chat with God agent
> "Scan my machine and tell me what I have"
> "Which of my projects is closest to making money?"
> "Clean up stale projects and free disk space"
> "Compare codeilus vs contentforge — which ships faster?"
> "What's my total lines of code across all projects?"
> "Show me projects with uncommitted changes"

# CLI
rusvel code scan-machine          # Index all projects
rusvel code projects              # List all known projects
rusvel infra system               # Live system stats
rusvel harvest analyze-portfolio  # Money potential ranking

# API
GET /api/machine/projects?sort=money_potential
GET /api/machine/system
POST /api/machine/scan
```

---

## Priority Order

1. **Phase 1** — MachinePort + fs adapter (enables everything else)
2. **Phase 2** — Project scanner (the most valuable single feature)
3. **Phase 4** — AI reasoning (God agent + chat, immediate user value)
4. **Phase 3** — Frontend dashboard (visibility)
5. **Phase 5** — Money analysis (strategic, builds on all above)

Phase 1+2 are the foundation. Phase 4 delivers the most value fastest (chat "what should I ship?"). Phase 3 and 5 are polish and strategy.

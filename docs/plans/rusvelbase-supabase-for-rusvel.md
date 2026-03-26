# RusvelBase — Supabase for RUSVEL

> **STATUS: IMPLEMENTED (Phases 1-3).** `rusvel-schema` crate for introspection, API routes at `/api/db/*` (tables, schema, rows, SQL runner), and Database Browser UI in the frontend. See `rusvel-schema` crate and `db_routes` API module.

> A Supabase-equivalent database platform adapted for the single-binary Rust + SQLite + SvelteKit stack.
> Created: 2026-03-24

## Motivation

Supabase provides PostgreSQL-backed infrastructure (auto-generated REST API, table editor, SQL editor, realtime subscriptions, auth, storage) via a polished dashboard. RUSVEL already has SQLite WAL, Axum API, embedded SvelteKit frontend, and SSE streaming — the foundation is there. This plan adds the missing pieces to give RUSVEL a full self-hosted database platform without any external dependencies.

There is no 1:1 "Supabase-for-SQLite" in the ecosystem — instead there's a toolbox of focused projects. RUSVEL's advantage is that we can integrate these concerns into a single binary rather than stitching separate services together.

---

## SQLite Ecosystem Landscape

Before building, it's important to understand what already exists and what we can learn from (or optionally integrate with) rather than reinventing.

### Higher-Level Platforms Around SQLite

| Project | What It Does | Relevance to RUSVEL |
|---|---|---|
| **Datasette** | Web UI + JSON/CSV APIs over SQLite, plugin ecosystem (auth, FTS, Litestream) | Closest to what we're building — our Table Editor + SQL Editor covers this. Learn from their filtering/faceting API design |
| **rqlite** | Distributed SQLite via Raft consensus + HTTP API | Not needed now (single binary), but the HTTP API design for SQLite CRUD is a good reference |
| **BedrockDB** | Distributed SQLite speaking MySQL protocol | Not relevant — we don't need MySQL compatibility |

### Replication & Durability

| Project | What It Does | When RUSVEL Needs It |
|---|---|---|
| **Litestream** | Continuous SQLite replication to S3 for point-in-time recovery | **Phase 5+** — when deploying to production. Can run as sidecar to the RUSVEL binary. Zero code changes needed |
| **LiteFS** (Fly.io) | FUSE-based distributed SQLite, single writer + read replicas | **Phase 5+** — if we ever need multi-region edge deployment |

### SQLite Extensions

| Extension | What It Does | RUSVEL Status |
|---|---|---|
| **JSON1** | Native JSON operations | Already used — `rusvel-db` stores JSON blobs extensively |
| **FTS5** | Full-text search | Already used — `rusvel-memory` uses FTS5 for session-namespaced search |
| **Sqlean** | Extended math, regex, Unicode, UUIDs, fuzzy match, CSV virtual tables | **Consider for Phase 2** — could enrich SQL Editor capabilities. Single shared library to load |
| **sqlite-vec** | Vector similarity search in SQLite | Already have embedding search via `rusvel-memory` + fastembed. Could replace/complement |
| **ICU** | Better Unicode collation and case-folding | Low priority for now |

### Mapping: Supabase Feature → SQLite Ecosystem → RUSVEL Approach

| Need | Supabase (Postgres) | SQLite Ecosystem | RUSVEL Approach |
|---|---|---|---|
| REST/JSON API + admin UI | Supabase Studio + PostgREST | Datasette | **Build it** — Phase 2 auto-generated API + Phase 3 dashboard |
| Managed DB + HA | Supabase Postgres | rqlite, LiteFS | **Skip** — single binary, single SQLite file. Add Litestream later for backups |
| Point-in-time backup | Supabase PITR | Litestream, LiteFS S3 | **Phase 5** — Litestream as optional sidecar, or embed backup logic |
| Realtime / change feeds | Supabase Realtime (Elixir + WAL) | DIY via `sqlite3_update_hook` + triggers | **Build it** — Phase 4a, SSE broadcast from update hooks |
| Auth + row-level policies | Supabase Auth (GoTrue) + Postgres RLS | App-side auth; no standard solution | **Build it** — Rust middleware injects WHERE clauses based on JWT claims |
| File storage | Supabase Storage (S3 + policies) | Local filesystem + metadata in SQLite | **Build it** — Phase 4b, local storage with SQLite metadata |
| Vector search | pgvector | sqlite-vec, sqliteai-vector | **Already have** — fastembed + custom index in `rusvel-memory` |
| Full-text search | Postgres tsvector | FTS5 | **Already have** — FTS5 in `rusvel-memory` |
| Serverless functions | Supabase Edge Functions (Deno) | N/A | **Already have** — Agents + Skills + Hooks (more powerful) |
| Extensions ecosystem | pg extensions | Sqlean bundle | **Consider** — load Sqlean for richer SQL in the editor |

### Key Insight

> The SQLite ecosystem is a **toolbox of focused projects** — RUSVEL's value-add is **integrating these into one coherent, embedded experience** inside the single binary. We're not competing with Datasette or rqlite; we're absorbing the best ideas from each into our stack.

---

## What Supabase Offers vs RUSVEL Status

| Supabase Feature | Priority | RUSVEL Today |
|---|---|---|
| Auto-generated REST API (PostgREST) | **HIGH** | Manual CRUD routes per entity |
| Table Editor UI | **HIGH** | No DB browser |
| SQL Editor | **HIGH** | No query UI |
| Realtime subscriptions | **MEDIUM** | SSE for chat only |
| Auth + RLS | **MEDIUM** | `rusvel-auth` basic credential store |
| File Storage | **MEDIUM** | No file storage |
| Backup/Replication | **LOW** | No backup strategy yet (Litestream candidate) |
| Schema Viewer | **LOW** | No schema UI |
| Edge Functions | **SKIP** | Agents/Skills/Hooks already more powerful |
| Migrations UI | **SKIP** | Handled in code via `rusvel-db/migrations.rs` |

---

## Phase 1: Schema Introspection Engine

**New crate:** `rusvel-schema` (~300 lines)

### Purpose

Read SQLite metadata and produce typed schema info usable by both the API and frontend.

### Core Types

```rust
pub struct TableInfo {
    pub name: String,
    pub columns: Vec<ColumnInfo>,
    pub indexes: Vec<IndexInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
    pub row_count: u64,
}

pub struct ColumnInfo {
    pub name: String,
    pub col_type: String,       // TEXT, INTEGER, REAL, BLOB, NULL
    pub nullable: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

pub struct ForeignKeyInfo {
    pub from_column: String,
    pub to_table: String,
    pub to_column: String,
}
```

### Implementation

Use SQLite PRAGMAs:

```sql
-- List all tables
SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';

-- Column info per table
PRAGMA table_info('{table}');

-- Index list
PRAGMA index_list('{table}');

-- Index columns
PRAGMA index_info('{index}');

-- Foreign keys
PRAGMA foreign_key_list('{table}');

-- Row count
SELECT count(*) FROM '{table}';
```

### Public API

```rust
impl SchemaIntrospector {
    pub fn new(db: &Connection) -> Self;
    pub fn list_tables(&self) -> Result<Vec<TableInfo>>;
    pub fn get_table(&self, name: &str) -> Result<TableInfo>;
    pub fn validate_table_name(&self, name: &str) -> bool;
    pub fn validate_column_name(&self, table: &str, col: &str) -> bool;
}
```

**Key:** All table/column names are validated against the introspected schema — never user-supplied strings in SQL.

---

## Phase 2: Auto-Generated REST API

**Location:** New module in `rusvel-api` (~500 lines)

### New Routes

```
GET    /api/db/tables                        → list all tables with row counts
GET    /api/db/tables/{table}/schema         → columns, indexes, foreign keys
GET    /api/db/tables/{table}/rows           → paginated, filtered, sorted rows
POST   /api/db/tables/{table}/rows           → insert one or more rows
PATCH  /api/db/tables/{table}/rows/{rowid}   → update row by rowid
DELETE /api/db/tables/{table}/rows/{rowid}   → delete row by rowid
POST   /api/db/sql                           → execute arbitrary SQL query
```

### Filtering Syntax (PostgREST-inspired)

Query parameters map to WHERE clauses:

```
?column=eq.value          → column = 'value'
?column=neq.value         → column != 'value'
?column=gt.5              → column > 5
?column=gte.5             → column >= 5
?column=lt.10             → column < 10
?column=lte.10            → column <= 10
?column=like.%pattern%    → column LIKE '%pattern%'
?column=in.(a,b,c)        → column IN ('a','b','c')
?column=is.null           → column IS NULL
?column=is.notnull        → column IS NOT NULL
```

### Pagination & Sorting

```
?limit=50                 → LIMIT 50 (default: 100, max: 1000)
?offset=100               → OFFSET 100
?order=created_at.desc    → ORDER BY created_at DESC
?order=name.asc,id.desc   → ORDER BY name ASC, id DESC
```

### Column Selection

```
?select=id,name,created_at   → SELECT id, name, created_at
?select=*                     → SELECT * (default)
```

### SQL Endpoint

```json
// POST /api/db/sql
{
  "query": "SELECT kind, count(*) as cnt FROM events GROUP BY kind ORDER BY cnt DESC",
  "params": []
}

// Response
{
  "columns": [
    { "name": "kind", "type": "TEXT" },
    { "name": "cnt", "type": "INTEGER" }
  ],
  "rows": [
    ["chat.message", 42],
    ["goal.created", 15]
  ],
  "row_count": 2,
  "duration_ms": 3
}
```

### Safety

- Table and column names validated against `SchemaIntrospector` (prevents SQL injection)
- All values passed as parameterized query bindings
- `POST /api/db/sql` has optional read-only mode (`PRAGMA query_only = ON` per connection)
- Dangerous statements (`DROP`, `ALTER`, `DELETE FROM` without WHERE) require explicit `"confirm": true` flag

---

## Phase 3: Dashboard UI (SvelteKit Frontend)

### 3a. Table Editor (`/database/tables`) — ~800 lines Svelte

**Left Sidebar:**
- List of all tables with row count badges
- Click to select table
- Search/filter tables

**Main Area — Spreadsheet View:**
- Column headers with type badge (TEXT, INT, etc.) and PK/FK indicators
- Rows rendered in a scrollable table
- **Inline editing:** click a cell to edit, blur or Enter to save (PATCH to API)
- **Add row:** button opens a form generated from column schema
- **Delete row:** row action with confirmation dialog
- **Bulk select:** checkbox column for multi-delete

**Filter Bar:**
- Column dropdown + operator dropdown (eq, like, gt, lt, etc.) + value input
- Multiple filters (AND logic)
- Clear filters button

**Pagination:**
- Page size selector (25, 50, 100)
- Previous/Next page buttons
- Total row count display

### 3b. SQL Editor (`/database/sql`) — ~500 lines Svelte

**Editor Area:**
- CodeMirror 6 with SQL syntax highlighting
- Ctrl+Enter to execute
- Multi-statement support (split on `;`)
- Tab to indent

**Results Panel:**
- Table view of results
- Column headers with types
- Row count and execution time
- Error display with line highlighting

**Sidebar:**
- Query history (saved to localStorage, last 50)
- Click to reload a previous query
- Pre-built templates dropdown:
  - "Show all tables"
  - "Table row counts"
  - "Recent events"
  - "Object counts by kind"

### 3c. Schema Viewer (`/database/schema`) — ~300 lines Svelte

- Expandable table cards showing columns
- Column details: name, type, nullable, default, PK/FK
- Foreign key links (click to navigate to referenced table)
- Index list per table
- "Copy CREATE TABLE" button

### Navigation

Add a "Database" section to the main sidebar/nav:

```
Database
├── Tables      → /database/tables
├── SQL Editor  → /database/sql
└── Schema      → /database/schema
```

---

## Phase 4: Realtime + Storage

### 4a. Realtime via SQLite Update Hooks (~200 lines Rust, ~100 lines Svelte)

**Backend:**

```rust
// In rusvel-db or rusvel-schema
// rusqlite exposes sqlite3_update_hook
db.update_hook(Some(|action: Action, _db: &str, table: &str, rowid: i64| {
    let change = DbChange {
        action: match action {
            Action::SQLITE_INSERT => "INSERT",
            Action::SQLITE_UPDATE => "UPDATE",
            Action::SQLITE_DELETE => "DELETE",
        },
        table: table.to_string(),
        rowid,
        timestamp: Utc::now(),
    };
    // Send to broadcast channel
    tx.send(change).ok();
}));
```

**API Route:**

```
GET /api/db/realtime?tables=events,objects   → SSE stream of DB changes
```

Uses `tokio::sync::broadcast` channel. Each SSE connection subscribes and filters by requested tables.

**Frontend:**

```javascript
// In table editor component
const source = new EventSource('/api/db/realtime?tables=' + currentTable);
source.onmessage = (event) => {
    const change = JSON.parse(event.data);
    if (change.table === currentTable) {
        refreshRows(); // or apply delta
    }
};
```

The table editor auto-refreshes when changes arrive from other sources (CLI, API, agents).

### 4b. File Storage (~400 lines Rust, ~400 lines Svelte)

**New table:**

```sql
CREATE TABLE storage_objects (
    id TEXT PRIMARY KEY,
    bucket TEXT NOT NULL,
    path TEXT NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT,
    size INTEGER NOT NULL,
    metadata TEXT DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(bucket, path)
);

CREATE TABLE storage_buckets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    public INTEGER DEFAULT 0,
    max_file_size INTEGER DEFAULT 52428800,  -- 50MB
    allowed_mime_types TEXT DEFAULT '[]',
    created_at TEXT NOT NULL
);
```

**File location:** `~/.rusvel/storage/{bucket}/{path}`

**API Routes:**

```
GET    /api/storage/buckets                    → list buckets
POST   /api/storage/buckets                    → create bucket
DELETE /api/storage/buckets/{id}               → delete bucket

POST   /api/storage/{bucket}/upload            → multipart file upload
GET    /api/storage/{bucket}/{path}            → download file
DELETE /api/storage/{bucket}/{path}            → delete file
GET    /api/storage/{bucket}?prefix=docs/      → list files in bucket

GET    /api/storage/{bucket}/{path}?transform=w200,h200  → resized image (optional)
```

**Frontend — Storage Browser (`/database/storage`):**
- Bucket list with create/delete
- File browser with folder-like navigation
- Drag-and-drop upload
- File preview (images, text, JSON)
- Copy public URL button
- File metadata display (size, mime, created)

---

## Build Order

| Step | Scope | Effort | Depends On |
|---|---|---|---|
| **1** | Phase 1: `rusvel-schema` crate | ~300 lines Rust | Nothing |
| **2** | Phase 2: Dynamic API routes | ~500 lines Rust | Step 1 |
| **3** | Phase 3a: Table Editor UI | ~800 lines Svelte | Step 2 |
| **4** | Phase 3b: SQL Editor UI | ~500 lines Svelte | Step 2 |
| **5** | Phase 4a: Realtime (update hooks + SSE) | ~300 lines | Step 2 |
| **6** | Phase 3c: Schema Viewer UI | ~300 lines Svelte | Step 1 |
| **7** | Phase 4b: File Storage | ~800 lines | Nothing |

**Steps 1+2** are the foundation — everything else builds on them.
**Steps 3a+3b** deliver the most user-visible value.
**Steps 5+6+7** are enhancements to build as needed.

---

## Architecture Fit

This integrates cleanly into RUSVEL's hexagonal architecture:

```
rusvel-schema (new)     → Pure introspection, no framework deps
rusvel-db               → Existing store, add update_hook + storage tables
rusvel-api              → New /api/db/* routes, new /api/storage/* routes
frontend                → New /database/* pages
rusvel-app              → Wire SchemaIntrospector into AppState
```

**No new port traits needed** — `rusvel-schema` operates directly on the `rusqlite::Connection` (same as `rusvel-db`). The API layer composes schema introspection with existing DB access.

---

## What This Is NOT

- **Not a full Postgres replacement** — SQLite has no stored procedures, no LISTEN/NOTIFY, no native RLS. We compensate in Rust.
- **Not a hosted SaaS** — Every node is the same binary. No control plane vs worker distinction.
- **Not single-node** — The single binary is for easy installation. The architecture is designed for N machines forming a mesh.

---

## Phase 5: Distribution & P2P Mesh

RUSVEL's single binary is designed to be installed on multiple machines that coordinate as a distributed agency. Same binary, `N` machines, forming a mesh.

### The Distributed RUSVEL Model

```
Machine A (home desktop)     Machine B (VPS)           Machine C (laptop)
┌─────────────────────┐     ┌─────────────────────┐   ┌─────────────────────┐
│  rusvel binary       │     │  rusvel binary       │   │  rusvel binary       │
│  ┌───────────────┐  │     │  ┌───────────────┐  │   │  ┌───────────────┐  │
│  │ SQLite (local) │  │     │  │ SQLite (local) │  │   │  │ SQLite (local) │  │
│  └───────┬───────┘  │     │  └───────┬───────┘  │   │  └───────┬───────┘  │
│          │          │     │          │          │   │          │          │
│  ┌───────┴───────┐  │     │  ┌───────┴───────┐  │   │  ┌───────┴───────┐  │
│  │  Replication   │◄─┼──P2P──┼►│  Replication   │◄─┼─P2P─┼►│  Replication   │  │
│  └───────────────┘  │  mesh  │  └───────────────┘  │ mesh │  └───────────────┘  │
│                     │     │                     │   │                     │
│  ┌───────────────┐  │     │  ┌───────────────┐  │   │  ┌───────────────┐  │
│  │ Network Layer  │  │     │  │ Network Layer  │  │   │  │ Network Layer  │  │
│  │ (rathole/ts)  │  │     │  │ (rathole/ts)  │  │   │  │ (rathole/ts)  │  │
│  └───────────────┘  │     │  └───────────────┘  │   │  └───────────────┘  │
└─────────────────────┘     └─────────────────────┘   └─────────────────────┘
```

### 5a. Network Layer — P2P Connectivity

How the nodes find and talk to each other. Three options, not mutually exclusive:

#### Option 1: Rathole (NAT Traversal Tunneling)

Reference: `good-repo/rathole` — Rust reverse proxy for NAT traversal.

- Lightweight tunnel between machines behind different NATs/firewalls
- One machine with a public IP acts as relay server
- Other machines connect out to it (no port forwarding needed)
- Already in Rust, can study and potentially embed the tunnel logic
- **Best for:** connecting home machines to a VPS relay

```
Home (behind NAT) ──rathole tunnel──► VPS (public IP) ◄──rathole tunnel── Laptop (behind NAT)
```

#### Option 2: Tailscale / WireGuard Mesh

- Encrypted mesh VPN — every machine gets a stable IP on a private network
- Zero-config NAT traversal via DERP relays + hole punching
- Each RUSVEL node just binds to its Tailscale IP
- **Best for:** easiest setup, zero networking code to write
- **Tradeoff:** external dependency (Tailscale daemon or embedded WireGuard)

Reference: `good-repo/mullvadvpn-app` — Rust WireGuard implementation for studying embedded VPN.

#### Option 3: Custom P2P (libp2p or QUIC)

- Full P2P with peer discovery, DHT, NAT traversal built in
- `libp2p` (Rust) or raw QUIC (`quinn` crate) for transport
- Most complex but most flexible — no external services
- **Best for:** long-term if we want zero external dependencies

#### Recommendation

Start with **Tailscale** for immediate multi-machine connectivity (zero code), study **rathole** for embedding tunnel logic into the binary later, keep **libp2p** as the endgame for full P2P autonomy.

### 5b. Database Replication — Distributed SQLite

How the SQLite databases stay in sync across nodes. The key decision:

#### Option A: libSQL (Turso's Fork) — Recommended

| Feature | rusqlite (current) | libSQL |
|---|---|---|
| Language | C library + Rust bindings | Rust-native |
| Async API | `spawn_blocking` workaround | Native async |
| Write concurrency | Single writer (WAL) | Improved concurrency |
| Vector search | Bolt-on (fastembed) | Built-in |
| Replication | None | Built-in (embedded replica mode) |
| Compatibility | 100% SQLite | 99%+ SQLite compatible |

libSQL's embedded replica mode:

```rust
// Each node can be a primary or replica
// Replicas sync automatically, can serve reads locally
let db = libsql::Builder::new_replica("local.db", "https://primary-node:8080")
    .build()
    .await?;

// Or multi-primary with conflict resolution
let db = libsql::Builder::new_synced("local.db", peers)
    .build()
    .await?;
```

**Migration path:** `rusvel-db` already isolates all database access behind port traits. Swapping `rusqlite` → `libsql` is contained to one crate.

#### Option B: Raft-Based Replication (rqlite-style)

- Implement Raft consensus in Rust (`openraft` crate)
- All writes go through the Raft leader
- Followers replicate the write-ahead log
- Strong consistency but single-writer bottleneck
- **More work** than libSQL but gives full control

#### Option C: CRDTs for Eventual Consistency

- Each node writes locally, syncs via CRDTs
- No leader election, no single-writer bottleneck
- Works offline, merges when reconnected
- Best for: collaborative data that can tolerate eventual consistency
- Crates: `yrs` (Yjs port to Rust), `automerge`

#### Recommendation

**libSQL for structured data** (events, objects, sessions, jobs) — it's Rust-native with built-in replication. **CRDTs for collaborative state** (agent configs, workflow definitions) where offline editing + merge is valuable.

### 5c. What Gets Replicated vs What Stays Local

Not everything should sync across nodes:

| Data | Replication Strategy | Why |
|---|---|---|
| **Events** | Replicate (append-only) | Full audit trail everywhere |
| **Objects** (agents, skills, rules) | Replicate (CRDT merge) | Same agency config on all nodes |
| **Sessions** | Replicate (primary → replicas) | Any node can continue a session |
| **Jobs** | Replicate (single writer) | Job queue needs consistency |
| **Metrics** | Local only, aggregate on query | Too high volume to sync |
| **Storage files** | On-demand pull | Don't sync 50MB files everywhere |
| **Chat history** | Replicate (append-only) | Access conversations from any node |
| **Config** | Replicate (last-write-wins) | Consistent settings across nodes |

### 5d. Node Discovery & Coordination

```rust
// New: rusvel-net crate (~800 lines)
pub struct NodeInfo {
    pub id: String,           // stable UUID per installation
    pub name: String,         // human-friendly name
    pub address: SocketAddr,  // reachable address (Tailscale IP, tunnel endpoint, etc.)
    pub role: NodeRole,       // Primary, Replica, or Peer
    pub capabilities: Vec<String>,  // which engines are active
    pub last_seen: DateTime<Utc>,
}

pub enum NodeRole {
    Primary,    // Accepts writes, replicates to others
    Replica,    // Read-only, syncs from primary
    Peer,       // Equal peer in CRDT mode
}

pub trait NetworkPort: Send + Sync {
    async fn discover_peers(&self) -> Result<Vec<NodeInfo>>;
    async fn send(&self, peer: &str, message: NetMessage) -> Result<()>;
    async fn broadcast(&self, message: NetMessage) -> Result<()>;
    async fn subscribe(&self) -> Result<Receiver<NetMessage>>;
}
```

### 5e. Distributed Job Execution

With multiple nodes, the job queue becomes a distributed scheduler:

```
Machine A: forge-engine, code-engine     (GPU, heavy compute)
Machine B: content-engine, harvest-engine (always-on VPS, web scraping)
Machine C: gtm-engine, finance-engine    (laptop, human-in-the-loop)
```

- Jobs tagged with required engine/capability
- Scheduler routes jobs to capable nodes
- Results replicated back via event stream
- Failed jobs can be retried on different nodes

---

## Phase 6: Enhanced SQLite Capabilities

### 6a. Sqlean Extension Loading

Load the Sqlean extension bundle at startup for richer SQL capabilities:

- `regexp` — regex support in WHERE clauses
- `uuid` — `uuid4()` function in SQL
- `stats` — median, percentile, stddev aggregates
- `fuzzy` — Levenshtein distance, soundex for fuzzy matching
- `csv` — virtual table for CSV file querying
- Makes the SQL Editor significantly more powerful
- Single `.so`/`.dylib` to ship alongside the binary (or statically link)

### 6b. DuckDB for Analytics

RUSVEL has OLAP workloads (growth-engine funnels, finance-engine ledger queries) that SQLite handles poorly:

- DuckDB's SQLite extension reads `rusvel.db` directly — no ETL needed
- Run analytical queries (GROUP BY, window functions, aggregates) at 10-100x SQLite speed
- **Option:** embed DuckDB alongside SQLite, route analytical queries to it
- Crate: `duckdb-rs`

```rust
// Route based on query type
match classify_query(&sql) {
    QueryType::OLTP => execute_on_sqlite(sql),    // INSERT, UPDATE, simple SELECT
    QueryType::OLAP => execute_on_duckdb(sql),    // GROUP BY, window functions, aggregates
}
```

### 6c. Multi-Database Browser

- Allow connecting to external SQLite databases (not just `~/.rusvel/rusvel.db`)
- Table Editor becomes a general-purpose SQLite browser
- Useful for: inspecting app databases, imported datasets, debugging
- Route: `POST /api/db/connect` with a file path
- In distributed mode: browse any node's database from any other node

---

## Updated Build Order

| Step | Phase | Scope | Effort | Depends On |
|---|---|---|---|---|
| **1** | Phase 1 | `rusvel-schema` crate | ~300 lines Rust | Nothing |
| **2** | Phase 2 | Dynamic API routes | ~500 lines Rust | Step 1 |
| **3** | Phase 3a | Table Editor UI | ~800 lines Svelte | Step 2 |
| **4** | Phase 3b | SQL Editor UI | ~500 lines Svelte | Step 2 |
| **5** | Phase 4a | Realtime (update hooks + SSE) | ~300 lines | Step 2 |
| **6** | Phase 3c | Schema Viewer UI | ~300 lines Svelte | Step 1 |
| **7** | Phase 4b | File Storage | ~800 lines | Nothing |
| **8** | Phase 5a | Network layer (Tailscale + rathole study) | ~200 lines + config | Nothing |
| **9** | Phase 5b | libSQL migration (`rusvel-db` swap) | ~400 lines refactor | Step 7 |
| **10** | Phase 5d | `rusvel-net` crate (discovery + messaging) | ~800 lines Rust | Step 8 |
| **11** | Phase 5c | Replication rules + sync logic | ~600 lines Rust | Steps 9+10 |
| **12** | Phase 5e | Distributed job scheduler | ~400 lines Rust | Steps 10+11 |
| **13** | Phase 6a | Sqlean extension loading | ~100 lines Rust | Nothing |
| **14** | Phase 6b | DuckDB analytics routing | ~300 lines Rust | Step 2 |
| **15** | Phase 6c | Multi-database browser | ~200 lines Rust+Svelte | Step 3 |

**Phases 1-4** = local RusvelBase platform (~3100 lines)
**Phase 5** = distributed mesh (~2400 lines)
**Phase 6** = enhanced capabilities (~600 lines)

---

## Architecture Fit

This integrates into RUSVEL's hexagonal architecture with two new crates:

```
rusvel-schema (new)     → Pure introspection, no framework deps
rusvel-net (new)        → P2P networking, peer discovery, message passing (Phase 5)
rusvel-db               → Swap rusqlite → libsql, add update_hook + storage tables
rusvel-api              → New /api/db/* routes, /api/storage/*, /api/nodes/* routes
frontend                → New /database/* pages, /network/* status page
rusvel-app              → Wire SchemaIntrospector + NetworkPort into AppState
```

New port trait in `rusvel-core`:

```rust
// Addition to ports.rs
#[async_trait]
pub trait NetworkPort: Send + Sync {
    async fn discover_peers(&self) -> Result<Vec<NodeInfo>>;
    async fn send(&self, peer_id: &str, message: NetMessage) -> Result<()>;
    async fn broadcast(&self, message: NetMessage) -> Result<()>;
    async fn subscribe(&self) -> Result<broadcast::Receiver<NetMessage>>;
    async fn node_info(&self) -> NodeInfo;
}
```

---

## Reference Repos

Already cloned in `good-repo/` for study:

| Repo | What to Study | Relevant Phase |
|---|---|---|
| `rathole` | NAT traversal, tunnel protocol, Rust networking | Phase 5a |
| `mullvadvpn-app` | WireGuard integration, Rust VPN, tunnel management | Phase 5a |
| `pocketbase` | Single-binary BaaS (Go) — auth, realtime, file storage, admin UI | Phases 1-4 (API design) |
| `meilisearch` | Rust search engine — API design, index management | Phase 6 |
| `qdrant` | Rust vector DB — distributed architecture, Raft consensus | Phase 5b |
| `greptimedb` | Rust time-series DB — distributed, metric storage | Phase 5e |
| `quickwit` | Rust search — distributed indexing, S3 storage | Phase 5 |
| `lancedb` | Rust vector DB — embedded, multimodal | Phase 6b |
| `dora` | Rust dataflow — distributed agent coordination | Phase 5e |
| `sonic` | Rust search backend — lightweight, fast | Phase 6 |

---

## Design Philosophy

RUSVEL takes the **"integrated toolbox"** approach:

1. **Same binary, N machines, one agency** — easy install via single binary, distributed via P2P mesh
2. **Datasette** inspired our Table Editor + SQL Editor + auto-API — embedded, not a separate service
3. **libSQL** replaces vanilla SQLite — Rust-native async, built-in replication, vector search
4. **rathole/Tailscale** provides the connectivity layer — NAT traversal without port forwarding
5. **Sqlean** enriches SQL capabilities — loaded as extensions at startup
6. **DuckDB** handles OLAP workloads — analytical queries routed automatically
7. **CRDTs** for conflict-free collaborative state across nodes

The result: `cargo install rusvel` on any machine, join the mesh, full agency capabilities.

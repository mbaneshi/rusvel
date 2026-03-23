//! Embedded SQL migration system.
//!
//! Migrations are numbered strings applied in order. The `schema_version`
//! PRAGMA tracks which have been applied.

use rusqlite::Connection;

/// Each migration is a (version, sql) pair.
const MIGRATIONS: &[(u32, &str)] = &[(1, MIGRATION_001)];

const MIGRATION_001: &str = r"
-- ════════════════════════════════════════════════════════════════
--  Events (append-only event log)
-- ════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS events (
    id          TEXT PRIMARY KEY,
    session_id  TEXT,
    run_id      TEXT,
    source      TEXT NOT NULL,
    kind        TEXT NOT NULL,
    payload     TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    metadata    TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_events_session   ON events(session_id);
CREATE INDEX IF NOT EXISTS idx_events_kind      ON events(kind);
CREATE INDEX IF NOT EXISTS idx_events_created   ON events(created_at);
CREATE INDEX IF NOT EXISTS idx_events_run       ON events(run_id);

-- ════════════════════════════════════════════════════════════════
--  Objects (CRUD store keyed by kind + id)
-- ════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS objects (
    kind        TEXT NOT NULL,
    id          TEXT NOT NULL,
    data        TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    PRIMARY KEY (kind, id)
);

CREATE INDEX IF NOT EXISTS idx_objects_kind ON objects(kind);

-- ════════════════════════════════════════════════════════════════
--  Sessions
-- ════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS sessions (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    kind        TEXT NOT NULL,
    tags        TEXT NOT NULL DEFAULT '[]',
    config      TEXT NOT NULL DEFAULT '{}',
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    metadata    TEXT NOT NULL DEFAULT '{}'
);

-- ════════════════════════════════════════════════════════════════
--  Runs
-- ════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS runs (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    engine          TEXT NOT NULL,
    input_summary   TEXT NOT NULL DEFAULT '',
    status          TEXT NOT NULL,
    llm_budget_used REAL NOT NULL DEFAULT 0.0,
    tool_calls_count INTEGER NOT NULL DEFAULT 0,
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    metadata        TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (session_id) REFERENCES sessions(id)
);

CREATE INDEX IF NOT EXISTS idx_runs_session ON runs(session_id);

-- ════════════════════════════════════════════════════════════════
--  Threads
-- ════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS threads (
    id          TEXT PRIMARY KEY,
    run_id      TEXT NOT NULL,
    channel     TEXT NOT NULL,
    messages    TEXT NOT NULL DEFAULT '[]',
    metadata    TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (run_id) REFERENCES runs(id)
);

CREATE INDEX IF NOT EXISTS idx_threads_run ON threads(run_id);

-- ════════════════════════════════════════════════════════════════
--  Jobs
-- ════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS jobs (
    id              TEXT PRIMARY KEY,
    session_id      TEXT NOT NULL,
    kind            TEXT NOT NULL,
    payload         TEXT NOT NULL DEFAULT '{}',
    status          TEXT NOT NULL DEFAULT 'Queued',
    scheduled_at    TEXT,
    started_at      TEXT,
    completed_at    TEXT,
    retries         INTEGER NOT NULL DEFAULT 0,
    max_retries     INTEGER NOT NULL DEFAULT 3,
    error           TEXT,
    metadata        TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_jobs_status  ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_kind    ON jobs(kind);
CREATE INDEX IF NOT EXISTS idx_jobs_session ON jobs(session_id);

-- ════════════════════════════════════════════════════════════════
--  Metrics (time-series)
-- ════════════════════════════════════════════════════════════════

CREATE TABLE IF NOT EXISTS metrics (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    name        TEXT NOT NULL,
    value       REAL NOT NULL,
    tags        TEXT NOT NULL DEFAULT '[]',
    recorded_at TEXT NOT NULL,
    metadata    TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_metrics_name        ON metrics(name);
CREATE INDEX IF NOT EXISTS idx_metrics_recorded_at ON metrics(recorded_at);
";

/// Apply all pending migrations to the database.
pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    let current_version: u32 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;

    for &(version, sql) in MIGRATIONS {
        if version > current_version {
            tracing::info!("Applying migration v{version}");
            conn.execute_batch(sql)?;
            conn.pragma_update(None, "user_version", version)?;
        }
    }

    Ok(())
}

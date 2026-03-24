//! SQLite-backed [`StoragePort`] implementation.
//!
//! A single [`Database`] struct owns the connection and implements all
//! five sub-store traits. Async wrappers use `tokio::task::spawn_blocking`
//! internally since rusqlite is synchronous.

use std::path::Path;
use std::sync::Mutex;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};

use rusvel_core::error::RusvelError;
use rusvel_core::ports::*;
use rusvel_core::{
    Event, EventFilter, Job, JobFilter, JobKind, JobResult, JobStatus, MetricFilter, MetricPoint,
    NewJob, ObjectFilter, Run, Session, SessionSummary, Thread,
};
use rusvel_core::{EventId, JobId, RunId, SessionId, ThreadId};

use crate::migrations;

// ════════════════════════════════════════════════════════════════════
//  Database
// ════════════════════════════════════════════════════════════════════

/// SQLite-backed storage adapter.
///
/// Thread-safe via an internal `Mutex<Connection>`. For the single-writer
/// nature of `SQLite` this is the simplest correct approach.
pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    /// Open (or create) a database at the given path, enable WAL mode,
    /// and run all pending migrations.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, RusvelError> {
        let conn = Connection::open(path).map_err(|e| RusvelError::Storage(e.to_string()))?;
        Self::init(conn)
    }

    /// Create an in-memory database (useful for tests).
    pub fn in_memory() -> Result<Self, RusvelError> {
        let conn = Connection::open_in_memory().map_err(|e| RusvelError::Storage(e.to_string()))?;
        Self::init(conn)
    }

    fn init(conn: Connection) -> Result<Self, RusvelError> {
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON;")
            .map_err(|e| RusvelError::Storage(e.to_string()))?;
        migrations::run_migrations(&conn).map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Acquire the connection lock. Panics only if a thread panicked
    /// while holding the lock (programming bug).
    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("database mutex poisoned")
    }
}

// ════════════════════════════════════════════════════════════════════
//  StoragePort
// ════════════════════════════════════════════════════════════════════

impl StoragePort for Database {
    fn events(&self) -> &dyn EventStore {
        self
    }
    fn objects(&self) -> &dyn ObjectStore {
        self
    }
    fn sessions(&self) -> &dyn SessionStore {
        self
    }
    fn jobs(&self) -> &dyn JobStore {
        self
    }
    fn metrics(&self) -> &dyn MetricStore {
        self
    }
}

// ════════════════════════════════════════════════════════════════════
//  EventStore
// ════════════════════════════════════════════════════════════════════

#[async_trait]
impl EventStore for Database {
    async fn append(&self, event: &Event) -> rusvel_core::Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO events (id, session_id, run_id, source, kind, payload, created_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                event.id.to_string(),
                event.session_id.map(|s| s.to_string()),
                event.run_id.map(|r| r.to_string()),
                serde_json::to_string(&event.source)?,
                event.kind,
                serde_json::to_string(&event.payload)?,
                event.created_at.to_rfc3339(),
                serde_json::to_string(&event.metadata)?,
            ],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, id: &EventId) -> rusvel_core::Result<Option<Event>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT id, session_id, run_id, source, kind, payload, created_at, metadata FROM events WHERE id = ?1")
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let result = stmt
            .query_row(params![id.to_string()], |row| Ok(row_to_event(row)))
            .optional()
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        match result {
            Some(event) => Ok(Some(event?)),
            None => Ok(None),
        }
    }

    async fn query(&self, filter: EventFilter) -> rusvel_core::Result<Vec<Event>> {
        let conn = self.conn();
        let mut sql = String::from(
            "SELECT id, session_id, run_id, source, kind, payload, created_at, metadata FROM events WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(ref sid) = filter.session_id {
            sql.push_str(&format!(" AND session_id = ?{idx}"));
            param_values.push(Box::new(sid.to_string()));
            idx += 1;
        }
        if let Some(ref rid) = filter.run_id {
            sql.push_str(&format!(" AND run_id = ?{idx}"));
            param_values.push(Box::new(rid.to_string()));
            idx += 1;
        }
        if let Some(ref kind) = filter.kind {
            sql.push_str(&format!(" AND kind = ?{idx}"));
            param_values.push(Box::new(kind.clone()));
            idx += 1;
        }
        if let Some(ref source) = filter.source {
            sql.push_str(&format!(" AND source = ?{idx}"));
            param_values.push(Box::new(serde_json::to_string(source)?));
            idx += 1;
        }
        if let Some(ref since) = filter.since {
            sql.push_str(&format!(" AND created_at >= ?{idx}"));
            param_values.push(Box::new(since.to_rfc3339()));
            idx += 1;
        }

        sql.push_str(" ORDER BY created_at ASC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT ?{idx}"));
            param_values.push(Box::new(i64::from(limit)));
            let _ = idx;
        }

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| Ok(row_to_event(row)))
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let mut events = Vec::new();
        for row in rows {
            let event = row.map_err(|e| RusvelError::Storage(e.to_string()))??;
            events.push(event);
        }
        Ok(events)
    }
}

fn row_to_event(row: &rusqlite::Row<'_>) -> rusvel_core::Result<Event> {
    let id_str: String = row
        .get(0)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let session_str: Option<String> = row
        .get(1)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let run_str: Option<String> = row
        .get(2)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let source_str: String = row
        .get(3)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let kind: String = row
        .get(4)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let payload_str: String = row
        .get(5)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let created_str: String = row
        .get(6)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let meta_str: String = row
        .get(7)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;

    Ok(Event {
        id: EventId::from_uuid(
            uuid::Uuid::parse_str(&id_str).map_err(|e| RusvelError::Storage(e.to_string()))?,
        ),
        session_id: session_str
            .map(|s| {
                uuid::Uuid::parse_str(&s)
                    .map(SessionId::from_uuid)
                    .map_err(|e| RusvelError::Storage(e.to_string()))
            })
            .transpose()?,
        run_id: run_str
            .map(|s| {
                uuid::Uuid::parse_str(&s)
                    .map(RunId::from_uuid)
                    .map_err(|e| RusvelError::Storage(e.to_string()))
            })
            .transpose()?,
        source: serde_json::from_str(&source_str)?,
        kind,
        payload: serde_json::from_str(&payload_str)?,
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| RusvelError::Storage(e.to_string()))?,
        metadata: serde_json::from_str(&meta_str)?,
    })
}

// ════════════════════════════════════════════════════════════════════
//  ObjectStore
// ════════════════════════════════════════════════════════════════════

#[async_trait]
impl ObjectStore for Database {
    async fn put(
        &self,
        kind: &str,
        id: &str,
        object: serde_json::Value,
    ) -> rusvel_core::Result<()> {
        let conn = self.conn();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO objects (kind, id, data, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(kind, id) DO UPDATE SET data = excluded.data, updated_at = excluded.updated_at",
            params![kind, id, serde_json::to_string(&object)?, &now, &now],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, kind: &str, id: &str) -> rusvel_core::Result<Option<serde_json::Value>> {
        let conn = self.conn();
        let result: Option<String> = conn
            .query_row(
                "SELECT data FROM objects WHERE kind = ?1 AND id = ?2",
                params![kind, id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        match result {
            Some(data) => Ok(Some(serde_json::from_str(&data)?)),
            None => Ok(None),
        }
    }

    async fn delete(&self, kind: &str, id: &str) -> rusvel_core::Result<()> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM objects WHERE kind = ?1 AND id = ?2",
            params![kind, id],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn list(
        &self,
        kind: &str,
        filter: ObjectFilter,
    ) -> rusvel_core::Result<Vec<serde_json::Value>> {
        let conn = self.conn();
        let mut sql = String::from("SELECT data FROM objects WHERE kind = ?1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        param_values.push(Box::new(kind.to_string()));
        let mut idx = 2;

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT ?{idx}"));
            param_values.push(Box::new(i64::from(limit)));
            idx += 1;
        }
        if let Some(offset) = filter.offset {
            // LIMIT is required for OFFSET in SQLite; add a large default if
            // no explicit limit was supplied.
            if filter.limit.is_none() {
                sql.push_str(&format!(" LIMIT ?{idx}"));
                param_values.push(Box::new(i64::MAX));
                idx += 1;
            }
            sql.push_str(&format!(" OFFSET ?{idx}"));
            param_values.push(Box::new(i64::from(offset)));
            let _ = idx;
        }

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                let data: String = row.get(0)?;
                Ok(data)
            })
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let mut objects = Vec::new();
        for row in rows {
            let data = row.map_err(|e| RusvelError::Storage(e.to_string()))?;
            objects.push(serde_json::from_str(&data)?);
        }
        Ok(objects)
    }
}

// ════════════════════════════════════════════════════════════════════
//  SessionStore
// ════════════════════════════════════════════════════════════════════

#[async_trait]
impl SessionStore for Database {
    async fn put_session(&self, session: &Session) -> rusvel_core::Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO sessions (id, name, kind, tags, config, created_at, updated_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
               name = excluded.name,
               kind = excluded.kind,
               tags = excluded.tags,
               config = excluded.config,
               updated_at = excluded.updated_at,
               metadata = excluded.metadata",
            params![
                session.id.to_string(),
                session.name,
                serde_json::to_string(&session.kind)?,
                serde_json::to_string(&session.tags)?,
                serde_json::to_string(&session.config)?,
                session.created_at.to_rfc3339(),
                session.updated_at.to_rfc3339(),
                serde_json::to_string(&session.metadata)?,
            ],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get_session(&self, id: &SessionId) -> rusvel_core::Result<Option<Session>> {
        let conn = self.conn();
        let result = conn
            .query_row(
                "SELECT id, name, kind, tags, config, created_at, updated_at, metadata FROM sessions WHERE id = ?1",
                params![id.to_string()],
                |row| {
                    Ok(row_to_session(row))
                },
            )
            .optional()
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        match result {
            Some(s) => Ok(Some(s?)),
            None => Ok(None),
        }
    }

    async fn list_sessions(&self) -> rusvel_core::Result<Vec<SessionSummary>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, kind, tags, updated_at FROM sessions ORDER BY updated_at DESC",
            )
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let id_str: String = row.get(0)?;
                let name: String = row.get(1)?;
                let kind_str: String = row.get(2)?;
                let tags_str: String = row.get(3)?;
                let updated_str: String = row.get(4)?;
                Ok((id_str, name, kind_str, tags_str, updated_str))
            })
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let mut summaries = Vec::new();
        for row in rows {
            let (id_str, name, kind_str, tags_str, updated_str) =
                row.map_err(|e| RusvelError::Storage(e.to_string()))?;
            summaries.push(SessionSummary {
                id: SessionId::from_uuid(
                    uuid::Uuid::parse_str(&id_str)
                        .map_err(|e| RusvelError::Storage(e.to_string()))?,
                ),
                name,
                kind: serde_json::from_str(&kind_str)?,
                tags: serde_json::from_str(&tags_str)?,
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| RusvelError::Storage(e.to_string()))?,
            });
        }
        Ok(summaries)
    }

    async fn put_run(&self, run: &Run) -> rusvel_core::Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO runs (id, session_id, engine, input_summary, status, llm_budget_used, tool_calls_count, started_at, completed_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
               status = excluded.status,
               llm_budget_used = excluded.llm_budget_used,
               tool_calls_count = excluded.tool_calls_count,
               completed_at = excluded.completed_at,
               metadata = excluded.metadata",
            params![
                run.id.to_string(),
                run.session_id.to_string(),
                serde_json::to_string(&run.engine)?,
                run.input_summary,
                serde_json::to_string(&run.status)?,
                run.llm_budget_used,
                run.tool_calls_count,
                run.started_at.to_rfc3339(),
                run.completed_at.map(|dt| dt.to_rfc3339()),
                serde_json::to_string(&run.metadata)?,
            ],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get_run(&self, id: &RunId) -> rusvel_core::Result<Option<Run>> {
        let conn = self.conn();
        let result = conn
            .query_row(
                "SELECT id, session_id, engine, input_summary, status, llm_budget_used, tool_calls_count, started_at, completed_at, metadata FROM runs WHERE id = ?1",
                params![id.to_string()],
                |row| Ok(row_to_run(row)),
            )
            .optional()
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        match result {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    async fn list_runs(&self, session_id: &SessionId) -> rusvel_core::Result<Vec<Run>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare("SELECT id, session_id, engine, input_summary, status, llm_budget_used, tool_calls_count, started_at, completed_at, metadata FROM runs WHERE session_id = ?1 ORDER BY started_at ASC")
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![session_id.to_string()], |row| Ok(row_to_run(row)))
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let mut runs = Vec::new();
        for row in rows {
            let run = row.map_err(|e| RusvelError::Storage(e.to_string()))??;
            runs.push(run);
        }
        Ok(runs)
    }

    async fn put_thread(&self, thread: &Thread) -> rusvel_core::Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO threads (id, run_id, channel, messages, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               messages = excluded.messages,
               metadata = excluded.metadata",
            params![
                thread.id.to_string(),
                thread.run_id.to_string(),
                serde_json::to_string(&thread.channel)?,
                serde_json::to_string(&thread.messages)?,
                serde_json::to_string(&thread.metadata)?,
            ],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get_thread(&self, id: &ThreadId) -> rusvel_core::Result<Option<Thread>> {
        let conn = self.conn();
        let result = conn
            .query_row(
                "SELECT id, run_id, channel, messages, metadata FROM threads WHERE id = ?1",
                params![id.to_string()],
                |row| Ok(row_to_thread(row)),
            )
            .optional()
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        match result {
            Some(t) => Ok(Some(t?)),
            None => Ok(None),
        }
    }

    async fn list_threads(&self, run_id: &RunId) -> rusvel_core::Result<Vec<Thread>> {
        let conn = self.conn();
        let mut stmt = conn
            .prepare(
                "SELECT id, run_id, channel, messages, metadata FROM threads WHERE run_id = ?1",
            )
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let rows = stmt
            .query_map(params![run_id.to_string()], |row| Ok(row_to_thread(row)))
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let mut threads = Vec::new();
        for row in rows {
            let thread = row.map_err(|e| RusvelError::Storage(e.to_string()))??;
            threads.push(thread);
        }
        Ok(threads)
    }
}

fn row_to_session(row: &rusqlite::Row<'_>) -> rusvel_core::Result<Session> {
    let id_str: String = row
        .get(0)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let name: String = row
        .get(1)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let kind_str: String = row
        .get(2)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let tags_str: String = row
        .get(3)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let config_str: String = row
        .get(4)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let created_str: String = row
        .get(5)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let updated_str: String = row
        .get(6)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let meta_str: String = row
        .get(7)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;

    Ok(Session {
        id: SessionId::from_uuid(
            uuid::Uuid::parse_str(&id_str).map_err(|e| RusvelError::Storage(e.to_string()))?,
        ),
        name,
        kind: serde_json::from_str(&kind_str)?,
        tags: serde_json::from_str(&tags_str)?,
        config: serde_json::from_str(&config_str)?,
        created_at: DateTime::parse_from_rfc3339(&created_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| RusvelError::Storage(e.to_string()))?,
        updated_at: DateTime::parse_from_rfc3339(&updated_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| RusvelError::Storage(e.to_string()))?,
        metadata: serde_json::from_str(&meta_str)?,
    })
}

fn row_to_run(row: &rusqlite::Row<'_>) -> rusvel_core::Result<Run> {
    let id_str: String = row
        .get(0)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let session_str: String = row
        .get(1)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let engine_str: String = row
        .get(2)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let input_summary: String = row
        .get(3)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let status_str: String = row
        .get(4)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let llm_budget_used: f64 = row
        .get(5)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let tool_calls_count: u32 = row
        .get(6)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let started_str: String = row
        .get(7)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let completed_str: Option<String> = row
        .get(8)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let meta_str: String = row
        .get(9)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;

    Ok(Run {
        id: RunId::from_uuid(
            uuid::Uuid::parse_str(&id_str).map_err(|e| RusvelError::Storage(e.to_string()))?,
        ),
        session_id: SessionId::from_uuid(
            uuid::Uuid::parse_str(&session_str).map_err(|e| RusvelError::Storage(e.to_string()))?,
        ),
        engine: serde_json::from_str(&engine_str)?,
        input_summary,
        status: serde_json::from_str(&status_str)?,
        llm_budget_used,
        tool_calls_count,
        started_at: DateTime::parse_from_rfc3339(&started_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| RusvelError::Storage(e.to_string()))?,
        completed_at: completed_str
            .map(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| RusvelError::Storage(e.to_string()))
            })
            .transpose()?,
        metadata: serde_json::from_str(&meta_str)?,
    })
}

fn row_to_thread(row: &rusqlite::Row<'_>) -> rusvel_core::Result<Thread> {
    let id_str: String = row
        .get(0)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let run_str: String = row
        .get(1)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let channel_str: String = row
        .get(2)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let messages_str: String = row
        .get(3)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let meta_str: String = row
        .get(4)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;

    Ok(Thread {
        id: ThreadId::from_uuid(
            uuid::Uuid::parse_str(&id_str).map_err(|e| RusvelError::Storage(e.to_string()))?,
        ),
        run_id: RunId::from_uuid(
            uuid::Uuid::parse_str(&run_str).map_err(|e| RusvelError::Storage(e.to_string()))?,
        ),
        channel: serde_json::from_str(&channel_str)?,
        messages: serde_json::from_str(&messages_str)?,
        metadata: serde_json::from_str(&meta_str)?,
    })
}

// ════════════════════════════════════════════════════════════════════
//  JobStore
// ════════════════════════════════════════════════════════════════════

#[async_trait]
impl JobStore for Database {
    async fn enqueue(&self, job: &Job) -> rusvel_core::Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO jobs (id, session_id, kind, payload, status, scheduled_at, started_at, completed_at, retries, max_retries, error, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                job.id.to_string(),
                job.session_id.to_string(),
                serde_json::to_string(&job.kind)?,
                serde_json::to_string(&job.payload)?,
                serde_json::to_string(&job.status)?,
                job.scheduled_at.map(|dt| dt.to_rfc3339()),
                job.started_at.map(|dt| dt.to_rfc3339()),
                job.completed_at.map(|dt| dt.to_rfc3339()),
                job.retries,
                job.max_retries,
                job.error,
                serde_json::to_string(&job.metadata)?,
            ],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn dequeue(&self, kinds: &[JobKind]) -> rusvel_core::Result<Option<Job>> {
        let conn = self.conn();
        let now = Utc::now().to_rfc3339();
        let queued_status = serde_json::to_string(&JobStatus::Queued)?;

        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        param_values.push(Box::new(queued_status.clone()));
        param_values.push(Box::new(now));

        let kind_filter = if kinds.is_empty() {
            String::new()
        } else {
            let placeholders: Vec<String> = (0..kinds.len())
                .map(|i| format!("?{}", i + 3))
                .collect();
            for k in kinds {
                param_values.push(Box::new(serde_json::to_string(k)?));
            }
            format!(" AND kind IN ({})", placeholders.join(", "))
        };

        let sql = format!(
            "SELECT id, session_id, kind, payload, status, scheduled_at, started_at, completed_at, retries, max_retries, error, metadata
             FROM jobs
             WHERE status = ?1 AND (scheduled_at IS NULL OR scheduled_at <= ?2){}
             ORDER BY rowid ASC
             LIMIT 1",
            kind_filter
        );

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let result = stmt
            .query_row(params_refs.as_slice(), |row| Ok(row_to_job(row)))
            .optional()
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        match result {
            Some(job) => {
                let job = job?;
                // Claim it by setting status to Running
                let running_status = serde_json::to_string(&JobStatus::Running)?;
                let now = Utc::now().to_rfc3339();
                conn.execute(
                    "UPDATE jobs SET status = ?1, started_at = ?2 WHERE id = ?3",
                    params![running_status, now, job.id.to_string()],
                )
                .map_err(|e| RusvelError::Storage(e.to_string()))?;

                // Return the job with updated status
                let mut claimed = job;
                claimed.status = JobStatus::Running;
                claimed.started_at = Some(Utc::now());
                Ok(Some(claimed))
            }
            None => Ok(None),
        }
    }

    async fn update(&self, job: &Job) -> rusvel_core::Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE jobs SET
               status = ?1, payload = ?2, scheduled_at = ?3, started_at = ?4,
               completed_at = ?5, retries = ?6, max_retries = ?7, error = ?8, metadata = ?9
             WHERE id = ?10",
            params![
                serde_json::to_string(&job.status)?,
                serde_json::to_string(&job.payload)?,
                job.scheduled_at.map(|dt| dt.to_rfc3339()),
                job.started_at.map(|dt| dt.to_rfc3339()),
                job.completed_at.map(|dt| dt.to_rfc3339()),
                job.retries,
                job.max_retries,
                job.error,
                serde_json::to_string(&job.metadata)?,
                job.id.to_string(),
            ],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn get(&self, id: &JobId) -> rusvel_core::Result<Option<Job>> {
        let conn = self.conn();
        let result = conn
            .query_row(
                "SELECT id, session_id, kind, payload, status, scheduled_at, started_at, completed_at, retries, max_retries, error, metadata FROM jobs WHERE id = ?1",
                params![id.to_string()],
                |row| Ok(row_to_job(row)),
            )
            .optional()
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        match result {
            Some(j) => Ok(Some(j?)),
            None => Ok(None),
        }
    }

    async fn list(&self, filter: JobFilter) -> rusvel_core::Result<Vec<Job>> {
        let conn = self.conn();
        let mut sql = String::from(
            "SELECT id, session_id, kind, payload, status, scheduled_at, started_at, completed_at, retries, max_retries, error, metadata FROM jobs WHERE 1=1",
        );
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(ref sid) = filter.session_id {
            sql.push_str(&format!(" AND session_id = ?{idx}"));
            param_values.push(Box::new(sid.to_string()));
            idx += 1;
        }

        if !filter.kinds.is_empty() {
            let placeholders: Vec<String> = filter
                .kinds
                .iter()
                .map(|_| {
                    let p = format!("?{idx}");
                    idx += 1;
                    p
                })
                .collect();
            sql.push_str(&format!(" AND kind IN ({})", placeholders.join(", ")));
            for k in &filter.kinds {
                param_values.push(Box::new(serde_json::to_string(k)?));
            }
        }

        if !filter.statuses.is_empty() {
            let placeholders: Vec<String> = filter
                .statuses
                .iter()
                .map(|_| {
                    let p = format!("?{idx}");
                    idx += 1;
                    p
                })
                .collect();
            sql.push_str(&format!(" AND status IN ({})", placeholders.join(", ")));
            for s in &filter.statuses {
                param_values.push(Box::new(serde_json::to_string(s)?));
            }
        }

        sql.push_str(" ORDER BY rowid ASC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT ?{idx}"));
            param_values.push(Box::new(i64::from(limit)));
            let _ = idx;
        }

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| Ok(row_to_job(row)))
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let mut jobs = Vec::new();
        for row in rows {
            let job = row.map_err(|e| RusvelError::Storage(e.to_string()))??;
            jobs.push(job);
        }
        Ok(jobs)
    }
}

fn row_to_job(row: &rusqlite::Row<'_>) -> rusvel_core::Result<Job> {
    let id_str: String = row
        .get(0)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let session_str: String = row
        .get(1)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let kind_str: String = row
        .get(2)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let payload_str: String = row
        .get(3)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let status_str: String = row
        .get(4)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let scheduled_str: Option<String> = row
        .get(5)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let started_str: Option<String> = row
        .get(6)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let completed_str: Option<String> = row
        .get(7)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let retries: u32 = row
        .get(8)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let max_retries: u32 = row
        .get(9)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let error: Option<String> = row
        .get(10)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
    let meta_str: String = row
        .get(11)
        .map_err(|e| RusvelError::Storage(e.to_string()))?;

    fn parse_opt_dt(s: Option<String>) -> rusvel_core::Result<Option<DateTime<Utc>>> {
        match s {
            Some(s) => Ok(Some(
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| RusvelError::Storage(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    Ok(Job {
        id: JobId::from_uuid(
            uuid::Uuid::parse_str(&id_str).map_err(|e| RusvelError::Storage(e.to_string()))?,
        ),
        session_id: SessionId::from_uuid(
            uuid::Uuid::parse_str(&session_str).map_err(|e| RusvelError::Storage(e.to_string()))?,
        ),
        kind: serde_json::from_str(&kind_str)?,
        payload: serde_json::from_str(&payload_str)?,
        status: serde_json::from_str(&status_str)?,
        scheduled_at: parse_opt_dt(scheduled_str)?,
        started_at: parse_opt_dt(started_str)?,
        completed_at: parse_opt_dt(completed_str)?,
        retries,
        max_retries,
        error,
        metadata: serde_json::from_str(&meta_str)?,
    })
}

fn job_from_new_job(new: NewJob) -> Job {
    Job {
        id: JobId::new(),
        session_id: new.session_id,
        kind: new.kind,
        payload: new.payload,
        status: JobStatus::Queued,
        scheduled_at: None,
        started_at: None,
        completed_at: None,
        retries: 0,
        max_retries: new.max_retries,
        error: None,
        metadata: new.metadata,
    }
}

// ════════════════════════════════════════════════════════════════════
//  JobPort (SQLite-backed; same store as JobStore / StoragePort::jobs)
// ════════════════════════════════════════════════════════════════════

#[async_trait]
impl JobPort for Database {
    async fn enqueue(&self, new: NewJob) -> rusvel_core::Result<JobId> {
        let job = job_from_new_job(new);
        let id = job.id;
        JobStore::enqueue(self, &job).await?;
        Ok(id)
    }

    async fn dequeue(&self, kinds: &[JobKind]) -> rusvel_core::Result<Option<Job>> {
        JobStore::dequeue(self, kinds).await
    }

    async fn complete(&self, id: &JobId, result: JobResult) -> rusvel_core::Result<()> {
        let mut job = JobStore::get(self, id)
            .await?
            .ok_or_else(|| RusvelError::NotFound {
                kind: "Job".into(),
                id: id.to_string(),
            })?;

        if job.status != JobStatus::Running {
            return Err(RusvelError::InvalidState {
                from: format!("{:?}", job.status),
                to: "Succeeded".into(),
            });
        }

        job.status = JobStatus::Succeeded;
        job.completed_at = Some(Utc::now());
        job.metadata["result"] =
            serde_json::to_value(&result).map_err(|e| RusvelError::Serialization(e.to_string()))?;
        JobStore::update(self, &job).await
    }

    async fn fail(&self, id: &JobId, error: String) -> rusvel_core::Result<()> {
        let mut job = JobStore::get(self, id)
            .await?
            .ok_or_else(|| RusvelError::NotFound {
                kind: "Job".into(),
                id: id.to_string(),
            })?;

        if job.status != JobStatus::Running {
            return Err(RusvelError::InvalidState {
                from: format!("{:?}", job.status),
                to: "Failed".into(),
            });
        }

        job.status = JobStatus::Failed;
        job.completed_at = Some(Utc::now());
        job.error = Some(error);
        JobStore::update(self, &job).await
    }

    async fn schedule(&self, new: NewJob, cron: &str) -> rusvel_core::Result<JobId> {
        let mut job = job_from_new_job(new);
        job.scheduled_at = Some(Utc::now());
        job.metadata["cron"] = serde_json::Value::String(cron.to_string());
        let id = job.id;
        JobStore::enqueue(self, &job).await?;
        Ok(id)
    }

    async fn cancel(&self, id: &JobId) -> rusvel_core::Result<()> {
        let mut job = JobStore::get(self, id)
            .await?
            .ok_or_else(|| RusvelError::NotFound {
                kind: "Job".into(),
                id: id.to_string(),
            })?;

        match job.status {
            JobStatus::Queued | JobStatus::AwaitingApproval => {
                job.status = JobStatus::Cancelled;
                job.completed_at = Some(Utc::now());
                JobStore::update(self, &job).await
            }
            _ => Err(RusvelError::InvalidState {
                from: format!("{:?}", job.status),
                to: "Cancelled".into(),
            }),
        }
    }

    async fn approve(&self, id: &JobId) -> rusvel_core::Result<()> {
        let mut job = JobStore::get(self, id)
            .await?
            .ok_or_else(|| RusvelError::NotFound {
                kind: "Job".into(),
                id: id.to_string(),
            })?;

        if job.status != JobStatus::AwaitingApproval {
            return Err(RusvelError::InvalidState {
                from: format!("{:?}", job.status),
                to: "Queued".into(),
            });
        }

        job.status = JobStatus::Queued;
        JobStore::update(self, &job).await
    }

    async fn list(&self, filter: JobFilter) -> rusvel_core::Result<Vec<Job>> {
        JobStore::list(self, filter).await
    }
}

// ════════════════════════════════════════════════════════════════════
//  MetricStore
// ════════════════════════════════════════════════════════════════════

#[async_trait]
impl MetricStore for Database {
    async fn record(&self, point: &MetricPoint) -> rusvel_core::Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO metrics (name, value, tags, recorded_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                point.name,
                point.value,
                serde_json::to_string(&point.tags)?,
                point.recorded_at.to_rfc3339(),
                serde_json::to_string(&point.metadata)?,
            ],
        )
        .map_err(|e| RusvelError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn query(&self, filter: MetricFilter) -> rusvel_core::Result<Vec<MetricPoint>> {
        let conn = self.conn();
        let mut sql =
            String::from("SELECT name, value, tags, recorded_at, metadata FROM metrics WHERE 1=1");
        let mut param_values: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();
        let mut idx = 1;

        if let Some(ref name) = filter.name {
            sql.push_str(&format!(" AND name = ?{idx}"));
            param_values.push(Box::new(name.clone()));
            idx += 1;
        }
        if let Some(ref since) = filter.since {
            sql.push_str(&format!(" AND recorded_at >= ?{idx}"));
            param_values.push(Box::new(since.to_rfc3339()));
            idx += 1;
        }
        if let Some(ref until) = filter.until {
            sql.push_str(&format!(" AND recorded_at <= ?{idx}"));
            param_values.push(Box::new(until.to_rfc3339()));
            idx += 1;
        }

        sql.push_str(" ORDER BY recorded_at ASC");

        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT ?{idx}"));
            param_values.push(Box::new(i64::from(limit)));
            let _ = idx;
        }

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let params_refs: Vec<&dyn rusqlite::types::ToSql> = param_values
            .iter()
            .map(std::convert::AsRef::as_ref)
            .collect();

        let rows = stmt
            .query_map(params_refs.as_slice(), |row| {
                let name: String = row.get(0)?;
                let value: f64 = row.get(1)?;
                let tags_str: String = row.get(2)?;
                let recorded_str: String = row.get(3)?;
                let meta_str: String = row.get(4)?;
                Ok((name, value, tags_str, recorded_str, meta_str))
            })
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let mut points = Vec::new();
        for row in rows {
            let (name, value, tags_str, recorded_str, meta_str) =
                row.map_err(|e| RusvelError::Storage(e.to_string()))?;
            points.push(MetricPoint {
                name,
                value,
                tags: serde_json::from_str(&tags_str)?,
                recorded_at: DateTime::parse_from_rfc3339(&recorded_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .map_err(|e| RusvelError::Storage(e.to_string()))?,
                metadata: serde_json::from_str(&meta_str)?,
            });
        }
        Ok(points)
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use rusvel_core::domain::*;
    #[allow(unused_imports)]
    use rusvel_core::id::*;

    fn test_db() -> Database {
        Database::in_memory().expect("failed to create test database")
    }

    fn make_session() -> Session {
        Session {
            id: SessionId::new(),
            name: "Test Session".into(),
            kind: SessionKind::General,
            tags: vec!["test".into()],
            config: SessionConfig::default(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    // ── EventStore tests ──────────────────────────────────────────

    #[tokio::test]
    async fn event_append_and_get() {
        let db = test_db();
        let event = Event {
            id: EventId::new(),
            session_id: Some(SessionId::new()),
            run_id: None,
            source: EngineKind::Forge,
            kind: "test.event".into(),
            payload: serde_json::json!({"key": "value"}),
            created_at: Utc::now(),
            metadata: serde_json::json!({}),
        };

        db.append(&event).await.unwrap();
        let retrieved = EventStore::get(&db, &event.id).await.unwrap().unwrap();
        assert_eq!(retrieved.id, event.id);
        assert_eq!(retrieved.kind, "test.event");
    }

    #[tokio::test]
    async fn event_query_by_kind() {
        let db = test_db();
        let sid = SessionId::new();

        for i in 0..3 {
            let event = Event {
                id: EventId::new(),
                session_id: Some(sid),
                run_id: None,
                source: EngineKind::Forge,
                kind: if i < 2 {
                    "test.a".into()
                } else {
                    "test.b".into()
                },
                payload: serde_json::json!({}),
                created_at: Utc::now(),
                metadata: serde_json::json!({}),
            };
            db.append(&event).await.unwrap();
        }

        let filter = EventFilter {
            kind: Some("test.a".into()),
            ..Default::default()
        };
        let results = EventStore::query(&db, filter).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    // ── ObjectStore tests ─────────────────────────────────────────

    #[tokio::test]
    async fn object_put_get_delete() {
        let db = test_db();
        let obj = serde_json::json!({"name": "Widget", "price": 42});

        ObjectStore::put(&db, "product", "p1", obj.clone())
            .await
            .unwrap();
        let got = ObjectStore::get(&db, "product", "p1")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(got["name"], "Widget");

        ObjectStore::delete(&db, "product", "p1").await.unwrap();
        let gone = ObjectStore::get(&db, "product", "p1").await.unwrap();
        assert!(gone.is_none());
    }

    #[tokio::test]
    async fn object_list() {
        let db = test_db();
        for i in 0..5 {
            ObjectStore::put(&db, "item", &format!("i{i}"), serde_json::json!({"n": i}))
                .await
                .unwrap();
        }

        let all = ObjectStore::list(&db, "item", ObjectFilter::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 5);

        let limited = ObjectStore::list(
            &db,
            "item",
            ObjectFilter {
                limit: Some(2),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(limited.len(), 2);
    }

    // ── SessionStore tests ────────────────────────────────────────

    #[tokio::test]
    async fn session_put_and_get() {
        let db = test_db();
        let session = make_session();

        db.put_session(&session).await.unwrap();
        let got = db.get_session(&session.id).await.unwrap().unwrap();
        assert_eq!(got.name, "Test Session");
    }

    #[tokio::test]
    async fn session_list_summaries() {
        let db = test_db();
        let s1 = make_session();
        let mut s2 = make_session();
        s2.name = "Second".into();

        db.put_session(&s1).await.unwrap();
        db.put_session(&s2).await.unwrap();

        let summaries = db.list_sessions().await.unwrap();
        assert_eq!(summaries.len(), 2);
    }

    #[tokio::test]
    async fn run_and_thread_crud() {
        let db = test_db();
        let session = make_session();
        db.put_session(&session).await.unwrap();

        let run = Run {
            id: RunId::new(),
            session_id: session.id,
            engine: EngineKind::Code,
            input_summary: "analyze repo".into(),
            status: RunStatus::Running,
            llm_budget_used: 0.0,
            tool_calls_count: 0,
            started_at: Utc::now(),
            completed_at: None,
            metadata: serde_json::json!({}),
        };
        db.put_run(&run).await.unwrap();
        let got_run = db.get_run(&run.id).await.unwrap().unwrap();
        assert_eq!(got_run.input_summary, "analyze repo");

        let runs = db.list_runs(&session.id).await.unwrap();
        assert_eq!(runs.len(), 1);

        let thread = Thread {
            id: ThreadId::new(),
            run_id: run.id,
            channel: ThreadChannel::User,
            messages: vec![],
            metadata: serde_json::json!({}),
        };
        db.put_thread(&thread).await.unwrap();
        let got_thread = db.get_thread(&thread.id).await.unwrap().unwrap();
        assert_eq!(got_thread.run_id, run.id);

        let threads = db.list_threads(&run.id).await.unwrap();
        assert_eq!(threads.len(), 1);
    }

    // ── JobStore tests ────────────────────────────────────────────

    #[tokio::test]
    async fn job_enqueue_dequeue() {
        let db = test_db();
        let job = Job {
            id: JobId::new(),
            session_id: SessionId::new(),
            kind: JobKind::AgentRun,
            payload: serde_json::json!({"task": "analyze"}),
            status: JobStatus::Queued,
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            retries: 0,
            max_retries: 3,
            error: None,
            metadata: serde_json::json!({}),
        };

        JobStore::enqueue(&db, &job).await.unwrap();

        let dequeued = JobStore::dequeue(&db, &[JobKind::AgentRun])
            .await
            .unwrap()
            .unwrap();
        assert_eq!(dequeued.id, job.id);
        assert_eq!(dequeued.status, JobStatus::Running);

        // No more jobs to dequeue
        let none = JobStore::dequeue(&db, &[JobKind::AgentRun])
            .await
            .unwrap();
        assert!(none.is_none());
    }

    #[tokio::test]
    async fn job_dequeue_empty_kinds_matches_any_kind() {
        let db = test_db();
        let job = Job {
            id: JobId::new(),
            session_id: SessionId::new(),
            kind: JobKind::HarvestScan,
            payload: serde_json::json!({}),
            status: JobStatus::Queued,
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            retries: 0,
            max_retries: 3,
            error: None,
            metadata: serde_json::json!({}),
        };

        JobStore::enqueue(&db, &job).await.unwrap();

        let dequeued = JobStore::dequeue(&db, &[]).await.unwrap().unwrap();
        assert_eq!(dequeued.id, job.id);
        assert_eq!(dequeued.kind, JobKind::HarvestScan);
    }

    #[tokio::test]
    async fn job_update_and_list() {
        let db = test_db();
        let sid = SessionId::new();
        let mut job = Job {
            id: JobId::new(),
            session_id: sid,
            kind: JobKind::ContentPublish,
            payload: serde_json::json!({}),
            status: JobStatus::Queued,
            scheduled_at: None,
            started_at: None,
            completed_at: None,
            retries: 0,
            max_retries: 3,
            error: None,
            metadata: serde_json::json!({}),
        };

        JobStore::enqueue(&db, &job).await.unwrap();

        job.status = JobStatus::Succeeded;
        job.completed_at = Some(Utc::now());
        JobStore::update(&db, &job).await.unwrap();

        let got = JobStore::get(&db, &job.id).await.unwrap().unwrap();
        assert_eq!(got.status, JobStatus::Succeeded);

        let filtered = JobStore::list(
            &db,
            JobFilter {
                statuses: vec![JobStatus::Succeeded],
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(filtered.len(), 1);
    }

    // ── MetricStore tests ─────────────────────────────────────────

    #[tokio::test]
    async fn metric_record_and_query() {
        let db = test_db();
        let now = Utc::now();

        for i in 0..5 {
            let point = MetricPoint {
                name: "cpu.usage".into(),
                value: 50.0 + i as f64,
                tags: vec!["host:alpha".into()],
                recorded_at: now + chrono::Duration::seconds(i),
                metadata: serde_json::json!({}),
            };
            MetricStore::record(&db, &point).await.unwrap();
        }

        // Also record a different metric
        MetricStore::record(
            &db,
            &MetricPoint {
                name: "mem.usage".into(),
                value: 80.0,
                tags: vec![],
                recorded_at: now,
                metadata: serde_json::json!({}),
            },
        )
        .await
        .unwrap();

        let cpu = MetricStore::query(
            &db,
            MetricFilter {
                name: Some("cpu.usage".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(cpu.len(), 5);

        let limited = MetricStore::query(
            &db,
            MetricFilter {
                name: Some("cpu.usage".into()),
                limit: Some(2),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(limited.len(), 2);

        let all = MetricStore::query(&db, MetricFilter::default())
            .await
            .unwrap();
        assert_eq!(all.len(), 6);
    }

    // ── StoragePort trait object ──────────────────────────────────

    #[tokio::test]
    async fn storage_port_dispatches_to_substores() {
        let db = test_db();
        let storage: &dyn StoragePort = &db;

        // Just verify we can call through the trait object
        let events = storage
            .events()
            .query(EventFilter::default())
            .await
            .unwrap();
        assert!(events.is_empty());

        let objects = storage
            .objects()
            .list("nonexistent", ObjectFilter::default())
            .await
            .unwrap();
        assert!(objects.is_empty());

        let sessions = storage.sessions().list_sessions().await.unwrap();
        assert!(sessions.is_empty());

        let jobs = storage.jobs().list(JobFilter::default()).await.unwrap();
        assert!(jobs.is_empty());

        let metrics = storage
            .metrics()
            .query(MetricFilter::default())
            .await
            .unwrap();
        assert!(metrics.is_empty());
    }
}

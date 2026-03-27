//! # rusvel-db
//!
//! SQLite-backed [`StoragePort`] adapter for RUSVEL.
//!
//! Uses `rusqlite` with WAL mode for concurrent reads. All five
//! sub-stores (events, objects, sessions, jobs, metrics) are backed
//! by a single `SQLite` database with proper indexes.

mod migrations;
mod store;

pub use store::{ColumnInfo, Database, ForeignKeyInfo, IndexInfo, SqlColumn, SqlResult, TableInfo};

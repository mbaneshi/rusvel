//! # rusvel-memory
//!
//! `SQLite` + FTS5 backed implementation of [`MemoryPort`] from `rusvel-core`.
//!
//! All memory entries are session-namespaced. Text search uses `SQLite` FTS5
//! for efficient full-text matching within a session's memory.

mod store;

pub use store::MemoryStore;

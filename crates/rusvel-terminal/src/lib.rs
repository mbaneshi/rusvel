//! PTY-backed [`rusvel_core::ports::TerminalPort`] implementation.

mod browser;
mod manager;

pub use browser::{ensure_browser_log_pane, inject_browser_event_log, spawn_browser_log_bridge};
pub use manager::TerminalManager;

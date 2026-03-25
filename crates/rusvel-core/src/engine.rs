//! Base trait that every domain engine implements.
//!
//! Engines receive port implementations via constructor injection and
//! expose a uniform lifecycle: initialize → (work) → health → shutdown.

use async_trait::async_trait;

use crate::domain::{Capability, HealthStatus};
use crate::error::Result;

/// Contract that every RUSVEL domain engine must satisfy.
///
/// The 5 engines (architecture-v2):
///
/// | Engine | Responsibility |
/// |--------|---------------|
/// | Forge | Agent orchestration + Mission (goals, planning) |
/// | Code | Code intelligence (Rust-only v0) |
/// | Harvest | Opportunity discovery |
/// | Content | Creation & publishing |
/// | `GoToMarket` | CRM + outreach + ops |
///
/// Engines depend **only** on port traits from `rusvel-core`.
/// They receive concrete adapters at construction time via
/// `Arc<dyn SomePort>`, injected by the composition root.
#[async_trait]
pub trait Engine: Send + Sync {
    /// Department string ID for this engine (e.g. `"forge"`, `"code"`).
    fn kind(&self) -> &str;

    /// Human-readable name (e.g. `"Forge Engine"`).
    fn name(&self) -> &str;

    /// Capabilities this engine advertises.
    fn capabilities(&self) -> Vec<Capability>;

    /// One-time startup (run migrations, warm caches, etc.).
    async fn initialize(&self) -> Result<()>;

    /// Graceful shutdown.
    async fn shutdown(&self) -> Result<()>;

    /// Liveness / readiness check.
    async fn health(&self) -> Result<HealthStatus>;
}

//! The core contract every department must implement.

use async_trait::async_trait;

use super::context::RegistrationContext;
use super::manifest::DepartmentManifest;
use crate::error::Result;

/// The contract every RUSVEL department implements.
///
/// This is the system's primary stability surface — version it carefully.
///
/// # Rules
///
/// - Departments depend **only** on `rusvel-core` (ADR-010 extended).
/// - Departments never import other department crates.
/// - Cross-department communication goes through [`EventPort`](crate::ports::EventPort).
/// - Departments use [`AgentPort`](crate::ports::AgentPort), never `LlmPort` (ADR-009).
#[async_trait]
pub trait DepartmentApp: Send + Sync {
    /// Static manifest declaring what this department contributes.
    ///
    /// Called before [`register()`](Self::register). Must be side-effect-free.
    /// The host uses this for dependency resolution, capability indexing,
    /// and UI rendering without executing department code.
    fn manifest(&self) -> DepartmentManifest;

    /// Register with host subsystems.
    ///
    /// Called once at boot, after dependency order is resolved.
    /// The department receives ports and registrars for each surface
    /// it participates in.
    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()>;

    /// Graceful shutdown. Default is no-op.
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

//! Flow Department — DepartmentApp implementation.
//!
//! Wraps `flow-engine` (DAG workflow automation) with the ADR-014
//! department contract: manifest declaration, subsystem registration,
//! and lifecycle.

mod manifest;
mod tools;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use flow_engine::FlowEngine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

/// The Flow Department app.
///
/// Implements [`DepartmentApp`] to register the flow engine's routes,
/// tools, event subscriptions, and job handlers with the host.
pub struct FlowDepartment {
    engine: OnceLock<Arc<FlowEngine>>,
}

impl FlowDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    /// Access the inner engine (available after registration).
    pub fn engine(&self) -> Option<&Arc<FlowEngine>> {
        self.engine.get()
    }
}

impl Default for FlowDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for FlowDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::flow_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(FlowEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            None,
            None,
        ));
        let _ = self.engine.set(engine.clone());

        tools::register(&engine, ctx);

        tracing::info!("Flow department registered");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        // FlowEngine does not implement Engine trait; no shutdown needed.
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn department_creates() {
        let dept = FlowDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "flow");
        assert_eq!(m.requires_ports.len(), 4);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn default_impl_works() {
        let dept = FlowDepartment::default();
        assert_eq!(dept.manifest().id, "flow");
    }

    #[test]
    fn manifest_is_pure() {
        let dept = FlowDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.routes.len(), m2.routes.len());
        assert_eq!(m1.tools.len(), m2.tools.len());
    }
}

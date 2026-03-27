//! Growth Department — DepartmentApp implementation.
//!
//! Wraps `growth-engine` (funnel, cohort, KPI) with the
//! ADR-014 department contract.

mod manifest;
mod tools;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use growth_engine::GrowthEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

pub struct GrowthDepartment {
    engine: OnceLock<Arc<GrowthEngine>>,
}

impl GrowthDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    pub fn engine(&self) -> Option<&Arc<GrowthEngine>> {
        self.engine.get()
    }
}

impl Default for GrowthDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for GrowthDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::growth_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(GrowthEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        ));
        let _ = self.engine.set(engine.clone());

        tools::register(&engine, ctx);

        tracing::info!("Growth department registered");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        if let Some(engine) = self.engine.get() {
            engine.shutdown().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn department_creates() {
        let dept = GrowthDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "growth");
        assert_eq!(m.requires_ports.len(), 4);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn manifest_is_pure() {
        let dept = GrowthDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.quick_actions.len(), m2.quick_actions.len());
    }
}

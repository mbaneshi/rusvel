//! Distribution Department — DepartmentApp implementation.
//!
//! Wraps `distro-engine` (marketplace, SEO, affiliate) with the
//! ADR-014 department contract.

mod manifest;
mod tools;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use distro_engine::DistroEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

pub struct DistroDepartment {
    engine: OnceLock<Arc<DistroEngine>>,
}

impl DistroDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    pub fn engine(&self) -> Option<&Arc<DistroEngine>> {
        self.engine.get()
    }
}

impl Default for DistroDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for DistroDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::distro_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(DistroEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        ));
        let _ = self.engine.set(engine.clone());

        tools::register(&engine, ctx);

        tracing::info!("Distro department registered");
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
        let dept = DistroDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "distro");
        assert_eq!(m.requires_ports.len(), 4);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn manifest_is_pure() {
        let dept = DistroDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.quick_actions.len(), m2.quick_actions.len());
    }
}

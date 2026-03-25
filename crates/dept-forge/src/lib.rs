//! Forge Department — DepartmentApp implementation.
//!
//! Wraps `forge-engine` (agent orchestration + mission planning) with the
//! ADR-014 department contract.

mod manifest;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use forge_engine::ForgeEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

/// The Forge Department app — the meta-engine.
pub struct ForgeDepartment {
    engine: OnceLock<Arc<ForgeEngine>>,
}

impl ForgeDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    pub fn engine(&self) -> Option<&Arc<ForgeEngine>> {
        self.engine.get()
    }
}

impl Default for ForgeDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for ForgeDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::forge_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(ForgeEngine::new(
            ctx.agent.clone(),
            ctx.events.clone(),
            ctx.memory.clone(),
            ctx.storage.clone(),
            ctx.jobs.clone(),
            ctx.sessions.clone(),
            ctx.config.clone(),
        ));
        let _ = self.engine.set(engine);

        tracing::info!("Forge department registered");
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
        let dept = ForgeDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "forge");
        assert_eq!(m.requires_ports.len(), 7);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn manifest_is_pure() {
        let dept = ForgeDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.personas.len(), m2.personas.len());
    }
}

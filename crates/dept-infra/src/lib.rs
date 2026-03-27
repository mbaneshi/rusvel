//! Infra Department — DepartmentApp implementation.
//!
//! Wraps `infra-engine` (deploy, monitoring, incidents) with the
//! ADR-014 department contract.

mod manifest;
mod tools;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use infra_engine::InfraEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

pub struct InfraDepartment {
    engine: OnceLock<Arc<InfraEngine>>,
}

impl InfraDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    pub fn engine(&self) -> Option<&Arc<InfraEngine>> {
        self.engine.get()
    }
}

impl Default for InfraDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for InfraDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::infra_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(InfraEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        ));
        let _ = self.engine.set(engine.clone());

        tools::register(&engine, ctx);

        tracing::info!("Infra department registered");
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
        let dept = InfraDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "infra");
        assert_eq!(m.requires_ports.len(), 4);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn manifest_is_pure() {
        let dept = InfraDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.quick_actions.len(), m2.quick_actions.len());
    }
}

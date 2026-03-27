//! Go-to-market department — [`DepartmentApp`](rusvel_core::department::DepartmentApp) implementation.
//!
//! Wraps `gtm-engine` (CRM, outreach, invoicing) with the
//! ADR-014 department contract.

mod manifest;
mod tools;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use gtm_engine::GtmEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

pub struct GtmDepartment {
    engine: OnceLock<Arc<GtmEngine>>,
}

impl GtmDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    pub fn engine(&self) -> Option<&Arc<GtmEngine>> {
        self.engine.get()
    }
}

impl Default for GtmDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for GtmDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::gtm_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(GtmEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        ));
        let _ = self.engine.set(engine.clone());

        tools::register(&engine, ctx);

        tracing::info!("GTM department registered");
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
        let dept = GtmDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "gtm");
        assert_eq!(m.requires_ports.len(), 4);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn manifest_is_pure() {
        let dept = GtmDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.quick_actions.len(), m2.quick_actions.len());
    }

    #[test]
    fn agent_tool_ids_match_sprint_manifest() {
        assert_eq!(crate::tools::GTM_TOOL_IDS.len(), 5);
    }
}

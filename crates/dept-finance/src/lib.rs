//! Finance Department — DepartmentApp implementation.
//!
//! Wraps `finance-engine` (ledger, tax, runway) with the
//! ADR-014 department contract.

mod manifest;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use finance_engine::FinanceEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

pub struct FinanceDepartment {
    engine: OnceLock<Arc<FinanceEngine>>,
}

impl FinanceDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    pub fn engine(&self) -> Option<&Arc<FinanceEngine>> {
        self.engine.get()
    }
}

impl Default for FinanceDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for FinanceDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::finance_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(FinanceEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        ));
        let _ = self.engine.set(engine);

        tracing::info!("Finance department registered");
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
        let dept = FinanceDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "finance");
        assert_eq!(m.requires_ports.len(), 4);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn manifest_is_pure() {
        let dept = FinanceDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.quick_actions.len(), m2.quick_actions.len());
    }
}

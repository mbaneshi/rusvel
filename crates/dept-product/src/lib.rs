//! Product Department — DepartmentApp implementation.
//!
//! Wraps `product-engine` (roadmap, pricing, feedback) with the
//! ADR-014 department contract.

mod manifest;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use product_engine::ProductEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

pub struct ProductDepartment {
    engine: OnceLock<Arc<ProductEngine>>,
}

impl ProductDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    pub fn engine(&self) -> Option<&Arc<ProductEngine>> {
        self.engine.get()
    }
}

impl Default for ProductDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for ProductDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::product_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(ProductEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        ));
        let _ = self.engine.set(engine);

        tracing::info!("Product department registered");
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
        let dept = ProductDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "product");
        assert_eq!(m.requires_ports.len(), 4);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn manifest_is_pure() {
        let dept = ProductDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.quick_actions.len(), m2.quick_actions.len());
    }
}

//! Content Department — DepartmentApp implementation.
//!
//! Wraps `content-engine` (domain logic) with the ADR-014 department
//! contract: manifest declaration, subsystem registration, and lifecycle.

mod events;
mod jobs;
mod manifest;
mod tools;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use content_engine::ContentEngine;
use content_engine::adapters;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

/// The Content Department app.
///
/// Implements [`DepartmentApp`] to register the content engine's routes,
/// tools, event subscriptions, and job handlers with the host.
pub struct ContentDepartment {
    engine: OnceLock<Arc<ContentEngine>>,
}

impl ContentDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    /// Access the inner engine (available after registration).
    pub fn engine(&self) -> Option<&Arc<ContentEngine>> {
        self.engine.get()
    }
}

impl Default for ContentDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for ContentDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::content_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        // ── Create engine with injected ports ────────────────────
        let engine = ContentEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        );

        // ── Register platform adapters (engine-internal, ADR-006) ──
        engine.register_platform(Arc::new(adapters::linkedin::LinkedInAdapter::new(
            ctx.config.clone(),
        )));
        engine.register_platform(Arc::new(adapters::twitter::TwitterAdapter::new(
            ctx.config.clone(),
        )));
        engine.register_platform(Arc::new(adapters::devto::DevToAdapter::new(
            ctx.config.clone(),
        )));

        let engine = Arc::new(engine);
        let _ = self.engine.set(engine.clone());

        tools::register_tools(&mut ctx.tools, engine.clone());

        ctx.event_handlers.on(
            "content",
            "code.analyzed",
            events::on_code_analyzed(engine.clone()),
        );

        ctx.job_handlers.handle(
            "content",
            "content.publish",
            jobs::content_publish(engine.clone()),
        );

        tracing::info!("Content department registered");
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
        let dept = ContentDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "content");
        assert!(dept.engine().is_none()); // not registered yet
    }

    #[test]
    fn default_impl_works() {
        let dept = ContentDepartment::default();
        assert_eq!(dept.manifest().id, "content");
    }

    #[test]
    fn manifest_is_pure() {
        // Calling manifest() multiple times returns same data, no side effects
        let dept = ContentDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.routes.len(), m2.routes.len());
        assert_eq!(m1.tools.len(), m2.tools.len());
    }
}

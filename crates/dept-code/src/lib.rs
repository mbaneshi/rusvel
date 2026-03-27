//! Code Department — DepartmentApp implementation.
//!
//! Wraps `code-engine` (code intelligence) with the ADR-014 department
//! contract: manifest declaration, subsystem registration, and lifecycle.

mod manifest;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use code_engine::CodeEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;

/// The Code Department app.
///
/// Implements [`DepartmentApp`] to register the code engine's routes,
/// tools, event subscriptions, and job handlers with the host.
pub struct CodeDepartment {
    engine: OnceLock<Arc<CodeEngine>>,
}

impl CodeDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    /// Access the inner engine (available after registration).
    pub fn engine(&self) -> Option<&Arc<CodeEngine>> {
        self.engine.get()
    }
}

impl Default for CodeDepartment {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DepartmentApp for CodeDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::code_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(CodeEngine::new(ctx.storage.clone(), ctx.events.clone()));
        let _ = self.engine.set(engine.clone());

        // ── Register tools ───────────────────────────────────────
        let eng = engine.clone();
        ctx.tools.add(
            "code",
            "code.analyze",
            "Analyze a codebase directory for symbols, metrics, and dependencies",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                    let analysis = eng.analyze(std::path::Path::new(path)).await?;
                    Ok(ToolOutput {
                        content: serde_json::to_string_pretty(&analysis)
                            .unwrap_or_else(|_| "analyzed".into()),
                        is_error: false,
                        metadata: serde_json::json!({
                            "total_symbols": analysis.metrics.total_symbols,
                            "total_files": analysis.metrics.total_files,
                        }),
                    })
                })
            }),
        );

        let eng = engine.clone();
        ctx.tools.add(
            "code",
            "code.search",
            "Search previously indexed code symbols",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 10 }
                },
                "required": ["query"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                    let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                    let results = eng.search(query, limit)?;
                    Ok(ToolOutput {
                        content: serde_json::to_string_pretty(&results)
                            .unwrap_or_else(|_| "no results".into()),
                        is_error: false,
                        metadata: serde_json::json!({"count": results.len()}),
                    })
                })
            }),
        );

        tracing::info!("Code department registered");
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
        let dept = CodeDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "code");
        assert_eq!(m.requires_ports.len(), 2);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn default_impl_works() {
        let dept = CodeDepartment::default();
        assert_eq!(dept.manifest().id, "code");
    }

    #[test]
    fn manifest_is_pure() {
        let dept = CodeDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.routes.len(), m2.routes.len());
        assert_eq!(m1.tools.len(), m2.tools.len());
    }
}

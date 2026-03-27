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
use rusvel_core::id::SessionId;

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

fn parse_session_id(args: &serde_json::Value) -> rusvel_core::error::Result<SessionId> {
    args.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(SessionId::from_uuid)
        .ok_or_else(|| {
            rusvel_core::error::RusvelError::Validation("session_id required or invalid".into())
        })
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
        let _ = self.engine.set(engine.clone());

        // -- product.roadmap.add_feature --
        let eng = engine.clone();
        ctx.tools.add(
            "product",
            "product.roadmap.add_feature",
            "Add a feature to the product roadmap with priority and optional milestone",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" },
                    "title": { "type": "string", "description": "Feature name" },
                    "description": { "type": "string", "description": "Feature description" },
                    "priority": { "type": "string", "enum": ["Critical", "High", "Medium", "Low"] },
                    "status": { "type": "string", "description": "Optional milestone tag" }
                },
                "required": ["session_id", "title", "description", "priority"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let title = args
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let desc = args
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let priority: product_engine::Priority = args
                        .get("priority")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(product_engine::Priority::Medium);
                    let milestone = args
                        .get("status")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    let id = eng
                        .roadmap()
                        .add_feature(sid, title, desc, priority, milestone)
                        .await?;
                    Ok(ToolOutput {
                        content: format!("Feature created: {id}"),
                        is_error: false,
                        metadata: serde_json::json!({ "feature_id": id.to_string() }),
                    })
                })
            }),
        );

        // -- product.roadmap.list_features --
        let eng = engine.clone();
        ctx.tools.add(
            "product",
            "product.roadmap.list_features",
            "List all features on the product roadmap, optionally filtered by status",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" },
                    "status": { "type": "string", "enum": ["Planned", "InProgress", "Done", "Cancelled"], "description": "Optional status filter" }
                },
                "required": ["session_id"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let features = eng.roadmap().list_features(sid).await?;
                    let status_filter = args.get("status").and_then(|v| v.as_str());
                    let filtered: Vec<_> = if let Some(sf) = status_filter {
                        features.into_iter().filter(|f| {
                            let s = serde_json::to_value(&f.status).ok().and_then(|v| v.as_str().map(String::from));
                            s.as_deref() == Some(sf)
                        }).collect()
                    } else {
                        features
                    };
                    Ok(ToolOutput {
                        content: serde_json::to_string_pretty(&filtered).unwrap_or_else(|_| "[]".into()),
                        is_error: false,
                        metadata: serde_json::json!({ "count": filtered.len() }),
                    })
                })
            }),
        );

        // -- product.pricing.create_tier --
        let eng = engine.clone();
        ctx.tools.add(
            "product",
            "product.pricing.create_tier",
            "Create a new pricing tier with monthly price and feature list",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" },
                    "name": { "type": "string", "description": "Tier name (e.g. Free, Pro, Enterprise)" },
                    "price": { "type": "number", "description": "Monthly price in USD" },
                    "features": { "type": "array", "items": { "type": "string" }, "description": "Included features" }
                },
                "required": ["session_id", "name", "price", "features"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let price = args.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let features: Vec<String> = args
                        .get("features")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or_default();
                    let id = eng.pricing().create_tier(sid, name, price, None, features).await?;
                    Ok(ToolOutput {
                        content: format!("Pricing tier created: {id}"),
                        is_error: false,
                        metadata: serde_json::json!({ "tier_id": id.to_string() }),
                    })
                })
            }),
        );

        // -- product.feedback.record --
        let eng = engine.clone();
        ctx.tools.add(
            "product",
            "product.feedback.record",
            "Record user feedback with source, sentiment (kind), and content",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" },
                    "source": { "type": "string", "description": "Feedback source (e.g. user email, survey)" },
                    "sentiment": { "type": "string", "enum": ["FeatureRequest", "Bug", "Praise", "Complaint"] },
                    "content": { "type": "string", "description": "Feedback text" }
                },
                "required": ["session_id", "source", "sentiment", "content"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let source = args.get("source").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let kind: product_engine::FeedbackKind = args
                        .get("sentiment")
                        .and_then(|v| serde_json::from_value(v.clone()).ok())
                        .unwrap_or(product_engine::FeedbackKind::FeatureRequest);
                    let content = args.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let id = eng.feedback().add_feedback(sid, source, kind, content).await?;
                    Ok(ToolOutput {
                        content: format!("Feedback recorded: {id}"),
                        is_error: false,
                        metadata: serde_json::json!({ "feedback_id": id.to_string() }),
                    })
                })
            }),
        );

        tracing::info!("Product department registered (4 tools)");
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

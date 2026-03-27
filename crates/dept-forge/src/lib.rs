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
use rusvel_core::domain::Timeframe;
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;

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

fn parse_session_id(args: &serde_json::Value) -> rusvel_core::error::Result<SessionId> {
    args.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(SessionId::from_uuid)
        .ok_or_else(|| {
            rusvel_core::error::RusvelError::Validation("session_id required or invalid".into())
        })
}

fn parse_timeframe(s: &str) -> rusvel_core::error::Result<Timeframe> {
    serde_json::from_value(serde_json::Value::String(s.to_string()))
        .map_err(|_| rusvel_core::error::RusvelError::Validation(format!("invalid timeframe: {s}")))
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
        let _ = self.engine.set(engine.clone());

        let eng = engine.clone();
        ctx.tools.add(
            "forge",
            "forge.mission.today",
            "Generate today's prioritized mission plan from active goals and recent activity",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" }
                },
                "required": ["session_id"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let plan = eng.mission_today(&sid).await?;
                    Ok(ToolOutput {
                        content: serde_json::to_string_pretty(&plan)
                            .unwrap_or_else(|_| "plan".into()),
                        is_error: false,
                        metadata: serde_json::json!({}),
                    })
                })
            }),
        );

        let eng = engine.clone();
        ctx.tools.add(
            "forge",
            "forge.mission.list_goals",
            "List all goals for a session",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" }
                },
                "required": ["session_id"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let goals = eng.list_goals(&sid).await?;
                    Ok(ToolOutput {
                        content: serde_json::to_string_pretty(&goals)
                            .unwrap_or_else(|_| "[]".into()),
                        is_error: false,
                        metadata: serde_json::json!({ "count": goals.len() }),
                    })
                })
            }),
        );

        let eng = engine.clone();
        ctx.tools.add(
            "forge",
            "forge.mission.set_goal",
            "Create a new goal for a session",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "title": { "type": "string" },
                    "description": { "type": "string" },
                    "timeframe": { "type": "string", "enum": ["Day", "Week", "Month", "Quarter"] }
                },
                "required": ["session_id", "title", "description", "timeframe"]
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
                    let description = args
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let tf = args
                        .get("timeframe")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            rusvel_core::error::RusvelError::Validation("timeframe required".into())
                        })?;
                    let timeframe = parse_timeframe(tf)?;
                    let goal = eng.set_goal(&sid, title, description, timeframe).await?;
                    Ok(ToolOutput {
                        content: serde_json::to_string_pretty(&goal)
                            .unwrap_or_else(|_| "goal".into()),
                        is_error: false,
                        metadata: serde_json::json!({"goal_id": goal.id.to_string()}),
                    })
                })
            }),
        );

        let eng = engine.clone();
        ctx.tools.add(
            "forge",
            "forge.mission.review",
            "Generate a periodic review (accomplishments, blockers, next actions)",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "period": { "type": "string", "enum": ["Day", "Week", "Month", "Quarter"] }
                },
                "required": ["session_id", "period"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let p = args.get("period").and_then(|v| v.as_str()).ok_or_else(|| {
                        rusvel_core::error::RusvelError::Validation("period required".into())
                    })?;
                    let period = parse_timeframe(p)?;
                    let review = eng.review(&sid, period).await?;
                    Ok(ToolOutput {
                        content: serde_json::to_string_pretty(&review)
                            .unwrap_or_else(|_| "review".into()),
                        is_error: false,
                        metadata: serde_json::json!({}),
                    })
                })
            }),
        );

        let eng = engine.clone();
        ctx.tools.add(
            "forge",
            "forge.persona.hire",
            "Build an AgentConfig from a named Forge persona (spawn-ready profile)",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string" },
                    "persona_name": { "type": "string" }
                },
                "required": ["session_id", "persona_name"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let name = args
                        .get("persona_name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            rusvel_core::error::RusvelError::Validation(
                                "persona_name required".into(),
                            )
                        })?;
                    let cfg = eng.hire_persona(name, &sid)?;
                    Ok(ToolOutput {
                        content: serde_json::to_string_pretty(&cfg).unwrap_or_else(|_| "{}".into()),
                        is_error: false,
                        metadata: serde_json::json!({}),
                    })
                })
            }),
        );

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

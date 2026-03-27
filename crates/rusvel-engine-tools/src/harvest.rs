//! Harvest engine tools: scan, score, propose, list, pipeline.

use std::sync::Arc;

use rusvel_core::domain::{Content, OpportunityStage, ToolDefinition, ToolResult};
use rusvel_core::id::SessionId;
use rusvel_tool::ToolRegistry;
use serde_json::json;

pub async fn register(registry: &ToolRegistry, engine: Arc<harvest_engine::HarvestEngine>) {
    // ── harvest_scan ──────────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "harvest_scan".into(),
                    description: "Scan for freelance opportunities using the mock source. Returns discovered opportunities.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" }
                        },
                        "required": ["session_id"]
                    }),
                    searchable: false,
                metadata: json!({"category": "harvest", "engine": "harvest"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args, "session_id")?;
                        let source = harvest_engine::source::MockSource::new();
                        match engine.scan(&sid, &source).await {
                            Ok(opps) => ok_json(&opps),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── harvest_score ─────────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "harvest_score".into(),
                    description: "Re-score an existing opportunity and update its stored score."
                        .into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" },
                            "opportunity_id": { "type": "string", "description": "Opportunity ID" }
                        },
                        "required": ["session_id", "opportunity_id"]
                    }),
                    searchable: false,
                    metadata: json!({"category": "harvest", "engine": "harvest"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args, "session_id")?;
                        let opp_id = args["opportunity_id"].as_str().unwrap_or_default();
                        match engine.score_opportunity(&sid, opp_id).await {
                            Ok(u) => ok_json(&json!({"score": u.score, "reasoning": u.reasoning})),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── harvest_propose ───────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "harvest_propose".into(),
                    description: "Generate a tailored proposal for a stored opportunity.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" },
                            "opportunity_id": { "type": "string", "description": "Opportunity ID" },
                            "profile": { "type": "string", "description": "Freelancer profile summary" }
                        },
                        "required": ["session_id", "opportunity_id", "profile"]
                    }),
                    searchable: false,
                metadata: json!({"category": "harvest", "engine": "harvest"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args, "session_id")?;
                        let opp_id = args["opportunity_id"].as_str().unwrap_or_default();
                        let profile = args["profile"].as_str().unwrap_or_default();
                        match engine.generate_proposal(&sid, opp_id, profile).await {
                            Ok(proposal) => ok_json(&proposal),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── harvest_list ──────────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "harvest_list".into(),
                    description: "List opportunities, optionally filtered by pipeline stage.".into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" },
                            "stage": {
                                "type": "string",
                                "description": "Filter by stage: Cold, Contacted, Qualified, ProposalSent, Won, Lost",
                                "enum": ["Cold", "Contacted", "Qualified", "ProposalSent", "Won", "Lost"]
                            }
                        },
                        "required": ["session_id"]
                    }),
                    searchable: false,
                metadata: json!({"category": "harvest", "engine": "harvest"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args, "session_id")?;
                        let stage = args.get("stage").and_then(|v| v.as_str()).and_then(parse_stage);
                        match engine.list_opportunities(&sid, stage.as_ref()).await {
                            Ok(opps) => ok_json(&opps),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }

    // ── harvest_pipeline ──────────────────────────────────────────
    {
        let engine = engine.clone();
        registry
            .register_with_handler(
                ToolDefinition {
                    name: "harvest_pipeline".into(),
                    description:
                        "Get pipeline statistics for a session (total count, breakdown by stage)."
                            .into(),
                    parameters: json!({
                        "type": "object",
                        "properties": {
                            "session_id": { "type": "string", "description": "Session UUID" }
                        },
                        "required": ["session_id"]
                    }),
                    searchable: false,
                    metadata: json!({"category": "harvest", "engine": "harvest"}),
                },
                Arc::new(move |args| {
                    let engine = engine.clone();
                    Box::pin(async move {
                        let sid = parse_session_id(&args, "session_id")?;
                        match engine.pipeline(&sid).await {
                            Ok(stats) => ok_json(&stats),
                            Err(e) => err_result(e),
                        }
                    })
                }),
            )
            .await
            .unwrap();
    }
}

fn parse_session_id(args: &serde_json::Value, key: &str) -> rusvel_core::error::Result<SessionId> {
    let s = args[key].as_str().unwrap_or_default();
    let uuid = s
        .parse::<uuid::Uuid>()
        .map_err(|e| rusvel_core::error::RusvelError::Validation(format!("invalid {key}: {e}")))?;
    Ok(SessionId::from_uuid(uuid))
}

fn parse_stage(s: &str) -> Option<OpportunityStage> {
    match s {
        "Cold" => Some(OpportunityStage::Cold),
        "Contacted" => Some(OpportunityStage::Contacted),
        "Qualified" => Some(OpportunityStage::Qualified),
        "ProposalSent" => Some(OpportunityStage::ProposalSent),
        "Won" => Some(OpportunityStage::Won),
        "Lost" => Some(OpportunityStage::Lost),
        _ => None,
    }
}

fn ok_json<T: serde::Serialize>(val: &T) -> rusvel_core::error::Result<ToolResult> {
    let output = serde_json::to_string_pretty(val).unwrap_or_default();
    Ok(ToolResult {
        success: true,
        output: Content::text(output),
        metadata: json!({}),
    })
}

fn err_result(e: rusvel_core::error::RusvelError) -> rusvel_core::error::Result<ToolResult> {
    Ok(ToolResult {
        success: false,
        output: Content::text(format!("Error: {e}")),
        metadata: json!({}),
    })
}

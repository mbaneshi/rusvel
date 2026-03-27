//! Event kind constants emitted by the Forge Engine.

pub const AGENT_CREATED: &str = "forge.agent.created";
pub const AGENT_STARTED: &str = "forge.agent.started";
pub const AGENT_COMPLETED: &str = "forge.agent.completed";
pub const AGENT_FAILED: &str = "forge.agent.failed";
pub const MISSION_PLAN_GENERATED: &str = "forge.mission.plan_generated";
pub const MISSION_GOAL_CREATED: &str = "forge.mission.goal_created";
pub const MISSION_GOAL_UPDATED: &str = "forge.mission.goal_updated";
pub const MISSION_REVIEW_COMPLETED: &str = "forge.mission.review_completed";
pub const BRIEF_GENERATED: &str = "forge.brief.generated";
/// Set on [`rusvel_core::domain::JobKind::ScheduledCron`] payloads (`event_kind`) to run [`crate::ForgeEngine::generate_brief`] (S-043).
pub const DAILY_BRIEFING_CRON_KIND: &str = "forge.daily_briefing";
pub const PERSONA_HIRED: &str = "forge.persona.hired";
pub const SAFETY_BUDGET_WARNING: &str = "forge.safety.budget_warning";
pub const SAFETY_CIRCUIT_OPEN: &str = "forge.safety.circuit_open";

pub const PIPELINE_STARTED: &str = "forge.pipeline.started";
pub const PIPELINE_STEP_STARTED: &str = "forge.pipeline.step_started";
pub const PIPELINE_STEP_COMPLETED: &str = "forge.pipeline.step_completed";
pub const PIPELINE_COMPLETED: &str = "forge.pipeline.completed";
pub const PIPELINE_FAILED: &str = "forge.pipeline.failed";

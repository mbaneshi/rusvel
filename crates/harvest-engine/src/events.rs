//! Event kind constants emitted by the Harvest engine.

pub const SCAN_STARTED: &str = "harvest.scan.started";
pub const SCAN_COMPLETED: &str = "harvest.scan.completed";
pub const OPPORTUNITY_DISCOVERED: &str = "harvest.opportunity.discovered";
pub const OPPORTUNITY_SCORED: &str = "harvest.opportunity.scored";
pub const PROPOSAL_GENERATED: &str = "harvest.proposal.generated";
pub const PROPOSAL_PERSISTED: &str = "harvest.proposal.persisted";
pub const PIPELINE_ADVANCED: &str = "harvest.pipeline.advanced";
pub const OUTCOME_RECORDED: &str = "harvest.outcome.recorded";
pub const OPPORTUNITY_INGESTED: &str = "harvest.opportunity.ingested";
pub const BATCH_INGEST_COMPLETED: &str = "harvest.batch.ingest.completed";
/// Cron `event_kind` to run [`crate::scan_execute::scan_from_params`] in the job worker.
pub const HARVEST_AUTO_SCAN_CRON_KIND: &str = "harvest.auto_scan";

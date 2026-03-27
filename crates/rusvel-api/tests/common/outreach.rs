//! Shared outreach worker simulation for integration tests (S-039).

use gtm_engine::GtmEngine;
use gtm_engine::email::EmailAdapter;
use gtm_engine::outreach::OutreachSendDispatch;
use rusvel_core::domain::Job;
use rusvel_core::ports::{EventPort, JobPort};

/// Mirrors `rusvel-app` job worker handling for [`JobKind::OutreachSend`].
///
/// Used only by integration tests that `mod common`; other test binaries still compile this module.
#[allow(dead_code)]
pub async fn process_outreach_like_app_worker(
    engine: &GtmEngine,
    jobs: &dyn JobPort,
    events: &dyn EventPort,
    email: &dyn EmailAdapter,
    job: &Job,
) -> rusvel_core::error::Result<()> {
    let job_id = job.id;
    match engine
        .outreach()
        .process_outreach_send_job(job, events, email)
        .await?
    {
        OutreachSendDispatch::HoldForApproval(job_result) => {
            jobs.hold_for_approval(&job_id, job_result).await?;
        }
        OutreachSendDispatch::Complete { result, next } => {
            if let Some(nj) = next {
                jobs.enqueue(nj).await?;
            }
            jobs.complete(&job_id, result).await?;
        }
    }
    Ok(())
}

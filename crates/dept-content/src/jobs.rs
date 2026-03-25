//! Job handlers for the Content department (ADR-014).

use std::sync::Arc;

use content_engine::ContentEngine;
use rusvel_core::department::JobHandlerFn;
use rusvel_core::domain::Job;

/// Handle `content.publish` / `JobKind::ContentPublish` jobs.
pub fn content_publish(engine: Arc<ContentEngine>) -> JobHandlerFn {
    Arc::new(move |job: Job| {
        let engine = engine.clone();
        Box::pin(async move { engine.execute_content_publish_job(job).await })
    })
}

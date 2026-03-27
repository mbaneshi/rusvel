//! Event subscriptions for the Content department (ADR-014).

use std::sync::Arc;

use content_engine::ContentEngine;
use rusvel_core::department::EventHandlerFn;
use rusvel_core::domain::Event;
use rusvel_core::id::SessionId;

/// On `code.analyzed`, draft a blog from the stored `code_analysis` snapshot when a session id is present.
pub fn on_code_analyzed(engine: Arc<ContentEngine>) -> EventHandlerFn {
    Arc::new(move |event: Event| {
        let engine = engine.clone();
        Box::pin(async move {
            if event.kind != "code.analyzed" {
                return Ok(());
            }

            let session_id = event
                .session_id
                .or_else(|| {
                    event
                        .payload
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| uuid::Uuid::parse_str(s).ok())
                        .map(SessionId::from_uuid)
                })
                .or_else(|| {
                    event
                        .metadata
                        .get("session_id")
                        .and_then(|v| v.as_str())
                        .and_then(|s| uuid::Uuid::parse_str(s).ok())
                        .map(SessionId::from_uuid)
                });

            let Some(sid) = session_id else {
                tracing::debug!(
                    kind = %event.kind,
                    "code.analyzed: no session_id on event; skip auto draft-from-code"
                );
                return Ok(());
            };

            let snapshot_id = event
                .payload
                .get("snapshot_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if snapshot_id.is_empty() {
                return Ok(());
            }

            match engine
                .draft_blog_from_code_snapshot(&sid, snapshot_id)
                .await
            {
                Ok(item) => {
                    tracing::info!(
                        content_id = %item.id,
                        "code.analyzed → drafted blog from snapshot {}",
                        snapshot_id
                    );
                }
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        snapshot_id = %snapshot_id,
                        "code-to-content draft failed (non-fatal)"
                    );
                }
            }
            Ok(())
        })
    })
}

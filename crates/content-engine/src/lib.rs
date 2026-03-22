//! Content Engine — creation, adaptation, scheduling, and publishing.
//!
//! Orchestrates AI-powered content writing, multi-platform adaptation,
//! calendar scheduling, and engagement analytics.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusvel_core::domain::*;
use rusvel_core::engine::Engine;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::*;
use rusvel_core::ports::{AgentPort, EventPort, JobPort, StoragePort};

pub mod analytics;
pub mod calendar;
pub mod platform;
pub mod writer;

pub use analytics::ContentAnalytics;
pub use calendar::{ContentCalendar, ScheduledPost};
pub use platform::{MockPlatformAdapter, PlatformAdapter, PostMetrics, PublishResult};
pub use writer::{ContentReview, ContentWriter};

// ════════════════════════════════════════════════════════════════════
//  Event constants
// ════════════════════════════════════════════════════════════════════

pub mod events {
    pub const CONTENT_DRAFTED: &str = "content.drafted";
    pub const CONTENT_ADAPTED: &str = "content.adapted";
    pub const CONTENT_SCHEDULED: &str = "content.scheduled";
    pub const CONTENT_PUBLISHED: &str = "content.published";
    pub const CONTENT_REVIEWED: &str = "content.reviewed";
    pub const CONTENT_CANCELLED: &str = "content.cancelled";
    pub const METRICS_RECORDED: &str = "content.metrics_recorded";
}

// ════════════════════════════════════════════════════════════════════
//  ContentEngine
// ════════════════════════════════════════════════════════════════════

/// Central content engine that wires together writing, publishing,
/// scheduling, and analytics.
pub struct ContentEngine {
    storage: Arc<dyn StoragePort>,
    event_bus: Arc<dyn EventPort>,
    writer: ContentWriter,
    calendar: ContentCalendar,
    analytics: ContentAnalytics,
    adapters: std::sync::Mutex<HashMap<String, Arc<dyn PlatformAdapter>>>,
}

impl ContentEngine {
    pub fn new(
        storage: Arc<dyn StoragePort>,
        event_bus: Arc<dyn EventPort>,
        agent: Arc<dyn AgentPort>,
        jobs: Arc<dyn JobPort>,
    ) -> Self {
        Self {
            writer: ContentWriter::new(agent),
            calendar: ContentCalendar::new(Arc::clone(&storage), jobs),
            analytics: ContentAnalytics::new(Arc::clone(&storage)),
            adapters: std::sync::Mutex::new(HashMap::new()),
            storage,
            event_bus,
        }
    }

    /// Register a platform adapter for publishing.
    pub fn register_platform(&self, adapter: Arc<dyn PlatformAdapter>) {
        let key = format!("{:?}", adapter.platform());
        self.adapters.lock().unwrap().insert(key, adapter);
    }

    fn get_adapter(&self, platform: &Platform) -> Result<Arc<dyn PlatformAdapter>> {
        let key = format!("{platform:?}");
        self.adapters
            .lock()
            .unwrap()
            .get(&key)
            .cloned()
            .ok_or_else(|| RusvelError::NotFound {
                kind: "PlatformAdapter".into(),
                id: key,
            })
    }

    // ── Domain operations ──────────────────────────────────────────

    /// Draft new content via AI.
    pub async fn draft(
        &self,
        session_id: &SessionId,
        topic: &str,
        kind: ContentKind,
    ) -> Result<ContentItem> {
        let item = self.writer.draft(session_id, topic, kind).await?;
        self.storage
            .objects()
            .put("content", &item.id.to_string(), serde_json::to_value(&item)?)
            .await?;
        self.emit(events::CONTENT_DRAFTED, Some(*session_id), &item.id)
            .await?;
        Ok(item)
    }

    /// Adapt existing content for a target platform.
    pub async fn adapt(
        &self,
        session_id: &SessionId,
        content_id: ContentId,
        platform: Platform,
    ) -> Result<ContentItem> {
        let json = self.load_content(&content_id).await?;
        let original: ContentItem = serde_json::from_value(json)?;
        let max_len = self
            .get_adapter(&platform)
            .ok()
            .and_then(|a| a.max_length());
        let adapted_body = self.writer.adapt(&original, platform.clone(), max_len).await?;

        let mut adapted = original;
        adapted.id = ContentId::new();
        adapted.session_id = *session_id;
        adapted.body_markdown = adapted_body;
        adapted.status = ContentStatus::Adapted;
        adapted.platform_targets = vec![platform];
        self.storage
            .objects()
            .put("content", &adapted.id.to_string(), serde_json::to_value(&adapted)?)
            .await?;
        self.emit(events::CONTENT_ADAPTED, Some(*session_id), &adapted.id)
            .await?;
        Ok(adapted)
    }

    /// Publish content to a platform. Requires approval status == Approved.
    pub async fn publish(
        &self,
        session_id: &SessionId,
        content_id: ContentId,
        platform: Platform,
    ) -> Result<PublishResult> {
        let json = self.load_content(&content_id).await?;
        let mut item: ContentItem = serde_json::from_value(json)?;

        if item.approval != ApprovalStatus::Approved
            && item.approval != ApprovalStatus::AutoApproved
        {
            return Err(RusvelError::Validation(
                "Content must be approved before publishing".into(),
            ));
        }

        let adapter = self.get_adapter(&platform)?;
        let result = adapter.publish(&item).await?;

        item.status = ContentStatus::Published;
        item.published_at = Some(result.published_at);
        self.storage
            .objects()
            .put("content", &content_id.to_string(), serde_json::to_value(&item)?)
            .await?;
        self.emit(events::CONTENT_PUBLISHED, Some(*session_id), &content_id)
            .await?;
        Ok(result)
    }

    /// Schedule content for future publication.
    pub async fn schedule(
        &self,
        session_id: &SessionId,
        content_id: ContentId,
        platform: Platform,
        at: DateTime<Utc>,
    ) -> Result<()> {
        self.calendar
            .schedule(content_id, platform, at, *session_id)
            .await?;
        self.emit(events::CONTENT_SCHEDULED, Some(*session_id), &content_id)
            .await?;
        Ok(())
    }

    /// List content items, optionally filtered by status.
    pub async fn list_content(
        &self,
        session_id: &SessionId,
        status_filter: Option<ContentStatus>,
    ) -> Result<Vec<ContentItem>> {
        let filter = ObjectFilter {
            session_id: Some(*session_id),
            tags: vec![],
            limit: None,
            offset: None,
        };
        let all = self.storage.objects().list("content", filter).await?;
        let mut items: Vec<ContentItem> = all
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        if let Some(status) = status_filter {
            items.retain(|i| i.status == status);
        }
        Ok(items)
    }

    /// Get engagement metrics for a content item.
    pub async fn get_metrics(
        &self,
        content_id: ContentId,
    ) -> Result<Vec<(Platform, PostMetrics)>> {
        self.analytics.get_metrics(content_id).await
    }

    // ── Internal helpers ───────────────────────────────────────────

    async fn load_content(&self, id: &ContentId) -> Result<serde_json::Value> {
        self.storage
            .objects()
            .get("content", &id.to_string())
            .await?
            .ok_or_else(|| RusvelError::NotFound {
                kind: "ContentItem".into(),
                id: id.to_string(),
            })
    }

    async fn emit(
        &self,
        kind: &str,
        session_id: Option<SessionId>,
        content_id: &ContentId,
    ) -> Result<()> {
        let event = Event {
            id: EventId::new(),
            session_id,
            run_id: None,
            source: EngineKind::Content,
            kind: kind.into(),
            payload: serde_json::json!({ "content_id": content_id.to_string() }),
            created_at: Utc::now(),
            metadata: serde_json::json!({}),
        };
        self.event_bus.emit(event).await?;
        Ok(())
    }
}

#[async_trait]
impl Engine for ContentEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::Content
    }
    fn name(&self) -> &str {
        "Content Engine"
    }
    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability::ContentCreation]
    }
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    async fn health(&self) -> Result<HealthStatus> {
        Ok(HealthStatus {
            healthy: true,
            message: None,
            metadata: serde_json::json!({}),
        })
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests;

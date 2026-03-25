//! Platform adapter trait and implementations for publishing content.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rusvel_core::domain::{ContentItem, Platform};
use rusvel_core::error::Result;
use serde::{Deserialize, Serialize};

/// Result of publishing content to a platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    pub post_id: String,
    pub url: String,
    pub published_at: DateTime<Utc>,
}

/// Engagement metrics for a published post.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PostMetrics {
    pub views: u64,
    pub likes: u64,
    pub comments: u64,
    pub shares: u64,
}

/// Trait for adapting and publishing content to a specific platform.
#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    /// Which platform this adapter targets.
    fn platform(&self) -> Platform;

    /// Publish content and return the result with post ID and URL.
    async fn publish(&self, content: &ContentItem) -> Result<PublishResult>;

    /// Fetch engagement metrics for a published post.
    async fn metrics(&self, post_id: &str) -> Result<PostMetrics>;

    /// Character/length limit for this platform, if any.
    fn max_length(&self) -> Option<usize>;

    /// Convert markdown to the platform's native format.
    fn format_content(&self, markdown: &str) -> String;
}

// ════════════════════════════════════════════════════════════════════
// DEV.to adapter — real implementation lives in `adapters::devto::DevToAdapter`.
// ════════════════════════════════════════════════════════════════════

// ════════════════════════════════════════════════════════════════════
//  Mock adapter (for testing)
// ════════════════════════════════════════════════════════════════════

/// In-memory mock adapter that records published content for testing.
pub struct MockPlatformAdapter {
    target: Platform,
    published: std::sync::Mutex<Vec<(String, ContentItem)>>,
}

impl MockPlatformAdapter {
    pub fn new(target: Platform) -> Self {
        Self {
            target,
            published: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Return all items published through this mock.
    pub fn published_items(&self) -> Vec<(String, ContentItem)> {
        self.published.lock().unwrap().clone()
    }
}

#[async_trait]
impl PlatformAdapter for MockPlatformAdapter {
    fn platform(&self) -> Platform {
        self.target.clone()
    }

    async fn publish(&self, content: &ContentItem) -> Result<PublishResult> {
        let post_id = uuid::Uuid::now_v7().to_string();
        let url = format!("https://mock.test/posts/{post_id}");
        self.published
            .lock()
            .unwrap()
            .push((post_id.clone(), content.clone()));
        Ok(PublishResult {
            post_id,
            url,
            published_at: Utc::now(),
        })
    }

    async fn metrics(&self, _post_id: &str) -> Result<PostMetrics> {
        Ok(PostMetrics {
            views: 100,
            likes: 10,
            comments: 3,
            shares: 5,
        })
    }

    fn max_length(&self) -> Option<usize> {
        match &self.target {
            Platform::Twitter => Some(280),
            Platform::LinkedIn => Some(3000),
            _ => None,
        }
    }

    fn format_content(&self, markdown: &str) -> String {
        markdown.to_string()
    }
}

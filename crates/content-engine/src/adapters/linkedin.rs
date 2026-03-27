//! LinkedIn UGC Posts API adapter (`POST /v2/ugcPosts`).

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use reqwest::StatusCode;
use rusvel_core::domain::{ContentItem, Platform};
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::ports::ConfigPort;
use serde_json::json;

use crate::platform::{PlatformAdapter, PostMetrics, PublishResult};

const DEFAULT_API_BASE: &str = "https://api.linkedin.com/v2";

/// Publishes share-style posts via LinkedIn's UGC API using a bearer token from [`ConfigPort`] key `linkedin_token`.
pub struct LinkedInAdapter {
    config: Arc<dyn ConfigPort>,
    client: reqwest::Client,
    api_base: String,
}

impl LinkedInAdapter {
    pub fn new(config: Arc<dyn ConfigPort>) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            api_base: DEFAULT_API_BASE.to_string(),
        }
    }

    /// Override API base URL (for tests with mock HTTP).
    pub fn with_base_url(config: Arc<dyn ConfigPort>, base_url: impl Into<String>) -> Self {
        let base = base_url.into();
        Self {
            config,
            client: reqwest::Client::new(),
            api_base: base.trim_end_matches('/').to_string(),
        }
    }

    fn bearer(&self) -> Result<String> {
        match self.config.get_value("linkedin_token")? {
            Some(v) => {
                serde_json::from_value(v).map_err(|e| RusvelError::Serialization(e.to_string()))
            }
            None => Err(RusvelError::Validation(
                "config key `linkedin_token` is not set".into(),
            )),
        }
    }
}

#[async_trait]
impl PlatformAdapter for LinkedInAdapter {
    fn platform(&self) -> Platform {
        Platform::LinkedIn
    }

    async fn publish(&self, content: &ContentItem) -> Result<PublishResult> {
        let token = self.bearer()?;
        let text = self.format_content(&content.body_markdown);
        let body = json!({
            "author": "urn:li:organization:0",
            "lifecycleState": "PUBLISHED",
            "specificContent": {
                "com.linkedin.ugc.ShareContent": {
                    "shareCommentary": { "text": text },
                    "shareMediaCategory": "NONE"
                }
            },
            "visibility": { "com.linkedin.ugc.MemberNetworkVisibility": "PUBLIC" }
        });
        let url = format!("{}/ugcPosts", self.api_base);
        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .header("X-Restli-Protocol-Version", "2.0.0")
            .json(&body)
            .send()
            .await
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        let status = resp.status();
        let body_bytes = resp
            .bytes()
            .await
            .map_err(|e| RusvelError::Storage(e.to_string()))?;

        if status == StatusCode::TOO_MANY_REQUESTS {
            return Err(RusvelError::Internal(
                "LinkedIn API rate limited (429)".into(),
            ));
        }
        if !status.is_success() {
            let snippet = String::from_utf8_lossy(&body_bytes);
            let short: String = snippet.chars().take(500).collect();
            return Err(RusvelError::Internal(format!(
                "LinkedIn publish failed: {status} — {short}"
            )));
        }

        let v: serde_json::Value = serde_json::from_slice(&body_bytes)
            .map_err(|e| RusvelError::Serialization(e.to_string()))?;
        let post_id = v["id"]
            .as_str()
            .map(String::from)
            .unwrap_or_else(|| "unknown".into());
        Ok(PublishResult {
            post_id: post_id.clone(),
            url: format!("https://www.linkedin.com/feed/update/{post_id}"),
            published_at: Utc::now(),
        })
    }

    async fn metrics(&self, _post_id: &str) -> Result<PostMetrics> {
        Err(RusvelError::Internal(
            "LinkedIn metrics API not wired in this adapter".into(),
        ))
    }

    fn max_length(&self) -> Option<usize> {
        Some(3000)
    }

    fn format_content(&self, markdown: &str) -> String {
        markdown.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusvel_core::domain::{ApprovalStatus, ContentKind, ContentStatus};
    use rusvel_core::id::{ContentId, SessionId};
    use serde_json::json;
    use std::sync::Mutex;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    struct TestCfg {
        token: Mutex<String>,
    }
    impl TestCfg {
        fn new(token: &str) -> Self {
            Self {
                token: Mutex::new(token.into()),
            }
        }
    }
    impl ConfigPort for TestCfg {
        fn get_value(&self, key: &str) -> rusvel_core::error::Result<Option<serde_json::Value>> {
            if key == "linkedin_token" {
                let t = self.token.lock().unwrap();
                return Ok(Some(json!(t.as_str())));
            }
            Ok(None)
        }
        fn set_value(&self, _: &str, _: serde_json::Value) -> rusvel_core::error::Result<()> {
            Ok(())
        }
    }

    fn sample_item() -> ContentItem {
        ContentItem {
            id: ContentId::new(),
            session_id: SessionId::new(),
            kind: ContentKind::LinkedInPost,
            title: "Hi".into(),
            body_markdown: "Hello **world**".into(),
            platform_targets: vec![],
            status: ContentStatus::Draft,
            approval: ApprovalStatus::Approved,
            scheduled_at: None,
            published_at: None,
            metadata: json!({}),
        }
    }

    #[tokio::test]
    async fn publish_success_against_mock_server() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/ugcPosts"))
            .and(header("Authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(201).set_body_json(json!({
                "id": "urn:li:ugcPost:123"
            })))
            .mount(&mock_server)
            .await;

        let cfg = Arc::new(TestCfg::new("test-token"));
        let adapter = LinkedInAdapter::with_base_url(cfg, mock_server.uri());

        let item = sample_item();
        let r = adapter.publish(&item).await.unwrap();
        assert!(r.post_id.contains("123"));
        assert!(r.url.contains("linkedin.com"));
    }

    #[tokio::test]
    async fn publish_maps_429_to_error() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/ugcPosts"))
            .respond_with(ResponseTemplate::new(429).set_body_string("slow down"))
            .mount(&mock_server)
            .await;

        let cfg = Arc::new(TestCfg::new("t"));
        let adapter = LinkedInAdapter::with_base_url(cfg, mock_server.uri());
        let err = adapter.publish(&sample_item()).await.unwrap_err();
        assert!(err.to_string().contains("429") || err.to_string().contains("rate"));
    }
}

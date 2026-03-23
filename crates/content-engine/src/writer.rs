//! AI-powered content generation, adaptation, and review.

use std::sync::Arc;

use rusvel_core::domain::*;
use rusvel_core::error::Result;
use rusvel_core::id::*;
use rusvel_core::ports::AgentPort;
use serde::{Deserialize, Serialize};

/// Quality review produced by the AI reviewer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentReview {
    /// 0.0 – 1.0 quality score.
    pub score: f64,
    /// Actionable improvement suggestions.
    pub suggestions: Vec<String>,
    /// Whether the content passes the quality bar.
    pub approved: bool,
}

/// AI content writer backed by an [`AgentPort`].
pub struct ContentWriter {
    agent: Arc<dyn AgentPort>,
}

impl ContentWriter {
    pub fn new(agent: Arc<dyn AgentPort>) -> Self {
        Self { agent }
    }

    /// Ask the LLM to draft a new content item for the given topic and kind.
    pub async fn draft(
        &self,
        session_id: &SessionId,
        topic: &str,
        kind: ContentKind,
    ) -> Result<ContentItem> {
        let prompt = format!(
            "Write a {kind:?} article about: {topic}\n\n\
             Return the result as markdown. Include a title on the first line \
             prefixed with `# `."
        );
        let config = AgentConfig {
            profile_id: None,
            session_id: *session_id,
            model: None,
            tools: vec![],
            instructions: Some("You are a professional content writer.".into()),
            budget_limit: None,
            metadata: serde_json::json!({}),
        };
        let run_id = self.agent.create(config).await?;
        let output = self.agent.run(&run_id, Content::text(prompt)).await?;

        let body = extract_text(&output.content);
        let title = body.lines().find(|l| l.starts_with("# ")).map_or_else(
            || topic.to_string(),
            |l| l.trim_start_matches("# ").to_string(),
        );

        Ok(ContentItem {
            id: ContentId::new(),
            session_id: *session_id,
            kind,
            title,
            body_markdown: body,
            platform_targets: vec![],
            status: ContentStatus::Draft,
            approval: ApprovalStatus::Pending,
            scheduled_at: None,
            published_at: None,
            metadata: serde_json::json!({}),
        })
    }

    /// Adapt content for a specific platform (respecting length, tone, format).
    pub async fn adapt(
        &self,
        content: &ContentItem,
        platform: Platform,
        max_length: Option<usize>,
    ) -> Result<String> {
        let length_hint = max_length
            .map(|n| format!("Maximum length: {n} characters. "))
            .unwrap_or_default();
        let prompt = format!(
            "Adapt the following content for {platform:?}. {length_hint}\
             Keep the core message but adjust tone and format.\n\n\
             ---\n{}\n---",
            content.body_markdown
        );
        let config = AgentConfig {
            profile_id: None,
            session_id: content.session_id,
            model: None,
            tools: vec![],
            instructions: Some("You are a social-media content strategist.".into()),
            budget_limit: None,
            metadata: serde_json::json!({}),
        };
        let run_id = self.agent.create(config).await?;
        let output = self.agent.run(&run_id, Content::text(prompt)).await?;
        Ok(extract_text(&output.content))
    }

    /// Ask the LLM to review content quality and provide feedback.
    pub async fn review(&self, content: &ContentItem) -> Result<ContentReview> {
        let prompt = format!(
            "Review the following content for quality, clarity, and engagement.\n\
             Score it 0.0-1.0 and list suggestions.\n\n---\n{}\n---",
            content.body_markdown
        );
        let config = AgentConfig {
            profile_id: None,
            session_id: content.session_id,
            model: None,
            tools: vec![],
            instructions: Some("You are an editorial reviewer.".into()),
            budget_limit: None,
            metadata: serde_json::json!({}),
        };
        let run_id = self.agent.create(config).await?;
        let output = self.agent.run(&run_id, Content::text(prompt)).await?;
        let text = extract_text(&output.content);

        // Parse a simple score from the response; fall back to 0.7.
        let score = text
            .lines()
            .find_map(|l| {
                l.split_whitespace().find_map(|w| {
                    w.trim_matches(|c: char| !c.is_ascii_digit() && c != '.')
                        .parse::<f64>()
                        .ok()
                        .filter(|&v| (0.0..=1.0).contains(&v))
                })
            })
            .unwrap_or(0.7);

        Ok(ContentReview {
            score,
            suggestions: text
                .lines()
                .filter(|l| l.starts_with("- "))
                .map(String::from)
                .collect(),
            approved: score >= 0.6,
        })
    }
}

/// Extract plain text from a [`Content`] value.
fn extract_text(content: &Content) -> String {
    content
        .parts
        .iter()
        .filter_map(|p| match p {
            Part::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

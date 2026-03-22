//! Source adapter trait and implementations for opportunity discovery.

use async_trait::async_trait;
use rusvel_core::{OpportunitySource, Result, RusvelError};
use serde::{Deserialize, Serialize};

/// A raw opportunity as scraped from a source, before scoring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawOpportunity {
    pub title: String,
    pub description: String,
    pub url: Option<String>,
    pub budget: Option<String>,
    pub skills: Vec<String>,
    pub posted_at: Option<String>,
    pub source_data: serde_json::Value,
}

/// Trait for opportunity sources that can be scanned.
#[async_trait]
pub trait HarvestSource: Send + Sync {
    /// Human-readable name of this source.
    fn name(&self) -> &str;

    /// Which source kind this maps to.
    fn source_kind(&self) -> OpportunitySource;

    /// Scan the source and return raw opportunities.
    async fn scan(&self) -> Result<Vec<RawOpportunity>>;
}

// ── MockSource ─────────────────────────────────────────────────────

/// Returns hardcoded test opportunities for development and testing.
pub struct MockSource;

#[async_trait]
impl HarvestSource for MockSource {
    fn name(&self) -> &str {
        "mock"
    }

    fn source_kind(&self) -> OpportunitySource {
        OpportunitySource::Manual
    }

    async fn scan(&self) -> Result<Vec<RawOpportunity>> {
        Ok(vec![
            RawOpportunity {
                title: "Build a REST API in Rust".into(),
                description: "Need an experienced Rust developer to build a REST API \
                              with Axum. Must know async, SQL, and testing."
                    .into(),
                url: Some("https://example.com/job/1".into()),
                budget: Some("$5000".into()),
                skills: vec!["rust".into(), "axum".into(), "sql".into()],
                posted_at: Some("2026-03-20".into()),
                source_data: serde_json::json!({"mock": true}),
            },
            RawOpportunity {
                title: "React Native Mobile App".into(),
                description: "Looking for a React Native developer for a mobile app. \
                              Experience with TypeScript and Firebase required."
                    .into(),
                url: Some("https://example.com/job/2".into()),
                budget: Some("$8000".into()),
                skills: vec!["react-native".into(), "typescript".into(), "firebase".into()],
                posted_at: Some("2026-03-19".into()),
                source_data: serde_json::json!({"mock": true}),
            },
            RawOpportunity {
                title: "CLI Tool for Data Processing".into(),
                description: "Build a command-line tool in Rust for processing CSV files. \
                              Should support streaming and be well-tested."
                    .into(),
                url: Some("https://example.com/job/3".into()),
                budget: Some("$3000".into()),
                skills: vec!["rust".into(), "cli".into(), "data-processing".into()],
                posted_at: Some("2026-03-21".into()),
                source_data: serde_json::json!({"mock": true}),
            },
        ])
    }
}

// ── RssSource ──────────────────────────────────────────────────────

/// Fetches and parses RSS/Atom feeds for job listings.
pub struct RssSource {
    pub name: String,
    pub url: String,
    pub source_kind: OpportunitySource,
}

impl RssSource {
    pub fn new(name: impl Into<String>, url: impl Into<String>, kind: OpportunitySource) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            source_kind: kind,
        }
    }

    /// Extract text content between XML tags using simple string matching.
    fn extract_tag(xml: &str, tag: &str) -> Option<String> {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        let start = xml.find(&open)?;
        let after_open = xml[start..].find('>')? + start + 1;
        let end = xml[after_open..].find(&close)? + after_open;
        let content = xml[after_open..end].trim();
        // Strip CDATA wrapper if present
        let content = content
            .strip_prefix("<![CDATA[")
            .and_then(|s| s.strip_suffix("]]>"))
            .unwrap_or(content);
        Some(content.to_string())
    }

    /// Parse RSS XML into raw opportunities.
    fn parse_rss(xml: &str) -> Vec<RawOpportunity> {
        let mut results = Vec::new();
        let mut search_from = 0;

        while let Some(item_start) = xml[search_from..].find("<item") {
            let abs_start = search_from + item_start;
            let Some(item_end_rel) = xml[abs_start..].find("</item>") else {
                break;
            };
            let item_xml = &xml[abs_start..abs_start + item_end_rel + 7];

            let title = Self::extract_tag(item_xml, "title").unwrap_or_default();
            let description = Self::extract_tag(item_xml, "description").unwrap_or_default();
            let link = Self::extract_tag(item_xml, "link");
            let pub_date = Self::extract_tag(item_xml, "pubDate");

            if !title.is_empty() {
                results.push(RawOpportunity {
                    title,
                    description,
                    url: link,
                    budget: None,
                    skills: Vec::new(),
                    posted_at: pub_date,
                    source_data: serde_json::json!({"rss": true}),
                });
            }

            search_from = abs_start + item_end_rel + 7;
        }

        results
    }
}

#[async_trait]
impl HarvestSource for RssSource {
    fn name(&self) -> &str {
        &self.name
    }

    fn source_kind(&self) -> OpportunitySource {
        self.source_kind.clone()
    }

    async fn scan(&self) -> Result<Vec<RawOpportunity>> {
        let body = reqwest::get(&self.url)
            .await
            .map_err(|e| RusvelError::Internal(format!("RSS fetch failed: {e}")))?
            .text()
            .await
            .map_err(|e| RusvelError::Internal(format!("RSS read failed: {e}")))?;

        Ok(Self::parse_rss(&body))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rss_parse_extracts_items() {
        let xml = r#"<?xml version="1.0"?>
        <rss><channel>
            <item>
                <title>Rust Developer Needed</title>
                <description>Build a CLI tool</description>
                <link>https://example.com/1</link>
                <pubDate>Mon, 20 Mar 2026</pubDate>
            </item>
            <item>
                <title><![CDATA[Python ML Engineer]]></title>
                <description><![CDATA[Train models]]></description>
                <link>https://example.com/2</link>
            </item>
        </channel></rss>"#;

        let items = RssSource::parse_rss(xml);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "Rust Developer Needed");
        assert_eq!(items[1].title, "Python ML Engineer");
        assert_eq!(items[0].url, Some("https://example.com/1".into()));
    }
}

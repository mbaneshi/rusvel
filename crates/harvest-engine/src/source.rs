//! Source adapter trait and implementations for opportunity discovery.

use std::sync::OnceLock;

use async_trait::async_trait;
use regex::Regex;
use reqwest::Url;
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

impl MockSource {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl HarvestSource for MockSource {
    fn name(&self) -> &'static str {
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
                skills: vec![
                    "react-native".into(),
                    "typescript".into(),
                    "firebase".into(),
                ],
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
                let mut row = RawOpportunity {
                    title,
                    description,
                    url: link,
                    budget: None,
                    skills: Vec::new(),
                    posted_at: pub_date,
                    source_data: serde_json::json!({"rss": true}),
                };
                enrich_from_description(&mut row);
                results.push(row);
            }

            search_from = abs_start + item_end_rel + 7;
        }

        results
    }
}

/// Best-effort budget and skills extraction from RSS description HTML / text.
pub fn enrich_from_description(raw: &mut RawOpportunity) {
    let plain = strip_html_tags(&raw.description);
    let combined = format!("{} {}", raw.title, plain);

    static BUDGET: OnceLock<Regex> = OnceLock::new();
    let budget_re = BUDGET.get_or_init(|| {
        Regex::new(r"\$[\d,]+(?:\.\d{2})?(?:\s*-\s*\$?[\d,]+(?:\.\d{2})?)?").expect("budget regex")
    });
    if raw.budget.is_none()
        && let Some(m) = budget_re.find(&combined)
    {
        raw.budget = Some(m.as_str().to_string());
    }

    if raw.skills.is_empty() {
        let lower = combined.to_lowercase();
        if let Some(idx) = lower.find("skills:").or_else(|| lower.find("skills :")) {
            let tail = combined[idx..]
                .split_once(':')
                .map(|(_, r)| r)
                .unwrap_or(&combined[idx..]);
            let end = tail.find('\n').unwrap_or(tail.len());
            let part = tail[..end].trim();
            raw.skills = part
                .split(|c| c == ',' || c == '·' || c == '|')
                .map(|s| s.trim().trim_matches(|c| c == ' ' || c == '\t'))
                .filter(|s| !s.is_empty() && s.len() < 48)
                .take(16)
                .map(String::from)
                .collect();
        }
    }

    static TAG: OnceLock<Regex> = OnceLock::new();
    let tag_re =
        TAG.get_or_init(|| Regex::new(r"(?i)<a[^>]*>([^<]{2,40})</a>").expect("tag regex"));
    if raw.skills.len() < 3 {
        for cap in tag_re.captures_iter(&raw.description) {
            if let Some(m) = cap.get(1) {
                let s = m.as_str().trim();
                if s.chars().any(|c| c.is_alphabetic()) && !raw.skills.contains(&s.to_string()) {
                    raw.skills.push(s.to_string());
                }
                if raw.skills.len() >= 12 {
                    break;
                }
            }
        }
    }
}

fn strip_html_tags(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(c),
            _ => {}
        }
    }
    out
}

/// Upwork job RSS (`q` + optional skills, sort by recency).
pub struct UpworkRssSource {
    inner: RssSource,
}

impl UpworkRssSource {
    pub fn new(query: String, skills: Vec<String>) -> Result<Self> {
        let mut q = query.trim().to_string();
        for s in skills {
            if !s.is_empty() {
                q.push(' ');
                q.push_str(s.trim());
            }
        }
        let mut url = Url::parse("https://www.upwork.com/ab/feed/jobs/rss")
            .map_err(|e| RusvelError::Internal(format!("invalid Upwork URL: {e}")))?;
        url.query_pairs_mut()
            .append_pair("q", q.trim())
            .append_pair("sort", "recency");
        Ok(Self {
            inner: RssSource::new("upwork_rss", url.to_string(), OpportunitySource::Upwork),
        })
    }
}

#[async_trait]
impl HarvestSource for UpworkRssSource {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn source_kind(&self) -> OpportunitySource {
        self.inner.source_kind()
    }

    async fn scan(&self) -> Result<Vec<RawOpportunity>> {
        self.inner.scan().await
    }
}

/// Freelancer.com project RSS.
pub struct FreelancerRssSource {
    inner: RssSource,
}

impl FreelancerRssSource {
    pub fn new(query: String, skills: Vec<String>) -> Result<Self> {
        let mut q = query.trim().to_string();
        for s in skills {
            if !s.is_empty() {
                q.push(' ');
                q.push_str(s.trim());
            }
        }
        let mut url = Url::parse("https://www.freelancer.com/rss/projects")
            .map_err(|e| RusvelError::Internal(format!("invalid Freelancer URL: {e}")))?;
        url.query_pairs_mut().append_pair("query", q.trim());
        Ok(Self {
            inner: RssSource::new(
                "freelancer_rss",
                url.to_string(),
                OpportunitySource::Freelancer,
            ),
        })
    }
}

#[async_trait]
impl HarvestSource for FreelancerRssSource {
    fn name(&self) -> &str {
        self.inner.name()
    }

    fn source_kind(&self) -> OpportunitySource {
        self.inner.source_kind()
    }

    async fn scan(&self) -> Result<Vec<RawOpportunity>> {
        self.inner.scan().await
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

    #[test]
    fn enrich_extracts_budget_and_skills_line() {
        let mut raw = RawOpportunity {
            title: "Gig".into(),
            description: "<p>Budget: $2,500 - $4,000</p><p>Skills: Rust, Tokio, SQL</p>".into(),
            url: None,
            budget: None,
            skills: vec![],
            posted_at: None,
            source_data: serde_json::json!({}),
        };
        enrich_from_description(&mut raw);
        assert_eq!(raw.budget.as_deref(), Some("$2,500 - $4,000"));
        assert!(raw.skills.iter().any(|s| s.contains("Rust")));
    }

    #[test]
    fn upwork_rss_url_contains_query() {
        let u = UpworkRssSource::new("rust developer".into(), vec!["wasm".into()]).unwrap();
        assert!(u.inner.url.contains("upwork.com"));
        assert!(u.inner.url.contains("q=rust"));
        assert!(u.inner.url.contains("wasm") || u.inner.url.contains("developer"));
    }

    #[test]
    fn freelancer_rss_url_contains_query() {
        let u = FreelancerRssSource::new("react".into(), vec![]).unwrap();
        assert!(u.inner.url.contains("freelancer.com"));
        assert!(u.inner.url.contains("query="));
    }
}

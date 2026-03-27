//! Browser-based harvest source using [`BrowserPort`](rusvel_core::ports::BrowserPort).
//!
//! Configure a listing page URL and either the default page-level extract script
//! ([`DEFAULT_CDP_EXTRACT_JS`]) or [`extract_js_listing_cards`] for CSS-selector-driven
//! extraction from repeated DOM nodes.
//!
//! When no browser is configured or any CDP step fails, scanning falls back to [`MockSource`](crate::source::MockSource).

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rusvel_core::ports::BrowserPort;
use rusvel_core::{OpportunitySource, Result, RusvelError};
use serde::Deserialize;
use tokio::time::sleep;
use tracing::warn;

use crate::source::{HarvestSource, MockSource, RawOpportunity};

/// Default JS: returns one listing synthesized from the loaded page (`JSON.stringify([...])`).
pub const DEFAULT_CDP_EXTRACT_JS: &str = r#"(function(){
  try {
    var t = document.body ? document.body.innerText.slice(0, 4000) : "";
    return JSON.stringify([{
      title: (document.title || "CDP listing").slice(0, 500),
      description: t,
      url: location.href,
      budget: null,
      skills: [],
      posted_at: null,
      source_data: { cdp: true }
    }]);
  } catch(e) {
    return "[]";
  }
})()"#;

/// Build extraction JS that walks `listing_selector` nodes and reads title/description/link via nested selectors.
///
/// Evaluates to a JSON array string compatible with [`parse_evaluate_to_rows`]. Intended for job boards that
/// render repeated cards (e.g. `article.job`, `.listing-row`).
pub fn extract_js_listing_cards(
    listing_selector: &str,
    title_selector: &str,
    description_selector: &str,
    link_selector: Option<&str>,
) -> String {
    let list_s = serde_json::to_string(listing_selector).expect("selector serializes");
    let title_s = serde_json::to_string(title_selector).expect("selector serializes");
    let desc_s = serde_json::to_string(description_selector).expect("selector serializes");
    let link_js = match link_selector {
        Some(ls) => {
            let l = serde_json::to_string(ls).expect("selector serializes");
            format!(
                "var le = el.querySelector({l}); row.url = le ? (le.href || (le.textContent || '').trim() || null) : null;"
            )
        }
        None => "row.url = null;".to_string(),
    };
    format!(
        r#"(function(){{
  try {{
    var nodes = document.querySelectorAll({list_s});
    var out = [];
    for (var i = 0; i < nodes.length; i++) {{
      var el = nodes[i];
      var t = el.querySelector({title_s});
      var d = el.querySelector({desc_s});
      var row = {{
        title: (t && t.textContent) ? t.textContent.trim() : "",
        description: (d && d.textContent) ? d.textContent.trim().slice(0, 8000) : "",
        url: null,
        budget: null,
        skills: [],
        posted_at: null,
        source_data: {{ "selector": true }}
      }};
      {link_js}
      if (row.title) out.push(row);
    }}
    return JSON.stringify(out);
  }} catch (e) {{
    return "[]";
  }}
}})()"#,
        list_s = list_s,
        title_s = title_s,
        desc_s = desc_s,
        link_js = link_js
    )
}

/// CDP-driven source: navigate to a URL and evaluate JS to produce `RawOpportunity` rows.
pub struct CdpSource {
    browser: Option<Arc<dyn BrowserPort>>,
    endpoint: String,
    listing_url: String,
    extract_js: String,
}

impl CdpSource {
    /// `listing_url` is typically the job board search or listing page to open.
    pub fn new(
        browser: Option<Arc<dyn BrowserPort>>,
        endpoint: impl Into<String>,
        listing_url: impl Into<String>,
    ) -> Self {
        Self {
            browser,
            endpoint: endpoint.into(),
            listing_url: listing_url.into(),
            extract_js: DEFAULT_CDP_EXTRACT_JS.to_string(),
        }
    }

    /// Override the extraction script (must evaluate to a JSON array of listing objects).
    pub fn with_extract_js(mut self, js: impl Into<String>) -> Self {
        self.extract_js = js.into();
        self
    }

    async fn scan_with_browser(
        &self,
        browser: &Arc<dyn BrowserPort>,
    ) -> Result<Vec<RawOpportunity>> {
        browser.connect(&self.endpoint).await?;
        let tabs = browser.tabs().await?;
        let tab_id = tabs.first().map(|t| t.id.as_str()).ok_or_else(|| {
            RusvelError::Internal("no browser tabs — open a page in Chrome".into())
        })?;
        browser.navigate(tab_id, &self.listing_url).await?;
        sleep(Duration::from_millis(800)).await;
        let evaluated = browser.evaluate_js(tab_id, &self.extract_js).await?;
        parse_evaluate_to_rows(evaluated)
    }

    async fn scan_fallback(&self) -> Result<Vec<RawOpportunity>> {
        MockSource::new().scan().await
    }
}

fn parse_evaluate_to_rows(evaluated: serde_json::Value) -> Result<Vec<RawOpportunity>> {
    let text = extract_eval_string(&evaluated)?;
    let rows: Vec<serde_json::Value> = serde_json::from_str(&text).map_err(|e| {
        RusvelError::Internal(format!(
            "CDP extract did not return a JSON array string: {e}"
        ))
    })?;
    let mut out = Vec::new();
    for v in rows {
        if let Ok(row) = serde_json::from_value::<CdpRow>(v) {
            out.push(row.into_raw());
        }
    }
    if out.is_empty() {
        return Err(RusvelError::Internal(
            "CDP extract returned no valid listings".into(),
        ));
    }
    Ok(out)
}

fn extract_eval_string(v: &serde_json::Value) -> Result<String> {
    if let Some(s) = v.as_str() {
        return Ok(s.to_string());
    }
    if let Some(obj) = v.as_object() {
        if let Some(r) = obj.get("result").and_then(|x| x.as_object()) {
            if let Some(val) = r.get("value") {
                if let Some(s) = val.as_str() {
                    return Ok(s.to_string());
                }
            }
        }
    }
    serde_json::to_string(v).map_err(|e| RusvelError::Serialization(e.to_string()))
}

#[derive(Debug, Deserialize)]
struct CdpRow {
    title: String,
    description: String,
    #[serde(default)]
    url: Option<String>,
    #[serde(default)]
    budget: Option<String>,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    posted_at: Option<String>,
    #[serde(default)]
    source_data: serde_json::Value,
}

impl CdpRow {
    fn into_raw(self) -> RawOpportunity {
        RawOpportunity {
            title: self.title,
            description: self.description,
            url: self.url,
            budget: self.budget,
            skills: self.skills,
            posted_at: self.posted_at,
            source_data: if self.source_data.is_null() {
                serde_json::json!({ "cdp": true })
            } else {
                self.source_data
            },
        }
    }
}

#[async_trait]
impl HarvestSource for CdpSource {
    fn name(&self) -> &str {
        "cdp"
    }

    fn source_kind(&self) -> OpportunitySource {
        OpportunitySource::Other("cdp".into())
    }

    async fn scan(&self) -> Result<Vec<RawOpportunity>> {
        let Some(browser) = &self.browser else {
            warn!("CDP harvest: no BrowserPort; using MockSource");
            return self.scan_fallback().await;
        };
        match self.scan_with_browser(browser).await {
            Ok(rows) => Ok(rows),
            Err(e) => {
                warn!("CDP harvest failed ({e}); falling back to MockSource");
                self.scan_fallback().await
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusvel_core::domain::BrowserEvent;
    use rusvel_core::domain::TabInfo;

    struct MockBrowser {
        eval_result: serde_json::Value,
    }

    #[async_trait::async_trait]
    impl BrowserPort for MockBrowser {
        async fn connect(&self, _endpoint: &str) -> Result<()> {
            Ok(())
        }

        async fn disconnect(&self) -> Result<()> {
            Ok(())
        }

        async fn tabs(&self) -> Result<Vec<TabInfo>> {
            Ok(vec![TabInfo {
                id: "tab1".into(),
                url: "about:blank".into(),
                title: String::new(),
                platform: None,
                metadata: Default::default(),
            }])
        }

        async fn observe(
            &self,
            _tab_id: &str,
        ) -> Result<tokio::sync::broadcast::Receiver<BrowserEvent>> {
            let (_tx, rx) = tokio::sync::broadcast::channel(1);
            Ok(rx)
        }

        async fn evaluate_js(&self, _tab_id: &str, _script: &str) -> Result<serde_json::Value> {
            Ok(self.eval_result.clone())
        }

        async fn navigate(&self, _tab_id: &str, _url: &str) -> Result<()> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn cdp_parses_json_array_from_evaluate_string() {
        let json = r#"[{"title":"Rust gig","description":"Do API work","url":"https://ex.com/1","budget":"$100","skills":["rust"],"posted_at":null,"source_data":{}}]"#;
        let browser = Arc::new(MockBrowser {
            eval_result: serde_json::Value::String(json.into()),
        });
        let src = CdpSource::new(
            Some(browser),
            "http://127.0.0.1:9222",
            "https://example.com/jobs",
        );
        let rows = src.scan().await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title, "Rust gig");
        assert_eq!(rows[0].url.as_deref(), Some("https://ex.com/1"));
    }

    #[tokio::test]
    async fn cdp_without_browser_falls_back_to_mock() {
        let src = CdpSource::new(None, "http://127.0.0.1:9222", "https://example.com");
        let rows = src.scan().await.unwrap();
        assert!(!rows.is_empty());
    }

    #[test]
    fn listing_cards_js_includes_selector_queries() {
        let js = extract_js_listing_cards(".job-card", "h3.title", "p.blurb", Some("a.permalink"));
        assert!(js.contains("querySelectorAll"));
        assert!(js.contains(".job-card"));
        assert!(js.contains("h3.title"));
        assert!(js.contains("a.permalink"));
    }

    /// Simulates `evaluate_js` output as if a listing page had two DOM cards (HTML parsed in-browser).
    #[tokio::test]
    async fn cdp_selector_extract_mock_returns_multiple_listings() {
        let json = r#"[{"title":"Scraped A","description":"From card one","url":"https://board.example/1","budget":null,"skills":[],"posted_at":null,"source_data":{"selector":true}},{"title":"Scraped B","description":"From card two","url":"https://board.example/2","budget":null,"skills":[],"posted_at":null,"source_data":{"selector":true}}]"#;
        let browser = Arc::new(MockBrowser {
            eval_result: serde_json::Value::String(json.into()),
        });
        let js = extract_js_listing_cards("article", "h2", ".body", None);
        let src = CdpSource::new(
            Some(browser),
            "http://127.0.0.1:9222",
            "https://board.example/jobs",
        )
        .with_extract_js(js);
        let rows = src.scan().await.unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].title, "Scraped A");
        assert_eq!(rows[1].url.as_deref(), Some("https://board.example/2"));
    }

    #[tokio::test]
    async fn cdp_parses_eval_result_wrapped_like_chrome_runtime() {
        let inner = r#"[{"title":"Wrapped","description":"x","url":null,"budget":null,"skills":[],"posted_at":null,"source_data":{}}]"#;
        let wrapped = serde_json::json!({
            "result": { "value": inner }
        });
        let browser = Arc::new(MockBrowser {
            eval_result: wrapped,
        });
        let src = CdpSource::new(
            Some(browser),
            "http://127.0.0.1:9222",
            "https://example.com",
        );
        let rows = src.scan().await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].title, "Wrapped");
    }
}

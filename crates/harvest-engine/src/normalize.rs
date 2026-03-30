//! Normalize capture payloads: stable keys, content hashing, shared ingest + scan paths.

use rusvel_core::OpportunitySource;
use rusvel_core::Result;
use rusvel_core::RusvelError;
use serde_json::Value;
use sha2::{Digest, Sha256};

use crate::source::RawOpportunity;

/// Build a [`RawOpportunity`] from a loose JSON job object (ingest API, extension, CDP rows).
pub fn raw_from_job_json(row: &Value) -> RawOpportunity {
    let title = row
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled opportunity");
    RawOpportunity {
        title: title.into(),
        description: row
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .into(),
        url: row.get("url").and_then(|v| v.as_str()).map(String::from),
        budget: row.get("budget").and_then(|v| {
            v.as_str()
                .map(String::from)
                .or_else(|| v.as_f64().map(|n| format!("${n:.0}")))
        }),
        skills: row
            .get("skills")
            .and_then(|v| v.as_array())
            .map(|a| {
                a.iter()
                    .filter_map(|x| x.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        posted_at: row
            .get("posted_at")
            .and_then(|v| v.as_str())
            .map(String::from),
        source_data: row.clone(),
        ..Default::default()
    }
}

/// Map API `platform` string to [`OpportunitySource`].
pub fn opportunity_source_from_platform(platform: &str) -> Result<OpportunitySource> {
    match platform.to_lowercase().as_str() {
        "upwork" => Ok(OpportunitySource::Upwork),
        "freelancer" => Ok(OpportunitySource::Freelancer),
        "linkedin" => Ok(OpportunitySource::LinkedIn),
        "github" => Ok(OpportunitySource::GitHub),
        "manual" => Ok(OpportunitySource::Manual),
        s if !s.is_empty() => Ok(OpportunitySource::Other(s.to_string())),
        _ => Err(RusvelError::Validation(
            "platform must be non-empty (e.g. upwork, freelancer)".into(),
        )),
    }
}

/// Same mapping as [`opportunity_source_from_platform`] but never fails (unknown → [`OpportunitySource::Other`]).
pub fn opportunity_source_from_str_loose(s: &str) -> OpportunitySource {
    opportunity_source_from_platform(s).unwrap_or_else(|_| OpportunitySource::Other(s.to_string()))
}

/// Ingest row: [`raw_from_job_json`] plus stable keys; per-row `"source"` overrides `default_source`.
pub fn raw_from_ingest_row(
    row: &Value,
    default_source: OpportunitySource,
) -> (OpportunitySource, RawOpportunity) {
    let mut raw = raw_from_job_json(row);
    if raw.description.is_empty() {
        if let Some(s) = row.get("snippet").and_then(|v| v.as_str()) {
            raw.description = s.into();
        }
    }
    if raw.platform_job_key.is_none() {
        raw.platform_job_key = platform_key_from_value(row);
    }
    if raw.upstream_score.is_none() {
        raw.upstream_score = upstream_score_from_value(row);
    }
    let source = row
        .get("source")
        .and_then(|v| v.as_str())
        .map(opportunity_source_from_str_loose)
        .unwrap_or(default_source);
    (source, raw)
}

/// SHA-256 hex of normalized title, description, and budget string (change detection).
pub fn content_hash_for_raw(raw: &RawOpportunity) -> String {
    let budget = raw.budget.as_deref().unwrap_or("");
    let norm = format!(
        "{}\n{}\n{}",
        raw.title.trim().to_lowercase(),
        raw.description.trim().to_lowercase(),
        budget.trim()
    );
    let mut h = Sha256::new();
    h.update(norm.as_bytes());
    hex::encode(h.finalize())
}

/// Writes [`RawOpportunity::content_hash`].
pub fn apply_content_hash(raw: &mut RawOpportunity) {
    raw.content_hash = Some(content_hash_for_raw(raw));
}

/// Best-effort platform job id from JSON (`platform_job_key`, `job_key`, or `id`).
pub fn platform_key_from_value(row: &Value) -> Option<String> {
    row.get("platform_job_key")
        .or_else(|| row.get("job_key"))
        .or_else(|| row.get("id"))
        .and_then(|v| v.as_str().map(str::trim))
        .filter(|s| !s.is_empty())
        .map(String::from)
}

/// Pull `upstream_score` from JSON (0–100 or 0.0–1.0 → stored as 0.0–1.0).
pub fn upstream_score_from_value(row: &Value) -> Option<f64> {
    let v = row.get("upstream_score").or_else(|| row.get("score"))?;
    let n = v.as_f64()?;
    Some(if n > 1.0 { n / 100.0 } else { n })
}

/// Fill `platform_job_key` and optional client/budget hints from a generic job object.
pub fn enrich_raw_from_json(raw: &mut RawOpportunity, source: OpportunitySource) {
    if raw.platform_job_key.is_none() {
        raw.platform_job_key = platform_key_from_value(&raw.source_data);
    }
    if raw.upstream_score.is_none() {
        raw.upstream_score = upstream_score_from_value(&raw.source_data);
    }
    let d = &raw.source_data;
    if raw.budget_min.is_none() {
        raw.budget_min = d.get("budget_min").and_then(|x| x.as_f64());
    }
    if raw.budget_max.is_none() {
        raw.budget_max = d.get("budget_max").and_then(|x| x.as_f64());
    }
    if raw.budget_currency.is_none() {
        raw.budget_currency = d
            .get("budget_currency")
            .and_then(|x| x.as_str())
            .map(String::from);
    }
    if raw.hourly.is_none() {
        raw.hourly = d.get("hourly").and_then(|x| x.as_bool());
    }
    if raw.client_hire_rate.is_none() {
        raw.client_hire_rate = d.get("client_hire_rate").and_then(|x| x.as_f64());
    }
    if raw.client_total_spent.is_none() {
        raw.client_total_spent = d.get("client_total_spent").and_then(|x| x.as_f64());
    }
    if raw.payment_verified.is_none() {
        raw.payment_verified = d.get("payment_verified").and_then(|x| x.as_bool());
    }
    if raw.proposal_count.is_none() {
        raw.proposal_count = d
            .get("proposal_count")
            .or_else(|| d.get("proposals_count"))
            .and_then(|x| x.as_u64())
            .map(|n| n as u32);
    }
    let _ = source;
}

/// Run enrich + content hash (call before scoring).
pub fn prepare_raw(raw: &mut RawOpportunity, source: OpportunitySource) {
    enrich_raw_from_json(raw, source);
    apply_content_hash(raw);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_hash_stable() {
        let raw = RawOpportunity {
            title: "A".into(),
            description: "B".into(),
            url: None,
            budget: Some("$100".into()),
            skills: vec![],
            posted_at: None,
            source_data: Value::Null,
            ..Default::default()
        };
        let h1 = content_hash_for_raw(&raw);
        let h2 = content_hash_for_raw(&raw);
        assert_eq!(h1, h2);
        assert_eq!(h1.len(), 64);
    }

    #[test]
    fn platform_key_prefers_explicit() {
        let v = serde_json::json!({"platform_job_key": "abc", "id": "z"});
        assert_eq!(platform_key_from_value(&v).as_deref(), Some("abc"));
    }

    #[test]
    fn raw_from_ingest_row_respects_per_row_source() {
        let row = serde_json::json!({
            "title": "T",
            "description": "D",
            "source": "freelancer"
        });
        let (src, raw) = raw_from_ingest_row(&row, OpportunitySource::Manual);
        assert_eq!(src, OpportunitySource::Freelancer);
        assert_eq!(raw.title, "T");
    }

    #[test]
    fn raw_from_ingest_row_falls_back_to_default_source() {
        let row = serde_json::json!({"title": "X", "description": "Y"});
        let (src, _) = raw_from_ingest_row(&row, OpportunitySource::Upwork);
        assert_eq!(src, OpportunitySource::Upwork);
    }
}

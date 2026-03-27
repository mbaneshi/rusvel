//! Aggregation helpers for [`MetricPoint`] rows produced by [`crate::CostTrackingLlm`]
//! (`llm.cost_usd` with `dept:` / `session:` tags).

use std::collections::HashMap;

use rusvel_core::domain::MetricPoint;

/// Metric name recorded for each LLM call (USD estimate).
pub const LLM_COST_METRIC_NAME: &str = "llm.cost_usd";

/// Department id from `dept:<id>` tag, or `"unknown"` when missing.
#[must_use]
pub fn department_from_tags(tags: &[String]) -> String {
    tags.iter()
        .find_map(|t| t.strip_prefix("dept:").map(str::to_string))
        .unwrap_or_else(|| "unknown".into())
}

/// Session id from `session:<uuid>` tag.
#[must_use]
pub fn session_from_tags(tags: &[String]) -> Option<String> {
    tags.iter()
        .find_map(|t| t.strip_prefix("session:").map(str::to_string))
}

/// Aggregated spend from raw metric points (typically `name == llm.cost_usd`).
#[derive(Debug, Clone, Default)]
pub struct SpendAggregation {
    pub by_department: HashMap<String, f64>,
    pub by_session: HashMap<String, f64>,
    pub total_usd: f64,
}

/// Sum [`MetricPoint::value`] across departments and sessions.
#[must_use]
pub fn aggregate_spend(points: &[MetricPoint]) -> SpendAggregation {
    let mut by_department: HashMap<String, f64> = HashMap::new();
    let mut by_session: HashMap<String, f64> = HashMap::new();
    for p in points {
        let d = department_from_tags(&p.tags);
        *by_department.entry(d).or_insert(0.0) += p.value;
        if let Some(sid) = session_from_tags(&p.tags) {
            *by_session.entry(sid).or_insert(0.0) += p.value;
        }
    }
    let total_usd = points.iter().map(|p| p.value).sum();
    SpendAggregation {
        by_department,
        by_session,
        total_usd,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use rusvel_core::domain::MetricPoint;

    fn point(tags: Vec<&str>, value: f64) -> MetricPoint {
        MetricPoint {
            name: LLM_COST_METRIC_NAME.into(),
            value,
            tags: tags.into_iter().map(String::from).collect(),
            recorded_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    #[test]
    fn aggregate_groups_by_department_and_session() {
        let pts = vec![
            point(vec!["dept:harvest", "session:aa"], 1.0),
            point(vec!["dept:harvest", "session:aa"], 2.0),
            point(vec!["dept:content", "session:bb"], 0.5),
        ];
        let a = aggregate_spend(&pts);
        assert!((a.total_usd - 3.5).abs() < f64::EPSILON);
        assert!((a.by_department["harvest"] - 3.0).abs() < f64::EPSILON);
        assert!((a.by_department["content"] - 0.5).abs() < f64::EPSILON);
        assert!((a.by_session["aa"] - 3.0).abs() < f64::EPSILON);
        assert!((a.by_session["bb"] - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn unknown_department_without_tag() {
        let pts = vec![point(vec!["session:xx"], 0.1)];
        let a = aggregate_spend(&pts);
        assert!((a.by_department["unknown"] - 0.1).abs() < f64::EPSILON);
    }
}

//! Session context bundle for department chat (S-045) — assembled by the API, formatted here.

use serde::{Deserialize, Serialize};

/// Standard sections for agent system prompts (goals + recent activity).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextPack {
    pub session_name: String,
    pub goal_titles: Vec<String>,
    pub recent_event_summaries: Vec<String>,
    pub metrics_summary: Option<String>,
}

/// Markdown-style block appended after rules and before department actions.
pub fn to_prompt_section(p: &ContextPack) -> String {
    if p.session_name.is_empty()
        && p.goal_titles.is_empty()
        && p.recent_event_summaries.is_empty()
        && p.metrics_summary.as_ref().map_or(true, |m| m.is_empty())
    {
        return String::new();
    }
    let mut s = String::from("\n\n--- Session context ---\n");
    if !p.session_name.is_empty() {
        s.push_str(&format!("Workspace session: **{}**\n", p.session_name));
    }
    if !p.goal_titles.is_empty() {
        s.push_str("Active goals:\n");
        for g in &p.goal_titles {
            s.push_str(&format!("- {g}\n"));
        }
    }
    if !p.recent_event_summaries.is_empty() {
        s.push_str(
            "Recent events (newest last in list may be oldest first in store — skim all):\n",
        );
        for e in &p.recent_event_summaries {
            s.push_str(&format!("- {e}\n"));
        }
    }
    if let Some(ref m) = p.metrics_summary {
        if !m.is_empty() {
            s.push_str(&format!("Quick metrics: {m}\n"));
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_includes_sections() {
        let p = ContextPack {
            session_name: "alpha".into(),
            goal_titles: vec!["ship v1".into()],
            recent_event_summaries: vec!["harvest.scan.completed: ok".into()],
            metrics_summary: None,
        };
        let t = to_prompt_section(&p);
        assert!(t.contains("alpha"));
        assert!(t.contains("ship v1"));
        assert!(t.contains("harvest.scan"));
    }
}

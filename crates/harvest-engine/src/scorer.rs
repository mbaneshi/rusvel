//! Opportunity scoring — keyword-based fallback with optional LLM scoring.

use std::sync::Arc;

use rusvel_core::ports::AgentPort;
use rusvel_core::{AgentConfig, Content, Result, RusvelError, SessionId};
use serde::{Deserialize, Serialize};

use crate::source::RawOpportunity;

/// A scored opportunity ready for pipeline insertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredOpportunity {
    pub raw: RawOpportunity,
    pub score: f64,
    pub reasoning: String,
}

/// Scores raw opportunities by skill match, budget fit, and complexity.
pub struct OpportunityScorer {
    agent: Option<Arc<dyn AgentPort>>,
    skills: Vec<String>,
    min_budget: Option<f64>,
}

impl OpportunityScorer {
    pub fn new(
        agent: Option<Arc<dyn AgentPort>>,
        skills: Vec<String>,
        min_budget: Option<f64>,
    ) -> Self {
        Self {
            agent,
            skills,
            min_budget,
        }
    }

    /// Score a raw opportunity. Uses LLM if available, otherwise keyword matching.
    pub async fn score(&self, raw: &RawOpportunity) -> Result<ScoredOpportunity> {
        if let Some(agent) = &self.agent {
            match self.score_with_agent(agent, raw).await {
                Ok(scored) => return Ok(scored),
                Err(e) => {
                    tracing::warn!("LLM scoring failed, falling back to keywords: {e}");
                }
            }
        }

        Ok(self.score_with_keywords(raw))
    }

    /// Keyword-based fallback scoring.
    fn score_with_keywords(&self, raw: &RawOpportunity) -> ScoredOpportunity {
        if self.skills.is_empty() {
            return ScoredOpportunity {
                raw: raw.clone(),
                score: 0.5,
                reasoning: "No skills configured for matching".into(),
            };
        }

        let text = format!("{} {}", raw.title, raw.description).to_lowercase();
        let raw_skills_lower: Vec<String> = raw.skills.iter().map(|s| s.to_lowercase()).collect();

        let mut matched = Vec::new();
        for skill in &self.skills {
            let skill_lower = skill.to_lowercase();
            if text.contains(&skill_lower) || raw_skills_lower.contains(&skill_lower) {
                matched.push(skill.clone());
            }
        }

        let skill_score = matched.len() as f64 / self.skills.len() as f64;

        // Budget bonus: if budget is stated and meets minimum, small boost
        let budget_bonus = if let (Some(budget_str), Some(min)) =
            (&raw.budget, self.min_budget)
        {
            let numeric: f64 = budget_str
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '.')
                .collect::<String>()
                .parse()
                .unwrap_or(0.0);
            if numeric >= min { 0.1 } else { 0.0 }
        } else {
            0.0
        };

        let score = (skill_score + budget_bonus).min(1.0);
        let reasoning = if matched.is_empty() {
            "No skill matches found".into()
        } else {
            format!("Matched skills: {}", matched.join(", "))
        };

        ScoredOpportunity {
            raw: raw.clone(),
            score,
            reasoning,
        }
    }

    /// LLM-based scoring via `AgentPort`.
    async fn score_with_agent(
        &self,
        agent: &Arc<dyn AgentPort>,
        raw: &RawOpportunity,
    ) -> Result<ScoredOpportunity> {
        let prompt = format!(
            "Score this freelance opportunity from 0.0 to 1.0 based on fit.\n\n\
             My skills: {}\n\
             Minimum budget: {}\n\n\
             Title: {}\n\
             Description: {}\n\
             Budget: {}\n\
             Required skills: {}\n\n\
             Respond with ONLY a JSON object: {{\"score\": 0.X, \"reasoning\": \"...\"}}",
            self.skills.join(", "),
            self.min_budget.map_or_else(|| "none".into(), |b| format!("${b}")),
            raw.title,
            raw.description,
            raw.budget.as_deref().unwrap_or("not specified"),
            raw.skills.join(", "),
        );

        let session_id = SessionId::new();
        let config = AgentConfig {
            profile_id: None,
            session_id,
            model: None,
            tools: vec![],
            instructions: Some("You are an opportunity scoring assistant.".into()),
            budget_limit: Some(0.01),
            metadata: serde_json::json!({}),
        };

        let run_id = agent.create(config).await?;
        let output = agent.run(&run_id, Content::text(prompt)).await?;

        let text = output
            .content
            .parts
            .iter()
            .filter_map(|p| match p {
                rusvel_core::Part::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect::<String>();

        // Try to parse JSON from the response
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| RusvelError::Internal(format!("Failed to parse LLM score: {e}")))?;

        let score = parsed["score"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0);
        let reasoning = parsed["reasoning"]
            .as_str()
            .unwrap_or("LLM scored")
            .to_string();

        Ok(ScoredOpportunity {
            raw: raw.clone(),
            score,
            reasoning,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn keyword_scoring_matches_skills() {
        let scorer = OpportunityScorer::new(
            None,
            vec!["rust".into(), "axum".into(), "python".into()],
            Some(2000.0),
        );

        let raw = RawOpportunity {
            title: "Build a REST API in Rust".into(),
            description: "Need Rust and Axum experience".into(),
            url: None,
            budget: Some("$5000".into()),
            skills: vec!["rust".into(), "axum".into()],
            posted_at: None,
            source_data: serde_json::json!({}),
        };

        let scored = scorer.score(&raw).await.unwrap();
        // 2/3 skills match + 0.1 budget bonus = ~0.77
        assert!(scored.score > 0.7, "score was {}", scored.score);
        assert!(scored.reasoning.contains("rust"));
    }

    #[tokio::test]
    async fn keyword_scoring_no_match() {
        let scorer = OpportunityScorer::new(None, vec!["golang".into()], None);

        let raw = RawOpportunity {
            title: "React Native App".into(),
            description: "Need React Native experience".into(),
            url: None,
            budget: None,
            skills: vec!["react-native".into()],
            posted_at: None,
            source_data: serde_json::json!({}),
        };

        let scored = scorer.score(&raw).await.unwrap();
        assert!(scored.score < 0.1, "score was {}", scored.score);
    }
}

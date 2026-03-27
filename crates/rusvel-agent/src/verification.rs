//! Self-correction verification for agent outputs.
//!
//! Provides a chain of [`VerificationStep`]s that inspect an agent's output
//! before it is returned to the caller. Steps can flag warnings or hard
//! failures, optionally suggesting a fix prompt for re-generation.

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use rusvel_core::domain::*;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::ports::LlmPort;

// ════════════════════════════════════════════════════════════════════
//  Result types
// ════════════════════════════════════════════════════════════════════

/// Outcome of a single verification step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationResult {
    Pass {
        confidence: f64,
    },
    Warn {
        issues: Vec<String>,
        confidence: f64,
    },
    Fail {
        issues: Vec<String>,
        suggested_fix: Option<String>,
    },
}

impl VerificationResult {
    pub fn is_pass(&self) -> bool {
        matches!(self, Self::Pass { .. })
    }

    pub fn is_fail(&self) -> bool {
        matches!(self, Self::Fail { .. })
    }
}

/// Context passed to each verification step.
#[derive(Debug, Clone)]
pub struct VerificationContext {
    pub department_id: String,
    pub tool_name: Option<String>,
    pub original_prompt: String,
}

// ════════════════════════════════════════════════════════════════════
//  Trait + chain
// ════════════════════════════════════════════════════════════════════

/// A single verification step in the chain.
#[async_trait]
pub trait VerificationStep: Send + Sync {
    fn name(&self) -> &str;

    async fn verify(&self, ctx: &VerificationContext, output: &str) -> Result<VerificationResult>;
}

/// Runs an ordered list of [`VerificationStep`]s, short-circuiting on the
/// first [`VerificationResult::Fail`].
pub struct VerificationChain {
    steps: Vec<Arc<dyn VerificationStep>>,
}

impl VerificationChain {
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    pub fn add(mut self, step: Arc<dyn VerificationStep>) -> Self {
        self.steps.push(step);
        self
    }

    /// Run all steps. Returns the first `Fail`, or collects all `Warn`/`Pass`.
    pub async fn run(
        &self,
        ctx: &VerificationContext,
        output: &str,
    ) -> Result<Vec<(String, VerificationResult)>> {
        let mut results = Vec::with_capacity(self.steps.len());
        for step in &self.steps {
            let result = step.verify(ctx, output).await?;
            let failed = result.is_fail();
            results.push((step.name().to_string(), result));
            if failed {
                break;
            }
        }
        Ok(results)
    }
}

impl Default for VerificationChain {
    fn default() -> Self {
        Self::new()
    }
}

// ════════════════════════════════════════════════════════════════════
//  Built-in steps
// ════════════════════════════════════════════════════════════════════

/// Sends the agent output to a fast/cheap LLM (e.g. Haiku) for critique.
pub struct LlmCritiqueStep {
    llm: Arc<dyn LlmPort>,
    model: ModelRef,
}

impl LlmCritiqueStep {
    pub fn new(llm: Arc<dyn LlmPort>, model: ModelRef) -> Self {
        Self { llm, model }
    }

    fn build_critique_request(&self, ctx: &VerificationContext, output: &str) -> LlmRequest {
        let system = format!(
            "You are a QA reviewer for the {} department. \
             Evaluate whether the following output correctly addresses the original prompt. \
             Respond with ONLY a JSON object: \
             {{\"pass\": true/false, \"confidence\": 0.0-1.0, \"issues\": [\"...\"]}}",
            ctx.department_id,
        );
        LlmRequest {
            model: self.model.clone(),
            messages: vec![
                LlmMessage {
                    role: LlmRole::System,
                    content: Content::text(system),
                },
                LlmMessage {
                    role: LlmRole::User,
                    content: Content::text(format!(
                        "Original prompt:\n{}\n\nAgent output:\n{}",
                        ctx.original_prompt, output
                    )),
                },
            ],
            tools: vec![],
            temperature: Some(0.0),
            max_tokens: Some(256),
            metadata: serde_json::json!({}),
        }
    }
}

#[async_trait]
impl VerificationStep for LlmCritiqueStep {
    fn name(&self) -> &str {
        "llm_critique"
    }

    async fn verify(&self, ctx: &VerificationContext, output: &str) -> Result<VerificationResult> {
        let req = self.build_critique_request(ctx, output);
        let resp = self.llm.generate(req).await?;

        let text = resp
            .content
            .parts
            .iter()
            .filter_map(|p| match p {
                Part::Text(t) => Some(t.as_str()),
                _ => None,
            })
            .collect::<String>();

        // Parse the JSON response; fall back to Pass on malformed output.
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .unwrap_or(serde_json::json!({"pass": true, "confidence": 0.5}));

        let pass = parsed["pass"].as_bool().unwrap_or(true);
        let confidence = parsed["confidence"].as_f64().unwrap_or(0.5);
        let issues: Vec<String> = parsed["issues"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        if pass && issues.is_empty() {
            Ok(VerificationResult::Pass { confidence })
        } else if pass {
            Ok(VerificationResult::Warn { issues, confidence })
        } else {
            Ok(VerificationResult::Fail {
                issues,
                suggested_fix: Some("Re-generate with the identified issues addressed.".into()),
            })
        }
    }
}

/// Checks agent output against a set of forbidden patterns (regex).
pub struct RulesComplianceStep {
    forbidden: Vec<(String, regex::Regex)>,
}

impl RulesComplianceStep {
    /// Create from a list of `(description, pattern)` pairs.
    pub fn new(patterns: Vec<(String, String)>) -> Result<Self> {
        let forbidden = patterns
            .into_iter()
            .map(|(desc, pat)| {
                let re = regex::Regex::new(&pat).map_err(|e| {
                    RusvelError::Config(format!("bad forbidden pattern '{pat}': {e}"))
                })?;
                Ok((desc, re))
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { forbidden })
    }
}

#[async_trait]
impl VerificationStep for RulesComplianceStep {
    fn name(&self) -> &str {
        "rules_compliance"
    }

    async fn verify(&self, _ctx: &VerificationContext, output: &str) -> Result<VerificationResult> {
        let mut issues = Vec::new();
        for (desc, re) in &self.forbidden {
            if re.is_match(output) {
                issues.push(desc.clone());
            }
        }
        if issues.is_empty() {
            Ok(VerificationResult::Pass { confidence: 1.0 })
        } else {
            Ok(VerificationResult::Fail {
                issues,
                suggested_fix: Some(
                    "Remove or rephrase content that violates compliance rules.".into(),
                ),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rules_compliance_passes_clean_output() {
        let step =
            RulesComplianceStep::new(vec![("no profanity".into(), r"(?i)\bbadword\b".into())])
                .unwrap();
        let ctx = VerificationContext {
            department_id: "content".into(),
            tool_name: None,
            original_prompt: "Write something nice".into(),
        };
        let result = step
            .verify(&ctx, "This is a perfectly fine output.")
            .await
            .unwrap();
        assert!(result.is_pass());
    }

    #[tokio::test]
    async fn rules_compliance_fails_on_forbidden() {
        let step =
            RulesComplianceStep::new(vec![("no secrets".into(), r"(?i)api[_\s]?key".into())])
                .unwrap();
        let ctx = VerificationContext {
            department_id: "code".into(),
            tool_name: None,
            original_prompt: "Generate config".into(),
        };
        let result = step
            .verify(&ctx, "Here is the API_KEY=abc123")
            .await
            .unwrap();
        assert!(result.is_fail());
    }

    #[tokio::test]
    async fn chain_short_circuits_on_fail() {
        let pass_step = RulesComplianceStep::new(vec![]).unwrap();
        let fail_step =
            RulesComplianceStep::new(vec![("always fail".into(), r".*".into())]).unwrap();

        let chain = VerificationChain::new()
            .add(Arc::new(pass_step))
            .add(Arc::new(fail_step));

        let ctx = VerificationContext {
            department_id: "test".into(),
            tool_name: None,
            original_prompt: "test".into(),
        };
        let results = chain.run(&ctx, "anything").await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results[0].1.is_pass());
        assert!(results[1].1.is_fail());
    }
}

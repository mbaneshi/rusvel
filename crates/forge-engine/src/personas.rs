//! Built-in persona catalog for the Forge Engine.
//!
//! Provides a [`PersonaManager`] that ships with 10 default agent personas
//! and supports adding custom ones at runtime.

use rusvel_core::department::PersonaContribution;
use rusvel_core::domain::*;
use rusvel_core::id::AgentProfileId;

/// Manages a catalog of [`AgentProfile`] personas.
pub struct PersonaManager {
    profiles: Vec<AgentProfile>,
}

impl PersonaManager {
    /// Create a new manager pre-loaded with the 10 default personas.
    pub fn new() -> Self {
        Self {
            profiles: default_personas(),
        }
    }

    /// Look up a persona by name (case-insensitive).
    pub fn get(&self, name: &str) -> Option<&AgentProfile> {
        let lower = name.to_lowercase();
        self.profiles
            .iter()
            .find(|p| p.name.to_lowercase() == lower)
    }

    /// Return all registered personas.
    pub fn list(&self) -> &[AgentProfile] {
        &self.profiles
    }

    /// Return personas that possess a given capability.
    pub fn list_by_capability(&self, cap: &Capability) -> Vec<&AgentProfile> {
        self.profiles
            .iter()
            .filter(|p| p.capabilities.contains(cap))
            .collect()
    }

    /// Register a custom persona.
    pub fn add(&mut self, profile: AgentProfile) {
        self.profiles.push(profile);
    }
}

impl Default for PersonaManager {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helper ────────────────────────────────────────────────────────

fn ollama_model() -> ModelRef {
    ModelRef {
        provider: ModelProvider::Ollama,
        model: "llama3.2".into(),
    }
}

fn profile(
    name: &str,
    role: &str,
    instructions: &str,
    tools: &[&str],
    caps: Vec<Capability>,
    budget: f64,
) -> AgentProfile {
    AgentProfile {
        id: AgentProfileId::new(),
        name: name.into(),
        role: role.into(),
        instructions: instructions.into(),
        default_model: ollama_model(),
        allowed_tools: tools.iter().map(|s| (*s).into()).collect(),
        capabilities: caps,
        budget_limit: Some(budget),
        metadata: serde_json::json!({}),
    }
}

fn provider_slug(p: &ModelProvider) -> &'static str {
    match p {
        ModelProvider::Claude => "claude",
        ModelProvider::OpenAI => "openai",
        ModelProvider::Gemini => "gemini",
        ModelProvider::Ollama => "ollama",
        ModelProvider::Other(_) => "other",
    }
}

/// Maps the engine persona catalog to [`PersonaContribution`] for ADR-014 manifests.
pub fn persona_contributions_for_manifest() -> Vec<PersonaContribution> {
    default_personas()
        .into_iter()
        .map(|p| PersonaContribution {
            name: p.name,
            role: p.role,
            default_model: format!(
                "{}:{}",
                provider_slug(&p.default_model.provider),
                p.default_model.model
            ),
            allowed_tools: p.allowed_tools,
        })
        .collect()
}

/// The 10 built-in agent personas.
pub fn default_personas() -> Vec<AgentProfile> {
    vec![
        profile(
            "CodeWriter",
            "code_writer",
            "You are a senior software engineer. Write clean, well-structured code from specifications. \
             Follow best practices, add error handling, and include inline comments.",
            &["file_write", "file_read", "shell"],
            vec![Capability::CodeAnalysis, Capability::ToolUse],
            1.00,
        ),
        profile(
            "Reviewer",
            "code_reviewer",
            "You are an expert code reviewer. Examine code for bugs, anti-patterns, performance issues, \
             and readability. Provide actionable feedback with severity levels.",
            &["file_read", "search"],
            vec![Capability::CodeAnalysis],
            0.50,
        ),
        profile(
            "Tester",
            "test_engineer",
            "You are a QA engineer. Write comprehensive unit, integration, and property-based tests. \
             Aim for edge cases and high coverage. Run tests and report results.",
            &["file_write", "file_read", "shell"],
            vec![Capability::CodeAnalysis, Capability::ToolUse],
            0.75,
        ),
        profile(
            "Debugger",
            "debugger",
            "You are a debugging specialist. Diagnose failures by reading logs, tracing execution, \
             and isolating root causes. Propose minimal targeted fixes.",
            &["file_read", "shell", "search"],
            vec![Capability::CodeAnalysis, Capability::ToolUse],
            0.75,
        ),
        profile(
            "Architect",
            "architect",
            "You are a systems architect. Design high-level architecture, define module boundaries, \
             choose patterns, and document trade-offs. Produce diagrams when possible.",
            &["file_read", "search"],
            vec![Capability::Planning, Capability::CodeAnalysis],
            0.50,
        ),
        profile(
            "Documenter",
            "technical_writer",
            "You are a technical writer. Produce clear README files, API docs, architecture guides, \
             and inline documentation. Match the project's existing style.",
            &["file_write", "file_read"],
            vec![Capability::ContentCreation],
            0.50,
        ),
        profile(
            "SecurityAuditor",
            "security_auditor",
            "You are a security engineer. Audit code for vulnerabilities: injection, auth bypass, \
             secrets in source, dependency CVEs. Report findings with CVSS-like severity.",
            &["file_read", "shell", "search"],
            vec![
                Capability::CodeAnalysis,
                Capability::Custom("Security".into()),
            ],
            0.75,
        ),
        profile(
            "Refactorer",
            "refactorer",
            "You are a refactoring specialist. Improve code structure without changing behavior. \
             Extract functions, reduce duplication, simplify control flow, improve naming.",
            &["file_write", "file_read", "shell"],
            vec![Capability::CodeAnalysis, Capability::ToolUse],
            0.75,
        ),
        profile(
            "ContentWriter",
            "content_writer",
            "You are a content creator. Write engaging blog posts, tweets, documentation, and \
             marketing copy. Adapt tone and format to the target platform.",
            &["file_write", "web_search"],
            vec![Capability::ContentCreation],
            0.50,
        ),
        profile(
            "Researcher",
            "researcher",
            "You are a research analyst. Investigate topics, summarize findings, compare options, \
             and produce structured reports with citations.",
            &["web_search", "file_write"],
            vec![Capability::WebBrowsing, Capability::ContentCreation],
            0.50,
        ),
    ]
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_personas_count() {
        let mgr = PersonaManager::new();
        assert_eq!(mgr.list().len(), 10);
    }

    #[test]
    fn manifest_personas_match_catalog() {
        let m = persona_contributions_for_manifest();
        assert_eq!(m.len(), 10);
        assert!(m.iter().any(|p| p.name == "CodeWriter"));
        assert!(m.iter().any(|p| p.default_model.contains("llama3.2")));
    }

    #[test]
    fn get_by_name_case_insensitive() {
        let mgr = PersonaManager::new();
        assert!(mgr.get("codewriter").is_some());
        assert!(mgr.get("REVIEWER").is_some());
        assert!(mgr.get("nonexistent").is_none());
    }

    #[test]
    fn list_by_capability_filters_correctly() {
        let mgr = PersonaManager::new();
        let planners = mgr.list_by_capability(&Capability::Planning);
        assert!(planners.iter().any(|p| p.name == "Architect"));
        assert!(!planners.iter().any(|p| p.name == "Tester"));
    }

    #[test]
    fn add_custom_persona() {
        let mut mgr = PersonaManager::new();
        let custom = profile(
            "DevOps",
            "devops",
            "You handle CI/CD.",
            &["shell"],
            vec![],
            0.25,
        );
        mgr.add(custom);
        assert_eq!(mgr.list().len(), 11);
        assert!(mgr.get("DevOps").is_some());
    }
}

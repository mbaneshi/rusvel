//! Agent persona catalog: named profiles with capabilities.
//!
//! Provides a registry of reusable [`AgentProfile`] personas that can
//! be looked up by name or filtered by [`Capability`].

use std::collections::HashMap;

use rusvel_core::domain::*;
use rusvel_core::id::AgentProfileId;

// ════════════════════════════════════════════════════════════════════
//  PersonaCatalog
// ════════════════════════════════════════════════════════════════════

/// An in-memory registry of named agent personas.
#[derive(Debug, Clone)]
pub struct PersonaCatalog {
    personas: HashMap<String, AgentProfile>,
}

impl PersonaCatalog {
    /// Build a catalog from a list of profiles, keyed by `profile.name`.
    pub fn from_profiles(profiles: Vec<AgentProfile>) -> Self {
        let personas = profiles.into_iter().map(|p| (p.name.clone(), p)).collect();
        Self { personas }
    }

    /// Deserialize a catalog from a JSON string (array of `AgentProfile`).
    pub fn from_json(json: &str) -> serde_json::Result<Self> {
        let profiles: Vec<AgentProfile> = serde_json::from_str(json)?;
        Ok(Self::from_profiles(profiles))
    }

    /// Look up a persona by name.
    pub fn get(&self, name: &str) -> Option<&AgentProfile> {
        self.personas.get(name)
    }

    /// List all personas.
    pub fn list(&self) -> Vec<&AgentProfile> {
        self.personas.values().collect()
    }

    /// List personas that have the given capability.
    pub fn list_by_capability(&self, cap: &Capability) -> Vec<&AgentProfile> {
        self.personas
            .values()
            .filter(|p| p.capabilities.contains(cap))
            .collect()
    }

    /// Number of personas in the catalog.
    pub fn len(&self) -> usize {
        self.personas.len()
    }

    /// Returns true if the catalog is empty.
    pub fn is_empty(&self) -> bool {
        self.personas.is_empty()
    }

    /// Build a catalog pre-loaded with 10 default personas.
    pub fn defaults() -> Self {
        Self::from_profiles(default_personas())
    }
}

// ════════════════════════════════════════════════════════════════════
//  Default personas
// ════════════════════════════════════════════════════════════════════

fn default_model() -> ModelRef {
    ModelRef {
        provider: ModelProvider::Claude,
        model: "claude-sonnet-4-20250514".into(),
    }
}

fn profile(name: &str, role: &str, instructions: &str, caps: Vec<Capability>) -> AgentProfile {
    AgentProfile {
        id: AgentProfileId::new(),
        name: name.into(),
        role: role.into(),
        instructions: instructions.into(),
        default_model: default_model(),
        allowed_tools: vec![],
        capabilities: caps,
        budget_limit: None,
        metadata: serde_json::json!({}),
    }
}

/// The 10 built-in personas.
fn default_personas() -> Vec<AgentProfile> {
    vec![
        profile(
            "CodeWriter",
            "Software Engineer",
            "Write clean, idiomatic code. Follow best practices.",
            vec![Capability::CodeAnalysis, Capability::ToolUse],
        ),
        profile(
            "Reviewer",
            "Code Reviewer",
            "Review code for correctness, style, and performance.",
            vec![Capability::CodeAnalysis],
        ),
        profile(
            "Tester",
            "QA Engineer",
            "Write comprehensive tests. Identify edge cases.",
            vec![Capability::CodeAnalysis],
        ),
        profile(
            "Debugger",
            "Debug Specialist",
            "Diagnose and fix bugs. Trace root causes.",
            vec![Capability::CodeAnalysis, Capability::ToolUse],
        ),
        profile(
            "Architect",
            "System Architect",
            "Design systems. Evaluate trade-offs. Write ADRs.",
            vec![Capability::Planning, Capability::CodeAnalysis],
        ),
        profile(
            "Documenter",
            "Technical Writer",
            "Write clear documentation, guides, and API references.",
            vec![Capability::ContentCreation],
        ),
        profile(
            "SecurityAuditor",
            "Security Engineer",
            "Audit code for vulnerabilities. Recommend mitigations.",
            vec![Capability::CodeAnalysis],
        ),
        profile(
            "Refactorer",
            "Refactoring Specialist",
            "Improve code structure without changing behavior.",
            vec![Capability::CodeAnalysis],
        ),
        profile(
            "ContentWriter",
            "Content Creator",
            "Write blog posts, social media content, and copy.",
            vec![Capability::ContentCreation],
        ),
        profile(
            "Researcher",
            "Research Analyst",
            "Research topics. Summarize findings. Cite sources.",
            vec![Capability::WebBrowsing, Capability::ContentCreation],
        ),
    ]
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_has_ten_personas() {
        let catalog = PersonaCatalog::defaults();
        assert_eq!(catalog.len(), 10);
    }

    #[test]
    fn get_by_name() {
        let catalog = PersonaCatalog::defaults();
        let cw = catalog.get("CodeWriter").unwrap();
        assert_eq!(cw.role, "Software Engineer");
        assert!(catalog.get("NonExistent").is_none());
    }

    #[test]
    fn list_returns_all() {
        let catalog = PersonaCatalog::defaults();
        assert_eq!(catalog.list().len(), 10);
    }

    #[test]
    fn filter_by_capability() {
        let catalog = PersonaCatalog::defaults();
        let code_analysts = catalog.list_by_capability(&Capability::CodeAnalysis);
        // CodeWriter, Reviewer, Tester, Debugger, Architect, SecurityAuditor, Refactorer
        assert_eq!(code_analysts.len(), 7);

        let content = catalog.list_by_capability(&Capability::ContentCreation);
        // Documenter, ContentWriter, Researcher
        assert_eq!(content.len(), 3);
    }

    #[test]
    fn from_profiles_and_json_roundtrip() {
        let profiles = vec![
            profile("A", "role-a", "do A", vec![Capability::Planning]),
            profile("B", "role-b", "do B", vec![Capability::ToolUse]),
        ];
        let json = serde_json::to_string(&profiles).unwrap();
        let catalog = PersonaCatalog::from_json(&json).unwrap();
        assert_eq!(catalog.len(), 2);
        assert!(catalog.get("A").is_some());
        assert!(catalog.get("B").is_some());
    }

    #[test]
    fn empty_catalog() {
        let catalog = PersonaCatalog::from_profiles(vec![]);
        assert!(catalog.is_empty());
        assert_eq!(catalog.list().len(), 0);
    }
}

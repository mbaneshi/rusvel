//! Static manifest for the Forge Department.

use rusvel_core::config::LayeredConfig;
use rusvel_core::department::*;

pub fn forge_manifest() -> DepartmentManifest {
    DepartmentManifest {
        id: "forge".into(),
        name: "Forge Department".into(),
        description: "Agent orchestration, goal planning, mission management, daily plans, reviews"
            .into(),
        icon: "=".into(),
        color: "indigo".into(),

        system_prompt: concat!(
            "You are the Forge department of RUSVEL.\n\n",
            "Focus: agent orchestration, goal planning, mission management, ",
            "daily plans, reviews.",
        )
        .into(),

        capabilities: vec!["planning".into(), "orchestration".into()],

        quick_actions: vec![
            QuickAction {
                label: "Daily plan".into(),
                prompt:
                    "Generate today's mission plan based on active goals and priorities.".into(),
            },
            QuickAction {
                label: "Review progress".into(),
                prompt: concat!(
                    "Review progress on all active goals. ",
                    "Summarize completed, in-progress, and blocked items."
                )
                .into(),
            },
            QuickAction {
                label: "Set new goal".into(),
                prompt:
                    "Help me define a new strategic goal. Ask me for context and desired outcome."
                        .into(),
            },
        ],

        routes: vec![],

        commands: vec![
            CommandContribution {
                name: "mission".into(),
                description: "Generate today's mission plan".into(),
                args: vec![ArgDef {
                    name: "subcommand".into(),
                    description: "today | goals | review".into(),
                    required: true,
                    default: Some("today".into()),
                }],
            },
        ],

        tools: vec![],

        personas: vec![
            PersonaContribution {
                name: "code_writer".into(),
                role: "Senior software engineer".into(),
                default_model: "sonnet".into(),
                allowed_tools: vec![
                    "file_write".into(),
                    "file_read".into(),
                    "shell".into(),
                ],
            },
            PersonaContribution {
                name: "code_reviewer".into(),
                role: "Expert code reviewer".into(),
                default_model: "sonnet".into(),
                allowed_tools: vec!["file_read".into(), "search".into()],
            },
            PersonaContribution {
                name: "test_engineer".into(),
                role: "QA and test engineer".into(),
                default_model: "sonnet".into(),
                allowed_tools: vec![
                    "file_write".into(),
                    "file_read".into(),
                    "shell".into(),
                ],
            },
            PersonaContribution {
                name: "architect".into(),
                role: "Systems architect".into(),
                default_model: "opus".into(),
                allowed_tools: vec!["file_read".into(), "search".into()],
            },
            PersonaContribution {
                name: "researcher".into(),
                role: "Technical researcher".into(),
                default_model: "sonnet".into(),
                allowed_tools: vec!["web_search".into(), "web_fetch".into()],
            },
        ],

        skills: vec![SkillContribution {
            name: "Daily Standup".into(),
            description: "Summarize progress and plan for the day".into(),
            prompt_template: concat!(
                "Based on recent activity, generate a standup summary:\n",
                "- What was accomplished yesterday\n",
                "- What is planned for today\n",
                "- Any blockers"
            )
            .into(),
        }],

        rules: vec![],

        jobs: vec![],

        ui: UiContribution {
            tabs: vec![
                "actions".into(),
                "agents".into(),
                "workflows".into(),
                "skills".into(),
                "rules".into(),
                "events".into(),
            ],
            dashboard_cards: vec![DashboardCard {
                title: "Mission Status".into(),
                description: "Active goals, today's plan, recent reviews".into(),
                size: "large".into(),
            }],
            has_settings: false,
            custom_components: vec![],
        },

        events_produced: vec![
            "forge.mission_planned".into(),
            "forge.goal_created".into(),
            "forge.goal_updated".into(),
            "forge.review_completed".into(),
            "forge.agent_hired".into(),
        ],
        events_consumed: vec![],

        requires_ports: vec![
            PortRequirement { port: "AgentPort".into(), optional: false },
            PortRequirement { port: "EventPort".into(), optional: false },
            PortRequirement { port: "MemoryPort".into(), optional: false },
            PortRequirement { port: "StoragePort".into(), optional: false },
            PortRequirement { port: "JobPort".into(), optional: false },
            PortRequirement { port: "SessionPort".into(), optional: false },
            PortRequirement { port: "ConfigPort".into(), optional: false },
        ],

        depends_on: vec![],

        config_schema: serde_json::json!({}),
        default_config: LayeredConfig::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_has_correct_id() {
        let m = forge_manifest();
        assert_eq!(m.id, "forge");
        assert_eq!(m.icon, "=");
        assert_eq!(m.color, "indigo");
    }

    #[test]
    fn manifest_declares_personas() {
        let m = forge_manifest();
        assert_eq!(m.personas.len(), 5);
        assert!(m.personas.iter().any(|p| p.name == "architect"));
    }

    #[test]
    fn manifest_requires_7_ports() {
        let m = forge_manifest();
        assert_eq!(m.requires_ports.len(), 7);
        assert!(m.requires_ports.iter().all(|p| !p.optional));
    }

    #[test]
    fn manifest_has_no_deps() {
        let m = forge_manifest();
        assert!(m.depends_on.is_empty());
    }
}

//! Capability Engine — `!build` command interceptor for department chat.
//!
//! When a user sends `!build <type>: <description>` in any department,
//! this module intercepts the message, asks Claude to generate the entity
//! JSON, parses it, persists it via `ObjectStore`, and returns a confirmation.

use std::sync::Arc;

use rusvel_core::domain::{AgentProfile, ModelProvider, ModelRef};
use rusvel_core::id::AgentProfileId;
use rusvel_core::ports::StoragePort;
use rusvel_llm::stream::ClaudeCliStreamer;
use rusvel_llm::stream::StreamEvent;
use tracing::debug;

use crate::hooks::HookDefinition;
use crate::mcp_servers::McpServerConfig;
use crate::rules::RuleDefinition;
use crate::skills::SkillDefinition;

/// Supported entity types for !build.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEntityType {
    Agent,
    Skill,
    Rule,
    Mcp,
    Hook,
}

impl BuildEntityType {
    fn from_str(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "agent" => Some(Self::Agent),
            "skill" => Some(Self::Skill),
            "rule" => Some(Self::Rule),
            "mcp" | "mcp-server" | "mcp_server" | "mcpserver" => Some(Self::Mcp),
            "hook" => Some(Self::Hook),
            _ => None,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Agent => "agent",
            Self::Skill => "skill",
            Self::Rule => "rule",
            Self::Mcp => "MCP server",
            Self::Hook => "hook",
        }
    }
}

/// Parsed !build command.
pub struct BuildCommand {
    pub entity_type: BuildEntityType,
    pub description: String,
}

/// Check if a message is a !build command and parse it.
pub fn parse_build_command(message: &str) -> Option<BuildCommand> {
    let trimmed = message.trim();
    if !trimmed.starts_with("!build") {
        return None;
    }

    // Strip "!build" prefix
    let rest = trimmed[6..].trim();

    // Try to parse "type: description" or "type description"
    // Formats: "!build agent: description" or "!build agent description"
    let (type_str, description) = if let Some(colon_pos) = rest.find(':') {
        let t = rest[..colon_pos].trim();
        let d = rest[colon_pos + 1..].trim();
        (t, d.to_string())
    } else {
        // Split on first whitespace
        let mut parts = rest.splitn(2, char::is_whitespace);
        let t = parts.next().unwrap_or("").trim();
        let d = parts.next().unwrap_or("").trim().to_string();
        (t, d)
    };

    let entity_type = BuildEntityType::from_str(type_str)?;

    if description.is_empty() {
        return None;
    }

    Some(BuildCommand {
        entity_type,
        description,
    })
}

/// Execute a !build command: ask Claude to generate the entity, persist it, return confirmation.
pub async fn execute_build(
    cmd: &BuildCommand,
    engine: &str,
    storage: &Arc<dyn StoragePort>,
) -> Result<String, String> {
    let prompt = build_generation_prompt(cmd, engine);

    debug!(entity_type = ?cmd.entity_type, description = %cmd.description, "executing !build");

    // Use ClaudeCliStreamer to get Claude's response (collect full text)
    let streamer = ClaudeCliStreamer::new();
    let args = vec![
        "--model".to_string(),
        "sonnet".to_string(),
        "--max-turns".to_string(),
        "1".to_string(),
        "--permission-mode".to_string(),
        "plan".to_string(),
    ];
    let mut rx = streamer.stream_with_args(&prompt, &args);

    let mut full_text = String::new();
    while let Some(event) = rx.recv().await {
        match event {
            StreamEvent::Delta { text } => {
                full_text.push_str(&text);
            }
            StreamEvent::Done {
                full_text: text, ..
            } => {
                full_text = text;
                break;
            }
            StreamEvent::Error { message } => {
                return Err(format!("Claude error: {message}"));
            }
        }
    }

    if full_text.is_empty() {
        return Err("No response from Claude".into());
    }

    // Extract JSON from the response (Claude may wrap it in markdown code blocks)
    let json_str = extract_json(&full_text)
        .ok_or_else(|| format!("Could not extract JSON from Claude response:\n{full_text}"))?;

    // Parse and persist based on entity type
    let confirmation = match cmd.entity_type {
        BuildEntityType::Agent => persist_agent(json_str, engine, storage).await?,
        BuildEntityType::Skill => persist_skill(json_str, engine, storage).await?,
        BuildEntityType::Rule => persist_rule(json_str, engine, storage).await?,
        BuildEntityType::Mcp => persist_mcp(json_str, engine, storage).await?,
        BuildEntityType::Hook => persist_hook(json_str, engine, storage).await?,
    };

    Ok(confirmation)
}

/// Build the prompt that asks Claude to generate the entity JSON.
fn build_generation_prompt(cmd: &BuildCommand, engine: &str) -> String {
    let schema_hint = match cmd.entity_type {
        BuildEntityType::Agent => {
            r#"Generate a JSON object with these exact fields:
{
  "name": "short-kebab-case-name",
  "role": "one-line role description",
  "instructions": "detailed system prompt / instructions for this agent (multi-paragraph is fine)",
  "model": "opus",
  "allowed_tools": ["Tool1", "Tool2"],
  "capabilities": [],
  "budget_limit": null
}

For model, choose: "opus" for complex/expert tasks, "sonnet" for general tasks, "haiku" for simple/fast tasks.
For allowed_tools, choose from: Read, Write, Edit, Bash, Glob, Grep, WebFetch, WebSearch, Agent.
Think carefully about which tools this agent needs."#
        }

        BuildEntityType::Skill => {
            r#"Generate a JSON object with these exact fields:
{
  "name": "short-kebab-case-name",
  "description": "what this skill does",
  "prompt_template": "The full prompt template. Use {{input}} as placeholder for user input."
}

The prompt_template should be a detailed, well-crafted prompt that produces high-quality results."#
        }

        BuildEntityType::Rule => {
            r#"Generate a JSON object with these exact fields:
{
  "name": "short-kebab-case-name",
  "content": "The rule text that will be injected into the system prompt.",
  "enabled": true
}

The content should be clear, actionable, and specific."#
        }

        BuildEntityType::Mcp => {
            r#"Generate a JSON object with these exact fields:
{
  "name": "short-kebab-case-name",
  "description": "what this MCP server provides",
  "server_type": "stdio",
  "command": "npx",
  "args": ["-y", "@modelcontextprotocol/server-xxx"],
  "url": null,
  "env": {},
  "enabled": true
}

For server_type, choose: "stdio" for local processes, "http"/"sse" for remote servers.
If stdio, provide command and args. If http/sse, provide url instead."#
        }

        BuildEntityType::Hook => {
            r#"Generate a JSON object with these exact fields:
{
  "name": "short-kebab-case-name",
  "event": "PreToolUse",
  "matcher": "*",
  "hook_type": "prompt",
  "action": "The prompt or command to execute when this hook fires.",
  "enabled": true
}

For event, choose from: SessionStart, SessionEnd, PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, Notification, SubagentStart, SubagentStop, TaskCompleted, ConfigChange, PreCompact, PostCompact, UserPromptSubmit, Stop, StopFailure.
For hook_type, choose: "command" (shell), "http" (webhook), or "prompt" (LLM prompt).
For matcher, use a regex/glob pattern (e.g., "Bash" for PreToolUse to match bash commands, "*" for all)."#
        }
    };

    format!(
        "You are a RUSVEL entity generator. Generate a single JSON object based on the user's description.\n\n\
         Entity type: {entity_type}\n\
         Department: {engine}\n\
         User description: {description}\n\n\
         {schema_hint}\n\n\
         IMPORTANT: Output ONLY the JSON object. No explanation, no markdown, no extra text.\n\
         Just the raw JSON.",
        entity_type = cmd.entity_type.label(),
        engine = engine,
        description = cmd.description,
    )
}

/// Extract JSON from Claude's response, handling potential markdown code blocks.
pub fn extract_json(text: &str) -> Option<&str> {
    let trimmed = text.trim();

    // Try direct parse first
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        return Some(trimmed);
    }

    // Try to extract from ```json ... ``` or ``` ... ```
    if let Some(start) = trimmed.find("```") {
        let after_backticks = &trimmed[start + 3..];
        // Skip optional language identifier
        let json_start = if after_backticks.starts_with("json") {
            after_backticks.find('\n').map(|i| start + 3 + i + 1)?
        } else if after_backticks.starts_with('\n') {
            start + 4
        } else {
            start + 3
        };
        let rest = &trimmed[json_start..];
        if let Some(end) = rest.find("```") {
            let candidate = rest[..end].trim();
            if candidate.starts_with('{') && candidate.ends_with('}') {
                return Some(candidate);
            }
        }
    }

    // Try to find first { and last }
    let first_brace = trimmed.find('{')?;
    let last_brace = trimmed.rfind('}')?;
    if first_brace < last_brace {
        return Some(&trimmed[first_brace..=last_brace]);
    }

    None
}

// ── Persistence helpers ──────────────────────────────────────

async fn persist_agent(
    json_str: &str,
    engine: &str,
    storage: &Arc<dyn StoragePort>,
) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct AgentGen {
        name: String,
        role: Option<String>,
        instructions: Option<String>,
        model: Option<String>,
        allowed_tools: Option<Vec<String>>,
        #[allow(dead_code)]
        capabilities: Option<Vec<String>>,
        budget_limit: Option<f64>,
    }

    let generated: AgentGen = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse agent JSON: {e}\nJSON: {json_str}"))?;

    let agent = AgentProfile {
        id: AgentProfileId::new(),
        name: generated.name.clone(),
        role: generated.role.unwrap_or_default(),
        instructions: generated.instructions.unwrap_or_default(),
        default_model: ModelRef {
            provider: ModelProvider::Claude,
            model: generated.model.unwrap_or_else(|| "sonnet".into()),
        },
        allowed_tools: generated.allowed_tools.unwrap_or_default(),
        capabilities: vec![],
        budget_limit: generated.budget_limit,
        metadata: serde_json::json!({ "engine": engine, "created_by": "!build" }),
    };

    let id = agent.id.to_string();
    storage
        .objects()
        .put(
            "agents",
            &id,
            serde_json::to_value(&agent).map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "**Agent created: {}**\n\n\
         - **ID:** `{}`\n\
         - **Role:** {}\n\
         - **Model:** {}\n\
         - **Tools:** {}\n\
         - **Department:** {}\n\n\
         You can now mention this agent with `@{}` in chat.",
        agent.name,
        id,
        agent.role,
        agent.default_model.model,
        if agent.allowed_tools.is_empty() {
            "all".into()
        } else {
            agent.allowed_tools.join(", ")
        },
        engine,
        agent.name,
    ))
}

async fn persist_skill(
    json_str: &str,
    engine: &str,
    storage: &Arc<dyn StoragePort>,
) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct SkillGen {
        name: String,
        description: Option<String>,
        prompt_template: Option<String>,
    }

    let generated: SkillGen = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse skill JSON: {e}\nJSON: {json_str}"))?;

    let id = uuid::Uuid::now_v7().to_string();
    let skill = SkillDefinition {
        id: id.clone(),
        name: generated.name.clone(),
        description: generated.description.unwrap_or_default(),
        prompt_template: generated.prompt_template.unwrap_or_default(),
        metadata: serde_json::json!({ "engine": engine, "created_by": "!build" }),
    };

    storage
        .objects()
        .put(
            "skills",
            &id,
            serde_json::to_value(&skill).map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "**Skill created: {}**\n\n\
         - **ID:** `{}`\n\
         - **Description:** {}\n\
         - **Department:** {}\n\n\
         Prompt template length: {} chars",
        skill.name,
        id,
        skill.description,
        engine,
        skill.prompt_template.len(),
    ))
}

async fn persist_rule(
    json_str: &str,
    engine: &str,
    storage: &Arc<dyn StoragePort>,
) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct RuleGen {
        name: String,
        content: Option<String>,
        enabled: Option<bool>,
    }

    let generated: RuleGen = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse rule JSON: {e}\nJSON: {json_str}"))?;

    let id = uuid::Uuid::now_v7().to_string();
    let rule = RuleDefinition {
        id: id.clone(),
        name: generated.name.clone(),
        content: generated.content.unwrap_or_default(),
        enabled: generated.enabled.unwrap_or(true),
        metadata: serde_json::json!({ "engine": engine, "created_by": "!build" }),
    };

    storage
        .objects()
        .put(
            "rules",
            &id,
            serde_json::to_value(&rule).map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "**Rule created: {}**\n\n\
         - **ID:** `{}`\n\
         - **Enabled:** {}\n\
         - **Department:** {}\n\
         - **Content:** {}\n\n\
         This rule is now {} in the {} department system prompt.",
        rule.name,
        id,
        rule.enabled,
        engine,
        if rule.content.len() > 100 {
            format!("{}...", &rule.content[..97])
        } else {
            rule.content.clone()
        },
        if rule.enabled { "active" } else { "disabled" },
        engine,
    ))
}

async fn persist_mcp(
    json_str: &str,
    engine: &str,
    storage: &Arc<dyn StoragePort>,
) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct McpGen {
        name: String,
        description: Option<String>,
        server_type: Option<String>,
        command: Option<String>,
        args: Option<Vec<String>>,
        url: Option<String>,
        env: Option<serde_json::Value>,
        enabled: Option<bool>,
    }

    let generated: McpGen = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse MCP server JSON: {e}\nJSON: {json_str}"))?;

    let id = uuid::Uuid::now_v7().to_string();
    let server = McpServerConfig {
        id: id.clone(),
        name: generated.name.clone(),
        description: generated.description.unwrap_or_default(),
        server_type: generated.server_type.unwrap_or_else(|| "stdio".into()),
        command: generated.command,
        args: generated.args.unwrap_or_default(),
        url: generated.url,
        env: generated.env.unwrap_or_else(|| serde_json::json!({})),
        enabled: generated.enabled.unwrap_or(true),
        metadata: serde_json::json!({ "engine": engine, "created_by": "!build" }),
    };

    storage
        .objects()
        .put(
            "mcp_servers",
            &id,
            serde_json::to_value(&server).map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "**MCP Server created: {}**\n\n\
         - **ID:** `{}`\n\
         - **Type:** {}\n\
         - **Command:** {}\n\
         - **Enabled:** {}\n\
         - **Department:** {}\n\n\
         This MCP server will be available in the next chat session.",
        server.name,
        id,
        server.server_type,
        server.command.as_deref().unwrap_or("N/A"),
        server.enabled,
        engine,
    ))
}

async fn persist_hook(
    json_str: &str,
    engine: &str,
    storage: &Arc<dyn StoragePort>,
) -> Result<String, String> {
    #[derive(serde::Deserialize)]
    struct HookGen {
        name: String,
        event: Option<String>,
        matcher: Option<String>,
        hook_type: Option<String>,
        action: Option<String>,
        enabled: Option<bool>,
    }

    let generated: HookGen = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse hook JSON: {e}\nJSON: {json_str}"))?;

    let id = uuid::Uuid::now_v7().to_string();
    let hook = HookDefinition {
        id: id.clone(),
        name: generated.name.clone(),
        event: generated.event.unwrap_or_else(|| "PreToolUse".into()),
        matcher: generated.matcher.unwrap_or_else(|| "*".into()),
        hook_type: generated.hook_type.unwrap_or_else(|| "prompt".into()),
        action: generated.action.unwrap_or_default(),
        enabled: generated.enabled.unwrap_or(true),
        metadata: serde_json::json!({ "engine": engine, "created_by": "!build" }),
    };

    storage
        .objects()
        .put(
            "hooks",
            &id,
            serde_json::to_value(&hook).map_err(|e| e.to_string())?,
        )
        .await
        .map_err(|e| e.to_string())?;

    Ok(format!(
        "**Hook created: {}**\n\n\
         - **ID:** `{}`\n\
         - **Event:** {}\n\
         - **Matcher:** {}\n\
         - **Type:** {}\n\
         - **Enabled:** {}\n\
         - **Department:** {}",
        hook.name, id, hook.event, hook.matcher, hook.hook_type, hook.enabled, engine,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_build_with_colon() {
        let cmd = parse_build_command("!build agent: a security auditor").unwrap();
        assert_eq!(cmd.entity_type, BuildEntityType::Agent);
        assert_eq!(cmd.description, "a security auditor");
    }

    #[test]
    fn parse_build_without_colon() {
        let cmd = parse_build_command("!build skill deploy to production").unwrap();
        assert_eq!(cmd.entity_type, BuildEntityType::Skill);
        assert_eq!(cmd.description, "deploy to production");
    }

    #[test]
    fn parse_build_rule() {
        let cmd = parse_build_command("!build rule: always use async/await").unwrap();
        assert_eq!(cmd.entity_type, BuildEntityType::Rule);
        assert_eq!(cmd.description, "always use async/await");
    }

    #[test]
    fn parse_build_mcp() {
        let cmd = parse_build_command("!build mcp: filesystem server").unwrap();
        assert_eq!(cmd.entity_type, BuildEntityType::Mcp);
        assert_eq!(cmd.description, "filesystem server");
    }

    #[test]
    fn parse_build_hook() {
        let cmd =
            parse_build_command("!build hook: validate bash commands before execution").unwrap();
        assert_eq!(cmd.entity_type, BuildEntityType::Hook);
        assert_eq!(cmd.description, "validate bash commands before execution");
    }

    #[test]
    fn parse_rejects_unknown_type() {
        assert!(parse_build_command("!build widget: something").is_none());
    }

    #[test]
    fn parse_rejects_no_description() {
        assert!(parse_build_command("!build agent:").is_none());
        assert!(parse_build_command("!build agent").is_none());
    }

    #[test]
    fn parse_rejects_non_build() {
        assert!(parse_build_command("hello world").is_none());
    }

    #[test]
    fn extract_json_raw() {
        let input = r#"{"name": "test"}"#;
        assert_eq!(extract_json(input), Some(r#"{"name": "test"}"#));
    }

    #[test]
    fn extract_json_from_code_block() {
        let input = "Here is the JSON:\n```json\n{\"name\": \"test\"}\n```\nDone.";
        assert_eq!(extract_json(input), Some(r#"{"name": "test"}"#));
    }

    #[test]
    fn extract_json_from_surrounding_text() {
        let input = "Sure! Here you go: {\"name\": \"test\"} Hope that helps!";
        assert_eq!(extract_json(input), Some(r#"{"name": "test"}"#));
    }
}

//! Configuration API — manages model, effort, tools, and budget settings.
//!
//! Settings are persisted in `ObjectStore` and used by the chat handler
//! to build [`AgentConfig`](rusvel_core::domain::AgentConfig) (Claude CLI,
//! Cursor agent, Ollama, etc. depending on `model`).

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::info;

use rusvel_core::ports::StoragePort;

use crate::AppState;

/// The user's chat configuration — assembled into `claude -p` flags.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatConfig {
    pub model: String,
    pub effort: String,
    pub max_budget_usd: Option<f64>,
    pub permission_mode: String,
    pub allowed_tools: Vec<String>,
    pub disallowed_tools: Vec<String>,
    pub max_turns: Option<u32>,
    /// Optional tier hint: `fast` | `balanced` | `premium` (see `rusvel_core::domain::ModelTier`).
    #[serde(default)]
    pub model_tier: Option<String>,
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            // Default to Cursor terminal agent (`rusvel-llm` CursorAgentProvider) to avoid
            // Claude Max / API limits when Cursor subscription is available. Pick
            // `claude/sonnet` etc. in the UI for Claude CLI routing.
            model: "cursor/sonnet-4".into(),
            effort: "medium".into(),
            max_budget_usd: None,
            permission_mode: "default".into(),
            allowed_tools: vec![],
            disallowed_tools: vec![],
            max_turns: None,
            model_tier: None,
        }
    }
}

impl ChatConfig {
    /// Model id for `claude -p --model` (strip `claude/` prefix; bare ids unchanged).
    fn claude_cli_model_flag(&self) -> String {
        match self.model.split_once('/') {
            Some(("claude", name)) => name.to_string(),
            _ => self.model.clone(),
        }
    }

    /// Build CLI arguments for `claude -p` from this config.
    pub fn to_claude_args(&self) -> Vec<String> {
        let mut args = vec![
            "--model".into(),
            self.claude_cli_model_flag(),
            "--effort".into(),
            self.effort.clone(),
            "--permission-mode".into(),
            self.permission_mode.clone(),
        ];
        if let Some(budget) = self.max_budget_usd {
            args.push("--max-budget-usd".into());
            args.push(budget.to_string());
        }
        if !self.allowed_tools.is_empty() {
            args.push("--allowedTools".into());
            args.push(self.allowed_tools.join(" "));
        }
        if !self.disallowed_tools.is_empty() {
            args.push("--disallowedTools".into());
            args.push(self.disallowed_tools.join(" "));
        }
        if let Some(turns) = self.max_turns {
            args.push("--max-turns".into());
            args.push(turns.to_string());
        }
        args
    }
}

const CONFIG_KEY: &str = "chat_config";
const CONFIG_ID: &str = "current";

/// Legacy picker values (before `cursor/…` and `claude/…` prefixes) routed to Claude CLI
/// and hit Max/API limits. Rewrite once to Cursor agent.
pub(crate) fn migrate_legacy_chat_model(config: &mut ChatConfig) -> bool {
    if matches!(config.model.as_str(), "sonnet" | "opus" | "haiku") {
        config.model = "cursor/sonnet-4".into();
        true
    } else {
        false
    }
}

/// Load persisted chat config and migrate legacy model ids to `cursor/sonnet-4` when needed.
pub async fn load_and_migrate_chat_config(
    storage: &Arc<dyn StoragePort>,
) -> Result<ChatConfig, (StatusCode, String)> {
    let stored = storage
        .objects()
        .get(CONFIG_KEY, CONFIG_ID)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut config: ChatConfig = stored
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    if migrate_legacy_chat_model(&mut config) {
        info!(
            target: "rusvel::config",
            model = %config.model,
            "migrated legacy bare Claude chat model to Cursor agent default"
        );
        let value = serde_json::to_value(&config)
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
        storage
            .objects()
            .put(CONFIG_KEY, CONFIG_ID, value)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    }

    Ok(config)
}

/// `GET /api/config` — get current chat configuration.
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ChatConfig>, (StatusCode, String)> {
    let config = load_and_migrate_chat_config(&state.storage).await?;
    Ok(Json(config))
}

/// `PUT /api/config` — update chat configuration.
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<ChatConfig>,
) -> Result<Json<ChatConfig>, (StatusCode, String)> {
    let value =
        serde_json::to_value(&config).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    state
        .storage
        .objects()
        .put(CONFIG_KEY, CONFIG_ID, value)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(config))
}

/// Available models for the picker.
#[derive(Serialize)]
pub struct ModelOption {
    pub value: String,
    pub label: String,
    pub description: String,
}

/// `GET /api/config/models` — list available models.
pub async fn list_models() -> Json<Vec<ModelOption>> {
    Json(vec![
        ModelOption {
            value: "cursor/sonnet-4".into(),
            label: "Cursor · Sonnet 4".into(),
            description: "Cursor terminal agent (cursor agent --print) — uses Cursor quota; RUSVEL tools not forwarded".into(),
        },
        ModelOption {
            value: "cursor/gpt-5".into(),
            label: "Cursor · GPT-5".into(),
            description: "Cursor agent with OpenAI-class model id (if available on your account)".into(),
        },
        ModelOption {
            value: "cursor/sonnet-4-thinking".into(),
            label: "Cursor · Sonnet thinking".into(),
            description: "Cursor agent with extended reasoning variant (if listed by cursor agent --list-models)".into(),
        },
        ModelOption {
            value: "claude/sonnet".into(),
            label: "Claude · Sonnet".into(),
            description: "Claude CLI — fast, capable (Claude Max / API)".into(),
        },
        ModelOption {
            value: "claude/opus".into(),
            label: "Claude · Opus".into(),
            description: "Claude CLI — most capable".into(),
        },
        ModelOption {
            value: "claude/haiku".into(),
            label: "Claude · Haiku".into(),
            description: "Claude CLI — fastest".into(),
        },
    ])
}

/// Available tools for toggle panel.
#[derive(Serialize)]
pub struct ToolOption {
    pub name: String,
    pub description: String,
    pub category: String,
}

/// `GET /api/config/tools` — list all available tools.
pub async fn list_tools() -> Json<Vec<ToolOption>> {
    Json(vec![
        ToolOption {
            name: "Read".into(),
            description: "Read file contents".into(),
            category: "Files".into(),
        },
        ToolOption {
            name: "Write".into(),
            description: "Create/overwrite files".into(),
            category: "Files".into(),
        },
        ToolOption {
            name: "Edit".into(),
            description: "Edit specific parts of files".into(),
            category: "Files".into(),
        },
        ToolOption {
            name: "Bash".into(),
            description: "Execute shell commands".into(),
            category: "System".into(),
        },
        ToolOption {
            name: "Glob".into(),
            description: "Find files by pattern".into(),
            category: "Search".into(),
        },
        ToolOption {
            name: "Grep".into(),
            description: "Search file contents".into(),
            category: "Search".into(),
        },
        ToolOption {
            name: "WebSearch".into(),
            description: "Search the web".into(),
            category: "Web".into(),
        },
        ToolOption {
            name: "WebFetch".into(),
            description: "Fetch URL content".into(),
            category: "Web".into(),
        },
        ToolOption {
            name: "Agent".into(),
            description: "Spawn sub-agents".into(),
            category: "Agents".into(),
        },
        ToolOption {
            name: "NotebookEdit".into(),
            description: "Edit Jupyter notebooks".into(),
            category: "Files".into(),
        },
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrate_legacy_opus_to_cursor() {
        let mut c = ChatConfig {
            model: "opus".into(),
            ..Default::default()
        };
        assert!(migrate_legacy_chat_model(&mut c));
        assert_eq!(c.model, "cursor/sonnet-4");
    }

    #[test]
    fn does_not_migrate_claude_prefixed() {
        let mut c = ChatConfig {
            model: "claude/opus".into(),
            ..Default::default()
        };
        assert!(!migrate_legacy_chat_model(&mut c));
    }

    #[test]
    fn to_claude_args_strips_claude_prefix() {
        let c = ChatConfig {
            model: "claude/opus".into(),
            ..Default::default()
        };
        let args = c.to_claude_args();
        let i = args.iter().position(|a| a == "--model").unwrap();
        assert_eq!(args.get(i + 1).map(String::as_str), Some("opus"));
    }
}

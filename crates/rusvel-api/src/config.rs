//! Configuration API — manages model, effort, tools, and budget settings.
//!
//! Settings are persisted in `ObjectStore` and used by the chat handler
//! to construct `claude -p` commands.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};

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
}

impl Default for ChatConfig {
    fn default() -> Self {
        Self {
            model: "sonnet".into(),
            effort: "medium".into(),
            max_budget_usd: None,
            permission_mode: "default".into(),
            allowed_tools: vec![],
            disallowed_tools: vec![],
            max_turns: None,
        }
    }
}

impl ChatConfig {
    /// Build CLI arguments for `claude -p` from this config.
    pub fn to_claude_args(&self) -> Vec<String> {
        let mut args = vec![
            "--model".into(),
            self.model.clone(),
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

/// `GET /api/config` — get current chat configuration.
pub async fn get_config(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ChatConfig>, (StatusCode, String)> {
    let stored = state
        .storage
        .objects()
        .get(CONFIG_KEY, CONFIG_ID)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let config = stored
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    Ok(Json(config))
}

/// `PUT /api/config` — update chat configuration.
pub async fn update_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<ChatConfig>,
) -> Result<Json<ChatConfig>, (StatusCode, String)> {
    let value = serde_json::to_value(&config)
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

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
            value: "sonnet".into(),
            label: "Sonnet".into(),
            description: "Fast, capable — daily coding".into(),
        },
        ModelOption {
            value: "opus".into(),
            label: "Opus".into(),
            description: "Most capable — complex reasoning".into(),
        },
        ModelOption {
            value: "haiku".into(),
            label: "Haiku".into(),
            description: "Fastest — quick tasks".into(),
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
        ToolOption { name: "Read".into(), description: "Read file contents".into(), category: "Files".into() },
        ToolOption { name: "Write".into(), description: "Create/overwrite files".into(), category: "Files".into() },
        ToolOption { name: "Edit".into(), description: "Edit specific parts of files".into(), category: "Files".into() },
        ToolOption { name: "Bash".into(), description: "Execute shell commands".into(), category: "System".into() },
        ToolOption { name: "Glob".into(), description: "Find files by pattern".into(), category: "Search".into() },
        ToolOption { name: "Grep".into(), description: "Search file contents".into(), category: "Search".into() },
        ToolOption { name: "WebSearch".into(), description: "Search the web".into(), category: "Web".into() },
        ToolOption { name: "WebFetch".into(), description: "Fetch URL content".into(), category: "Web".into() },
        ToolOption { name: "Agent".into(), description: "Spawn sub-agents".into(), category: "Agents".into() },
        ToolOption { name: "NotebookEdit".into(), description: "Edit Jupyter notebooks".into(), category: "Files".into() },
    ])
}

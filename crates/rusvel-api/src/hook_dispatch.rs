//! Hook dispatch — fires matching hooks when events occur.
//!
//! Hooks are stored in `ObjectStore`["hooks"]. When an event fires (e.g.,
//! `code.chat.completed`), this module finds all enabled hooks whose `event`
//! field matches, and executes them asynchronously (fire-and-forget).
//!
//! Hook types:
//! - `command` — runs a shell command via `sh -c`
//! - `http`    — POSTs the event payload to a URL
//! - `prompt`  — sends the action text as a prompt to `claude -p`

use std::sync::Arc;

use rusvel_core::ports::StoragePort;
use rusvel_llm::stream::ClaudeCliStreamer;

use crate::hooks::HookDefinition;

/// Dispatch all matching hooks for an event. Runs asynchronously — does not block.
///
/// Call this after emitting events (e.g., in `department_chat_handler`, workflow execution).
/// Each matched hook spawns its own tokio task; errors are logged, not propagated.
pub fn dispatch_hooks(
    event_kind: &str,
    payload: serde_json::Value,
    engine: &str,
    storage: Arc<dyn StoragePort>,
) {
    let event_kind = event_kind.to_string();
    let engine = engine.to_string();

    tokio::spawn(async move {
        let hooks = match load_matching_hooks(&storage, &event_kind, &engine).await {
            Ok(h) => h,
            Err(e) => {
                tracing::warn!("Failed to load hooks for event {event_kind}: {e}");
                return;
            }
        };

        for hook in hooks {
            let payload = payload.clone();
            tokio::spawn(async move {
                if let Err(e) = execute_hook(&hook, &payload).await {
                    tracing::warn!("Hook '{}' failed: {e}", hook.name);
                }
            });
        }
    });
}

/// Load enabled hooks that match the given event kind and engine scope.
async fn load_matching_hooks(
    storage: &Arc<dyn StoragePort>,
    event_kind: &str,
    engine: &str,
) -> Result<Vec<HookDefinition>, String> {
    let all = storage
        .objects()
        .list("hooks", rusvel_core::domain::ObjectFilter::default())
        .await
        .map_err(|e| e.to_string())?;

    let hooks: Vec<HookDefinition> = all
        .into_iter()
        .filter_map(|v| serde_json::from_value(v).ok())
        .filter(|h: &HookDefinition| h.enabled)
        .filter(|h| {
            // Engine scope: match engine or global (no engine in metadata)
            let hook_engine = h.metadata.get("engine").and_then(|e| e.as_str());
            hook_engine == Some(engine) || hook_engine.is_none()
        })
        .filter(|h| matches_event(&h.event, &h.matcher, event_kind))
        .collect();

    Ok(hooks)
}

/// Check if a hook's event + matcher pattern matches the given event kind.
///
/// - Exact match: hook.event == `event_kind`
/// - Wildcard: hook.matcher == "*" matches any event
/// - Suffix match: hook.event == "chat.completed" matches "code.chat.completed"
fn matches_event(hook_event: &str, hook_matcher: &str, event_kind: &str) -> bool {
    if hook_matcher == "*" {
        return true;
    }
    if event_kind == hook_event {
        return true;
    }
    // Allow partial match: "chat.completed" matches "code.chat.completed"
    if event_kind.ends_with(hook_event) {
        return true;
    }
    false
}

/// Execute a single hook based on its type.
async fn execute_hook(hook: &HookDefinition, payload: &serde_json::Value) -> Result<(), String> {
    match hook.hook_type.as_str() {
        "command" => execute_command_hook(hook, payload).await,
        "http" => execute_http_hook(hook, payload).await,
        "prompt" => execute_prompt_hook(hook, payload).await,
        other => Err(format!("unknown hook type: {other}")),
    }
}

/// Run a shell command. The event payload is available as $`HOOK_PAYLOAD` env var.
async fn execute_command_hook(
    hook: &HookDefinition,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let payload_str = serde_json::to_string(payload).unwrap_or_default();

    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(&hook.action)
        .env("HOOK_PAYLOAD", &payload_str)
        .output()
        .await
        .map_err(|e| format!("command exec failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!(
            "Hook '{}' command exited {}: {stderr}",
            hook.name,
            output.status
        );
    }

    Ok(())
}

/// POST the event payload to the hook's action URL.
async fn execute_http_hook(
    hook: &HookDefinition,
    payload: &serde_json::Value,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(&hook.action)
        .json(payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("http hook failed: {e}"))?;

    if !resp.status().is_success() {
        tracing::warn!("Hook '{}' HTTP returned {}", hook.name, resp.status());
    }

    Ok(())
}

/// Send the hook's action text as a prompt to claude -p (fire-and-forget).
async fn execute_prompt_hook(
    hook: &HookDefinition,
    payload: &serde_json::Value,
) -> Result<(), String> {
    use tokio_stream::StreamExt;
    use tokio_stream::wrappers::ReceiverStream;

    // Interpolate {{payload}} in the action text
    let prompt = hook.action.replace(
        "{{payload}}",
        &serde_json::to_string_pretty(payload).unwrap_or_default(),
    );

    let streamer = ClaudeCliStreamer::new();
    let args = vec![
        "--model".into(),
        "haiku".to_string(),
        "--max-turns".into(),
        "1".to_string(),
        "--permission-mode".into(),
        "plan".to_string(),
    ];
    let rx = streamer.stream_with_args(&prompt, &args);

    // Drain the stream (we don't need the output, just let it run)
    let mut stream = ReceiverStream::new(rx);
    while let Some(event) = stream.next().await {
        if let rusvel_llm::stream::StreamEvent::Error { message } = event {
            return Err(format!("prompt hook error: {message}"));
        }
    }

    Ok(())
}

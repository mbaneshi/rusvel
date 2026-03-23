//! Streaming adapter for Claude CLI (`claude -p --output-format stream-json`).
//!
//! Unlike [`ClaudeCliProvider`] which waits for the full response, this streams
//! text deltas as they arrive — suitable for SSE endpoints and real-time chat UI.

use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tracing::debug;

/// An event emitted during streaming.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Incremental text chunk from the assistant.
    Delta { text: String },
    /// Stream completed successfully.
    Done { full_text: String, cost_usd: f64 },
    /// An error occurred.
    Error { message: String },
}

/// Streams Claude CLI responses line-by-line.
pub struct ClaudeCliStreamer {
    command: String,
    timeout_secs: u64,
}

impl ClaudeCliStreamer {
    /// Create a streamer using Max subscription (no API key).
    pub fn new() -> Self {
        Self {
            command: "claude".into(),
            timeout_secs: 300,
        }
    }

    /// Stream a prompt through Claude CLI. Returns a receiver that yields
    /// [`StreamEvent`] items as text arrives.
    pub fn stream(&self, prompt: &str) -> mpsc::Receiver<StreamEvent> {
        self.stream_with_args(prompt, &[])
    }

    /// Stream with additional CLI arguments (e.g., --model, --effort, --allowedTools).
    pub fn stream_with_args(
        &self,
        prompt: &str,
        extra_args: &[String],
    ) -> mpsc::Receiver<StreamEvent> {
        let (tx, rx) = mpsc::channel(64);
        let command = self.command.clone();
        let timeout_secs = self.timeout_secs;
        let prompt = prompt.to_string();
        let extra = extra_args.to_vec();

        tokio::spawn(async move {
            if let Err(e) = run_stream(&command, &prompt, timeout_secs, &extra, &tx).await {
                let _ = tx.send(StreamEvent::Error { message: e }).await;
            }
        });

        rx
    }
}

impl Default for ClaudeCliStreamer {
    fn default() -> Self {
        Self::new()
    }
}

async fn run_stream(
    command: &str,
    prompt: &str,
    timeout_secs: u64,
    extra_args: &[String],
    tx: &mpsc::Sender<StreamEvent>,
) -> Result<(), String> {
    let mut cmd = Command::new(command);
    cmd.arg("-p")
        .arg(prompt)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--no-session-persistence");

    // Apply extra args (model, effort, tools, etc.)
    for arg in extra_args {
        cmd.arg(arg);
    }

    // Max subscription env vars (same as ClaudeCliProvider)
    cmd.env_remove("ANTHROPIC_API_KEY");
    cmd.env_remove("CLAUDECODE");
    cmd.env("CLAUDE_CODE_ENTRYPOINT", "sdk-max");
    cmd.env("CLAUDE_USE_SUBSCRIPTION", "true");
    cmd.env("CLAUDE_BYPASS_BALANCE_CHECK", "true");

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn claude: {e}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture stdout".to_string())?;

    let mut lines = BufReader::new(stdout).lines();
    let mut full_text = String::new();
    let mut cost_usd = 0.0;

    let stream_result = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), async {
        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            // Each line is a JSON object
            let parsed: serde_json::Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let event_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");

            match event_type {
                "assistant" => {
                    // Extract text from message.content[].text
                    if let Some(content) = parsed
                        .pointer("/message/content")
                        .and_then(|c| c.as_array())
                    {
                        for block in content {
                            if block.get("type").and_then(|t| t.as_str()) == Some("text")
                                && let Some(text) = block.get("text").and_then(|t| t.as_str())
                                && !text.is_empty()
                            {
                                debug!(len = text.len(), "stream delta");
                                let _ = tx
                                    .send(StreamEvent::Delta {
                                        text: text.to_string(),
                                    })
                                    .await;
                            }
                        }
                    }
                }
                "result" => {
                    // Final result
                    if let Some(result) = parsed.get("result").and_then(|r| r.as_str()) {
                        full_text = result.to_string();
                    }
                    cost_usd = parsed
                        .get("total_cost_usd")
                        .and_then(serde_json::Value::as_f64)
                        .unwrap_or(0.0);

                    let is_error = parsed
                        .get("is_error")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);

                    if is_error {
                        let _ = tx
                            .send(StreamEvent::Error {
                                message: full_text.clone(),
                            })
                            .await;
                    } else {
                        let _ = tx
                            .send(StreamEvent::Done {
                                full_text: full_text.clone(),
                                cost_usd,
                            })
                            .await;
                    }
                }
                _ => {
                    // system, content_block_delta, etc. — skip for now
                }
            }
        }
    })
    .await;

    // Clean up child process
    let _ = child.kill().await;

    if stream_result.is_err() {
        return Err("claude CLI stream timed out".into());
    }

    // If we never got a result event, send what we have
    if full_text.is_empty() {
        return Err("no result received from claude CLI".into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streamer_defaults() {
        let s = ClaudeCliStreamer::new();
        assert_eq!(s.command, "claude");
        assert_eq!(s.timeout_secs, 300);
    }
}

//! System self-improvement endpoints.
//!
//! Allows RUSVEL to test, build, and inspect itself.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use rusvel_core::id::SessionId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::process::Command;

use crate::AppState;

#[derive(Serialize)]
pub struct CommandResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Serialize)]
pub struct SystemStatus {
    pub build: CommandResult,
    pub test: CommandResult,
    pub frontend_check: CommandResult,
}

/// `POST /api/system/test` — run cargo test + pnpm check
pub async fn run_tests(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<SystemStatus>, (StatusCode, String)> {
    let project_dir = find_project_dir();

    let build = run_command("cargo", &["build"], &project_dir).await;
    let test = run_command("cargo", &["test"], &project_dir).await;

    let frontend_dir = format!("{}/frontend", project_dir);
    let frontend_check = if std::path::Path::new(&frontend_dir).exists() {
        run_command("pnpm", &["exec", "svelte-check"], &frontend_dir).await
    } else {
        CommandResult {
            success: true,
            stdout: "frontend dir not found, skipped".into(),
            stderr: String::new(),
            exit_code: Some(0),
        }
    };

    Ok(Json(SystemStatus {
        build,
        test,
        frontend_check,
    }))
}

#[derive(Debug, Deserialize)]
pub struct NotifyBody {
    pub session_id: SessionId,
    pub text: String,
}

/// `POST /api/system/notify` — send text via configured outbound channel (e.g. Telegram).
pub async fn notify(
    State(state): State<Arc<AppState>>,
    Json(body): Json<NotifyBody>,
) -> Result<StatusCode, (StatusCode, String)> {
    let Some(ref ch) = state.channel else {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "notify channel not configured".into(),
        ));
    };
    let payload = json!({ "text": body.text });
    ch.send_message(&body.session_id, payload)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e.to_string()))?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/system/build` — rebuild backend + frontend
pub async fn run_build(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let project_dir = find_project_dir();

    let cargo = run_command("cargo", &["build"], &project_dir).await;

    let frontend_dir = format!("{}/frontend", project_dir);
    let frontend = if std::path::Path::new(&frontend_dir).exists() {
        run_command("pnpm", &["build"], &frontend_dir).await
    } else {
        CommandResult {
            success: true,
            stdout: "skipped".into(),
            stderr: String::new(),
            exit_code: Some(0),
        }
    };

    Ok(Json(serde_json::json!({
        "cargo_build": cargo,
        "frontend_build": frontend,
    })))
}

/// `GET /api/system/status` — read current state + gaps
pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let project_dir = find_project_dir();

    // Read status docs if they exist
    let current_state =
        std::fs::read_to_string(format!("{}/docs/status/current-state.md", project_dir))
            .unwrap_or_else(|_| "Not found".into());
    let gap_analysis =
        std::fs::read_to_string(format!("{}/docs/status/gap-analysis.md", project_dir))
            .unwrap_or_else(|_| "Not found".into());

    // Count entities
    let agents = state
        .storage
        .objects()
        .list("agents", rusvel_core::domain::ObjectFilter::default())
        .await
        .map(|v| v.len())
        .unwrap_or(0);
    let skills = state
        .storage
        .objects()
        .list("skills", rusvel_core::domain::ObjectFilter::default())
        .await
        .map(|v| v.len())
        .unwrap_or(0);
    let rules = state
        .storage
        .objects()
        .list("rules", rusvel_core::domain::ObjectFilter::default())
        .await
        .map(|v| v.len())
        .unwrap_or(0);

    // Git info
    let git_log = run_command("git", &["log", "--oneline", "-10"], &project_dir).await;
    let git_status = run_command("git", &["status", "--short"], &project_dir).await;

    Ok(Json(serde_json::json!({
        "current_state_md": current_state.chars().take(2000).collect::<String>(),
        "gap_analysis_md": gap_analysis.chars().take(2000).collect::<String>(),
        "entities": {
            "agents": agents,
            "skills": skills,
            "rules": rules,
        },
        "git": {
            "recent_commits": git_log.stdout,
            "working_tree": git_status.stdout,
        },
    })))
}

/// `POST /api/system/fix` — ask Code department to fix an issue
pub async fn self_fix(
    State(_state): State<Arc<AppState>>,
    Json(body): Json<FixRequest>,
) -> Result<Json<CommandResult>, (StatusCode, String)> {
    let project_dir = find_project_dir();

    // Run the fix command via claude -p with full tools
    let prompt = format!(
        "Fix this issue in the RUSVEL codebase at {}:\n\n{}\n\n\
         After fixing, run `cargo test` to verify. If tests fail, fix those too.",
        project_dir, body.issue
    );

    let result = run_command(
        "claude",
        &[
            "-p",
            &prompt,
            "--output-format",
            "text",
            "--model",
            "sonnet",
            "--allowedTools",
            "Read Write Edit Bash Glob Grep",
            "--add-dir",
            &project_dir,
            "--no-session-persistence",
            "--permission-mode",
            "acceptEdits",
        ],
        &project_dir,
    )
    .await;

    Ok(Json(result))
}

#[derive(serde::Deserialize)]
pub struct FixRequest {
    pub issue: String,
}

/// `POST /api/system/ingest-docs` — ingest RUSVEL's own docs into knowledge base (on-demand)
pub async fn ingest_docs(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let embed = state.embedding.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Embedding adapter not available".into(),
    ))?;
    let vs = state.vector_store.as_ref().ok_or((
        StatusCode::SERVICE_UNAVAILABLE,
        "Vector store not available".into(),
    ))?;

    let project_dir = find_project_dir();
    let docs = [
        "CLAUDE.md",
        "docs/design/architecture-v2.md",
        "docs/design/decisions.md",
        "docs/status/current-state.md",
        "docs/status/gap-analysis.md",
    ];

    let mut total_chunks = 0usize;
    let mut ingested_files = Vec::new();

    for doc_path in &docs {
        let full_path = format!("{}/{}", project_dir, doc_path);
        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let chunks: Vec<&str> = content
            .split("\n\n")
            .map(|c| c.trim())
            .filter(|c| c.len() > 50)
            .collect();

        for (i, chunk) in chunks.iter().enumerate() {
            let id = format!("doc-{}-{}", doc_path.replace('/', "-"), i);
            match embed.embed_one(chunk).await {
                Ok(embedding) => {
                    let _ = vs
                        .upsert(
                            &id,
                            chunk,
                            embedding,
                            serde_json::json!({ "source": doc_path, "chunk_index": i }),
                        )
                        .await;
                    total_chunks += 1;
                }
                Err(e) => {
                    tracing::warn!("Failed to embed chunk from {doc_path}: {e}");
                }
            }
        }
        ingested_files.push(doc_path.to_string());
    }

    // Also ingest profile.toml if it exists
    let profile_path = format!(
        "{}/.rusvel/profile.toml",
        std::env::var("HOME").unwrap_or_else(|_| ".".into())
    );
    if let Ok(profile_content) = std::fs::read_to_string(&profile_path) {
        if let Ok(embedding) = embed.embed_one(&profile_content).await {
            let _ = vs
                .upsert(
                    "doc-profile",
                    &profile_content,
                    embedding,
                    serde_json::json!({ "source": "profile.toml" }),
                )
                .await;
            total_chunks += 1;
            ingested_files.push("profile.toml".into());
        }
    }

    Ok(Json(serde_json::json!({
        "chunks_ingested": total_chunks,
        "files_processed": ingested_files,
    })))
}

// ── Helpers ──────────────────────────────────────────────────

async fn run_command(cmd: &str, args: &[&str], cwd: &str) -> CommandResult {
    match Command::new(cmd)
        .args(args)
        .current_dir(cwd)
        .env_remove("ANTHROPIC_API_KEY")
        .env("CLAUDE_CODE_ENTRYPOINT", "sdk-max")
        .env("CLAUDE_USE_SUBSCRIPTION", "true")
        .env("CLAUDE_BYPASS_BALANCE_CHECK", "true")
        .output()
        .await
    {
        Ok(output) => CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout)
                .chars()
                .take(5000)
                .collect(),
            stderr: String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(2000)
                .collect(),
            exit_code: output.status.code(),
        },
        Err(e) => CommandResult {
            success: false,
            stdout: String::new(),
            stderr: format!("Failed to run {cmd}: {e}"),
            exit_code: None,
        },
    }
}

pub fn find_project_dir() -> String {
    // Try common locations
    for path in &[
        std::env::current_dir()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        "/Users/bm/rusvel".into(),
    ] {
        if std::path::Path::new(path).join("Cargo.toml").exists() {
            return path.clone();
        }
    }
    ".".into()
}

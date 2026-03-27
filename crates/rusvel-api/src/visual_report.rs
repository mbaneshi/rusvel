//! Visual regression report endpoints.
//!
//! Stores visual test analysis reports from the Playwright + Claude Vision pipeline,
//! and provides a self-correction endpoint that auto-generates fix skills and
//! prevention rules via the `!build` system.

use std::sync::Arc;

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::build_cmd::{BuildCommand, BuildEntityType, execute_build};

// ── Report Types ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualIssue {
    pub r#type: String,
    pub description: String,
    pub element: String,
    pub suggested_fix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendedAction {
    pub action_type: String,
    pub entity_description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteAnalysis {
    pub route: String,
    pub severity: String,
    pub issues: Vec<VisualIssue>,
    pub recommended_actions: Vec<RecommendedAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_routes: usize,
    pub regressions: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualReport {
    pub run_id: String,
    pub timestamp: String,
    pub analyses: Vec<RouteAnalysis>,
    pub summary: ReportSummary,
    // manifest is accepted but not stored (too large)
    #[serde(default, skip_serializing)]
    pub manifest: serde_json::Value,
}

// ── Handlers ────────────────────────────────────────────────────

const REPORT_KIND: &str = "visual_reports";

/// `POST /api/system/visual-report` — Store a visual analysis report.
pub async fn store_report(
    State(state): State<Arc<AppState>>,
    Json(report): Json<VisualReport>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, String)> {
    let report_id = report.run_id.clone();
    let json =
        serde_json::to_value(&report).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    state
        .storage
        .objects()
        .put(REPORT_KIND, &report_id, json)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Emit event for hook dispatch
    let event = rusvel_core::domain::Event {
        id: rusvel_core::id::EventId::new(),
        session_id: None,
        run_id: None,
        source: "code".into(),
        kind: "visual.regression.detected".into(),
        payload: serde_json::json!({
            "run_id": report.run_id,
            "regressions": report.summary.regressions,
            "critical": report.summary.critical,
            "high": report.summary.high,
        }),
        created_at: chrono::Utc::now(),
        metadata: serde_json::json!({}),
    };
    let _ = state.events.emit(event).await;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": report_id,
            "regressions": report.summary.regressions,
        })),
    ))
}

/// `GET /api/system/visual-report` — Get the latest visual reports.
pub async fn get_reports(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, String)> {
    let filter = rusvel_core::domain::ObjectFilter::default();
    let reports = state
        .storage
        .objects()
        .list(REPORT_KIND, filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(reports))
}

// ── Self-Correction ─────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct CorrectionResult {
    pub skills_created: Vec<String>,
    pub rules_created: Vec<String>,
    pub errors: Vec<String>,
}

/// `POST /api/system/visual-report/self-correct` — Auto-generate fix skills + rules.
///
/// Reads the latest visual report, generates `!build skill:` and `!build rule:`
/// commands for each issue, and persists them via the existing build pipeline.
pub async fn self_correct(
    State(state): State<Arc<AppState>>,
) -> Result<Json<CorrectionResult>, (StatusCode, String)> {
    // Load the latest report
    let filter = rusvel_core::domain::ObjectFilter::default();
    let reports = state
        .storage
        .objects()
        .list(REPORT_KIND, filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let latest = reports
        .last()
        .ok_or_else(|| (StatusCode::NOT_FOUND, "No visual reports found".into()))?;

    let report: VisualReport = serde_json::from_value(latest.clone())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = CorrectionResult {
        skills_created: Vec::new(),
        rules_created: Vec::new(),
        errors: Vec::new(),
    };

    for analysis in &report.analyses {
        // Skip low-severity issues
        if analysis.severity == "low" {
            continue;
        }

        // Generate fix skills from recommended actions
        for action in &analysis.recommended_actions {
            let entity_type = match action.action_type.as_str() {
                "skill" => BuildEntityType::Skill,
                "rule" => BuildEntityType::Rule,
                "hook" => BuildEntityType::Hook,
                _ => continue,
            };

            let cmd = BuildCommand {
                entity_type,
                description: format!(
                    "Fix visual regression on route {}: {}",
                    analysis.route, action.entity_description
                ),
            };

            match execute_build(&cmd, "code", &state.storage).await {
                Ok(confirmation) => match entity_type {
                    BuildEntityType::Skill => result.skills_created.push(confirmation),
                    BuildEntityType::Rule => result.rules_created.push(confirmation),
                    _ => {}
                },
                Err(e) => result.errors.push(format!(
                    "Failed to build {} for {}: {}",
                    cmd.entity_type.label(),
                    analysis.route,
                    e
                )),
            }
        }

        // Auto-generate a prevention rule for each high/critical issue
        if analysis.severity == "high" || analysis.severity == "critical" {
            for issue in &analysis.issues {
                let cmd = BuildCommand {
                    entity_type: BuildEntityType::Rule,
                    description: format!(
                        "Prevent visual regression on route {}: {} — element '{}' must maintain: {}",
                        analysis.route, issue.description, issue.element, issue.suggested_fix
                    ),
                };

                match execute_build(&cmd, "code", &state.storage).await {
                    Ok(confirmation) => result.rules_created.push(confirmation),
                    Err(e) => result.errors.push(format!(
                        "Failed to create prevention rule for {}: {}",
                        analysis.route, e
                    )),
                }
            }
        }
    }

    Ok(Json(result))
}

/// `POST /api/system/visual-test` — Run visual tests and return results.
///
/// Spawns Playwright in the project's frontend directory and returns the output.
pub async fn run_visual_tests(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<crate::system::CommandResult>, (StatusCode, String)> {
    let project_dir = crate::system::find_project_dir();
    let frontend_dir = format!("{project_dir}/frontend");

    let result = tokio::process::Command::new("pnpm")
        .args([
            "exec",
            "playwright",
            "test",
            "--project=visual",
            "--reporter=json",
        ])
        .current_dir(&frontend_dir)
        .output()
        .await
        .map(|output| crate::system::CommandResult {
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout)
                .chars()
                .take(10_000)
                .collect(),
            stderr: String::from_utf8_lossy(&output.stderr)
                .chars()
                .take(5_000)
                .collect(),
            exit_code: output.status.code(),
        })
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to run playwright: {e}"),
            )
        })?;

    Ok(Json(result))
}

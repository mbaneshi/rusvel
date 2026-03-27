//! Deployment adapters — Fly.io via `flyctl` CLI (see [`DeployPort`] in rusvel-core).

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::process::Command;

use rusvel_core::domain::{DeployStatus, DeployedUrl};
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::ports::{ConfigPort, DeployPort};

/// Fly.io deploy using `flyctl` and `FLY_ACCESS_TOKEN` (from config or env).
pub struct FlyDeployPort {
    config: Arc<dyn ConfigPort>,
}

impl FlyDeployPort {
    pub fn new(config: Arc<dyn ConfigPort>) -> Self {
        Self { config }
    }

    fn api_token(&self) -> Result<String> {
        if let Ok(Some(v)) = self.config.get_value("deploy.fly.api_token") {
            if let Some(s) = v.as_str() {
                if !s.is_empty() {
                    return Ok(s.to_string());
                }
            }
        }
        std::env::var("FLY_API_TOKEN")
            .or_else(|_| std::env::var("FLY_ACCESS_TOKEN"))
            .map_err(|_| {
                RusvelError::Config(
                    "Set deploy.fly.api_token in config or FLY_API_TOKEN / FLY_ACCESS_TOKEN in the environment"
                        .into(),
                )
            })
    }
}

#[async_trait]
impl DeployPort for FlyDeployPort {
    async fn deploy(&self, artifact_path: &Path, service_name: &str) -> Result<DeployedUrl> {
        let token = self.api_token()?;
        let output = Command::new("flyctl")
            .current_dir(artifact_path)
            .args(["deploy", "--app", service_name, "--yes"])
            .env("FLY_ACCESS_TOKEN", &token)
            .output()
            .await
            .map_err(|e| RusvelError::Internal(format!("flyctl spawn failed: {e}")))?;

        if !output.status.success() {
            return Err(RusvelError::Internal(format!(
                "flyctl deploy failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let url = format!("https://{service_name}.fly.dev");
        Ok(DeployedUrl {
            url: url.clone(),
            deployment_id: service_name.to_string(),
            metadata: Default::default(),
        })
    }

    async fn status(&self, deployment_id: &str) -> Result<DeployStatus> {
        let token = self.api_token()?;
        let output = Command::new("flyctl")
            .args(["status", "--app", deployment_id, "--json"])
            .env("FLY_ACCESS_TOKEN", &token)
            .output()
            .await
            .map_err(|e| RusvelError::Internal(format!("flyctl status failed: {e}")))?;

        if !output.status.success() {
            return Err(RusvelError::Internal(format!(
                "flyctl status failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|_| {
            serde_json::json!({
                "Status": String::from_utf8_lossy(&output.stdout).trim()
            })
        });

        let state = v
            .get("Status")
            .or_else(|| v.get("status"))
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(DeployStatus {
            id: deployment_id.to_string(),
            state,
            url: None,
            metadata: Default::default(),
        })
    }
}

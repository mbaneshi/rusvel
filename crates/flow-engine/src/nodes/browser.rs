//! Browser-related flow nodes: event trigger gate and CDP actions.

use std::sync::Arc;

use async_trait::async_trait;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::ports::BrowserPort;

use super::{NodeContext, NodeHandler, NodeOutput};

/// Validates trigger payload for flows started from browser capture (or manual run with trigger JSON).
pub struct BrowserTriggerNode;

#[async_trait]
impl NodeHandler for BrowserTriggerNode {
    fn node_type(&self) -> &str {
        "browser_trigger"
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let trigger = ctx.inputs.get("trigger").ok_or_else(|| {
            RusvelError::Validation("browser_trigger requires upstream trigger input".into())
        })?;
        let expect = ctx
            .node
            .parameters
            .get("event_kind")
            .and_then(|v| v.as_str())
            .unwrap_or("browser.data.captured");
        if expect != "*" {
            let kind = trigger.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            if kind != expect {
                return Err(RusvelError::Validation(format!(
                    "browser_trigger: trigger.kind '{kind}' does not match expected '{expect}'"
                )));
            }
        }
        if trigger.get("platform").is_none() && trigger.get("data").is_none() {
            return Err(RusvelError::Validation(
                "browser_trigger: trigger must include platform or data".into(),
            ));
        }
        Ok(NodeOutput {
            data: trigger.clone(),
            output_name: "main".into(),
        })
    }
}

/// Executes a [`BrowserPort`] action (navigate / evaluate) as a flow step.
pub struct BrowserActionNode {
    browser: Option<Arc<dyn BrowserPort>>,
}

impl BrowserActionNode {
    pub fn new(browser: Option<Arc<dyn BrowserPort>>) -> Self {
        Self { browser }
    }
}

#[async_trait]
impl NodeHandler for BrowserActionNode {
    fn node_type(&self) -> &str {
        "browser_action"
    }

    async fn execute(&self, ctx: &NodeContext) -> Result<NodeOutput> {
        let browser = self.browser.as_ref().ok_or_else(|| {
            RusvelError::Internal("browser_action requires BrowserPort (not configured)".into())
        })?;
        if ctx
            .node
            .parameters
            .get("requires_approval")
            .and_then(|v| v.as_bool())
            == Some(true)
        {
            return Err(RusvelError::Validation(
                "browser_action: set requires_approval=false or enqueue a BrowserAction job (ADR-008)"
                    .into(),
            ));
        }
        let tab_id = ctx
            .node
            .parameters
            .get("tab_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RusvelError::Validation("browser_action requires tab_id".into()))?;
        let action = ctx
            .node
            .parameters
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("navigate");
        match action {
            "navigate" => {
                let url = ctx
                    .node
                    .parameters
                    .get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        RusvelError::Validation("browser_action navigate requires url".into())
                    })?;
                browser.navigate(tab_id, url).await?;
                Ok(NodeOutput {
                    data: serde_json::json!({ "ok": true, "action": "navigate", "url": url }),
                    output_name: "main".into(),
                })
            }
            "evaluate" | "evaluate_js" => {
                let script = ctx
                    .node
                    .parameters
                    .get("script")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        RusvelError::Validation("browser_action evaluate requires script".into())
                    })?;
                let v = browser.evaluate_js(tab_id, script).await?;
                Ok(NodeOutput {
                    data: serde_json::json!({ "result": v }),
                    output_name: "main".into(),
                })
            }
            other => Err(RusvelError::Validation(format!(
                "browser_action: unknown action '{other}'"
            ))),
        }
    }
}

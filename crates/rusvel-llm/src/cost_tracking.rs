//! [`LlmPort`] wrapper: tier resolution + [`MetricStore`] spend recording.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::mpsc;

use rusvel_core::domain::*;
use rusvel_core::error::Result;
use rusvel_core::ports::{LlmPort, MetricStore};

use crate::tier_routing::apply_model_tier;

/// Wraps an inner [`LlmPort`], applies [`apply_model_tier`], and records estimated USD per call to [`MetricStore`].
#[derive(Clone)]
pub struct CostTrackingLlm {
    inner: Arc<dyn LlmPort>,
    metrics: Option<Arc<dyn MetricStore>>,
}

impl CostTrackingLlm {
    pub fn new(inner: Arc<dyn LlmPort>) -> Self {
        Self {
            inner,
            metrics: None,
        }
    }

    pub fn with_metrics(inner: Arc<dyn LlmPort>, metrics: Arc<dyn MetricStore>) -> Self {
        Self {
            inner,
            metrics: Some(metrics),
        }
    }

    async fn record_cost(&self, req: &LlmRequest, resp: &LlmResponse) {
        let Some(store) = &self.metrics else {
            return;
        };
        let usd = estimate_llm_cost_usd(&req.model.provider, &req.model.model, &resp.usage);
        let tier = ModelTier::from_request_metadata(&req.metadata);
        let mut tags = vec![
            format!("provider:{:?}", req.model.provider),
            format!("model:{}", req.model.model),
        ];
        if let Some(t) = tier {
            tags.push(format!("tier:{t}"));
        }
        if let Some(sid) = req
            .metadata
            .get(RUSVEL_META_SESSION_ID)
            .and_then(|v| v.as_str())
        {
            tags.push(format!("session:{sid}"));
        }
        let point = MetricPoint {
            name: "llm.cost_usd".into(),
            value: usd,
            tags,
            recorded_at: Utc::now(),
            metadata: serde_json::json!({
                "input_tokens": resp.usage.input_tokens,
                "output_tokens": resp.usage.output_tokens,
            }),
        };
        if let Err(e) = store.record(&point).await {
            tracing::warn!(error = %e, "metric store record failed for llm.cost_usd");
        }
    }
}

#[async_trait]
impl LlmPort for CostTrackingLlm {
    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse> {
        let req = apply_model_tier(request);
        let resp = self.inner.generate(req.clone()).await?;
        self.record_cost(&req, &resp).await;
        Ok(resp)
    }

    async fn stream(&self, request: LlmRequest) -> Result<mpsc::Receiver<LlmStreamEvent>> {
        let req = apply_model_tier(request);
        let req_snapshot = req.clone();
        let mut inner_rx = self.inner.stream(req).await?;
        let this = self.clone();
        let (tx, out_rx) = mpsc::channel(32);
        tokio::spawn(async move {
            while let Some(ev) = inner_rx.recv().await {
                match ev {
                    LlmStreamEvent::Done(resp) => {
                        this.record_cost(&req_snapshot, &resp).await;
                        let _ = tx.send(LlmStreamEvent::Done(resp)).await;
                    }
                    other => {
                        if tx.send(other).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Ok(out_rx)
    }

    async fn embed(&self, model: &ModelRef, text: &str) -> Result<Vec<f32>> {
        self.inner.embed(model, text).await
    }

    async fn list_models(&self) -> Result<Vec<ModelRef>> {
        self.inner.list_models().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rusvel_core::domain::MetricFilter;
    use rusvel_core::error::RusvelError;
    use std::sync::Mutex;

    struct RecordingMetrics {
        points: Mutex<Vec<MetricPoint>>,
    }

    impl RecordingMetrics {
        fn new() -> Arc<Self> {
            Arc::new(Self {
                points: Mutex::new(Vec::new()),
            })
        }
    }

    #[async_trait]
    impl MetricStore for RecordingMetrics {
        async fn record(&self, point: &MetricPoint) -> rusvel_core::error::Result<()> {
            self.points.lock().unwrap().push(point.clone());
            Ok(())
        }

        async fn query(
            &self,
            _filter: MetricFilter,
        ) -> rusvel_core::error::Result<Vec<MetricPoint>> {
            Ok(self.points.lock().unwrap().clone())
        }
    }

    struct EchoModelProvider;

    #[async_trait]
    impl LlmPort for EchoModelProvider {
        async fn generate(&self, request: LlmRequest) -> Result<LlmResponse> {
            Ok(LlmResponse {
                content: Content::text(request.model.model.clone()),
                finish_reason: FinishReason::Stop,
                usage: LlmUsage {
                    input_tokens: 1000,
                    output_tokens: 500,
                },
                metadata: serde_json::json!({}),
            })
        }

        async fn embed(&self, _: &ModelRef, _: &str) -> Result<Vec<f32>> {
            Err(RusvelError::Llm("no embed".into()))
        }

        async fn list_models(&self) -> Result<Vec<ModelRef>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn records_metric_after_generate() {
        let metrics = RecordingMetrics::new();
        let llm = CostTrackingLlm::with_metrics(
            Arc::new(EchoModelProvider),
            metrics.clone() as Arc<dyn MetricStore>,
        );
        let req = LlmRequest {
            model: ModelRef {
                provider: ModelProvider::Claude,
                model: "claude-sonnet-4-20250514".into(),
            },
            messages: vec![],
            tools: vec![],
            temperature: None,
            max_tokens: None,
            metadata: serde_json::json!({
                RUSVEL_META_MODEL_TIER: "fast",
                RUSVEL_META_SESSION_ID: "sess-test-1",
            }),
        };
        let resp = llm.generate(req).await.unwrap();
        match &resp.content.parts[0] {
            Part::Text(t) => assert!(t.contains("haiku")),
            _ => panic!("expected text"),
        }
        let pts = metrics.points.lock().unwrap();
        assert_eq!(pts.len(), 1);
        assert_eq!(pts[0].name, "llm.cost_usd");
        assert!(pts[0].value > 0.0);
        assert!(pts[0].tags.iter().any(|t| t.contains("session:sess-test-1")));
        assert!(pts[0].tags.iter().any(|t| t.starts_with("tier:")));
    }
}

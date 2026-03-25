//! Multi-provider router that dispatches by [`ModelProvider`].

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use rusvel_core::domain::*;
use rusvel_core::error::RusvelError;
use rusvel_core::ports::LlmPort;

// ════════════════════════════════════════════════════════════════════
//  MultiProvider
// ════════════════════════════════════════════════════════════════════

/// Routes LLM requests to the correct provider based on
/// [`ModelRef::provider`].
///
/// ```ignore
/// let mut multi = MultiProvider::new();
/// multi.register(ModelProvider::Ollama, Arc::new(OllamaProvider::new()));
/// multi.register(ModelProvider::Claude, Arc::new(ClaudeProvider::new(key)));
/// multi.register(ModelProvider::OpenAI, Arc::new(OpenAiProvider::new(key)));
/// ```
pub struct MultiProvider {
    providers: HashMap<ModelProvider, Arc<dyn LlmPort>>,
}

impl MultiProvider {
    /// Create an empty router with no providers registered.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
        }
    }

    /// Register a provider for a given [`ModelProvider`] variant.
    pub fn register(&mut self, provider: ModelProvider, llm: Arc<dyn LlmPort>) {
        self.providers.insert(provider, llm);
    }

    /// Look up the adapter for a provider, returning a user-friendly error
    /// if it has not been registered.
    fn get(&self, provider: &ModelProvider) -> rusvel_core::error::Result<&dyn LlmPort> {
        self.providers
            .get(provider)
            .map(std::convert::AsRef::as_ref)
            .ok_or_else(|| {
                RusvelError::Llm(format!("no adapter registered for provider {provider:?}"))
            })
    }
}

impl Default for MultiProvider {
    fn default() -> Self {
        Self::new()
    }
}

// ════════════════════════════════════════════════════════════════════
//  LlmPort implementation — delegate to inner provider
// ════════════════════════════════════════════════════════════════════

#[async_trait]
impl LlmPort for MultiProvider {
    async fn generate(&self, request: LlmRequest) -> rusvel_core::error::Result<LlmResponse> {
        self.get(&request.model.provider)?.generate(request).await
    }

    async fn stream(
        &self,
        request: LlmRequest,
    ) -> rusvel_core::error::Result<tokio::sync::mpsc::Receiver<LlmStreamEvent>> {
        self.get(&request.model.provider)?.stream(request).await
    }

    async fn embed(&self, model: &ModelRef, text: &str) -> rusvel_core::error::Result<Vec<f32>> {
        self.get(&model.provider)?.embed(model, text).await
    }

    async fn list_models(&self) -> rusvel_core::error::Result<Vec<ModelRef>> {
        let mut all = Vec::new();
        for provider in self.providers.values() {
            match provider.list_models().await {
                Ok(models) => all.extend(models),
                Err(e) => {
                    tracing::warn!("failed to list models from a provider: {e}");
                }
            }
        }
        Ok(all)
    }

    async fn submit_batch(
        &self,
        batch: LlmBatchRequest,
    ) -> rusvel_core::error::Result<LlmBatchSubmitResult> {
        let first = batch
            .items
            .first()
            .ok_or_else(|| RusvelError::Validation("batch has no items".into()))?;
        let p = &first.request.model.provider;
        for item in &batch.items[1..] {
            if &item.request.model.provider != p {
                return Err(RusvelError::Validation(
                    "batch items must use the same model provider".into(),
                ));
            }
        }
        self.get(p)?.submit_batch(batch).await
    }

    async fn poll_batch(
        &self,
        handle: &BatchHandle,
    ) -> rusvel_core::error::Result<LlmBatchPollResult> {
        self.get(&handle.provider)?.poll_batch(handle).await
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny fake provider for testing the router.
    struct FakeProvider {
        tag: &'static str,
    }

    #[async_trait]
    impl LlmPort for FakeProvider {
        async fn generate(&self, _request: LlmRequest) -> rusvel_core::error::Result<LlmResponse> {
            Ok(LlmResponse {
                content: Content::text(format!("from {}", self.tag)),
                finish_reason: FinishReason::Stop,
                usage: LlmUsage::default(),
                metadata: serde_json::json!({}),
            })
        }

        async fn embed(
            &self,
            _model: &ModelRef,
            _text: &str,
        ) -> rusvel_core::error::Result<Vec<f32>> {
            Ok(vec![0.0])
        }

        async fn list_models(&self) -> rusvel_core::error::Result<Vec<ModelRef>> {
            Ok(vec![ModelRef {
                provider: ModelProvider::Other(self.tag.into()),
                model: "fake-model".into(),
            }])
        }
    }

    fn make_request(provider: ModelProvider) -> LlmRequest {
        LlmRequest {
            model: ModelRef {
                provider,
                model: "test".into(),
            },
            messages: vec![LlmMessage {
                role: LlmRole::User,
                content: Content::text("hi"),
            }],
            tools: vec![],
            temperature: None,
            max_tokens: None,
            metadata: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn routes_to_correct_provider() {
        let mut multi = MultiProvider::new();
        multi.register(
            ModelProvider::Ollama,
            Arc::new(FakeProvider { tag: "ollama" }),
        );
        multi.register(
            ModelProvider::Claude,
            Arc::new(FakeProvider { tag: "claude" }),
        );

        let resp = multi
            .generate(make_request(ModelProvider::Claude))
            .await
            .unwrap();
        match &resp.content.parts[0] {
            Part::Text(t) => assert_eq!(t, "from claude"),
            _ => panic!("expected text"),
        }

        let resp = multi
            .generate(make_request(ModelProvider::Ollama))
            .await
            .unwrap();
        match &resp.content.parts[0] {
            Part::Text(t) => assert_eq!(t, "from ollama"),
            _ => panic!("expected text"),
        }
    }

    #[tokio::test]
    async fn unregistered_provider_returns_error() {
        let multi = MultiProvider::new();
        let result = multi.generate(make_request(ModelProvider::OpenAI)).await;
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("no adapter registered"), "got: {msg}");
    }

    #[tokio::test]
    async fn list_models_aggregates() {
        let mut multi = MultiProvider::new();
        multi.register(
            ModelProvider::Ollama,
            Arc::new(FakeProvider { tag: "ollama" }),
        );
        multi.register(
            ModelProvider::Claude,
            Arc::new(FakeProvider { tag: "claude" }),
        );
        let models = multi.list_models().await.unwrap();
        assert_eq!(models.len(), 2);
    }
}

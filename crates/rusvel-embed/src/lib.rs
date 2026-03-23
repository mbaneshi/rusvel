//! # rusvel-embed
//!
//! Fastembed adapter implementing [`EmbeddingPort`] for RUSVEL.
//!
//! Uses the `all-MiniLM-L6-v2` model (384 dimensions) by default.
//! Since fastembed is synchronous, all embedding calls are dispatched
//! to a blocking thread via [`tokio::task::spawn_blocking`].

use std::sync::Arc;

use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::ports::EmbeddingPort;

/// Fastembed-backed implementation of [`EmbeddingPort`].
///
/// Wraps `fastembed::TextEmbedding` in an `Arc` so it can be shared
/// across async tasks and sent to blocking threads.
pub struct FastEmbedAdapter {
    model: Arc<TextEmbedding>,
    model_name: String,
    dimensions: usize,
}

impl FastEmbedAdapter {
    /// Create a new adapter with the default model (`all-MiniLM-L6-v2`, 384 dims).
    pub fn new() -> Result<Self> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2)
    }

    /// Create a new adapter with a specific fastembed model.
    pub fn with_model(model_id: EmbeddingModel) -> Result<Self> {
        let (name, dims) = model_info(&model_id);

        let options = InitOptions::new(model_id).with_show_download_progress(true);

        let model = TextEmbedding::try_new(options)
            .map_err(|e| RusvelError::Internal(format!("fastembed init failed: {e}")))?;

        tracing::info!(model = %name, dimensions = dims, "embedding model loaded");

        Ok(Self {
            model: Arc::new(model),
            model_name: name.to_string(),
            dimensions: dims,
        })
    }
}

#[async_trait]
impl EmbeddingPort for FastEmbedAdapter {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let model = Arc::clone(&self.model);
        // fastembed wants Vec<String>
        let owned: Vec<String> = texts.iter().map(std::string::ToString::to_string).collect();

        tokio::task::spawn_blocking(move || {
            model
                .embed(owned, None)
                .map_err(|e| RusvelError::Internal(format!("embedding failed: {e}")))
        })
        .await
        .map_err(|e| RusvelError::Internal(format!("spawn_blocking join error: {e}")))?
    }

    async fn embed_one(&self, text: &str) -> Result<Vec<f32>> {
        let mut results = self.embed(&[text]).await?;
        results
            .pop()
            .ok_or_else(|| RusvelError::Internal("embed returned empty results".into()))
    }

    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }
}

/// Map a fastembed model enum variant to its human-readable name and dimensions.
fn model_info(model: &EmbeddingModel) -> (&'static str, usize) {
    match model {
        EmbeddingModel::AllMiniLML6V2 => ("all-MiniLM-L6-v2", 384),
        EmbeddingModel::AllMiniLML12V2 => ("all-MiniLM-L12-v2", 384),
        EmbeddingModel::BGESmallENV15 => ("bge-small-en-v1.5", 384),
        EmbeddingModel::BGEBaseENV15 => ("bge-base-en-v1.5", 768),
        EmbeddingModel::BGELargeENV15 => ("bge-large-en-v1.5", 1024),
        _ => ("unknown", 384),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_info_default() {
        let (name, dims) = model_info(&EmbeddingModel::AllMiniLML6V2);
        assert_eq!(name, "all-MiniLM-L6-v2");
        assert_eq!(dims, 384);
    }
}

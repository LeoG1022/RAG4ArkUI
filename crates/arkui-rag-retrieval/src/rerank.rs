//! CrossEncoderReranker —— BGE-Reranker-v2-m3 精排（Day 1 stub）。
//!
//! Week 3 起接入真实 ONNX 推理。当前实现仅做 identity（不改变顺序）。

use arkui_rag_core::{Hit, RagError, Reranker, Result};
use async_trait::async_trait;

pub struct CrossEncoderReranker {
    model_id: String,
}

impl Default for CrossEncoderReranker {
    fn default() -> Self {
        Self::new("bge-reranker-v2-m3-stub")
    }
}

impl CrossEncoderReranker {
    pub fn new(model_id: impl Into<String>) -> Self {
        Self {
            model_id: model_id.into(),
        }
    }
}

#[async_trait]
impl Reranker for CrossEncoderReranker {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn rerank(&self, _query: &str, mut hits: Vec<Hit>, top_n: usize) -> Result<Vec<Hit>> {
        tracing::warn!("CrossEncoderReranker 是 Day 1 stub，做 identity + truncate(top_n)");
        if hits.is_empty() {
            return Err(RagError::Rerank("empty hits".into()));
        }
        hits.truncate(top_n);
        Ok(hits)
    }
}

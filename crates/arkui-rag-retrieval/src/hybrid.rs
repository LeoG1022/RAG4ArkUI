//! HybridRetriever —— 向量 + BM25 并行检索 + RRF 融合。
//!
//! Day 1 stub：构造函数与 `Retriever` 实现就位，但 `retrieve()` 返回空 hits 并 warn。
//! Week 2 起接入真实 VectorStore 与 BM25Index。

use crate::rrf::{rrf_fuse, RRF_DEFAULT_K};
use arkui_rag_core::{EnhancedQuery, Hit, RagError, Result, Retriever};
use async_trait::async_trait;
use std::sync::Arc;

pub struct HybridRetriever {
    name: String,
    // Week 2 字段：vector: Arc<dyn VectorStore>, bm25: Arc<dyn BM25Index>, embedder: Arc<dyn Embedder>
    _placeholder: Arc<()>,
}

impl Default for HybridRetriever {
    fn default() -> Self {
        Self::new()
    }
}

impl HybridRetriever {
    pub fn new() -> Self {
        Self {
            name: "hybrid-rrf-day1-stub".to_string(),
            _placeholder: Arc::new(()),
        }
    }
}

#[async_trait]
impl Retriever for HybridRetriever {
    fn name(&self) -> &str {
        &self.name
    }

    async fn retrieve(&self, _query: &EnhancedQuery, _top_k: usize) -> Result<Vec<Hit>> {
        tracing::warn!(
            "HybridRetriever 是 Day 1 stub，返回空 hits。Week 2 会接入 VectorStore + BM25Index。"
        );
        // 调用 rrf_fuse 让 dead-code 检查通过（同时验证类型链）
        let _ = rrf_fuse(vec![], RRF_DEFAULT_K);
        Err(RagError::NotImplemented(
            "HybridRetriever::retrieve - 见 Week 2 backlog".into(),
        ))
    }
}

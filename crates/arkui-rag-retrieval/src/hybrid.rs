//! HybridRetriever —— 向量 + BM25 并行检索 + RRF 融合。
//!
//! Day 2 起真实可用：
//! - 接收 Embedder + VectorStore + BM25Index 三个依赖（trait object）
//! - retrieve() 内部并行执行两路检索，然后 RRF 融合
//! - BM25 路径返回空时退化为纯向量（仍然走 RRF，无副作用）
//!
//! Day 2 不做的：Reranker（CrossEncoderReranker 仍是 truncate 占位）。
//! 上层调用方负责把 Reranker 串到 HybridRetriever 后面。

use crate::rrf::{rrf_fuse, RRF_DEFAULT_K};
use arkui_rag_core::{Embedder, EnhancedQuery, Hit, HitSource, RagError, Result, Retriever};
use arkui_rag_storage::{BM25Index, VectorStore};
use async_trait::async_trait;
use std::sync::Arc;

pub struct HybridRetriever {
    name: String,
    embedder: Arc<dyn Embedder>,
    vector: Arc<dyn VectorStore>,
    bm25: Arc<dyn BM25Index>,
    /// 每路召回数量上限（送入 RRF 前的 top_k）。
    per_branch_topk: usize,
    rrf_k: f32,
}

impl HybridRetriever {
    pub fn new(
        embedder: Arc<dyn Embedder>,
        vector: Arc<dyn VectorStore>,
        bm25: Arc<dyn BM25Index>,
    ) -> Self {
        Self {
            name: "hybrid-rrf".to_string(),
            embedder,
            vector,
            bm25,
            per_branch_topk: 50,
            rrf_k: RRF_DEFAULT_K,
        }
    }

    pub fn with_per_branch_topk(mut self, n: usize) -> Self {
        self.per_branch_topk = n.max(1);
        self
    }

    pub fn with_rrf_k(mut self, k: f32) -> Self {
        self.rrf_k = k;
        self
    }
}

#[async_trait]
impl Retriever for HybridRetriever {
    fn name(&self) -> &str {
        &self.name
    }

    async fn retrieve(&self, query: &EnhancedQuery, top_k: usize) -> Result<Vec<Hit>> {
        // 1. 编码（向量路径用 rewritten + hyde_doc 优先）
        let text_for_vector = query
            .hyde_doc
            .as_deref()
            .unwrap_or(query.rewritten.as_str());
        let vec_q = self.embedder.encode_single(text_for_vector).await?;

        // 2. 并行两路检索
        let vec_fut = self
            .vector
            .search(&vec_q, self.per_branch_topk, &query.filters);
        let bm_fut = self
            .bm25
            .search(&query.rewritten, self.per_branch_topk, &query.filters);
        let (vec_hits, bm_hits) = tokio::try_join!(vec_fut, bm_fut)?;

        let vec_count = vec_hits.len();
        let bm_count = bm_hits.len();
        tracing::debug!(
            "hybrid: vector={} bm25={} (will RRF-fuse)",
            vec_count,
            bm_count
        );

        // 3. RRF 融合
        let mut fused = rrf_fuse(vec![vec_hits, bm_hits], self.rrf_k);
        fused.truncate(top_k);

        // 4. 标记来源为 Hybrid
        for h in fused.iter_mut() {
            h.source = HitSource::Hybrid;
        }

        if fused.is_empty() {
            // 不是错误（索引为空 / 都被 filter 过滤了），只是空结果
            tracing::warn!(
                "HybridRetriever 返回空 hits（vector={}, bm25={}, top_k={}）",
                vec_count,
                bm_count,
                top_k
            );
        }
        // 故意触发一次类型链：让 RagError 在签名里被引用
        let _: Result<()> = Ok(()).map_err(|_: std::convert::Infallible| {
            RagError::Retrieval("unreachable".into())
        });
        Ok(fused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_core::{Chunk, ChunkId, ChunkMetadata, ChunkType, QueryFilters};
    use arkui_rag_embedding::MockEmbedder;
    use arkui_rag_storage::{InMemoryBM25Index, InMemoryVectorStore};

    fn mk_chunk(id: &str, content: &str) -> Chunk {
        Chunk {
            id: ChunkId::new(id),
            content: content.to_string(),
            metadata: ChunkMetadata {
                source: format!("{}.md", id),
                r#type: ChunkType::Generic,
                ..ChunkMetadata::default()
            },
        }
    }

    #[tokio::test]
    async fn retrieves_topk_via_vector_path() {
        let embedder = Arc::new(MockEmbedder::new(64));
        let vector = Arc::new(InMemoryVectorStore::new("mock-64", 64));
        let bm25 = Arc::new(InMemoryBM25Index);

        // 索引 3 个 chunk
        let chunks = vec![
            mk_chunk("a", "ArkUI-X List 下拉刷新 Refresh"),
            mk_chunk("b", "Kotlin Coroutine launch"),
            mk_chunk("c", "Android Activity Lifecycle"),
        ];
        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embs = embedder.encode(&texts).await.unwrap();
        vector.upsert(&chunks, &embs).await.unwrap();

        let retriever = HybridRetriever::new(embedder, vector, bm25);
        let q = EnhancedQuery::passthrough("ArkUI-X List 下拉刷新 Refresh");
        let hits = retriever.retrieve(&q, 3).await.unwrap();
        assert_eq!(hits.len(), 3);
        // MockEmbedder 对同样文本返回同样向量 → chunk "a" cosine sim 必然为 1
        assert_eq!(hits[0].chunk.id.as_str(), "a");
        for h in &hits {
            assert!(matches!(h.source, HitSource::Hybrid));
        }
    }

    #[tokio::test]
    async fn empty_store_returns_empty_hits() {
        let embedder = Arc::new(MockEmbedder::new(16));
        let vector = Arc::new(InMemoryVectorStore::new("mock-16", 16));
        let bm25 = Arc::new(InMemoryBM25Index);

        let retriever = HybridRetriever::new(embedder, vector, bm25);
        let q = EnhancedQuery::passthrough("nothing here");
        let hits = retriever.retrieve(&q, 5).await.unwrap();
        assert!(hits.is_empty());
    }

    #[tokio::test]
    async fn respects_top_k() {
        let embedder = Arc::new(MockEmbedder::new(32));
        let vector = Arc::new(InMemoryVectorStore::new("mock-32", 32));
        let bm25 = Arc::new(InMemoryBM25Index);

        for i in 0..10 {
            let c = mk_chunk(&format!("c{}", i), &format!("content {}", i));
            let e = embedder.encode(&[c.content.as_str()]).await.unwrap();
            vector.upsert(&[c], &e).await.unwrap();
        }

        let retriever = HybridRetriever::new(embedder, vector, bm25);
        let q = EnhancedQuery::passthrough("content 0");
        let hits = retriever
            .retrieve(&q, 3)
            .await
            .unwrap();
        assert_eq!(hits.len(), 3);
    }

    #[tokio::test]
    async fn filters_by_platform() {
        let embedder = Arc::new(MockEmbedder::new(16));
        let vector = Arc::new(InMemoryVectorStore::new("mock-16", 16));
        let bm25 = Arc::new(InMemoryBM25Index);

        let mut harmony_only = mk_chunk("harmony", "harmony content");
        harmony_only.metadata.platforms = vec![arkui_rag_core::chunk::Platform::HarmonyOs];
        let mut android_only = mk_chunk("android", "android content");
        android_only.metadata.platforms = vec![arkui_rag_core::chunk::Platform::Android];

        let chunks = vec![harmony_only, android_only];
        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embs = embedder.encode(&texts).await.unwrap();
        vector.upsert(&chunks, &embs).await.unwrap();

        let retriever = HybridRetriever::new(embedder, vector, bm25);
        let q = EnhancedQuery {
            raw: "content".into(),
            rewritten: "content".into(),
            hyde_doc: None,
            entities: vec![],
            intent: Default::default(),
            filters: QueryFilters {
                platforms: vec![arkui_rag_core::chunk::Platform::HarmonyOs],
                ..Default::default()
            },
        };
        let hits = retriever.retrieve(&q, 10).await.unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk.id.as_str(), "harmony");
    }
}

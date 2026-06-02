#![doc = include_str!("../README.md")]

pub mod memory;

#[cfg(feature = "tantivy")]
pub mod tantivy_bm25;

#[cfg(feature = "lancedb")]
pub mod lancedb_store;

use arkui_rag_core::{Chunk, ChunkId, Hit, QueryFilters, Result};
use async_trait::async_trait;

pub use memory::{InMemoryBM25Index, InMemoryVectorStore, IndexSnapshot};

#[cfg(feature = "tantivy")]
pub use tantivy_bm25::TantivyBM25Index;

#[cfg(feature = "lancedb")]
pub use lancedb_store::LanceVectorStore;

/// 向量存储后端。LanceDB 适配是 `feature = "lancedb"` 的 Week 2 backlog；
/// Day 2 内置 `InMemoryVectorStore` 让 Mock Demo 端到端跑通。
#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn upsert(&self, chunks: &[Chunk], embeddings: &[Vec<f32>]) -> Result<()>;
    async fn search(
        &self,
        query_vec: &[f32],
        top_k: usize,
        filters: &QueryFilters,
    ) -> Result<Vec<Hit>>;
    async fn delete(&self, ids: &[ChunkId]) -> Result<()>;
    /// 当前已索引的 chunk 数（用于 stats / debug）。
    async fn len(&self) -> Result<usize>;
    /// 索引是否为空（默认基于 len · 与 clippy `len_without_is_empty` 兼容）。
    async fn is_empty(&self) -> Result<bool> {
        Ok(self.len().await? == 0)
    }
}

/// BM25 倒排索引后端。Tantivy 适配是 Week 3 backlog；
/// Day 2 内置 `InMemoryBM25Index` 返回空结果（让 RRF 退化为纯向量）。
#[async_trait]
pub trait BM25Index: Send + Sync {
    async fn upsert(&self, chunks: &[Chunk]) -> Result<()>;
    async fn search(&self, query: &str, top_k: usize, filters: &QueryFilters) -> Result<Vec<Hit>>;
    async fn delete(&self, ids: &[ChunkId]) -> Result<()>;
}

/// 元数据 + 原文存储。SQLite 适配是 Week 2 backlog。
#[async_trait]
pub trait MetadataStore: Send + Sync {
    async fn get(&self, id: &ChunkId) -> Result<Option<Chunk>>;
    async fn parent_of(&self, id: &ChunkId) -> Result<Option<Chunk>>;
}

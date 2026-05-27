#![doc = include_str!("../README.md")]

use arkui_rag_core::{Chunk, ChunkId, Hit, QueryFilters, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

/// 向量存储后端（Week 2 接 LanceDB）。
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
}

/// BM25 倒排索引后端（Week 2 接 Tantivy）。
#[async_trait]
pub trait BM25Index: Send + Sync {
    async fn upsert(&self, chunks: &[Chunk]) -> Result<()>;
    async fn search(
        &self,
        query: &str,
        top_k: usize,
        filters: &QueryFilters,
    ) -> Result<Vec<Hit>>;
    async fn delete(&self, ids: &[ChunkId]) -> Result<()>;
}

/// 元数据 + 原文存储（Week 2 接 SQLite）。
#[async_trait]
pub trait MetadataStore: Send + Sync {
    async fn get(&self, id: &ChunkId) -> Result<Option<Chunk>>;
    async fn parent_of(&self, id: &ChunkId) -> Result<Option<Chunk>>;
}

/// Day 1 占位：进程内 HashMap，不做实际向量检索/BM25。
/// 仅用于让上游 crate 能编译通过。
pub struct InMemoryStore {
    chunks: RwLock<HashMap<String, Chunk>>,
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            chunks: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl MetadataStore for InMemoryStore {
    async fn get(&self, id: &ChunkId) -> Result<Option<Chunk>> {
        Ok(self.chunks.read().unwrap().get(id.as_str()).cloned())
    }

    async fn parent_of(&self, id: &ChunkId) -> Result<Option<Chunk>> {
        let guard = self.chunks.read().unwrap();
        let me = guard.get(id.as_str());
        let parent_id = me.and_then(|c| c.metadata.parent_id.clone());
        Ok(parent_id.and_then(|pid| guard.get(pid.as_str()).cloned()))
    }
}

//! In-memory 后端实现 —— Day 2 端到端 Mock Demo 用。
//!
//! 设计：
//! - `InMemoryVectorStore`：`Vec<(Chunk, Vec<f32>)>` + cosine 相似度暴力扫；
//!   `save_to(path)` / `load_from(path)` JSON 序列化，让 CLI 的 index 和 query
//!   两次进程之间共享索引。
//! - `InMemoryBM25Index`：占位实现，返回空 hits。让 HybridRetriever 的 BM25
//!   路径不报错，RRF 退化为纯向量。
//!
//! **性能**：100 chunks 内毫秒级；超过 10k chunks 需要换 LanceDB。

use crate::{BM25Index, MetadataStore, VectorStore};
use arkui_rag_core::{Chunk, ChunkId, Hit, HitSource, QueryFilters, RagError, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexSnapshot {
    pub format_version: u32,
    pub embedder_model_id: String,
    pub embedder_dim: usize,
    pub entries: Vec<IndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub chunk: Chunk,
    pub embedding: Vec<f32>,
}

const CURRENT_FORMAT_VERSION: u32 = 1;

pub struct InMemoryVectorStore {
    entries: RwLock<Vec<IndexEntry>>,
    embedder_model_id: String,
    embedder_dim: usize,
}

impl InMemoryVectorStore {
    pub fn new(embedder_model_id: impl Into<String>, dim: usize) -> Self {
        Self {
            entries: RwLock::new(Vec::new()),
            embedder_model_id: embedder_model_id.into(),
            embedder_dim: dim,
        }
    }

    pub async fn save_to(&self, path: &Path) -> Result<()> {
        let entries = self.entries.read().unwrap().clone();
        let snap = IndexSnapshot {
            format_version: CURRENT_FORMAT_VERSION,
            embedder_model_id: self.embedder_model_id.clone(),
            embedder_dim: self.embedder_dim,
            entries,
        };
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let bytes = serde_json::to_vec_pretty(&snap)?;
        tokio::fs::write(path, bytes).await?;
        Ok(())
    }

    pub async fn load_from(path: &Path) -> Result<Self> {
        let bytes = tokio::fs::read(path).await?;
        let snap: IndexSnapshot = serde_json::from_slice(&bytes)?;
        if snap.format_version != CURRENT_FORMAT_VERSION {
            return Err(RagError::Storage(format!(
                "index format v{} not supported (current v{})",
                snap.format_version, CURRENT_FORMAT_VERSION
            )));
        }
        Ok(Self {
            entries: RwLock::new(snap.entries),
            embedder_model_id: snap.embedder_model_id,
            embedder_dim: snap.embedder_dim,
        })
    }

    pub fn embedder_model_id(&self) -> &str {
        &self.embedder_model_id
    }

    pub fn dim(&self) -> usize {
        self.embedder_dim
    }
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    // 假设两边都已 L2 归一化 → 点积即余弦相似度
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn passes_filter(chunk: &Chunk, filters: &QueryFilters) -> bool {
    if !filters.platforms.is_empty() {
        let any_match = chunk
            .metadata
            .platforms
            .iter()
            .any(|p| filters.platforms.contains(p));
        // 若 chunk.platforms 为空 → 视为通用，通过
        if !chunk.metadata.platforms.is_empty() && !any_match {
            return false;
        }
    }
    if let Some(v) = &filters.api_version {
        if let Some(cv) = &chunk.metadata.api_version {
            if cv != v {
                return false;
            }
        }
    }
    if !filters.include_deprecated && chunk.metadata.deprecated {
        return false;
    }
    if !filters.tags.is_empty() {
        let any_match = chunk.metadata.tags.iter().any(|t| filters.tags.contains(t));
        if !any_match {
            return false;
        }
    }
    true
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn upsert(&self, chunks: &[Chunk], embeddings: &[Vec<f32>]) -> Result<()> {
        if chunks.len() != embeddings.len() {
            return Err(RagError::Storage(format!(
                "upsert: chunks.len={} != embeddings.len={}",
                chunks.len(),
                embeddings.len()
            )));
        }
        let mut entries = self.entries.write().unwrap();
        for (c, e) in chunks.iter().zip(embeddings.iter()) {
            if e.len() != self.embedder_dim {
                return Err(RagError::Storage(format!(
                    "embedding dim {} != store dim {}",
                    e.len(),
                    self.embedder_dim
                )));
            }
            // upsert by id
            if let Some(slot) = entries.iter_mut().find(|x| x.chunk.id == c.id) {
                slot.chunk = c.clone();
                slot.embedding = e.clone();
            } else {
                entries.push(IndexEntry {
                    chunk: c.clone(),
                    embedding: e.clone(),
                });
            }
        }
        Ok(())
    }

    async fn search(
        &self,
        query_vec: &[f32],
        top_k: usize,
        filters: &QueryFilters,
    ) -> Result<Vec<Hit>> {
        if query_vec.len() != self.embedder_dim {
            return Err(RagError::Storage(format!(
                "query dim {} != store dim {}",
                query_vec.len(),
                self.embedder_dim
            )));
        }
        let entries = self.entries.read().unwrap();
        let mut scored: Vec<(f32, &IndexEntry)> = entries
            .iter()
            .filter(|e| passes_filter(&e.chunk, filters))
            .map(|e| (cosine_sim(query_vec, &e.embedding), e))
            .collect();
        scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(top_k);
        Ok(scored
            .into_iter()
            .map(|(s, e)| Hit {
                chunk: e.chunk.clone(),
                score: s,
                source: HitSource::Vector,
                vector_score: None,
                bm25_score: None,
            })
            .collect())
    }

    async fn delete(&self, ids: &[ChunkId]) -> Result<()> {
        let mut entries = self.entries.write().unwrap();
        entries.retain(|e| !ids.contains(&e.chunk.id));
        Ok(())
    }

    async fn len(&self) -> Result<usize> {
        Ok(self.entries.read().unwrap().len())
    }
}

#[async_trait]
impl MetadataStore for InMemoryVectorStore {
    async fn get(&self, id: &ChunkId) -> Result<Option<Chunk>> {
        let entries = self.entries.read().unwrap();
        Ok(entries
            .iter()
            .find(|e| e.chunk.id == *id)
            .map(|e| e.chunk.clone()))
    }

    async fn parent_of(&self, id: &ChunkId) -> Result<Option<Chunk>> {
        let entries = self.entries.read().unwrap();
        let me = entries.iter().find(|e| e.chunk.id == *id);
        let parent_id = me.and_then(|e| e.chunk.metadata.parent_id.clone());
        Ok(parent_id.and_then(|pid| {
            entries
                .iter()
                .find(|e| e.chunk.id == pid)
                .map(|e| e.chunk.clone())
        }))
    }
}

/// Day 2 占位：BM25 路径返回空 hits。让 HybridRetriever 调用不报错，
/// RRF 自然退化为只跑向量。Week 3 起接 Tantivy。
pub struct InMemoryBM25Index;

impl Default for InMemoryBM25Index {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl BM25Index for InMemoryBM25Index {
    async fn upsert(&self, _chunks: &[Chunk]) -> Result<()> {
        Ok(())
    }
    async fn search(
        &self,
        _query: &str,
        _top_k: usize,
        _filters: &QueryFilters,
    ) -> Result<Vec<Hit>> {
        tracing::debug!("InMemoryBM25Index::search returning empty (Week 3 will use Tantivy)");
        Ok(Vec::new())
    }
    async fn delete(&self, _ids: &[ChunkId]) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_core::{ChunkMetadata, ChunkType};

    fn mk_chunk(id: &str, src: &str) -> Chunk {
        Chunk {
            id: ChunkId::new(id),
            content: format!("content of {}", id),
            metadata: ChunkMetadata {
                source: src.to_string(),
                r#type: ChunkType::Generic,
                ..ChunkMetadata::default()
            },
        }
    }

    #[tokio::test]
    async fn upsert_and_search_topk() {
        let store = InMemoryVectorStore::new("mock-4", 4);
        let chunks = vec![
            mk_chunk("a", "p.md"),
            mk_chunk("b", "p.md"),
            mk_chunk("c", "p.md"),
        ];
        let embeddings = vec![
            // 已 L2 归一
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
        ];
        store.upsert(&chunks, &embeddings).await.unwrap();
        assert_eq!(store.len().await.unwrap(), 3);

        let q = [0.9, 0.1, 0.0, 0.0];
        // 归一化
        let norm = (q.iter().map(|x| x * x).sum::<f32>()).sqrt();
        let q: Vec<f32> = q.iter().map(|x| x / norm).collect();

        let hits = store.search(&q, 2, &QueryFilters::default()).await.unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].chunk.id.as_str(), "a"); // 最相近
    }

    #[tokio::test]
    async fn upsert_overwrites_same_id() {
        let store = InMemoryVectorStore::new("mock-2", 2);
        let c = mk_chunk("a", "p.md");
        store
            .upsert(std::slice::from_ref(&c), &[vec![1.0, 0.0]])
            .await
            .unwrap();
        store
            .upsert(std::slice::from_ref(&c), &[vec![0.0, 1.0]])
            .await
            .unwrap();
        assert_eq!(store.len().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn dim_mismatch_rejected() {
        let store = InMemoryVectorStore::new("mock-4", 4);
        let r = store
            .upsert(&[mk_chunk("a", "p.md")], &[vec![1.0, 0.0]])
            .await;
        assert!(r.is_err());
    }

    #[tokio::test]
    async fn save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("idx.json");

        let store = InMemoryVectorStore::new("mock-2", 2);
        store
            .upsert(&[mk_chunk("a", "p.md")], &[vec![1.0, 0.0]])
            .await
            .unwrap();
        store.save_to(&path).await.unwrap();

        let loaded = InMemoryVectorStore::load_from(&path).await.unwrap();
        assert_eq!(loaded.len().await.unwrap(), 1);
        assert_eq!(loaded.embedder_model_id(), "mock-2");
        assert_eq!(loaded.dim(), 2);
    }

    #[tokio::test]
    async fn bm25_stub_returns_empty() {
        let bm = InMemoryBM25Index;
        let hits = bm
            .search("anything", 5, &QueryFilters::default())
            .await
            .unwrap();
        assert!(hits.is_empty());
    }
}

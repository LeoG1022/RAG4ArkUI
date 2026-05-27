#![doc = include_str!("../README.md")]

use arkui_rag_core::{chunker::SourceLang, ASTChunker, Chunk, Embedder, RagError, Result};
use arkui_rag_storage::{BM25Index, VectorStore};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;
use walkdir::WalkDir;

/// 索引一次的统计结果。
#[derive(Debug, Clone, Serialize)]
pub struct IndexStats {
    pub files: usize,
    pub chunks: usize,
    pub skipped: usize,
    pub elapsed_ms: u128,
    pub embedder_model_id: String,
}

/// 索引流水线编排器。所有后端通过 trait object 注入，便于切换。
pub struct Indexer {
    chunker: Arc<dyn ASTChunker>,
    embedder: Arc<dyn Embedder>,
    vector: Arc<dyn VectorStore>,
    bm25: Arc<dyn BM25Index>,
    batch_size: usize,
}

impl Indexer {
    pub fn new(
        chunker: Arc<dyn ASTChunker>,
        embedder: Arc<dyn Embedder>,
        vector: Arc<dyn VectorStore>,
        bm25: Arc<dyn BM25Index>,
    ) -> Self {
        Self {
            chunker,
            embedder,
            vector,
            bm25,
            batch_size: 32,
        }
    }

    pub fn with_batch_size(mut self, n: usize) -> Self {
        self.batch_size = n.max(1);
        self
    }

    /// 索引一个目录。返回统计信息。
    ///
    /// 行为：
    /// 1. 递归遍历 `source`（跳过 `_index/`、隐藏目录、`.gitkeep`）
    /// 2. 按扩展名分发 chunker —— Day 2 只处理 `.md`，其他文件计入 skipped
    /// 3. 批量 embed → upsert vector + bm25
    pub async fn index_directory(&self, source: &Path) -> Result<IndexStats> {
        let start = Instant::now();
        let mut stats = IndexStats {
            files: 0,
            chunks: 0,
            skipped: 0,
            elapsed_ms: 0,
            embedder_model_id: self.embedder.model_id().to_string(),
        };

        let files = walk_corpus(source);
        let mut buffered_chunks: Vec<Chunk> = Vec::new();

        for path in files {
            let ext = path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_lowercase();
            let lang = match ext.as_str() {
                "md" | "markdown" => SourceLang::Markdown,
                _ => {
                    stats.skipped += 1;
                    tracing::debug!("skip {} (ext={})", path.display(), ext);
                    continue;
                }
            };

            let content = tokio::fs::read_to_string(&path).await?;
            let rel = path
                .strip_prefix(source)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let chunks = self.chunker.chunk(&rel, &content, lang).await?;
            if chunks.is_empty() {
                tracing::warn!("chunker produced 0 chunks for {}", rel);
                continue;
            }
            stats.files += 1;
            stats.chunks += chunks.len();
            buffered_chunks.extend(chunks);

            while buffered_chunks.len() >= self.batch_size {
                let drained: Vec<Chunk> = buffered_chunks.drain(..self.batch_size).collect();
                self.flush_batch(&drained).await?;
            }
        }

        if !buffered_chunks.is_empty() {
            let drained: Vec<Chunk> = std::mem::take(&mut buffered_chunks);
            self.flush_batch(&drained).await?;
        }

        stats.elapsed_ms = start.elapsed().as_millis();
        Ok(stats)
    }

    async fn flush_batch(&self, chunks: &[Chunk]) -> Result<()> {
        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embedder.encode(&texts).await?;
        if embeddings.len() != chunks.len() {
            return Err(RagError::Embedding(format!(
                "encode 返回 {} 向量但传入 {} chunks",
                embeddings.len(),
                chunks.len()
            )));
        }
        self.vector.upsert(chunks, &embeddings).await?;
        self.bm25.upsert(chunks).await?;
        Ok(())
    }
}

fn walk_corpus(source: &Path) -> Vec<PathBuf> {
    WalkDir::new(source)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            // 跳过隐藏目录 + _index/
            if e.file_type().is_dir() && (name.starts_with('.') || name == "_index") {
                return false;
            }
            true
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| {
            // 跳过 .gitkeep + 隐藏文件
            let name = p.file_name().and_then(|s| s.to_str()).unwrap_or("");
            !name.starts_with('.') && name != ".gitkeep"
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use arkui_rag_chunker::MarkdownChunker;
    use arkui_rag_embedding::MockEmbedder;
    use arkui_rag_storage::{InMemoryBM25Index, InMemoryVectorStore};

    #[tokio::test]
    async fn index_two_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(
            dir.path().join("a.md"),
            "# Top\n\n## A1\nbody a1\n\n## A2\nbody a2\n",
        )
        .await
        .unwrap();
        tokio::fs::write(
            dir.path().join("b.md"),
            "# Top\n\n## B1\nbody b1\n",
        )
        .await
        .unwrap();
        tokio::fs::write(dir.path().join(".gitkeep"), "").await.unwrap();
        tokio::fs::write(dir.path().join("ignored.kt"), "fun main(){}").await.unwrap();

        let embedder = Arc::new(MockEmbedder::new(64));
        let vector = Arc::new(InMemoryVectorStore::new("mock-64", 64));
        let bm25 = Arc::new(InMemoryBM25Index);
        let chunker = Arc::new(MarkdownChunker::new());

        let indexer = Indexer::new(chunker, embedder, vector.clone(), bm25);
        let stats = indexer.index_directory(dir.path()).await.unwrap();
        assert_eq!(stats.files, 2);
        assert_eq!(stats.chunks, 3); // A1 + A2 + B1
        assert_eq!(stats.skipped, 1); // ignored.kt
        assert_eq!(vector.len().await.unwrap(), 3);
    }
}

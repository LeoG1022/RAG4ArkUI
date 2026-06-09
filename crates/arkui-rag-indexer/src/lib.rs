#![doc = include_str!("../README.md")]

use arkui_rag_chunker::ChunkerDispatcher;
use arkui_rag_core::{chunker::SourceLang, Chunk, Embedder, RagError, Result};
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
/// Day 10 起：chunker 改为 `Arc<ChunkerDispatcher>` 支持多语言路由。
pub struct Indexer {
    dispatcher: Arc<ChunkerDispatcher>,
    embedder: Arc<dyn Embedder>,
    vector: Arc<dyn VectorStore>,
    bm25: Arc<dyn BM25Index>,
    batch_size: usize,
}

impl Indexer {
    pub fn new(
        dispatcher: Arc<ChunkerDispatcher>,
        embedder: Arc<dyn Embedder>,
        vector: Arc<dyn VectorStore>,
        bm25: Arc<dyn BM25Index>,
    ) -> Self {
        Self {
            dispatcher,
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

    /// 索引一个目录。返回统计信息。无 checkpoint（兼容老路径）。
    pub async fn index_directory(&self, source: &Path) -> Result<IndexStats> {
        self.index_directory_with_checkpoint(source, 0, None).await
    }

    /// Round 55: 索引一个目录 · 支持中途 checkpoint persist
    ///
    /// - `checkpoint_every_files = 0` 关 checkpoint（同 index_directory）
    /// - `checkpoint_every_files > 0` 每处理 N files 持久化一次
    /// - `persist_path` 传给 `VectorStore::persist_checkpoint`（InMemoryVectorStore 写 index.json）
    ///
    /// 设计：长 build（数小时）死掉时不丢失全部进度 · 重启可从 checkpoint 接续（需上层逻辑）
    pub async fn index_directory_with_checkpoint(
        &self,
        source: &Path,
        checkpoint_every_files: usize,
        persist_path: Option<&Path>,
    ) -> Result<IndexStats> {
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
        let mut files_since_checkpoint = 0usize;

        for path in files {
            let lang = ChunkerDispatcher::detect_lang(&path);
            if matches!(lang, SourceLang::Auto) || !self.dispatcher.has(lang) {
                stats.skipped += 1;
                tracing::debug!("skip {} (lang={:?}; not registered)", path.display(), lang);
                continue;
            }

            // Round 54: 容错读 · 非 UTF-8 文件（如 GBK 编码）lossy 替换坏字节为 U+FFFD
            let bytes = tokio::fs::read(&path).await?;
            let content = match std::str::from_utf8(&bytes) {
                Ok(s) => s.to_string(),
                Err(e) => {
                    tracing::warn!(
                        "{} 非 UTF-8 (pos {}): {} · 用 from_utf8_lossy 兜底",
                        path.display(),
                        e.valid_up_to(),
                        e
                    );
                    String::from_utf8_lossy(&bytes).into_owned()
                }
            };
            let rel = path
                .strip_prefix(source)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            let chunks_result = self.dispatcher.chunk_as(&rel, &content, lang).await;
            let chunks = match chunks_result {
                Ok(c) => c,
                Err(RagError::NotImplemented(msg)) => {
                    tracing::warn!("skip {} (chunker NotImplemented: {})", rel, msg);
                    stats.skipped += 1;
                    continue;
                }
                Err(e) => return Err(e),
            };
            if chunks.is_empty() {
                tracing::warn!("chunker produced 0 chunks for {}", rel);
                continue;
            }
            stats.files += 1;
            stats.chunks += chunks.len();
            files_since_checkpoint += 1;
            buffered_chunks.extend(chunks);

            while buffered_chunks.len() >= self.batch_size {
                let drained: Vec<Chunk> = buffered_chunks.drain(..self.batch_size).collect();
                self.flush_batch(&drained).await?;
            }

            // Round 55: checkpoint 持久化（每 N files · 0 = off）
            if checkpoint_every_files > 0 && files_since_checkpoint >= checkpoint_every_files {
                // 先 flush buffer 让所有已收 chunk 进入 store
                if !buffered_chunks.is_empty() {
                    let drained: Vec<Chunk> = std::mem::take(&mut buffered_chunks);
                    self.flush_batch(&drained).await?;
                }
                match self.vector.persist_checkpoint(persist_path).await {
                    Ok(_) => tracing::info!(
                        "✅ checkpoint · files={} chunks={} persisted to {:?}",
                        stats.files, stats.chunks, persist_path
                    ),
                    Err(e) => tracing::warn!(
                        "⚠️ checkpoint 失败（继续 build）: {} · files={} chunks={}",
                        e, stats.files, stats.chunks
                    ),
                }
                files_since_checkpoint = 0;
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
            // 跳过隐藏目录 + _index/ · 但不能 reject root（depth==0）
            // 否则 tempfile::tempdir() 这种 /tmp/.tmpXXXX 路径会被整个跳过
            if e.depth() > 0
                && e.file_type().is_dir()
                && (name.starts_with('.') || name == "_index")
            {
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
    use arkui_rag_chunker::{ChunkerDispatcher, MarkdownChunker};
    use arkui_rag_embedding::MockEmbedder;
    use arkui_rag_storage::{InMemoryBM25Index, InMemoryVectorStore};

    fn dispatcher_markdown_only() -> Arc<ChunkerDispatcher> {
        Arc::new(
            ChunkerDispatcher::new()
                .register(SourceLang::Markdown, Arc::new(MarkdownChunker::new())),
        )
    }

    #[tokio::test]
    async fn index_two_markdown_files() {
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(
            dir.path().join("a.md"),
            "# Top\n\n## A1\nbody a1\n\n## A2\nbody a2\n",
        )
        .await
        .unwrap();
        tokio::fs::write(dir.path().join("b.md"), "# Top\n\n## B1\nbody b1\n")
            .await
            .unwrap();
        tokio::fs::write(dir.path().join(".gitkeep"), "")
            .await
            .unwrap();
        tokio::fs::write(dir.path().join("ignored.kt"), "fun main(){}")
            .await
            .unwrap();

        let embedder = Arc::new(MockEmbedder::new(64));
        let vector = Arc::new(InMemoryVectorStore::new("mock-64", 64));
        let bm25 = Arc::new(InMemoryBM25Index);

        let indexer = Indexer::new(dispatcher_markdown_only(), embedder, vector.clone(), bm25);
        let stats = indexer.index_directory(dir.path()).await.unwrap();
        assert_eq!(stats.files, 2);
        assert_eq!(stats.chunks, 3); // A1 + A2 + B1
        assert_eq!(stats.skipped, 1); // ignored.kt
        assert_eq!(vector.len().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn ets_files_skipped_when_no_ts_chunker_registered() {
        // 默认 dispatcher 只注册 Markdown，.ets 文件应被 skipped
        let dir = tempfile::tempdir().unwrap();
        tokio::fs::write(dir.path().join("a.ets"), "@Component struct A {}")
            .await
            .unwrap();
        tokio::fs::write(dir.path().join("b.md"), "# Top\n\nbody\n")
            .await
            .unwrap();

        let embedder = Arc::new(MockEmbedder::new(32));
        let vector = Arc::new(InMemoryVectorStore::new("mock-32", 32));
        let bm25 = Arc::new(InMemoryBM25Index);

        let indexer = Indexer::new(dispatcher_markdown_only(), embedder, vector.clone(), bm25);
        let stats = indexer.index_directory(dir.path()).await.unwrap();
        assert_eq!(stats.files, 1);
        assert_eq!(stats.skipped, 1);
    }
}

//! `OnnxReranker` —— `Reranker` trait 的 ONNX 真实实现。
//!
//! 包装 `RerankerModel`（同步 API）为 async trait 实现，
//! 用 `tokio::task::spawn_blocking` 桥接，避免阻塞 tokio worker。

use crate::reranker_onnx::RerankerModel;
use arkui_rag_core::{Hit, HitSource, RagError, Reranker, Result};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

pub struct OnnxReranker {
    inner: Arc<RerankerModel>,
    model_id: String,
}

impl OnnxReranker {
    /// 加载本地 BGE-Reranker-v2 ONNX 模型目录。
    ///
    /// `model_id` 用于日志 / 标识（推荐 "bge-reranker-v2-m3" / "bge-reranker-v2-gemma"）。
    pub fn load(model_dir: &Path, model_id: impl Into<String>) -> Result<Self> {
        let inner = RerankerModel::load(model_dir)
            .map_err(|e| RagError::Rerank(format!("OnnxReranker::load 失败: {}", e)))?;
        Ok(Self {
            inner: Arc::new(inner),
            model_id: model_id.into(),
        })
    }
}

#[async_trait]
impl Reranker for OnnxReranker {
    fn model_id(&self) -> &str {
        &self.model_id
    }

    async fn rerank(&self, query: &str, hits: Vec<Hit>, top_n: usize) -> Result<Vec<Hit>> {
        if hits.is_empty() {
            return Ok(Vec::new());
        }

        // 准备 (query, content) 对
        let pairs: Vec<(String, String)> = hits
            .iter()
            .map(|h| (query.to_string(), h.chunk.content.clone()))
            .collect();

        let inner = self.inner.clone();
        let scores = tokio::task::spawn_blocking(move || inner.score(&pairs))
            .await
            .map_err(|e| RagError::Rerank(format!("spawn_blocking join 失败: {}", e)))?
            .map_err(|e| RagError::Rerank(format!("ONNX rerank 推理失败: {}", e)))?;

        if scores.len() != hits.len() {
            return Err(RagError::Rerank(format!(
                "scores 数 {} 与 hits 数 {} 不匹配",
                scores.len(),
                hits.len()
            )));
        }

        // 用新 score 替换 hits 的 score，按降序排
        let mut scored: Vec<Hit> = hits
            .into_iter()
            .zip(scores.into_iter())
            .map(|(mut h, s)| {
                h.score = s;
                h.source = HitSource::Reranked;
                h
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(top_n);
        Ok(scored)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn load_missing_model_returns_err() {
        let r = OnnxReranker::load(Path::new("/nonexistent/path/to/reranker"), "test-reranker");
        assert!(r.is_err());
    }

    /// 真模型集成测试：需 ARKUI_RAG_RERANKER_DIR 环境变量
    #[tokio::test]
    #[ignore = "需 BGE-Reranker-v2 模型；设环境变量 ARKUI_RAG_RERANKER_DIR=/path 后 cargo test -- --ignored"]
    async fn rerank_with_real_model() {
        use arkui_rag_core::{Chunk, ChunkId, ChunkMetadata};
        let dir = match std::env::var("ARKUI_RAG_RERANKER_DIR") {
            Ok(d) => d,
            Err(_) => return,
        };
        let r = OnnxReranker::load(Path::new(&dir), "bge-reranker-v2-m3").unwrap();

        let mk_hit = |id: &str, content: &str, score: f32| Hit {
            chunk: Chunk {
                id: ChunkId::new(id),
                content: content.to_string(),
                metadata: ChunkMetadata::default(),
            },
            score,
            source: HitSource::Hybrid,
            vector_score: None,
            bm25_score: None,
        };

        let hits = vec![
            mk_hit("rel", "ArkUI-X 用 Refresh 组件实现下拉刷新", 0.3),
            mk_hit("noise", "完全无关的天气预报今天晴朗", 0.5),
            mk_hit("partial", "Refresh 是一个组件", 0.2),
        ];
        let reranked = r.rerank("下拉刷新怎么实现", hits, 3).await.unwrap();
        assert_eq!(reranked.len(), 3);
        // 真模型应把 "rel" 排第 1
        assert_eq!(reranked[0].chunk.id.as_str(), "rel");
        // 噪音应当被排最后
        assert_eq!(reranked[2].chunk.id.as_str(), "noise");
    }
}

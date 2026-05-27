//! `OnnxEmbedder` —— `Embedder` trait 的 ONNX 真实实现。
//!
//! 桥接策略：底层的 `EmbeddingModel`（§7.2 verbatim）是同步 API，
//! 这里用 `tokio::task::spawn_blocking` 把它移到 blocking 线程池，
//! 避免阻塞 tokio runtime 的 worker。
//!
//! 返回值从 `ndarray::Array2<f32>` 转成 `Vec<Vec<f32>>`，与 `Embedder::encode`
//! 签名对齐（trait 不依赖 ndarray，保持 core 轻量）。

use crate::onnx::EmbeddingModel;
use arkui_rag_core::{Embedder, RagError, Result};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

pub struct OnnxEmbedder {
    inner: Arc<EmbeddingModel>,
    model_id: String,
}

impl OnnxEmbedder {
    /// 加载本地 ONNX 模型目录（包含 `model.onnx` 和 `tokenizer.json`）。
    ///
    /// `model_id` 用于：
    /// - 索引文件里写入 `embedder_model_id`，加载时校验防错配
    /// - 检索 / 日志里标识当前模型
    /// 推荐值："bge-m3"、"qwen3-embedding-0.6b"。
    pub fn load(model_dir: &Path, model_id: impl Into<String>) -> Result<Self> {
        let inner = EmbeddingModel::load(model_dir)
            .map_err(|e| RagError::Embedding(format!("OnnxEmbedder::load 失败: {}", e)))?;
        Ok(Self {
            inner: Arc::new(inner),
            model_id: model_id.into(),
        })
    }
}

#[async_trait]
impl Embedder for OnnxEmbedder {
    async fn encode(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Err(RagError::Embedding(
                "OnnxEmbedder::encode called with empty batch".into(),
            ));
        }
        // 把 &str 借用拷贝为 owned String 才能跨线程移动
        let owned: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        let inner = self.inner.clone();

        let arr = tokio::task::spawn_blocking(move || {
            let refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
            inner.encode(&refs)
        })
        .await
        .map_err(|e| RagError::Embedding(format!("spawn_blocking join 失败: {}", e)))?
        .map_err(|e| RagError::Embedding(format!("ONNX encode 推理失败: {}", e)))?;

        // Array2<f32> → Vec<Vec<f32>>（按行）
        let batch = arr.nrows();
        let mut out = Vec::with_capacity(batch);
        for i in 0..batch {
            out.push(arr.row(i).to_vec());
        }
        Ok(out)
    }

    fn dim(&self) -> usize {
        self.inner.dim()
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[cfg(test)]
mod tests {
    // 这里不能放真实的端到端测试 —— 需要 ~2GB BGE-M3 模型文件。
    // 真实模型测试走 examples/ 或单独的 ignore=true 测试。
    use super::*;

    #[tokio::test]
    async fn load_missing_model_returns_err() {
        let r = OnnxEmbedder::load(Path::new("/nonexistent/path/to/bge-m3"), "bge-m3-test");
        assert!(r.is_err());
        let msg = format!("{}", r.unwrap_err());
        assert!(msg.contains("OnnxEmbedder::load") || msg.contains("加载"));
    }

    // 真模型集成测试：仅在环境变量 ARKUI_RAG_BGE_M3_DIR 指向本地模型时运行
    #[tokio::test]
    #[ignore = "需 BGE-M3 模型；设环境变量 ARKUI_RAG_BGE_M3_DIR=/path/to/bge-m3 后跑 cargo test -- --ignored"]
    async fn encode_with_real_bge_m3() {
        let dir = match std::env::var("ARKUI_RAG_BGE_M3_DIR") {
            Ok(d) => d,
            Err(_) => return,
        };
        let emb = OnnxEmbedder::load(Path::new(&dir), "bge-m3").unwrap();
        let v = emb
            .encode_single("ArkUI-X 下拉刷新 Refresh 组件")
            .await
            .unwrap();
        assert_eq!(v.len(), emb.dim());
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "L2 归一化应近似 1.0，实际 {}", norm);
    }
}

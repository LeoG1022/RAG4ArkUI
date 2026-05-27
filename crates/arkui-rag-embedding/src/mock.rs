//! MockEmbedder —— 确定性伪随机向量。
//!
//! 用途：Day 1 没有模型也能跑通端到端类型链。
//! 哈希文本得到一个固定 seed，从 seed 派生 `dim` 维 L2 归一化向量。
//! 相同文本永远得到相同向量（便于测试断言）。

use arkui_rag_core::{Embedder, RagError, Result};
use async_trait::async_trait;
use std::hash::{Hash, Hasher};

pub struct MockEmbedder {
    dim: usize,
    model_id: String,
}

impl MockEmbedder {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            model_id: format!("mock-{}", dim),
        }
    }

    fn embed_one(&self, text: &str) -> Vec<f32> {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        // 简易线性同余生成器，避免引入 rand crate
        let mut state = seed;
        let mut v = Vec::with_capacity(self.dim);
        for _ in 0..self.dim {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            // 映射到 [-1, 1]
            let f = ((state >> 33) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            v.push(f);
        }
        // L2 归一化
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-9);
        for x in v.iter_mut() {
            *x /= norm;
        }
        v
    }
}

#[async_trait]
impl Embedder for MockEmbedder {
    async fn encode(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Err(RagError::Embedding("encode called with empty batch".into()));
        }
        Ok(texts.iter().map(|t| self.embed_one(t)).collect())
    }

    fn dim(&self) -> usize {
        self.dim
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn deterministic_same_text() {
        let m = MockEmbedder::new(64);
        let a = m.encode_single("ArkUI-X").await.unwrap();
        let b = m.encode_single("ArkUI-X").await.unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 64);
    }

    #[tokio::test]
    async fn different_text_different_vector() {
        let m = MockEmbedder::new(64);
        let a = m.encode_single("ArkUI-X").await.unwrap();
        let b = m.encode_single("KMP").await.unwrap();
        assert_ne!(a, b);
    }

    #[tokio::test]
    async fn l2_normalized() {
        let m = MockEmbedder::new(128);
        let v = m.encode_single("any text").await.unwrap();
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
    }
}

//! `Embedder` trait —— 文本嵌入接口。
//!
//! 用在两处：
//! 1. 索引时：每个 chunk 的 content 编码后写入向量库
//! 2. 检索时：用户 query 编码后查询向量库
//!
//! 不同模型（BGE-M3 / Qwen3 / 自训）通过实现本 trait 接入。

use crate::error::Result;
use async_trait::async_trait;

/// 文本嵌入接口。
///
/// 实现必须保证：
/// - 输出向量已 L2 归一化（向量库默认走余弦 / 点积）
/// - `dim()` 与 `encode()` 返回向量维度一致
/// - 批量编码（`encode`）比逐条编码（`encode_single`）有性能优势
#[async_trait]
pub trait Embedder: Send + Sync {
    /// 批量编码。返回形状为 `[batch_size, dim]` 的浮点矩阵（按行存储）。
    async fn encode(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// 单条编码（默认实现走 batch=1）。
    async fn encode_single(&self, text: &str) -> Result<Vec<f32>> {
        let mut batch = self.encode(&[text]).await?;
        batch
            .pop()
            .ok_or_else(|| crate::error::RagError::Embedding("encode returned empty batch".into()))
    }

    /// 向量维度（如 BGE-M3 = 1024）。
    fn dim(&self) -> usize;

    /// 模型标识（用于索引版本绑定 —— 模型升级要触发重建索引）。
    fn model_id(&self) -> &str;
}

//! `Reranker` trait —— 精排接口。
//!
//! 二阶段检索的第二阶段：召回器给 Top-50，精排器精排到 Top-10。
//! 见技术方案 §6.2 模型 2、§7.1 模型 3。

use crate::error::Result;
use crate::hit::Hit;
use async_trait::async_trait;

/// 精排器接口。
///
/// 实现通常是 cross-encoder（query × document 拼接送进单一模型），
/// 比 embedding 的双塔模型精度高，但每对都要现场推理 → 必须先召回缩小到 Top-K。
#[async_trait]
pub trait Reranker: Send + Sync {
    /// 给定 query 和候选 hits，返回重排后的 hits（按精排 score 降序）。
    ///
    /// 实现可以选择 truncate 到 `top_n`（典型 50 → 10）。
    async fn rerank(&self, query: &str, hits: Vec<Hit>, top_n: usize) -> Result<Vec<Hit>>;

    /// 模型标识。
    fn model_id(&self) -> &str;
}

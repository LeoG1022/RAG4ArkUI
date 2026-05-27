//! `Retriever` trait —— 检索引擎的统一接口。
//!
//! 所有具体实现（Vector / BM25 / Hybrid / Graph）都实现本 trait，
//! 让上层 `RagEngine` 可以热插拔。

use crate::error::Result;
use crate::hit::Hit;
use crate::query::EnhancedQuery;
use async_trait::async_trait;

/// 检索器统一接口。
///
/// **契约**：
/// - 必须异步（流水线在 server 形态下需要异步并发）
/// - 必须支持 top_k 截断（避免返回数百条命中拖垮后续 rerank）
/// - 必须遵守 `query.filters` 做元数据预过滤
#[async_trait]
pub trait Retriever: Send + Sync {
    /// 给定增强后的查询和返回数量上限，返回命中列表（按 score 降序）。
    async fn retrieve(&self, query: &EnhancedQuery, top_k: usize) -> Result<Vec<Hit>>;

    /// 检索器名（用于调试 / RRF source 标记）。
    fn name(&self) -> &str;
}

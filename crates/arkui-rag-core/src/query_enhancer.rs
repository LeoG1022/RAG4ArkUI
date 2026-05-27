//! `QueryEnhancer` trait —— Query 改写 / 增强接口。
//!
//! 流水线：原始 query → `QueryEnhancer::enhance()` → `EnhancedQuery`（含 hyde_doc / entities / intent / filters）
//!                  → `HybridRetriever::retrieve()` → Top-K Hits
//!
//! 见技术方案 §2.4 检索流水线设计、§6.2 模型 3、§1.2 Advanced RAG。

use crate::error::Result;
use crate::query::EnhancedQuery;
use async_trait::async_trait;

/// Query 改写 / 增强接口。
///
/// **典型实现**：
/// - `PassthroughEnhancer`：透传，不改写（默认）
/// - `MockHydeEnhancer`：用确定性规则生成 "假代码"（Day 7 占位）
/// - `RemoteHydeEnhancer`：调远程 LLM 生成假文档（Week 3+ 接入）
///
/// **契约**：
/// - `enhance()` 返回的 `EnhancedQuery.raw` 必须保留原始 query 文本（便于审计 / 报告）
/// - `EnhancedQuery.rewritten` 可与 raw 相同（不改写）也可改写
/// - `EnhancedQuery.hyde_doc` 是 Optional —— 仅 HyDE 类实现填充
#[async_trait]
pub trait QueryEnhancer: Send + Sync {
    /// 增强一个原始 query 为 EnhancedQuery。
    async fn enhance(&self, raw: &str) -> Result<EnhancedQuery>;

    /// 实现标识（用于日志 / 报告区分）。
    fn name(&self) -> &str;
}

/// 透传实现：raw → EnhancedQuery::passthrough，不改写。
///
/// 用作默认值 / 关闭 enhance 时的占位。
pub struct PassthroughEnhancer;

impl Default for PassthroughEnhancer {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl QueryEnhancer for PassthroughEnhancer {
    async fn enhance(&self, raw: &str) -> Result<EnhancedQuery> {
        Ok(EnhancedQuery::passthrough(raw))
    }

    fn name(&self) -> &str {
        "passthrough"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn passthrough_preserves_raw_and_no_hyde() {
        let e = PassthroughEnhancer;
        let q = e.enhance("ArkUI-X 下拉刷新").await.unwrap();
        assert_eq!(q.raw, "ArkUI-X 下拉刷新");
        assert_eq!(q.rewritten, "ArkUI-X 下拉刷新");
        assert!(q.hyde_doc.is_none());
        assert_eq!(e.name(), "passthrough");
    }
}

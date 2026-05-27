//! 查询的中间表示。
//!
//! 流水线：原始 query → `QueryRouter` 分类 → `QueryEnhancer` 增强 → `EnhancedQuery` → 检索器。
//! 见技术方案 §2.4 检索流水线设计。

use crate::chunk::Platform;
use serde::{Deserialize, Serialize};

/// 查询意图（由 Query Router 给出）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryIntent {
    /// 新建组件 / 写代码
    NewComponent,
    /// 修复编译 / 运行错误
    ErrorFix,
    /// API 查询
    ApiLookup,
    /// 迁移适配（KMP / Android / iOS → ArkUI-X）
    Migration,
    /// 一多改造
    Adaptive,
    /// 闲聊 / 不需要 RAG
    Chitchat,
    /// 未分类（默认走完整 hybrid）
    Generic,
}

impl Default for QueryIntent {
    fn default() -> Self {
        Self::Generic
    }
}

/// 查询过滤条件（用于元数据预过滤，§4.2 决策 6）。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryFilters {
    #[serde(default)]
    pub platforms: Vec<Platform>,
    #[serde(default)]
    pub api_version: Option<String>,
    #[serde(default)]
    pub source_framework: Option<String>,
    #[serde(default)]
    pub include_deprecated: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// 增强后的查询（HyDE / 实体抽取 / 改写后的形态）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedQuery {
    /// 原始 query。
    pub raw: String,
    /// 改写后的 query（可能跟 raw 一致）。
    pub rewritten: String,
    /// HyDE 生成的假代码 / 假文档（用于向量检索）。
    #[serde(default)]
    pub hyde_doc: Option<String>,
    /// 抽取出的实体（组件名 / API 名 / 平台关键词）。
    #[serde(default)]
    pub entities: Vec<String>,
    /// 意图分类。
    #[serde(default)]
    pub intent: QueryIntent,
    /// 元数据过滤。
    #[serde(default)]
    pub filters: QueryFilters,
}

impl EnhancedQuery {
    /// 从原始字符串构造一个最小可用 EnhancedQuery（无增强）。
    pub fn passthrough(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        Self {
            rewritten: raw.clone(),
            raw,
            hyde_doc: None,
            entities: Vec::new(),
            intent: QueryIntent::Generic,
            filters: QueryFilters::default(),
        }
    }
}

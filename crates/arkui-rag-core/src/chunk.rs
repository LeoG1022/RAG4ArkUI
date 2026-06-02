//! Chunk 与元数据类型。
//!
//! 元数据 schema 与技术方案 §4.2 决策 6 对齐：
//! - `platforms` 支持多平台过滤（HarmonyOS / Android / iOS）
//! - `api_version` 用于过滤已弃用 / 未来 API
//! - `type` 区分 api_doc / code_example / migration_rule / error_fix
//! - `parent_id` 支持 §1.4 "Parent-Child 索引"

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 全局唯一的 chunk 标识。
///
/// 推荐生成方式：`{source_path}#{heading_path}@{line_range}` 的 SHA-1 前 12 位。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChunkId(pub String);

impl ChunkId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 切分得到的文本单元。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: ChunkId,
    /// 文本内容（已去掉 frontmatter 等噪声）。
    pub content: String,
    /// 元数据 —— 用于过滤、引用、Parent-Child 扩展。
    pub metadata: ChunkMetadata,
}

/// Chunk 元数据。字段名与技术方案 §4.2 决策 6 的 JSON schema 对齐，
/// 命名沿用 snake_case 以便 HTTP / MCP 直接序列化。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChunkMetadata {
    /// 原始文档路径（相对 corpus/ 根）。
    pub source: String,
    /// Markdown heading 链 / AST 路径，用于 Parent-Child 扩展。
    #[serde(default)]
    pub heading_path: Vec<String>,
    /// 起止行号 [start, end]，闭区间。
    #[serde(default)]
    pub line_range: Option<(u32, u32)>,
    /// 支持的平台。空向量表示通用。
    #[serde(default)]
    pub platforms: Vec<Platform>,
    /// ArkUI-X API 版本（如 "ArkUI-X 1.2"）。
    #[serde(default)]
    pub api_version: Option<String>,
    /// 是否已废弃。
    #[serde(default)]
    pub deprecated: bool,
    /// chunk 类型（api_doc / code_example / migration_rule / error_fix）。
    pub r#type: ChunkType,
    /// 源框架（仅迁移规则用：KMP / Android / iOS）。
    #[serde(default)]
    pub source_framework: Option<String>,
    /// 复杂度（basic / intermediate / advanced）。
    #[serde(default)]
    pub complexity: Option<String>,
    /// 自由 tag。
    #[serde(default)]
    pub tags: Vec<String>,
    /// 父 chunk id（Parent-Child 索引，§1.4）。
    #[serde(default)]
    pub parent_id: Option<ChunkId>,
    /// 任意扩展字段。
    #[serde(default)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    HarmonyOs,
    Android,
    Ios,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ChunkType {
    ApiDoc,
    CodeExample,
    MigrationRule,
    ErrorFix,
    #[default]
    Generic,
}

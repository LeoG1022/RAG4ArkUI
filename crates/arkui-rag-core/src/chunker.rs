//! `ASTChunker` trait —— 文档切分接口。
//!
//! 切分质量决定 RAG 上限（§1.6 第 2 条产品级 RAG 原则）。
//! 实现按文档语言走策略模式：tree-sitter（ArkTS / Kotlin / Swift）+ markdown AST + PDF。

use crate::chunk::Chunk;
use crate::error::Result;
use async_trait::async_trait;

/// 切分器输入语言 / 格式。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SourceLang {
    Markdown,
    ArkTs,
    Kotlin,
    Swift,
    Json,
    Auto,
}

/// 文档切分接口。
///
/// **契约**：
/// - 必须保留语义边界（Component / function / heading），绝不按固定字符数切
/// - 返回的 Chunk 必须填好 metadata.source / heading_path / line_range
/// - 必须支持 Parent-Child 结构：父 chunk 的 id 写入子 chunk 的 `parent_id`
#[async_trait]
pub trait ASTChunker: Send + Sync {
    /// 切分内容。`source_path` 是相对 corpus/ 根的路径（写入元数据）。
    async fn chunk(
        &self,
        source_path: &str,
        content: &str,
        lang: SourceLang,
    ) -> Result<Vec<Chunk>>;

    /// 切分器名。
    fn name(&self) -> &str;
}

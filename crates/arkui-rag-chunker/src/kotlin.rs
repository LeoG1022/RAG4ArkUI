//! KotlinChunker —— Kotlin (`.kt`) 代码切分（**Day 10 stub**）。
//!
//! 当前实现：feature `kotlin` 启用时返回 `NotImplemented` 错误，
//! 让 indexer 知道有这个 chunker 但跳过 .kt 文件。
//! 真实接入 `tree-sitter-kotlin` 是 Week 2-3 backlog（社区维护活跃度需评估）。

#![cfg(feature = "kotlin")]

use arkui_rag_core::{chunker::SourceLang, ASTChunker, Chunk, RagError, Result};
use async_trait::async_trait;

pub struct KotlinChunker {
    name: String,
}

impl Default for KotlinChunker {
    fn default() -> Self {
        Self {
            name: "treesitter-kotlin-stub".to_string(),
        }
    }
}

impl KotlinChunker {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ASTChunker for KotlinChunker {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chunk(
        &self,
        _source_path: &str,
        _content: &str,
        lang: SourceLang,
    ) -> Result<Vec<Chunk>> {
        if !matches!(lang, SourceLang::Kotlin | SourceLang::Auto) {
            return Err(RagError::Chunker(format!(
                "KotlinChunker 不支持语言 {:?}",
                lang
            )));
        }
        Err(RagError::NotImplemented(
            "KotlinChunker 实装待 Week 2-3 backlog（接 tree-sitter-kotlin）".into(),
        ))
    }
}

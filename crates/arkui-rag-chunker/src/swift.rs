//! SwiftChunker —— Swift (`.swift`) 代码切分（**Day 10 stub**）。
//!
//! 当前实现：feature `swift` 启用时返回 `NotImplemented` 错误。
//! 真实接入 `tree-sitter-swift` 是 Week 2-3 backlog。

#![cfg(feature = "swift")]

use arkui_rag_core::{chunker::SourceLang, ASTChunker, Chunk, RagError, Result};
use async_trait::async_trait;

pub struct SwiftChunker {
    name: String,
}

impl Default for SwiftChunker {
    fn default() -> Self {
        Self {
            name: "treesitter-swift-stub".to_string(),
        }
    }
}

impl SwiftChunker {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl ASTChunker for SwiftChunker {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chunk(
        &self,
        _source_path: &str,
        _content: &str,
        lang: SourceLang,
    ) -> Result<Vec<Chunk>> {
        if !matches!(lang, SourceLang::Swift | SourceLang::Auto) {
            return Err(RagError::Chunker(format!(
                "SwiftChunker 不支持语言 {:?}",
                lang
            )));
        }
        Err(RagError::NotImplemented(
            "SwiftChunker 实装待 Week 2-3 backlog（接 tree-sitter-swift）".into(),
        ))
    }
}

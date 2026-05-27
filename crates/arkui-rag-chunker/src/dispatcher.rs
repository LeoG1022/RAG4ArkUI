//! ChunkerDispatcher —— 按 SourceLang 路由到对应的 ASTChunker。
//!
//! 让 Indexer 不再死绑单一 chunker，能处理混合文档类型的 corpus
//! （markdown + .ets + .kt + .swift）。

use arkui_rag_core::{chunker::SourceLang, ASTChunker, Chunk, RagError, Result};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

pub struct ChunkerDispatcher {
    by_lang: HashMap<SourceLang, Arc<dyn ASTChunker>>,
}

impl Default for ChunkerDispatcher {
    fn default() -> Self {
        Self {
            by_lang: HashMap::new(),
        }
    }
}

impl ChunkerDispatcher {
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个 lang → chunker 的路由。
    /// 重复注册同 lang 会覆盖（with-replace 语义）。
    pub fn register(mut self, lang: SourceLang, chunker: Arc<dyn ASTChunker>) -> Self {
        self.by_lang.insert(lang, chunker);
        self
    }

    /// 当前已注册的语言列表。
    pub fn supported_langs(&self) -> Vec<SourceLang> {
        self.by_lang.keys().copied().collect()
    }

    /// 给定文件路径推断语言（按扩展名）。
    pub fn detect_lang(path: &Path) -> SourceLang {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_lowercase();
        match ext.as_str() {
            "md" | "markdown" => SourceLang::Markdown,
            "ets" | "ts" | "tsx" => SourceLang::ArkTs,
            "kt" | "kts" => SourceLang::Kotlin,
            "swift" => SourceLang::Swift,
            "json" => SourceLang::Json,
            _ => SourceLang::Auto,
        }
    }

    /// 按 path 推断 lang 然后路由到 chunker。
    pub async fn chunk(
        &self,
        source_path: &str,
        content: &str,
        path_for_detect: &Path,
    ) -> Result<Vec<Chunk>> {
        let lang = Self::detect_lang(path_for_detect);
        match self.by_lang.get(&lang) {
            Some(c) => c.chunk(source_path, content, lang).await,
            None => Err(RagError::Chunker(format!(
                "未注册 {:?} chunker（已注册：{:?}）",
                lang,
                self.supported_langs()
            ))),
        }
    }

    /// 仅按 lang 调用（不走 path 检测）。
    pub async fn chunk_as(
        &self,
        source_path: &str,
        content: &str,
        lang: SourceLang,
    ) -> Result<Vec<Chunk>> {
        match self.by_lang.get(&lang) {
            Some(c) => c.chunk(source_path, content, lang).await,
            None => Err(RagError::Chunker(format!(
                "未注册 {:?} chunker",
                lang
            ))),
        }
    }

    /// 是否包含给定 lang 的路由。
    pub fn has(&self, lang: SourceLang) -> bool {
        self.by_lang.contains_key(&lang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MarkdownChunker;

    #[test]
    fn detect_by_extension() {
        assert_eq!(
            ChunkerDispatcher::detect_lang(Path::new("a.md")),
            SourceLang::Markdown
        );
        assert_eq!(
            ChunkerDispatcher::detect_lang(Path::new("a.ets")),
            SourceLang::ArkTs
        );
        assert_eq!(
            ChunkerDispatcher::detect_lang(Path::new("a.ts")),
            SourceLang::ArkTs
        );
        assert_eq!(
            ChunkerDispatcher::detect_lang(Path::new("a.kt")),
            SourceLang::Kotlin
        );
        assert_eq!(
            ChunkerDispatcher::detect_lang(Path::new("a.swift")),
            SourceLang::Swift
        );
        assert_eq!(
            ChunkerDispatcher::detect_lang(Path::new("a.unknown")),
            SourceLang::Auto
        );
    }

    #[tokio::test]
    async fn dispatch_to_markdown() {
        let d = ChunkerDispatcher::new()
            .register(SourceLang::Markdown, Arc::new(MarkdownChunker::new()));
        let chunks = d
            .chunk("a.md", "# Top\n\n## A\nbody\n", Path::new("a.md"))
            .await
            .unwrap();
        assert!(!chunks.is_empty());
    }

    #[tokio::test]
    async fn missing_lang_returns_err() {
        let d = ChunkerDispatcher::new()
            .register(SourceLang::Markdown, Arc::new(MarkdownChunker::new()));
        let r = d
            .chunk("a.kt", "fun main(){}", Path::new("a.kt"))
            .await;
        assert!(r.is_err());
        let msg = format!("{}", r.unwrap_err());
        assert!(msg.contains("Kotlin"));
    }

    #[test]
    fn has_and_supported_langs() {
        let d = ChunkerDispatcher::new()
            .register(SourceLang::Markdown, Arc::new(MarkdownChunker::new()));
        assert!(d.has(SourceLang::Markdown));
        assert!(!d.has(SourceLang::ArkTs));
        assert_eq!(d.supported_langs(), vec![SourceLang::Markdown]);
    }
}

//! MarkdownChunker —— 按 heading 切分。
//!
//! Day 1 最小可用：识别 `#` / `##` / `###` / `####` 行作为 chunk 边界，
//! 维护 heading 栈生成 `heading_path`，记录 chunk 在原文中的行号区间。
//!
//! **不处理**：HTML 嵌入、frontmatter、代码块跨段、表格——这些是 Week 2 的 enrich 阶段。

use arkui_rag_core::{
    chunker::SourceLang, ASTChunker, Chunk, ChunkId, ChunkMetadata, ChunkType, RagError, Result,
};
use async_trait::async_trait;

pub struct MarkdownChunker {
    name: String,
}

impl Default for MarkdownChunker {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownChunker {
    pub fn new() -> Self {
        Self {
            name: "markdown-heading-v1".to_string(),
        }
    }

    fn make_id(source_path: &str, heading_path: &[String], start_line: u32) -> ChunkId {
        // 简易稳定 id（避免引入 sha1 / uuid 额外依赖）
        let joined = heading_path.join("/");
        ChunkId::new(format!("{}#{}@{}", source_path, joined, start_line))
    }
}

#[async_trait]
impl ASTChunker for MarkdownChunker {
    fn name(&self) -> &str {
        &self.name
    }

    async fn chunk(
        &self,
        source_path: &str,
        content: &str,
        lang: SourceLang,
    ) -> Result<Vec<Chunk>> {
        if !matches!(lang, SourceLang::Markdown | SourceLang::Auto) {
            return Err(RagError::Chunker(format!(
                "MarkdownChunker 不支持语言 {:?}",
                lang
            )));
        }

        let lines: Vec<&str> = content.lines().collect();
        let mut chunks: Vec<Chunk> = Vec::new();
        let mut heading_stack: Vec<(usize, String)> = Vec::new(); // (level, title)
        let mut cur_start: usize = 0;
        let mut cur_path: Vec<String> = Vec::new();

        // 哨兵：扫到行尾后再 flush 一次
        for (idx, raw_line) in lines.iter().enumerate().chain(std::iter::once((lines.len(), &""))) {
            let line = *raw_line;
            let is_sentinel = idx == lines.len();

            if let Some((level, title)) = is_sentinel.then(|| (0, String::new())).or_else(|| parse_heading(line)) {
                // 遇到 heading（或哨兵）→ flush 当前 chunk
                if !cur_path.is_empty() && idx > cur_start {
                    let body = lines[cur_start..idx].join("\n");
                    if !body.trim().is_empty() {
                        let id = MarkdownChunker::make_id(source_path, &cur_path, cur_start as u32 + 1);
                        chunks.push(Chunk {
                            id,
                            content: body,
                            metadata: ChunkMetadata {
                                source: source_path.to_string(),
                                heading_path: cur_path.clone(),
                                line_range: Some((cur_start as u32 + 1, idx as u32)),
                                r#type: ChunkType::Generic,
                                ..ChunkMetadata::default()
                            },
                        });
                    }
                }

                if is_sentinel {
                    break;
                }

                // 更新 heading 栈
                while heading_stack.last().map_or(false, |(lv, _)| *lv >= level) {
                    heading_stack.pop();
                }
                heading_stack.push((level, title.clone()));
                cur_path = heading_stack.iter().map(|(_, t)| t.clone()).collect();
                cur_start = idx + 1;
            }
        }

        Ok(chunks)
    }
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|c| *c == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = &trimmed[level..];
    if !rest.starts_with(' ') && !rest.is_empty() {
        return None;
    }
    Some((level, rest.trim().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn splits_by_h2() {
        let md = "# Top\n\n## A\nbody a\n\n## B\nbody b\n";
        let ch = MarkdownChunker::new();
        let chunks = ch.chunk("t.md", md, SourceLang::Markdown).await.unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].metadata.heading_path, vec!["Top", "A"]);
        assert_eq!(chunks[1].metadata.heading_path, vec!["Top", "B"]);
        assert!(chunks[0].content.contains("body a"));
    }

    #[tokio::test]
    async fn ignores_non_heading_hash() {
        let md = "# Title\n\nsome `#hash` reference\n";
        let ch = MarkdownChunker::new();
        let chunks = ch.chunk("t.md", md, SourceLang::Markdown).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].metadata.heading_path, vec!["Title"]);
    }

    #[tokio::test]
    async fn rejects_non_markdown() {
        let ch = MarkdownChunker::new();
        let err = ch.chunk("t.kt", "fun main(){}", SourceLang::Kotlin).await;
        assert!(err.is_err());
    }
}

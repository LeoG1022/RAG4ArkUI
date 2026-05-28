//! MarkdownChunker —— 按 heading 切分 + YAML frontmatter 解析。
//!
//! Day 2 升级：识别 `---\n…\n---\n` 顶部 frontmatter，按 corpus/README.md 的元数据
//! schema 解析（platforms / api_version / deprecated / type / tags 等），把字段
//! 注入到每个 chunk 的 ChunkMetadata 里。
//!
//! **不处理**：HTML 嵌入、代码块跨段、表格——这些是 Week 3+ 的 enrich 阶段。

use arkui_rag_core::{
    chunk::Platform, chunker::SourceLang, ASTChunker, Chunk, ChunkId, ChunkMetadata, ChunkType,
    RagError, Result,
};
use async_trait::async_trait;
use serde::Deserialize;

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
            name: "markdown-heading-v2-frontmatter".to_string(),
        }
    }

    fn make_id(source_path: &str, heading_path: &[String], start_line: u32) -> ChunkId {
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

        let (frontmatter, body, line_offset) = split_frontmatter(content)?;
        let lines: Vec<&str> = body.lines().collect();
        let mut chunks: Vec<Chunk> = Vec::new();
        // (level, title, chunk_id) —— chunk_id 在当前 heading flush 时填入栈顶
        // Day 11：父子索引 —— 子 chunk 的 parent_id = 栈中前一层的 chunk_id
        let mut heading_stack: Vec<(usize, String, Option<ChunkId>)> = Vec::new();
        let mut cur_start: usize = 0;
        let mut cur_path: Vec<String> = Vec::new();

        // 哨兵：扫到行尾后再 flush 一次
        let total = lines.len();
        for idx in 0..=total {
            let is_sentinel = idx == total;
            let line = if is_sentinel { "" } else { lines[idx] };

            let trigger = if is_sentinel {
                Some((0_usize, String::new()))
            } else {
                parse_heading(line)
            };

            if let Some((level, title)) = trigger {
                if !cur_path.is_empty() && idx > cur_start {
                    let body_slice = lines[cur_start..idx].join("\n");
                    if !body_slice.trim().is_empty() {
                        let abs_start = (cur_start as u32) + 1 + line_offset;
                        let abs_end = (idx as u32) + line_offset;
                        // 父 chunk = 栈顶上一层（heading_stack 在 pop 后栈顶就是直接父级）
                        // 当前正要 flush 的 chunk 对应 heading_stack 最后一项（即"当前作用域"）
                        // 父 = 倒数第 2 项
                        let parent_id = if heading_stack.len() >= 2 {
                            heading_stack[heading_stack.len() - 2].2.clone()
                        } else {
                            None
                        };
                        let mut md = ChunkMetadata {
                            source: source_path.to_string(),
                            heading_path: cur_path.clone(),
                            line_range: Some((abs_start, abs_end)),
                            r#type: ChunkType::Generic,
                            parent_id,
                            ..ChunkMetadata::default()
                        };
                        if let Some(fm) = &frontmatter {
                            fm.apply_to(&mut md);
                        }
                        let id = MarkdownChunker::make_id(source_path, &cur_path, abs_start);
                        // 把刚生成的 chunk_id 写回栈顶 → 让"我"成为后续子 chunk 的 parent
                        if let Some(last) = heading_stack.last_mut() {
                            last.2 = Some(id.clone());
                        }
                        chunks.push(Chunk {
                            id,
                            content: body_slice,
                            metadata: md,
                        });
                    }
                }
                if is_sentinel {
                    break;
                }
                while heading_stack
                    .last()
                    .map_or(false, |(lv, _, _)| *lv >= level)
                {
                    heading_stack.pop();
                }
                heading_stack.push((level, title.clone(), None));
                cur_path = heading_stack.iter().map(|(_, t, _)| t.clone()).collect();
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

/// 解析 frontmatter（如果存在）。返回 (frontmatter, body_without_frontmatter, lines_skipped)。
///
/// 约定：仅当文件**首行**是 `---` 且能找到对应闭合 `---` 才识别为 frontmatter；
/// 否则原样返回 body。失败的 YAML 解析 → 报 Chunker 错误（让用户知道）。
fn split_frontmatter(content: &str) -> Result<(Option<Frontmatter>, &str, u32)> {
    let mut lines = content.lines();
    let Some(first) = lines.next() else {
        return Ok((None, content, 0));
    };
    if first.trim() != "---" {
        return Ok((None, content, 0));
    }
    // 找闭合 ---
    let mut yaml_lines: Vec<&str> = Vec::new();
    let mut close_found = false;
    let mut consumed_chars = first.len() + 1; // 包括 \n
    for line in lines {
        consumed_chars += line.len() + 1;
        if line.trim() == "---" {
            close_found = true;
            break;
        }
        yaml_lines.push(line);
    }
    if !close_found {
        // 没闭合 → 视为正文
        return Ok((None, content, 0));
    }
    let yaml = yaml_lines.join("\n");
    let fm: Frontmatter = serde_yaml::from_str(&yaml).map_err(|e| {
        RagError::Chunker(format!("frontmatter YAML 解析失败: {} (yaml: {:?})", e, yaml))
    })?;
    let body = &content[consumed_chars.min(content.len())..];
    let line_offset = (yaml_lines.len() as u32) + 2; // 两行 `---` + yaml 行数
    Ok((Some(fm), body, line_offset))
}

/// frontmatter schema —— 与 corpus/README.md 元数据 schema 对齐。
#[derive(Debug, Clone, Default, Deserialize)]
struct Frontmatter {
    #[serde(default)]
    api_name: Option<String>,
    #[serde(default)]
    platforms: Vec<String>,
    #[serde(default)]
    api_version: Option<String>,
    #[serde(default)]
    deprecated: bool,
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    source_framework: Option<String>,
    #[serde(default)]
    complexity: Option<String>,
    #[serde(default)]
    tags: Vec<String>,
}

impl Frontmatter {
    fn apply_to(&self, md: &mut ChunkMetadata) {
        if !self.platforms.is_empty() {
            md.platforms = self
                .platforms
                .iter()
                .filter_map(|s| match s.to_lowercase().as_str() {
                    "harmonyos" | "harmony" | "openharmony" => Some(Platform::HarmonyOs),
                    "android" => Some(Platform::Android),
                    "ios" => Some(Platform::Ios),
                    _ => None,
                })
                .collect();
        }
        if self.api_version.is_some() {
            md.api_version = self.api_version.clone();
        }
        md.deprecated = self.deprecated;
        if let Some(t) = &self.r#type {
            md.r#type = match t.as_str() {
                "api_doc" => ChunkType::ApiDoc,
                "code_example" => ChunkType::CodeExample,
                "migration_rule" => ChunkType::MigrationRule,
                "error_fix" => ChunkType::ErrorFix,
                _ => ChunkType::Generic,
            };
        }
        if self.source_framework.is_some() {
            md.source_framework = self.source_framework.clone();
        }
        if self.complexity.is_some() {
            md.complexity = self.complexity.clone();
        }
        if !self.tags.is_empty() {
            md.tags = self.tags.clone();
        }
        if let Some(api) = &self.api_name {
            md.extra
                .insert("api_name".to_string(), serde_json::Value::String(api.clone()));
        }
    }
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

    #[tokio::test]
    async fn frontmatter_parsed_and_applied() {
        let md = "---\nplatforms: [HarmonyOS, Android]\napi_version: \"ArkUI-X 1.2\"\ntype: api_doc\ntags: [routing]\n---\n\n# Router\n\n## pushUrl\nuse this to push.\n";
        let ch = MarkdownChunker::new();
        let chunks = ch.chunk("r.md", md, SourceLang::Markdown).await.unwrap();
        assert_eq!(chunks.len(), 1);
        let m = &chunks[0].metadata;
        assert_eq!(m.platforms, vec![Platform::HarmonyOs, Platform::Android]);
        assert_eq!(m.api_version.as_deref(), Some("ArkUI-X 1.2"));
        assert_eq!(m.r#type, ChunkType::ApiDoc);
        assert_eq!(m.tags, vec!["routing"]);
        // line_range 应该是相对原始文件（含 frontmatter 偏移）
        let (start, _end) = m.line_range.unwrap();
        assert!(start > 6, "expected start > 6 because frontmatter takes 6 lines, got {}", start);
    }

    #[tokio::test]
    async fn no_frontmatter_still_works() {
        let md = "# Plain\n\ncontent\n";
        let ch = MarkdownChunker::new();
        let chunks = ch.chunk("p.md", md, SourceLang::Markdown).await.unwrap();
        assert_eq!(chunks.len(), 1);
        assert!(chunks[0].metadata.platforms.is_empty());
    }

    #[tokio::test]
    async fn malformed_frontmatter_errors() {
        // YAML 解析失败：缩进不一致
        let md = "---\nplatforms:\n  -bad indent\nname: x\n---\n\n# X\nbody\n";
        let ch = MarkdownChunker::new();
        let r = ch.chunk("x.md", md, SourceLang::Markdown).await;
        assert!(r.is_err());
    }
}

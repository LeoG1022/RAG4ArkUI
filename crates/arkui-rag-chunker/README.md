# arkui-rag-chunker

**定位**：文档切分。把原始文档 → 一组 `Chunk`（带元数据 + heading_path + line_range）。

## Day 1 提供

| 实现 | 状态 |
|---|---|
| `MarkdownChunker` | ✅ 最小可用：按 `#` / `##` / `###` heading 切分，保留 heading_path 与 line_range |
| `ArkTsChunker` | ⏳ stub（Week 2 接 tree-sitter-typescript） |
| `KotlinChunker` | ⏳ stub |
| `SwiftChunker` | ⏳ stub |

技术方案对应：§2.3 代码感知的 Chunking 策略、§4.2 决策 6。

## 设计原则

1. **绝不按固定字符数切**——破坏语义边界
2. **必须保留 heading_path / line_range**——用于 Parent-Child 扩展和引用回链
3. **元数据由切分器初步填充**（type / source / heading_path），后续 enrich 阶段补 platforms / version

## 用法

```rust,ignore
use arkui_rag_chunker::MarkdownChunker;
use arkui_rag_core::{ASTChunker, chunker::SourceLang};

let ch = MarkdownChunker::new();
let chunks = ch.chunk(
    "corpus/official/router.md",
    "# Router\n\n## pushUrl\n...",
    SourceLang::Markdown,
).await.unwrap();
assert!(!chunks.is_empty());
```

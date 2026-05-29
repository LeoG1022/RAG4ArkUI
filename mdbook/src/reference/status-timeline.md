# STATUS 时间线

每个 Day 提交后 agent 强制写 `docs/STATUS-<slug>.md`（FAIL 级规则 M-STATUS-PER-ROUND）。
18 个文档按时间排列：

## Week 1 · Bootstrap

| Day | STATUS | 主要内容 |
|---|---|---|
| 2 | [STATUS-day2](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day2.md) | 端到端 Mock Demo + indexer crate |

## Week 2 · 真后端 + 评估

| Day | STATUS | 主要内容 |
|---|---|---|
| 4 | [STATUS-day4-bm25-tantivy](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day4-bm25-tantivy.md) | TantivyBM25 真活 |
| Bootstrap | [STATUS-bootstrap-status-rule](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-bootstrap-status-rule.md) | 规则 #17 STATUS-PER-ROUND |
| 5 | [STATUS-day5-reranker](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day5-reranker.md) | OnnxReranker async wrapper |
| 6 | [STATUS-day6-eval](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day6-eval.md) | 检索质量评估 arkui-rag-eval crate |
| — | [STATUS-roadmap-doc](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-roadmap-doc.md) | ROADMAP.md 全景图归档 |
| 7 | [STATUS-day7-hyde](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day7-hyde.md) | HyDE QueryEnhancer |
| 9 | [STATUS-day9-lancedb](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day9-lancedb.md) | LanceDB 嵌入式向量库 |
| 10 | [STATUS-day10-tree-sitter](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day10-tree-sitter.md) | tree-sitter ArkTS / TS 代码切分 |
| 11 | [STATUS-day11-parent-child](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day11-parent-child.md) | Parent-Child 父子索引 + ContextAssembler |

## Week 3-4 · 协议层 3/3 ⭐

| Day | STATUS | 主要内容 |
|---|---|---|
| 14 | [STATUS-day14-http](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day14-http.md) | HTTP/REST Server（axum） |
| 15 | [STATUS-day15-mcp](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day15-mcp.md) | MCP Server（JSON-RPC stdio · 4 tools） |
| 16 | [STATUS-day16-lsp](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day16-lsp.md) | LSP Server（Content-Length framing · 5 method） |

## Week 5 · Agent 接入

| Day | STATUS | 主要内容 |
|---|---|---|
| 19 | [STATUS-day19-claude-code](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day19-claude-code.md) | Claude Code MCP 接入指南 + bash demo |

## Week 6 · 发布 ⭐

| Day | STATUS | 主要内容 |
|---|---|---|
| 20a | [STATUS-day20a-release-local](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day20a-release-local.md) | 本地 host release artifact |
| 20b | [STATUS-day20b-ci-matrix](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-day20b-ci-matrix.md) | CI matrix 4 平台自动 release |
| 20c | [STATUS-pre-existing-fixes](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-pre-existing-fixes.md) | pre-existing 阻塞清理（typescript + chrono） |
| 21 | [STATUS-corpus-pull](https://github.com/LeoG1022/RAG4ArkUI/blob/master/docs/STATUS-corpus-pull.md) | `arkui-rag corpus pull` 真活 |
| 22 | **本轮** | mdBook 文档站 + 1.0 release page |

## 文件命名约定

`docs/STATUS-<slug>.md` · slug 与 `feedback/features/.../<N>-<date>-<slug>.md` 的 slug 完全对齐（M-STATUS-PER-ROUND FAIL 级规则强制）。

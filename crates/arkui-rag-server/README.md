# arkui-rag-server

**定位**：协议适配层。同一个引擎对接三套消费者：

| 协议 | feature | 消费者 |
|---|---|---|
| HTTP/REST | `--features http` | IDE 插件、curl 调试 |
| MCP (stdio + SSE) | `--features mcp` | Claude Code / Cursor / OpenCode |
| LSP | `--features lsp` | DevEco / IntelliJ inline 提示 |

技术方案对应：§4.2 决策 2、§9 图 2、§9 图 8。

## Day 1 状态

全部是路由 stub —— 函数签名与返回类型就位，handler 内打印 TODO。Week 4 实施真实路由。

## MCP 暴露的 4 个工具（规约）

照搬技术方案 §4.2 决策 2：

| 工具 | 用途 |
|---|---|
| `arkui_search_docs(query, platform_filter, top_k)` | 检索 API 文档 / 规范 |
| `arkui_search_code(query, mode, top_k)` | 检索代码示例 |
| `arkui_migrate_snippet(source_code, from, to)` | 迁移建议 |
| `arkui_validate_api(code)` | API 时效性 + 平台兼容性校验 |

## HTTP 路由（规约）

```
POST /search          { query, top_k, filters } → { hits[], citations[], latency_ms }
POST /index           { source_path }           → { indexed: N, skipped: M, errors: [] }
GET  /health                                    → { status, model_id, corpus_version }
GET  /corpus/list                               → 列出 corpus/ 子目录与文档数
```
